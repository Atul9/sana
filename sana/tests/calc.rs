use sana::{Sana, Spanned};

#[test]
fn calc_sin() {
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

    let input = "sin(3.14)";
    let mut lexer = Token::lexer(&input);

    let tok = lexer.next().unwrap();
    assert_eq!(
        tok,
        Spanned {
            value: Token::Ident,
            start: 0,
            end: 3
        }
    );

    let tok = lexer.next().unwrap();
    assert_eq!(
        tok,
        Spanned {
            value: Token::LParren,
            start: 3,
            end: 4
        }
    );

    let tok = lexer.next().unwrap();
    assert_eq!(
        tok,
        Spanned {
            value: Token::Float,
            start: 4,
            end: 8
        }
    );

    let tok = lexer.next().unwrap();
    assert_eq!(
        tok,
        Spanned {
            value: Token::RParren,
            start: 8,
            end: 9
        }
    );

    assert!(lexer.next().is_none());
}

#[test]
fn dot_vm() {
    #[derive(Debug, Clone, Copy, PartialEq, Sana)]
    #[backend(vm)]
    pub enum Token {
        #[regex(r"\.")]
        Dot,
        #[error]
        Error,
    }
    let input = ".";
    let mut lexer = Token::lexer(&input);

    let tok = lexer.next().unwrap();
    assert_eq!(
        tok,
        Spanned {
            value: Token::Dot,
            start: 0,
            end: 1
        }
    );

    assert!(lexer.next().is_none());
}

#[test]
fn dot_ct() {
    #[derive(Debug, Clone, Copy, PartialEq, Sana)]
    #[backend(rust)]
    pub enum Token {
        #[regex(r"\.")]
        Dot,
        #[error]
        Error,
    }
    let input = ".";
    let mut lexer = Token::lexer(&input);

    let tok = lexer.next().unwrap();
    assert_eq!(
        tok,
        Spanned {
            value: Token::Dot,
            start: 0,
            end: 1
        }
    );

    assert!(lexer.next().is_none());
}
