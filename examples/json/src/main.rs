use sana::Sana;

use std::io::Read;

#[derive(Debug, Clone, Copy, PartialEq, Sana)]
enum Token {
    #[regex("\"" . r#"([^"\\]|\\["\\/bfnrt]|\\u\d\d\d\d)*"# . "\"")]
    String,
    #[regex(r"-?(0|[1-9]\d*)(\.\d+)?([eE][+-]?\d+)?")]
    Number,

    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("null")]
    Null,

    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,

    #[token(",")]
    Comma,
    #[token(":")]
    Colon,

    #[regex("[\r\n\t ]+")]
    Whitespace,

    #[error]
    Error,
}

fn main() {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();

    let lexer = Token::lexer(&input);

    for tok in lexer {
        println!("{:?} at {}..{}", tok.value, tok.start, tok.end);

        if tok.value == Token::Error {
            break;
        }
    }
}
