use sana::{Sana, Spanned};

#[test]
fn or() {
    #[derive(Clone, Copy, Sana, PartialEq, Debug)]
    enum Token {
        #[regex("[[:punct:]]+" | "\\d+")]
        PunctOrDigit,
        #[token(" ")]
        Space,

        #[error]
        Error,
    }

    let input = "; 1,23 .";
    let mut lexer = Token::lexer(&input);

    let tok = lexer.next().unwrap();
    assert_eq!(
        tok,
        Spanned {
            value: Token::PunctOrDigit,
            start: 0,
            end: 1
        }
    ); // ;

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
            value: Token::PunctOrDigit,
            start: 2,
            end: 3
        }
    ); // 1

    let tok = lexer.next().unwrap();
    assert_eq!(
        tok,
        Spanned {
            value: Token::PunctOrDigit,
            start: 3,
            end: 4
        }
    ); // ,

    let tok = lexer.next().unwrap();
    assert_eq!(
        tok,
        Spanned {
            value: Token::PunctOrDigit,
            start: 4,
            end: 6
        }
    ); // 23

    let tok = lexer.next().unwrap();
    assert_eq!(
        tok,
        Spanned {
            value: Token::Space,
            start: 6,
            end: 7
        }
    );

    let tok = lexer.next().unwrap();
    assert_eq!(
        tok,
        Spanned {
            value: Token::PunctOrDigit,
            start: 7,
            end: 8
        }
    ); // .

    assert!(lexer.next().is_none());
}

#[test]
fn or_attribute() {
    #[derive(Clone, Copy, Sana, PartialEq, Debug)]
    enum Token {
        #[regex("A")]
        #[regex("B")]
        AorB,
        #[token(" ")]
        Space,

        #[error]
        Error,
    }

    let input = "A B";
    let mut lexer = Token::lexer(&input);

    let tok = lexer.next().unwrap();
    assert_eq!(
        tok,
        Spanned {
            value: Token::AorB,
            start: 0,
            end: 1
        }
    ); // A

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
            value: Token::AorB,
            start: 2,
            end: 3
        }
    ); // B

    assert!(lexer.next().is_none());
}

#[test]
fn and_nonintersect_rules() {
    #[derive(Clone, Copy, Sana, PartialEq, Debug)]
    enum Token {
        #[regex("[[:punct:]]+" & "\\d+")] // does not make any sense
        #[allow(dead_code)]
        PunctAndDigit,

        #[error]
        Error,
    }

    let input = ";";
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
fn and_intersect_rules() {
    #[derive(Clone, Copy, Sana, PartialEq, Debug)]
    enum Token {
        #[regex("[123]+" & "\\d+")]
        Digits,

        #[token(" ")]
        Space,

        #[error]
        Error,
    }

    let input = "32 1";
    let mut lexer = Token::lexer(&input);

    let tok = lexer.next().unwrap();
    assert_eq!(
        tok,
        Spanned {
            value: Token::Digits,
            start: 0,
            end: 2
        }
    );

    let tok = lexer.next().unwrap();
    assert_eq!(
        tok,
        Spanned {
            value: Token::Space,
            start: 2,
            end: 3
        }
    );

    let tok = lexer.next().unwrap();
    assert_eq!(
        tok,
        Spanned {
            value: Token::Digits,
            start: 3,
            end: 4
        }
    );

    assert!(lexer.next().is_none());
}

#[test]
fn and_not() {
    #[derive(Clone, Copy, Sana, PartialEq, Debug)]
    enum Token {
        #[regex(r"/\*", priority = 616)]
        CommentStart,
        #[regex(r"[~!@#\^\&|`?+\-*/%<>=]+" & !r"/\*.*")]
        Op,

        #[error]
        Error,
    }

    let input = "/**/";
    let mut lexer = Token::lexer(&input);

    let tok = lexer.next().unwrap();
    assert_eq!(
        tok,
        Spanned {
            value: Token::CommentStart,
            start: 0,
            end: 2
        }
    );

    let tok = lexer.next().unwrap();
    assert_eq!(
        tok,
        Spanned {
            value: Token::Op,
            start: 2,
            end: 4
        }
    );

    assert!(lexer.next().is_none());
}

#[test]
fn concat() {
    #[derive(Clone, Copy, Sana, PartialEq, Debug)]
    enum Token {
        #[regex("[[:punct:]]+" . "\\d+")]
        PunctThenDigit,
        #[token(" ")]
        Space,

        #[error]
        Error,
    }

    let input = ",23 ";
    let mut lexer = Token::lexer(&input);

    let tok = lexer.next().unwrap();
    assert_eq!(
        tok,
        Spanned {
            value: Token::PunctThenDigit,
            start: 0,
            end: 3
        }
    );

    let tok = lexer.next().unwrap();
    assert_eq!(
        tok,
        Spanned {
            value: Token::Space,
            start: 3,
            end: 4
        }
    );

    assert!(lexer.next().is_none());
}
