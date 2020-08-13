use syn::Ident;
use heck::ShoutySnakeCase;
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::{format_ident, quote};

use sana_core::ir::{Op, Ir};
use crate::{SanaSpec, Backend};

pub(crate) fn pprint_ir(spec: SanaSpec) -> ! {
    let dfa = match spec.rules.construct_dfa() {
        Ok(dfa) => dfa,
        Err(sana_core::Error::AmbiguityError(ix, i)) =>
            abort!(spec.variants[i].span(), "Ambiguous rule";
            note = spec.variants[ix].span() => "Resolve conflicts with {}", spec.variants[ix]),
    };

    let ir = Ir::from_automata(dfa);

    abort!(
        spec.pprint_ir.unwrap(), "The user wants IR";
        help = "\n{}", ir.pretty()
    );
}

pub(crate) fn generate(spec: SanaSpec) -> TokenStream {
    let dfa = match spec.rules.construct_dfa() {
        Ok(dfa) => dfa,
        Err(sana_core::Error::AmbiguityError(ix, i)) =>
            abort!(spec.variants[i].span(), "Ambiguous rule";
            note = spec.variants[ix].span() => "Resolve conflicts with {}", spec.variants[ix]),
    };

    let ir = Ir::from_automata(dfa);

    let enum_ident = spec.enum_ident;
    let enum_const_name = enum_ident.to_string()
        .to_shouty_snake_case();
    let ir_var = format_ident!("_{}_IR", enum_const_name);
    let ir_code = generate_ir(&enum_ident, &ir, &spec.variants);

    let lexer_name = format_ident!("_{}_LEXER", enum_const_name);
    let bytecode = analyze_ir(&ir);
    let rust_code = compile_bytecode(bytecode, &enum_ident, &spec.variants);
    let error = spec.terminal;

    let uses_vm = spec.backend == Backend::Vm;

    quote! {
        #[doc(hidden)]
        const #ir_var: &'static [sana::ir::Op<#enum_ident>] = #ir_code;

        impl sana::Sana for #enum_ident {
            const ERROR: Self = #enum_ident::#error;
            const USES_VM: bool = #uses_vm;

            fn ir() -> &'static [sana::ir::Op<Self>] { #ir_var }
            fn lex<'input>(cursor: &mut sana::ir::Cursor<'input>) -> sana::ir::VmResult<Self> {
                let mut lexer = #lexer_name::new();
                lexer.run(cursor)
            }
        }

        struct #lexer_name {
            action: ::core::option::Option<#enum_ident>,
            end: usize,
        }

        impl #lexer_name {
            fn new() -> Self {
                let action = None;
                let end = 0;
                Self { action, end }
            }

            fn run<'input>(&mut self, cursor: &mut sana::ir::Cursor<'input>) -> sana::ir::VmResult<#enum_ident> {
                self.action = None;

                if cursor.is_eoi() {
                    return sana::ir::VmResult::Eoi
                }

                let start = cursor.position();

                // l0 is the entry point
                self._l0(cursor);

                if self.action.is_none() && !cursor.is_eoi() {
                    return sana::ir::VmResult::Error {
                        start,
                        end: cursor.position(),
                    }
                }

                if self.end != cursor.position() { cursor.rewind(self.end) }

                if let Some(action) = self.action.take() {
                    sana::ir::VmResult::Action { start, end: self.end, action }
                } else {
                    sana::ir::VmResult::Eoi
                }
            }

            // The implementation of compile-time lexer
            #rust_code
        }
    }
}

