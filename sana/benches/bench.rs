use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use sana::Sana;

#[derive(Debug, Clone, Copy, PartialEq, Sana)]
enum Token {
    #[regex(r"[ \n\t\f]+")]
    Whitespace,

    #[regex("[a-zA-Z_$][a-zA-Z0-9_$]*", priority = 1)]
    Identifier,

    #[regex(r#""([^"\\]|\\t|\\u|\\n|\\")*""#)]
    String,

    #[token("private", priority = 2)]
    Private,

    #[token("primitive", priority = 2)]
    Primitive,

    #[token("protected", priority = 2)]
    Protected,

    #[token("in", priority = 2)]
    In,

    #[token("instanceof", priority = 2)]
    Instanceof,

    #[token(".")]
    Accessor,

    #[token("...")]
    Ellipsis,

    #[token("(")]
    ParenOpen,

    #[token(")")]
    ParenClose,

    #[token("{")]
    BraceOpen,

    #[token("}")]
    BraceClose,

    #[token("+")]
    OpAddition,

    #[token("++")]
    OpIncrement,

    #[token("=")]
    OpAssign,

    #[token("==")]
    OpEquality,

    #[token("===")]
    OpStrictEquality,

    #[token("=>")]
    FatArrow,

    #[error]
    InvalidToken,
}

static SOURCE: &str = "
foobar(protected primitive private instanceof in) { + ++ = == === => }
foobar(protected primitive private instanceof in) { + ++ = == === => }
foobar(protected primitive private instanceof in) { + ++ = == === => }
foobar(protected primitive private instanceof in) { + ++ = == === => }
foobar(protected primitive private instanceof in) { + ++ = == === => }
foobar(protected primitive private instanceof in) { + ++ = == === => }
foobar(protected primitive private instanceof in) { + ++ = == === => }
foobar(protected primitive private instanceof in) { + ++ = == === => }
foobar(protected primitive private instanceof in) { + ++ = == === => }
foobar(protected primitive private instanceof in) { + ++ = == === => }
foobar(protected primitive private instanceof in) { + ++ = == === => }
foobar(protected primitive private instanceof in) { + ++ = == === => }
foobar(protected primitive private instanceof in) { + ++ = == === => }
foobar(protected primitive private instanceof in) { + ++ = == === => }
foobar(protected primitive private instanceof in) { + ++ = == === => }
foobar(protected primitive private instanceof in) { + ++ = == === => }
foobar(protected primitive private instanceof in) { + ++ = == === => }
foobar(protected primitive private instanceof in) { + ++ = == === => }
foobar(protected primitive private instanceof in) { + ++ = == === => }
foobar(protected primitive private instanceof in) { + ++ = == === => }
foobar(protected primitive private instanceof in) { + ++ = == === => }
foobar(protected primitive private instanceof in) { + ++ = == === => }
foobar(protected primitive private instanceof in) { + ++ = == === => }
foobar(protected primitive private instanceof in) { + ++ = == === => }
foobar(protected primitive private instanceof in) { + ++ = == === => }
foobar(protected primitive private instanceof in) { + ++ = == === => }
foobar(protected primitive private instanceof in) { + ++ = == === => }
foobar(protected primitive private instanceof in) { + ++ = == === => }
foobar(protected primitive private instanceof in) { + ++ = == === => }
foobar(protected primitive private instanceof in) { + ++ = == === => }
";

static IDENTIFIERS: &str = "It was the year when they finally immanentized the Eschaton \
                            It was the year when they finally immanentized the Eschaton \
                            It was the year when they finally immanentized the Eschaton \
                            It was the year when they finally immanentized the Eschaton \
                            It was the year when they finally immanentized the Eschaton \
                            It was the year when they finally immanentized the Eschaton \
                            It was the year when they finally immanentized the Eschaton \
                            It was the year when they finally immanentized the Eschaton \
                            It was the year when they finally immanentized the Eschaton \
                            It was the year when they finally immanentized the Eschaton \
                            It was the year when they finally immanentized the Eschaton \
                            It was the year when they finally immanentized the Eschaton \
                            It was the year when they finally immanentized the Eschaton";

static STRINGS: &str = r#""tree" "to" "a" "graph" "that can" "more adequately represent" "loops and arbitrary state jumps" "with\"\"\"out" "the\n\n\n\n\n" "expl\"\"\"osive" "nature\"""of trying to build up all possible permutations in a tree." "tree" "to" "a" "graph" "that can" "more adequately represent" "loops and arbitrary state jumps" "with\"\"\"out" "the\n\n\n\n\n" "expl\"\"\"osive" "nature\"""of trying to build up all possible permutations in a tree." "tree" "to" "a" "graph" "that can" "more adequately represent" "loops and arbitrary state jumps" "with\"\"\"out" "the\n\n\n\n\n" "expl\"\"\"osive" "nature\"""of trying to build up all possible permutations in a tree." "tree" "to" "a" "graph" "that can" "more adequately represent" "loops and arbitrary state jumps" "with\"\"\"out" "the\n\n\n\n\n" "expl\"\"\"osive" "nature\"""of trying to build up all possible permutations in a tree.""#;

fn identifiers(c: &mut Criterion) {
    let mut group = c.benchmark_group("identifiers");
    let size = IDENTIFIERS.len();
    group.throughput(Throughput::Bytes(size as u64));
    group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
        b.iter(|| {
            let lex = Token::lexer(IDENTIFIERS);
            black_box(lex.count());
        });
    });
}

fn keywords_operators_and_punctators(c: &mut Criterion) {
    let mut group = c.benchmark_group("keywords_operators_and_punctators");
    let size = SOURCE.len();
    group.throughput(Throughput::Bytes(size as u64));
    group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
        b.iter(|| {
            let lex = Token::lexer(SOURCE);
            black_box(lex.count());
        });
    });
}

fn strings(c: &mut Criterion) {
    let mut group = c.benchmark_group("strings");
    let size = STRINGS.len();
    group.throughput(Throughput::Bytes(size as u64));
    group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
        b.iter(|| {
            let lex = Token::lexer(STRINGS);
            black_box(lex.count());
        });
    });
}

criterion_group!(
    benches,
    identifiers,
    keywords_operators_and_punctators,
    strings
);
criterion_main!(benches);
