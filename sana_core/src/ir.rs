use std::{ops::Not, collections::VecDeque};

use crate::automata::{Automata, NodeKind, State};

/// An intermediate representation
pub struct Ir<T> {
    pub blocks: Vec<Block<T>>
}

pub enum Block<T> {
    Block(Vec<Op<T>>),
    Func(Vec<Op<T>>),
}

impl<T> Block<T> {
    fn push(&mut self, op: Op<T>) {
        match self {
            Block::Block(ops)
            | Block::Func(ops) =>
                ops.push(op)
        }
    }

    fn ops(&self) -> &[Op<T>] {
        match self {
            Block::Block(ops)
            | Block::Func(ops) =>
                ops
        }
    }
}

/// IR opcodes
#[derive(Debug, Clone, PartialEq)]
pub enum Op<T> {
    /// Shift the cursor to the next character
    Shift,
    /// Jump if matches
    JumpMatches {
        from: char,
        to: char,
        on_success: usize,
    },
    /// Jump if not matches
    JumpNotMatches {
        from: char,
        to: char,
        on_failure: usize,
    },
    LoopMatches {
        from: char,
        to: char,
    },
    /// Just jump
    Jump(usize),
    /// Set current action
    Set(T),
    /// Halt and return an action, if any
    Halt,
}

/// Pretty print the IR
pub fn pprint_ir<T: std::fmt::Debug>(ir: &Ir<T>) {
    for (i, block) in ir.blocks.iter().enumerate() {
        match block {
            Block::Block(ops) => {
                println!("l{}:", i);

                for op in ops { pprint_op(op) }
            },
            Block::Func(ops) => {
                println!("l{}(λ):", i);

                for op in ops { pprint_op(op) }
            },
        };
    }
}

fn pprint_op<T: std::fmt::Debug>(op: &Op<T>) {
    use Op::*;

    print!("    ");
    match op {
        Shift =>
            println!("shift"),
        JumpMatches { from, to, on_success } =>
            println!("jm {:?} {:?} l{}", from, to, on_success),
        JumpNotMatches { from, to, on_failure } =>
            println!("jnm {:?} {:?} l{}", from, to, on_failure),
        LoopMatches { from, to } =>
            println!("lm {:?} {:?}", from, to),
        Jump(to) =>
            println!("jump l{}", to),
        Set(act) =>
            println!("set {:?}", act),
        Halt =>
            println!("halt"),
    }
}

impl<T: Clone> Ir<T> {
    /// Create IR from DFA
    pub fn from_automata(automata: Automata<T>) -> Ir<T> {
        let terminal = automata.find_terminal_node();
        let node_kinds = automata.node_kinds();

        let mut state_blocks: Vec<Option<usize>> =
            vec![None; automata.states.len()];
        let mut blocks = vec![
            Block::Func::<T>(vec![])
        ];
        state_blocks[0] = Some(0);

        fn insert_block<T>(
            st: usize,
            state_blocks: &mut [Option<usize>],
            blocks: &mut Vec<Block<T>>,
            node_kinds: &[NodeKind]
        ) -> usize {
            if let Some(i) = state_blocks[st] {
                i
            }
            else {
                let i = blocks.len();

                let block =
                    if node_kinds[st] == NodeKind::Sink {
                        Block::Func(vec![])
                    }
                    else { Block::Block(vec![]) };

                blocks.push(block);
                state_blocks[st] = Some(i);

                i
            }
        }

        let mut queue = VecDeque::new();
        queue.push_back(0usize);
        queue.push_back(terminal);

        let terminal_block = insert_block(
            terminal,
            &mut state_blocks,
            &mut blocks,
            &node_kinds
        );

        while let Some(st) = queue.pop_front() {
            let block_ix = state_blocks[st].unwrap();

            // Inital and terminal do not shift
            if st != 0 && st != terminal {
                blocks[block_ix].push(Op::Shift)
            }

            if let Some(State::Action(act)) = automata.get(st) {
                blocks[block_ix].push(Op::Set(act.clone()))
            }

            match node_kinds[st] {
                NodeKind::Sink | NodeKind::Fork => {
                    let (loops, next) = automata.transitions_from(st)
                        .partition::<Vec<_>, _>(|&(_, to)| to == st);

                    for (ch, _) in loops {
                        blocks[block_ix].push(Op::LoopMatches {
                            from: ch.start,
                            to: ch.end,
                        });
                    }

                    for (ch, to) in next {
                        if state_blocks[to].is_none() {
                            queue.push_back(to)
                        }

                        let to_block = insert_block(
                            to,
                            &mut state_blocks,
                            &mut blocks,
                            &node_kinds
                        );

                        blocks[block_ix].push(Op::JumpMatches {
                            from: ch.start,
                            to: ch.end,
                            on_success: to_block,
                        });
                    }
                },
                NodeKind::Link => {
                    let (loops, next) = automata.transitions_from(st)
                        .partition::<Vec<_>, _>(|&(_, to)| to == st);

                    for (ch, _) in loops {
                        blocks[block_ix].push(Op::LoopMatches {
                            from: ch.start,
                            to: ch.end,
                        });
                    }

                    let mut jumps = 0;
                    for (ch, to) in next {
                        if to == terminal { continue }

                        jumps += 1;

                        if state_blocks[to].is_none() {
                            queue.push_back(to)
                        }

                        let to_block = insert_block(
                            to,
                            &mut state_blocks,
                            &mut blocks,
                            &node_kinds
                        );

                        blocks[block_ix].push(Op::JumpNotMatches {
                            from: ch.start,
                            to: ch.end,
                            on_failure: terminal_block,
                        });
                        blocks[block_ix].push(Op::Jump(to_block));
                    }

                    if jumps == 0 {
                        blocks[block_ix].push(Op::Halt)
                    }
                },
                NodeKind::Leaf => {
                    for (ch, to) in automata.transitions_from(st) {
                        if to == terminal { continue }

                        blocks[block_ix].push(Op::LoopMatches {
                            from: ch.start,
                            to: ch.end,
                        });
                    }

                    blocks[block_ix].push(Op::Halt)
                },
                NodeKind::Terminal =>
                    blocks[block_ix].push(Op::Halt),
            }
        }

        Ir { blocks }
    }