fn generate_ir(enum_ident: &Ident, ir: &Ir<usize>, variants: &[Ident]) -> TokenStream {
    let code = ir.flatten();
    let mut ops = vec![];

    for op in code {
        let op = match op {
            Op::Shift => quote! {
                sana::ir::Op::Shift
            },
            Op::JumpMatches { from, to, on_success } => quote! {
                sana::ir::Op::JumpMatches {
                    from: #from,
                    to: #to,
                    on_success: #on_success,
                }
            },
            Op::JumpNotMatches { from, to, on_failure } => quote! {
                sana::ir::Op::JumpNotMatches {
                    from: #from,
                    to: #to,
                    on_failure: #on_failure,
                }
            },
            Op::LoopMatches { from, to } => quote! {
                sana::ir::Op::LoopMatches {
                    from: #from,
                    to: #to,
                }
            },
            Op::Jump(loc) => quote! {
                sana::ir::Op::Jump(#loc)
            },
            Op::Set(act) => {
                let var = &variants[act];

                quote! { sana::ir::Op::Set(#enum_ident::#var) }
            },
            Op::Halt => quote! {
                sana::ir::Op::Halt
            },
        };

        ops.push(op)
    }

    quote! {
        &[
            #(#ops),*
        ]
    }
}

// A block is a list of of statements with additional information.
#[derive(Debug)]
struct Block {
    id: usize,
    // whether there are jumps from inner blocks to the beginning of this block
    is_loop: bool,
    // whether there are multiple jumps into this block
    // and it should not be inlined
    is_func: bool,
    code: Vec<Stmt>,
}

type BlockId = usize;

#[derive(Debug)]
enum Stmt {
    /// Set current action
    Set(usize),
    /// Shift the cursor to the next character
    Shift,
    /// A set of guards to jump to a guard's suitable block
    Match(Match),
    /// jump to the block if the head does not match the given range
    JumpNotMatches {
        range: (char, char),
        block: BlockId,
    },
    /// An unconditional jump to the block
    Jump(BlockId),
    /// Halt and return an action, if any
    Halt,
}

/// A set of guards to jump to a guard's suitable block
#[derive(Debug)]
struct Match {
    arms: Vec<MatchArm>
}

/// Jump if matches
#[derive(Debug, Clone)]
struct MatchArm {
    ranges: Vec<(char, char)>,
    block: BlockId,
}

#[derive(Debug)]
pub struct Bytecode {
    blocks: Vec<Block>,
}

fn analyze_ir(ir: &Ir<usize>) -> Bytecode {
    let mut blocks: Vec<Block> = ir.blocks.iter()
        .enumerate()
        .map(|(id, block)| {
            let (is_func, ops) = match block {
                sana_core::ir::Block::Block(ops) => (false, ops),
                sana_core::ir::Block::Func(ops) => (true, ops),
            };

            let mut is_loop = false;

            let mut code = vec![];

            // we collect match guards into a single match statement if they follow each other
            // once the are broken with another statement we should dump the accumulator
            let mut match_acc: Option<Match> = None;

            for op in ops {
                match op {
                    Op::Set(act) => {
                        if let Some(match_acc) = match_acc.take() {
                            // dump match acc
                            code.push(Stmt::Match(match_acc))
                        }
                        code.push(Stmt::Set(*act));
                    },
                    Op::Shift => {
                        if let Some(match_acc) = match_acc.take() {
                            // dump match acc
                            code.push(Stmt::Match(match_acc))
                        }
                        code.push(Stmt::Shift);
                    },
                    Op::JumpMatches { from, to, on_success } => {
                        let match_arm = MatchArm {
                            ranges: vec![(*from, *to)],
                            block: *on_success
                        };
                        if let Some(match_acc) = match_acc.as_mut() {
                            match_acc.arms.push(match_arm);
                        } else {
                            match_acc = Some(Match {
                                arms: vec![match_arm],
                            });
                        }
                    },
                    Op::LoopMatches { from, to } => {
                        is_loop = true;

                        let match_arm = MatchArm {
                            ranges: vec![(*from, *to)],
                            block: id // self block id
                        };
                        if let Some(match_acc) = match_acc.as_mut() {
                            match_acc.arms.push(match_arm);
                        } else {
                            match_acc = Some(Match {
                                arms: vec![match_arm],
                            });
                        }
                    },
                    Op::JumpNotMatches { from, to, on_failure } => {
                        if let Some(match_acc) = match_acc.take() {
                            // dump match acc
                            code.push(Stmt::Match(match_acc))
                        }
                        code.push(Stmt::JumpNotMatches { range: (*from, *to), block: *on_failure });
                    },
                    Op::Jump(id) => {
                        if let Some(match_acc) = match_acc.take() {
                            // dump match acc
                            code.push(Stmt::Match(match_acc))
                        }
                        code.push(Stmt::Jump(*id));
                    },
                    Op::Halt => {
                        if let Some(match_acc) = match_acc.take() {
                            code.push(Stmt::Match(match_acc))
                        }
                        code.push(Stmt::Halt);
                    },
                }
            }
            // if the last statement was a match guard, then we should dump match statement
            if let Some(match_acc) = match_acc.take() {
                code.push(Stmt::Match(match_acc))
            }

            Block {
                id,
                is_loop,
                is_func,
                code,
            }
        })
        .collect();

    for block in blocks.iter_mut() {
        for op in block.code.iter_mut() {
            if let Stmt::Match(match_stmt) = op {
                optimize_match(match_stmt);
            }
        }
    }

    Bytecode { blocks }
}

fn optimize_match(match_stmt: &mut Match) {
    if !match_stmt.arms.is_empty() {
        let mut arms = match_stmt.arms.clone();

        // sort by block
        arms.sort_by(|l, r| l.block.cmp(&r.block));

        // group by block
        let mut new_arms: Vec<MatchArm> = vec![];
        for arm in arms.iter() {
            match new_arms.last_mut() {
                Some(new_arm) if new_arm.block == arm.block => {
                    new_arm.ranges.extend(&arm.ranges);
                }
                _ => {
                    let match_arm = MatchArm {
                        ranges: arm.ranges.clone(),
                        block: arm.block,
                    };
                    new_arms.push(match_arm);
                },
            }
        }

        match_stmt.arms = new_arms;
    }
}

use std::collections::HashSet;

pub fn compile_bytecode(bytecode: Bytecode, enum_ident: &Ident, variants: &[Ident]) -> TokenStream {
    let mut fns: Vec<TokenStream> = vec![];

    // compile only functions
    for block in bytecode.blocks.iter().filter(|b| b.is_func) {
        let name = format_ident!("_l{}", block.id);

        let body = func_to_rust(&bytecode, block.id, enum_ident, variants);

        fns.push(quote! {
            #[allow(clippy::needless_return)]
            fn #name<'input>(&mut self, cursor: &mut sana::ir::Cursor<'input>) { #body }
        });
    }

    quote! {
        #(#fns)*
    }
}

macro_rules! format_lifetime {
    ($($arg:tt)*) => {{
        let name = format!($($arg)*);

        syn::Lifetime::new(&name, proc_macro2::Span::call_site())
    }}
}

fn func_to_rust(bytecode: &Bytecode, block_id: BlockId, enum_ident: &Ident, variants: &[Ident]) -> TokenStream {
    let block = &bytecode.blocks[block_id];
    assert_eq!(block.id, block_id);

    let mut call_stack = HashSet::new();
    call_stack.insert(block.id);

    let code = block.code.iter()
        .map(|stmt| stmt_to_rust(&mut call_stack, bytecode, stmt, enum_ident, variants))
        .collect::<Vec::<_>>();

    if block.is_loop {
        let label = format_lifetime!("'l{}", block.id);
        let break_op = match block.code.last() {
            Some(Stmt::Halt) => quote!{ },
            _ => quote!{ break },
        };

        quote! {
            #label: loop {
                #(#code);*;

                #break_op
            }
        }
    }
    else {
        quote! {
            #(#code);*;
        }
    }
}

fn block_to_rust(call_stack: &mut HashSet<BlockId>, bytecode: &Bytecode, block_id: BlockId, enum_ident: &Ident, variants: &[Ident]) -> TokenStream {
    let block = &bytecode.blocks[block_id];
    assert_eq!(block.id, block_id);

    if block.is_loop && call_stack.contains(&block.id) {
        // we are trying to jump into the beginning of a loop from the middle of a block
        let label = format_lifetime!("'l{}", block_id);
        return quote! { continue #label }
    }

    if block.is_func {
        // any call to a function inside a block results into a call
        let func = format_ident!("_l{}", block.id);
        return quote! { return self.#func(cursor); }
    }

    if call_stack.contains(&block.id) {
        panic!("recursion in block {} which is not a function nor a loop. call stack: {:?}", block.id, &call_stack);
    }

    call_stack.insert(block.id);

    let code = block.code.iter()
        .map(|stmt| stmt_to_rust(call_stack, bytecode, stmt, enum_ident, variants))
        .collect::<Vec::<_>>();

    call_stack.remove(&block.id);

    if block.is_loop {
        let label = format_lifetime!("'l{}", block.id);
        let break_op = match block.code.last() {
            Some(Stmt::Halt) => quote!{ },
            _ => quote!{ break },
        };

        quote! {
            #label: loop {
                #(#code);*;

                #break_op
            }
        }
    }
    else {
        quote! {
            #(#code);*;
        }
    }
}

fn match_arm_to_rust(call_stack: &mut HashSet<BlockId>, bytecode: &Bytecode, arm: &MatchArm, enum_ident: &Ident, variants: &[Ident]) -> TokenStream {
    let ranges = arm.ranges.iter()
        .map(|(from, to)| quote! { #from ..= #to });
    let block = block_to_rust(call_stack, bytecode, arm.block, enum_ident, variants);

    quote! {
        #(#ranges)|* => { #block }
    }
}

fn stmt_to_rust(call_stack: &mut HashSet<BlockId>, bytecode: &Bytecode, stmt: &Stmt, enum_ident: &Ident, variants: &[Ident]) -> TokenStream {
    match stmt {
        Stmt::Set(act) => {
            let var = &variants[*act];
            quote! {
                self.action = Some(#enum_ident::#var);
                self.end = cursor.position();
            }
        },
        Stmt::Shift => {
            quote! { cursor.shift() }
        },
        Stmt::Match(Match { arms }) => {
            let arms = arms.iter()
                .map(|arm| match_arm_to_rust(call_stack, bytecode, arm, enum_ident, variants))
                .collect::<Vec::<_>>();

            quote! {
                let ch = match cursor.head {
                    Some(ch) => ch,
                    None => return,
                };

                match ch {
                    #(#arms),*,
                    _ => (),
                }
            }
        },
        Stmt::JumpNotMatches { range, block } => {
            let (from, to) = range;
            let block = block_to_rust(call_stack, bytecode, *block, enum_ident, variants);

            quote! {
                let ch = match cursor.head {
                    Some(ch) => ch,
                    None => return,
                };

                if !(#from..=#to).contains(&ch) { #block }
            }
        },
        Stmt::Jump(block_id) => {
            block_to_rust(call_stack, bytecode, *block_id, enum_ident, variants)
        },
        Stmt::Halt =>
            quote! { return },
    }
}
