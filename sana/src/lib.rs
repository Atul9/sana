//! Sana is a crate that allows you to create lexers easily.
//!
//! # Example
//!
//! ```rust
//! use sana::{Sana, Spanned};
//!
//! #[derive(Debug, Clone, Copy, PartialEq, Sana)]
//! enum Token {
//!     #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
//!     Ident,
//!     #[regex("[0-9]+")]
//!     Integer,
//!
//!     #[token("let", priority = 1)]
//!     Let,
//!     #[token("=")]
//!     Equals,
//!     #[regex(";")]
//!     Semicolon,
//!
//!     #[regex("[ \t\r\n]")]
//!     Whitespace,
//!     #[error]
//!     Error,
//! }
//!
//! let mut lexer = Token::lexer("let answer = 42;");
//!
//! assert_eq!(
//!     lexer.next(),
//!     Some(Spanned { value: Token::Let, start: 0, end: 3 })
//! );
//! assert_eq!(lexer.next(),
//!     Some(Spanned { value: Token::Whitespace, start: 3, end: 4 })
//! );
//! assert_eq!(lexer.next(),
//!     Some(Spanned { value: Token::Ident, start: 4, end: 10 })
//! );
//! assert_eq!(lexer.next(),
//!     Some(Spanned { value: Token::Whitespace, start: 10, end: 11 })
//! );
//! assert_eq!(
//!     lexer.next(),
//!     Some(Spanned { value: Token::Equals, start: 11, end: 12 })
//! );
//! assert_eq!(
//!     lexer.next(),
//!     Some(Spanned { value: Token::Whitespace, start: 12, end: 13 })
//! );
//! assert_eq!(
//!     lexer.next(),
//!     Some(Spanned { value: Token::Integer, start: 13, end: 15 })
//! );
//! assert_eq!(
//!     lexer.next(),
//!     Some(Spanned { value: Token::Semicolon, start: 15, end: 16 })
//! );
//!
//! // No tokens left
//! assert_eq!(
//!     lexer.next(),
//!     None
//! );
//! ```
//!
//! # Lexers – how do they work?
//!
//! A lexer is defined by a set of rules. A rule is a struct
//! `{ regex, priority, action }` where `regex` is a regular expression, `priority`
//! is the priority of the rule, and `action` is the action that corresponds to
//! the token that the rule matches.
//!
//! We say, that a string `s` *matches* regular expression `r`, if `s` belongs to the
//! language (a set of strings) defined by `r`. For example, if `r = [a-z]`, then the
//! string `"a"` matches `r`, because `"a" ∈ { "a", "b", ..., "z" }` but the string
//! `"9"` does not match `r`.
//!
//! Note, that unlike many regular expression implementations like
//! [regex](https://docs.rs/regex), we consider only the full string match rather
//! than a substring match. So the string `"abc"` does *not* match `[a-z]`.
//!
//! A string `s` *matches a rule set* if the rule set contains at least one rule `r`
//! such as `r.regex` matches `s`.
//!
//! A *prefix set* of a string is a set of all prefixes of that string. For example,
//! the prefix set of `"let"` is `{ "l", "le", "let" }`.
//!
//! The *longest match* of the string `s` by a rule set `R` is the longest string
//! `s_max` in the prefix set of `s` such as `s_max` matches `R`.
//!
//! The job of the lexer is to find the longest match of the rule set and emit the
//! action of the rule that matches the longest match. If there more than one such
//! rules, the rule with the highest priority is selected.
//!
//! Consider example above. The longest match of `"let answer = 42;"` is `"let"`.
//! There are two rules that match this string:
//!
//! - `{ regex: "[a-zA-Z_][a-zA-Z0-9_]*", priority: 0, action: Ident }`
//! - `{ regex: "let", priority: 1, action: Let }`.
//!
//! From these rules, the `Let` rule is selected, because it has the highest priority.
//!
//! Implementation-wise, a lexer generator constructs a DFA to match a string by a
//! rule set. The generated lexer consumes the input until it reaches the teminal state. While
//! walking the DFA, lexer remembers the last seen action state. When the teminal
//! state is reached, the lexer returns the last action state or an error, if the
//! lexer visited no action states.

#[doc(hidden)]
pub use sana_core::ir;
pub use sana_derive::Sana;

use sana_core::ir::{Op, Vm};

/// Trait implemented for an enum representing all tokens.
///
/// The trait implemented by `#[derive(Sana)]`. You should not implement it yourself.
pub trait Sana: Sized + Clone + Copy {
    const ERROR: Self;
    #[doc(hidden)]
    const USES_VM: bool = false;

    #[doc(hidden)]
    fn ir() -> &'static [Op<Self>];

    #[doc(hidden)]
    fn lex<'input>(_cursor: &mut ir::Cursor<'input>) -> ir::VmResult<Self> {
        ir::VmResult::Eoi
    }

    /// Create a new `Lexer` that will produce tokens of this type
    fn lexer(input: &str) -> Lexer<'_, Self> {
        Lexer::new(input)
    }
}

/// The `Lexer` is an `Iterator` of tokens
#[derive(Debug, Clone)]
pub struct Lexer<'input, Token: Sana + 'static> {
    vm: Vm<'static, 'input, Token>,
}

impl<'input, Token: Sana> Lexer<'input, Token> {
    /// Create a new `Lexer` on the given input
    ///
    /// **NOTE:** for better type inference it's prefered
    /// to use `Sana::lexer` instead
    pub fn new(input: &'input str) -> Self {
        let ir = Token::ir();
        let vm = Vm::new(ir, input);

        Lexer { vm }
    }

    /// Morth the lexer into another lexer, which scans a different token
    ///
    /// The cursor position of the new lexer is the same as the cursor
    /// position of the old lexer before the metamorphosis
    pub fn morph<Lexeme: Sana + 'static>(self) -> Lexer<'input, Lexeme> {
        let mut lexer = Lexeme::lexer(self.source());
        lexer.rewind(self.position());

        lexer
    }

    /// Set the cursor at position `pos`
    pub fn rewind(&mut self, pos: usize) {
        self.vm.cursor.rewind(pos)
    }

    /// The current position of the cursor
    pub fn position(&self) -> usize {
        self.vm.cursor.position()
    }

    /// The source string of the lexer
    pub fn source(&self) -> &'input str {
        self.vm.cursor.input
    }
}

/// A value (for example, token) together with its range
///
/// The range includes the start but excludes the end, similar to `start..end` ranges
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spanned<T> {
    pub start: usize,
    pub end: usize,
    pub value: T,
}

impl<'input, Token: Sana> Iterator for Lexer<'input, Token> {
    type Item = Spanned<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        use sana_core::ir::VmResult::*;

        let res = if Token::USES_VM {
            self.vm.run()
        } else {
            Token::lex(&mut self.vm.cursor)
        };

        let token = match res {
            Action { start, end, action } => Spanned {
                start,
                end,
                value: action,
            },
            Error { start, end } => Spanned {
                start,
                end,
                value: Token::ERROR,
            },
            Eoi => return None,
        };

        Some(token)
    }
}
