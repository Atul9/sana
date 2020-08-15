use sana::{Sana, Spanned};

#[test]
fn err_first_token() {
    #[derive(Clone, Copy, Sana, PartialEq, Debug)]
    enum Token {
        #[error]
        Error,
        #[token("x")]
        #[token("y")]
        Xy,
        #[token(" ")]
        Space,
    }

    let input = "x y";
    let mut lexer = Token::lexer(&input);

    let tok = lexer.next().unwrap();
    assert_eq!(
        tok,
        Spanned {
            value: Token::Xy,
            start: 0,
            end: 1
        }
    );

    let tok = lexer.next().unwrap();
    assert_eq!(
        tok,
        Spanned {
            value: Token::Space,
            start: 1,
            end: 2
        }
    );

    let tok = lexer.next().unwrap();
    assert_eq!(
        tok,
        Spanned {
            value: Token::Xy,
            start: 2,
            end: 3
        }
    );

    assert!(lexer.next().is_none());
}
