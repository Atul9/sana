mod lexer;
mod parser;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Call { name: String, args: Vec<Expr> },

    UMinus(Box<Expr>),
    Plus { left: Box<Expr>, right: Box<Expr> },
    Minus { left: Box<Expr>, right: Box<Expr> },
    Mul { left: Box<Expr>, right: Box<Expr> },
    Div { left: Box<Expr>, right: Box<Expr> },

    Number(Number),
}

impl Expr {
    fn eval(&self) -> Result<Number, ()> {
        let res = match self {
            Expr::Call { name, args } => match name.as_str() {
                "sin" => eval_sine(&args)?,
                "cos" => eval_cosine(&args)?,
                _ => return Err(()),
            },
            Expr::UMinus(e) => e.eval()?.negate(),
            Expr::Plus { left, right } => {
                left.eval()?
                    .join_with(right.eval()?, |x, y| x + y, |x, y| x + y)
            }
            Expr::Minus { left, right } => {
                left.eval()?
                    .join_with(right.eval()?, |x, y| x - y, |x, y| x - y)
            }
            Expr::Mul { left, right } => {
                left.eval()?
                    .join_with(right.eval()?, |x, y| x * y, |x, y| x * y)
            }
            Expr::Div { left, right } => {
                Number::Float(left.eval()?.to_float() / right.eval()?.to_float())
            }
            Expr::Number(num) => *num,
        };

        Ok(res)
    }
}

fn eval_sine(exprs: &[Expr]) -> Result<Number, ()> {
    if exprs.len() != 1 {
        return Err(());
    }

    Ok(Number::Float(exprs[0].eval()?.to_float().sin()))
}

fn eval_cosine(exprs: &[Expr]) -> Result<Number, ()> {
    if exprs.len() != 1 {
        return Err(());
    }

    Ok(Number::Float(exprs[0].eval()?.to_float().cos()))
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Number {
    Integer(i64),
    Float(f64),
}

impl Number {
    fn negate(self) -> Self {
        match self {
            Number::Integer(i) => Number::Integer(-i),
            Number::Float(f) => Number::Float(-f),
        }
    }

    fn join_with<I, F>(self, other: Number, ifun: I, ffun: F) -> Self
    where
        I: FnOnce(i64, i64) -> i64,
        F: FnOnce(f64, f64) -> f64,
    {
        use Number::*;

        match (self, other) {
            (Integer(ix), Integer(iy)) => Integer(ifun(ix, iy)),
            (Integer(i), Float(f)) => Float(ffun(i as f64, f)),
            (Float(f), Integer(i)) => Float(ffun(f, i as f64)),
            (Float(fx), Float(fy)) => Float(ffun(fx, fy)),
        }
    }

    fn to_float(self) -> f64 {
        match self {
            Number::Integer(i) => i as f64,
            Number::Float(f) => f,
        }
    }
}

impl std::fmt::Display for Number {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Number::Integer(i) => i.fmt(fmt),
            Number::Float(f) => f.fmt(fmt),
        }
    }
}

fn parse_expr(input: &str) -> Result<Expr, ()> {
    let lexer = lexer::token_iter(input);

    parser::ExprParser::new()
        .parse(input, lexer)
        .map_err(|_| ())
}

fn main() {
    use rustyline::error::ReadlineError;
    use rustyline::Editor;

    let mut rl = Editor::<()>::new();

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());

                let res = parse_expr(line.as_str()).and_then(|expr| expr.eval());

                match res {
                    Ok(num) => println!("{}", num),
                    Err(()) => println!("Invalid expression"),
                }
            }
            Err(ReadlineError::Interrupted) => break,
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
}
