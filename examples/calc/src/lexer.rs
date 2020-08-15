use sana::{Sana, Spanned};

type ParserItem = Result<(usize, Token, usize), crate::parser::ParserError>;

pub fn token_iter<'input>(input: &'input str) -> impl Iterator<Item = ParserItem> + 'input {
    Token::lexer(input)
        .filter(|tok| tok.value != Token::Whitespace)
        .map(|tok| match tok {
            Spanned {
                value: Token::Error,
                ..
            } => Err(crate::parser::ParserError),
            Spanned { value, start, end } => Ok((start, value, end)),
        })
}

#[derive(Debug, Clone, Copy, PartialEq, Sana)]
pub enum Token {
    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Ident,
    #[regex("[0-9]+", priority = 1)]
    Integer,
    #[regex(r"[0-9]+(\.[0-9]+)?([eE][+-]\d+)?")]
    Float,

    #[token("(")]
    LParren,
    #[token(")")]
    RParren,
    #[token(",")]
    Comma,

    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Mul,
    #[token("/")]
    Div,

    #[regex("[ \t\r\n]+")]
    Whitespace,

    #[error]
    Error,
}
