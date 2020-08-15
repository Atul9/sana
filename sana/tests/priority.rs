use sana::{Sana, Spanned};

#[test]
fn priority_first() {
    // the first regex has the highest priority
    #[derive(Clone, Copy, Sana, PartialEq, Debug)]
    enum AmbigToken {
        #[regex(".+", priority = 1)]
        A,
        #[regex(".+")] // default priority = 0
        #[allow(dead_code)]
        B, // this one should not be constructed

        #[error]
        Error,
    }

    let input = "x";
    let mut lexer = AmbigToken::lexer(&input);

    let tok = lexer.next().unwrap();
    assert_eq!(
        tok,
        Spanned {
            value: AmbigToken::A,
            start: 0,
            end: 1
        }
    );

    assert!(lexer.next().is_none());
}

#[test]
fn priority_last() {
    // the last regex has the highest priority
    #[derive(Clone, Copy, Sana, PartialEq, Debug)]
    enum AmbigToken {
        #[regex(".+")] // default priority = 0
        #[allow(dead_code)]
        A, // this one should not be constructed
        #[regex(".+", priority = 1)]
        B,

        #[error]
        Error,
    }

    let input = "x";
    let mut lexer = AmbigToken::lexer(&input);

    let tok = lexer.next().unwrap();
    assert_eq!(
        tok,
        Spanned {
            value: AmbigToken::B,
            start: 0,
            end: 1
        }
    );

    assert!(lexer.next().is_none());
}
