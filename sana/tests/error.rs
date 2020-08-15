use sana::{Sana, Spanned};

#[test]
fn everything_is_an_error() {
    #[derive(Debug, Clone, Copy, PartialEq, Sana)]
    pub enum Token {
        #[error]
        Error,
    }
    let input = "asd";
    let mut lexer = Token::lexer(&input);

    let tok = lexer.next().unwrap();
    assert_eq!(
        tok,
        Spanned {
            value: Token::Error,
            start: 0,
            end: 0
        }
    );
}

#[test]
fn error_after_match() {
    #[derive(Debug, Clone, Copy, PartialEq, Sana)]
    pub enum Token {
        #[token("x")]
        X,
        #[error]
        Error,
    }
    let input = "xyz";
    let mut lexer = Token::lexer(&input);

    let tok = lexer.next().unwrap();
    assert_eq!(
        tok,
        Spanned {
            value: Token::X,
            start: 0,
            end: 1
        }
    );

    let tok = lexer.next().unwrap();
    assert_eq!(
        tok,
        Spanned {
            value: Token::Error,
            start: 1,
            end: 1
        }
    );

    let tok = lexer.next().unwrap(); // still an error
    assert_eq!(
        tok,
        Spanned {
            value: Token::Error,
            start: 1,
            end: 1
        }
    );
}