    /// Convert IR to the code suitable for VM execution
    pub fn flatten(&self) -> Vec<Op<T>> {
        let mut code = vec![];
        let mut symbol_map = Vec::with_capacity(self.blocks.len());

        let mut code_len = 0;
        for block in &self.blocks {
            code.extend(block.ops().iter().cloned());
            symbol_map.push(code_len);
            code_len += block.ops().len();
        }

        for op in &mut code {
            match op {
                Op::JumpMatches { on_success: loc, .. }
                | Op::JumpNotMatches { on_failure: loc, ..}
                | Op::Jump(loc) =>
                    *loc = symbol_map[*loc],
                _ => (),
            }
        }

        code
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// Result returned by `Vm`
pub enum VmResult<T> {
    /// Action with span `start..end`
    Action {
        start: usize,
        end: usize,
        action: T
    },
    /// Error with span `start..end`
    Error {
        start: usize,
        end: usize,
    },
    /// End of input
    Eoi,
}

#[derive(Debug, Clone)]
pub struct Vm<'code, 'input, T> {
    pub input: &'input str,
    code: &'code [Op<T>],
    iter: std::str::Chars<'input>,
    cursor: Option<char>,
    pos: usize,
}

impl<'code, 'input, T: Clone> Vm<'code, 'input, T> {
    pub fn new(code: &'code [Op<T>], input: &'input str) -> Self {
        let mut iter = input.chars();
        let cursor = iter.next();
        let pos = 0;

        Vm { code, input, iter, cursor, pos }
    }

    pub fn position(&self) -> usize {
        self.pos
    }

    fn shift(&mut self) {
        self.pos += self.cursor
            .map(char::len_utf8)
            .unwrap_or(1);
        self.cursor = self.iter.next();
    }

    /// Set the cursor position
    pub fn rewind(&mut self, pos: usize) {
        self.iter = self.input[pos..].chars();
        self.cursor = self.iter.next();
        self.pos = pos;
    }

    /// Execute the loaded code
    pub fn run(&mut self) -> VmResult<T> {
        let mut inst_ptr = 0;
        let mut jump_ptr = 0;

        let mut action = None;
        let start = self.pos;
        let mut end = start;

        if self.cursor.is_none() {
            return VmResult::Eoi
        }

        loop {
            match &self.code[inst_ptr] {
                Op::Shift => {
                    self.shift();
                },
                Op::JumpMatches { from, to, on_success } => {
                    let cursor =
                        if let Some(ch) = self.cursor { ch }
                        else { break };

                    if (*from..=*to).contains(&cursor) {
                        inst_ptr = *on_success;
                        jump_ptr = *on_success;

                        continue;
                    }

                },
                Op::JumpNotMatches { from, to, on_failure } => {
                    let cursor =
                        if let Some(ch) = self.cursor { ch }
                        else { break };

                    if (*from..=*to).contains(&cursor).not() {
                        inst_ptr = *on_failure;
                        jump_ptr = *on_failure;

                        continue;
                    }
                },
                Op::LoopMatches { from, to} => {
                    let cursor =
                        if let Some(ch) = self.cursor { ch }
                        else { break };

                    if (*from..=*to).contains(&cursor) {
                        inst_ptr = jump_ptr;

                        continue
                    }
                },
                Op::Jump(loc) => {
                    inst_ptr = *loc;
                    jump_ptr = *loc;

                    continue
                },
                Op::Set(act) => {
                    action = Some(act.clone());
                    end = self.pos;
                },
                Op::Halt => break,
            };

            inst_ptr += 1;
        }

        if action.is_none() && self.pos != self.input.len() {
            return VmResult::Error {
                start,
                end: self.pos,
            }
        }

        if end != self.pos { self.rewind(end) }

        match action {
            Some(action) =>
                VmResult::Action { start, end, action },
            None =>
                VmResult::Eoi
        }
    }
}
