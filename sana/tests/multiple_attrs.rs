use sana::{Sana, Spanned};

#[test]
fn multiple_attrs() {
    #[derive(Clone, Copy, Sana, PartialEq, Debug)]
    enum Token {
        #[token("x")]
        #[token("y")]
        Xy,
        #[token(" ")]
        Space,

        #[error]
        Error,
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
