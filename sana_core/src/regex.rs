use regex_syntax::hir;

use std::{
    convert::TryFrom,
    hash::{Hash, Hasher},
};

use crate::automata::CharRange;
use std::ops::Not;

// Hashing is used for regular expression normalization
fn hash<T: Hash>(t: &T) -> u64 {
    let mut s = fnv::FnvHasher::default();
    t.hash(&mut s);
    s.finish()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Class(hir::ClassUnicode);

impl Class {
    fn full() -> Self {
        let range = hir::ClassUnicodeRange::new('\0', '\u{10ffff}');
        let class = Class(hir::ClassUnicode::new(vec![range]));

        assert_ne!(class.0.ranges().len(), 0);

        class
    }

    fn from_literal(literal: char) -> Self {
        let range = hir::ClassUnicodeRange::new(literal, literal);
        let class = Class(hir::ClassUnicode::new(vec![range]));

        assert_ne!(class.0.ranges().len(), 0);

        class
    }

    fn is_empty(&self) -> bool {
        self.0.ranges().is_empty()
    }

    fn contains(&self, ch: char) -> bool {
        for r in self.0.ranges() {
            if (r.start()..=r.end()).contains(&ch) {
                return true;
            }
        }

        false
    }

    pub fn pick(&self) -> char {
        self.0.ranges()[0].start()
    }

    pub fn ranges<'a>(&'a self) -> impl Iterator<Item = CharRange> + 'a {
        self.0
            .ranges()
            .iter()
            .map(|r| CharRange::new(r.start(), r.end()))
    }
}

#[allow(clippy::derive_hash_xor_eq)]
impl std::hash::Hash for Class {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        for r in self.0.ranges() {
            (r.start(), r.end()).hash(hasher)
        }
    }
}

/// A trait for regular expressions derivatives
///
/// This trait is the heart of regular expression processing
pub trait Derivative {
    /// Find a derivative of regular expression
    ///
    /// Let `input = 'c' ⋅ tail`. Then the derivative of `r` by `c`, written
    /// `c⁻¹ r` is a regular expression that matches `tail` iff `r` matches `input`
    fn derivative(&self, ch: char) -> Self;

    /// Return the derivative classes of a regular expression
    fn class_set(&self) -> ClassSet;
}

/// Regular expression with logical operations
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Regex {
    /// `∅`
    Nothing,
    /// Empty string
    Empty,
    /// A single character
    Literal(char),
    /// A class of characters
    Class(Class),
    /// A regular expression concatenation
    Concat(Vec<Regex>),
    /// Klenee star
    Loop(Box<Regex>),
    /// Logical or (alteration)
    Or(Vec<Regex>),
    /// Logical and (intersection)
    And(Vec<Regex>),
    /// Complement
    Not(Box<Regex>),
    /// `¬∅`
    Anything,
}

pub fn pprint_regex(regex: &Regex) {
    print_regex_indent(regex, 0)
}

fn print_regex_indent(regex: &Regex, indent: usize) {
    let ws: String = std::iter::repeat(' ').take(indent).collect();

    match regex {
        Regex::Nothing => println!("{}Nothing", ws),
        Regex::Empty => println!("{}Empty", ws),
        Regex::Literal(l) => println!("{}{}", ws, l),
        Regex::Class(class) => println!("{}{:?}", ws, class),
        Regex::Concat(list) => {
            println!("{}Concat", ws);

            for r in list {
                print_regex_indent(r, indent + 2)
            }
        }
        Regex::Loop(r) => {
            println!("{}Loop", ws);

            print_regex_indent(r, indent + 2)
        }
        Regex::Or(list) => {
            println!("{}Or", ws);

            for r in list {
                print_regex_indent(r, indent + 2)
            }
        }
        Regex::And(list) => {
            println!("{}And", ws);

            for r in list {
                print_regex_indent(r, indent + 2)
            }
        }
        Regex::Not(r) => {
            println!("{}Not", ws);

            print_regex_indent(r, indent + 2)
        }
        Regex::Anything => println!("{}Anything", ws),
    }
}

fn flatten_concat(list: &mut Vec<Regex>) {
    if list.iter().all(|e| matches!(*e, Regex::Concat(_)).not()) {
        return;
    }

    let mut new = vec![];

    for e in list.drain(..) {
        if let Regex::Concat(list) = e {
            new.extend(list)
        } else {
            new.push(e)
        }
    }

    *list = new
}

fn flatten_or(list: &mut Vec<Regex>) {
    if list.iter().all(|e| matches!(*e, Regex::Or(_)).not()) {
        return;
    }

    let mut new = vec![];

    for e in list.drain(..) {
        if let Regex::Or(list) = e {
            new.extend(list)
        } else {
            new.push(e)
        }
    }

    *list = new
}

fn flatten_and(list: &mut Vec<Regex>) {
    if list.iter().all(|e| matches!(*e, Regex::And(_)).not()) {
        return;
    }

    let mut new = vec![];

    for e in list.drain(..) {
        if let Regex::And(list) = e {
            new.extend(list)
        } else {
            new.push(e)
        }
    }

    *list = new
}

impl Regex {
    /// Create a regular expression that matches the given string
    pub fn literal_str(string: &str) -> Regex {
        if string.is_empty() {
            return Regex::Empty;
        }

        Regex::Concat(string.chars().map(Regex::Literal).collect())
    }

    /// Normalize the regular expression
    ///
    /// The purpose of normalization is to make equivalent expressions equal. This
    /// limits the number of all possible derivatives to a finite set, allowing
    /// DFA construction.
    ///
    /// The approach is rather straightforward:
    /// - The effective data structure for a monoid is a flat list
    /// - For a *commutative* monoid, it's a sorted flat list
    /// - Idempotence eliminates repetitions
    ///
    /// Since a particular order does not matter, lists are sorted by hash
    pub fn normalize(&mut self) {
        match self {
            Regex::Concat(list) => {
                for e in list.as_mut_slice() {
                    e.normalize()
                }

                flatten_concat(list);

                if list.iter().any(|e| *e == Regex::Nothing) {
                    return *self = Regex::Nothing;
                }

                list.retain(|e| *e != Regex::Empty);

                if list.len() == 1 {
                    return *self = list[0].clone();
                }
                if list.is_empty() {
                    return *self = Regex::Empty;
                }
            }
            Regex::Loop(e) => {
                e.normalize();

                match e.as_mut() {
                    Regex::Loop(inner) => *e = inner.clone(),
                    Regex::Nothing => *self = Regex::Empty,
                    Regex::Empty => *self = Regex::Empty,
                    _ => (),
                }
            }
            Regex::Or(list) => {
                for e in list.as_mut_slice() {
                    e.normalize()
                }

                flatten_or(list);

                if list.iter().any(|e| *e == Regex::Anything) {
                    return *self = Regex::Anything;
                }

                list.retain(|e| *e != Regex::Nothing);

                if list.len() == 1 {
                    return *self = list[0].clone();
                }
                if list.is_empty() {
                    return *self = Regex::Nothing;
                }

                list.sort_by(|l, r| hash(l).cmp(&hash(r)));
                list.dedup()
            }
            Regex::And(list) => {
                for e in list.as_mut_slice() {
                    e.normalize()
                }

                flatten_and(list);

                if list.iter().any(|e| *e == Regex::Nothing) {
                    return *self = Regex::Nothing;
                }

                list.retain(|e| *e != Regex::Anything);

                if list.len() == 1 {
                    return *self = list[0].clone();
                }
                if list.is_empty() {
                    return *self = Regex::Anything;
                }

                list.sort_by(|l, r| hash(l).cmp(&hash(r)));
                list.dedup()
            }
            Regex::Not(e) => {
                e.normalize();

                match e.as_mut() {
                    Regex::Not(inner) => *self = *inner.clone(),
                    Regex::Nothing => *self = Regex::Anything,
                    Regex::Anything => *self = Regex::Nothing,
                    _ => (),
                }
            }
            _ => (),
        }
    }

    /// Check if a regular expression is nullable
    ///
    /// A regular expression is *nullable* if it matches the empty string
    pub fn is_nullable(&self) -> bool {
        match self {
            Regex::Nothing => false,
            Regex::Empty => true,
            Regex::Literal(_) => false,
            Regex::Class(c) => c.is_empty(),
            Regex::Concat(list) => list.iter().all(|e| e.is_nullable()),
            Regex::Loop(_) => true,
            Regex::Or(list) => list.iter().any(|e| e.is_nullable()),
            Regex::And(list) => list.iter().all(|e| e.is_nullable()),
            Regex::Not(e) => e.is_nullable().not(),
            Regex::Anything => true,
        }
    }
}

impl Derivative for Regex {
    fn derivative(&self, ch: char) -> Regex {
        let mut res = match self {
            Regex::Nothing => Regex::Nothing,
            Regex::Empty => Regex::Nothing,
            Regex::Literal(c) => {
                if *c == ch {
                    Regex::Empty
                } else {
                    Regex::Nothing
                }
            }
            Regex::Class(c) => {
                if c.contains(ch) {
                    Regex::Empty
                } else {
                    Regex::Nothing
                }
            }
            Regex::Concat(list) => {
                let mut sum = vec![];

                for i in 0..list.len() {
                    let mut conc = vec![list[i].derivative(ch)];
                    conc.extend(list[i..].iter().skip(1).cloned());

                    sum.push(Regex::Concat(conc));

                    if list[i].is_nullable().not() || i == list.len() {
                        break;
                    }
                }

                Regex::Or(sum)
            }
            Regex::Loop(e) => Regex::Concat(vec![e.derivative(ch), Regex::Loop(e.clone())]),
            Regex::Or(list) => Regex::Or(list.iter().map(|e| e.derivative(ch)).collect()),
            Regex::And(list) => Regex::And(list.iter().map(|e| e.derivative(ch)).collect()),
            Regex::Not(e) => Regex::Not(Box::new(e.derivative(ch))),
            Regex::Anything => Regex::Anything,
        };

        res.normalize();

        res
    }

    fn class_set(&self) -> ClassSet {
        ClassSet::from_regex(self)
    }
}

impl TryFrom<hir::Hir> for Regex {
    type Error = &'static str;

    fn try_from(hir: hir::Hir) -> Result<Regex, Self::Error> {
        use hir::*;

        match hir.into_kind() {
            HirKind::Empty => Ok(Regex::Empty),
            HirKind::Concat(concat) => {
                let mut out = Vec::with_capacity(concat.len());

                fn extend(mir: Regex, out: &mut Vec<Regex>) {
                    match mir {
                        Regex::Concat(nested) => {
                            for child in nested {
                                extend(child, out);
                            }
                        }
                        mir => out.push(mir),
                    }
                }

                for hir in concat {
                    extend(Regex::try_from(hir)?, &mut out);
                }

                Ok(Regex::Concat(out))
            }
            HirKind::Alternation(alternation) => {
                let alternation = alternation
                    .into_iter()
                    .map(Regex::try_from)
                    .collect::<Result<_, _>>()?;

                Ok(Regex::Or(alternation))
            }
            HirKind::Literal(hir::Literal::Unicode(literal)) => Ok(Regex::Literal(literal)),
            HirKind::Literal(_) => Err("Only Unicode literals are supported"),
            HirKind::Class(hir::Class::Unicode(class)) => Ok(Regex::Class(Class(class))),
            HirKind::Class(_) => Err("Only Unicode classes are supported"),
            HirKind::Repetition(repetition) => {
                if repetition.greedy.not() {
                    return Err("#[regex]: non-greedy parsing is currently unsupported.");
                }

                let kind = repetition.kind;
                let mir = Regex::try_from(*repetition.hir)?;

                match kind {
                    RepetitionKind::ZeroOrOne => Ok(Regex::Or(vec![mir, Regex::Empty])),
                    RepetitionKind::ZeroOrMore => Ok(Regex::Loop(Box::new(mir))),
                    RepetitionKind::OneOrMore => {
                        Ok(Regex::Concat(vec![mir.clone(), Regex::Loop(Box::new(mir))]))
                    }
                    RepetitionKind::Range(..) => {
                        Err("#[regex]: {n,m} repetition range is currently unsupported.")
                    }
                }
            }
            HirKind::Group(group) => Regex::try_from(*group.hir),
            HirKind::WordBoundary(_) => Err("#[regex]: word boundaries are currently unsupported."),
            HirKind::Anchor(_) => Err("#[regex]: anchors in #[regex] are currently unsupported."),
        }
    }
}

fn collect_classes(classes: &mut Vec<Class>, regex: &Regex) {
    match regex {
        Regex::Nothing | Regex::Empty | Regex::Anything => (),
        Regex::Literal(literal) => classes.push(Class::from_literal(*literal)),
        Regex::Class(class) => classes.push(class.clone()),
        Regex::Concat(list) => {
            collect_classes(classes, &list[0]);

            for i in 1..list.len() {
                if list[i - 1].is_nullable().not() {
                    break;
                }

                collect_classes(classes, &list[i])
            }
        }
        Regex::Loop(e) => collect_classes(classes, e),
        Regex::Or(list) => {
            for e in list {
                collect_classes(classes, e)
            }
        }
        Regex::And(list) => {
            for e in list {
                collect_classes(classes, e)
            }
        }
        Regex::Not(e) => collect_classes(classes, e),
    };
}

/// Derivative class set
///
/// Two characters `a` and `b` belong to the same derivative class of
/// regular expression `r` iff `a⁻¹ r = b⁻¹ r`
///
/// A class set of a regular expression is a set of all derivative classes
/// of that expression
#[derive(Debug, Clone, PartialEq)]
pub struct ClassSet {
    set: Vec<Class>,
}

impl ClassSet {
    fn from_classes(classes: &[Class]) -> Self {
        let mut set: Vec<Class> = Vec::with_capacity(2 * classes.len());
        set.push(Class::full());

        for cl in classes {
            let len = set.len();
            for i in 0..len {
                set.push(set[i].clone())
            }

            for inner in &mut set[0..len] {
                inner.0.intersect(&cl.0)
            }
            for inner in &mut set[len..] {
                inner.0.difference(&cl.0)
            }

            if set.len() > 64 {
                set.sort_by_key(|l| hash(l));
                set.dedup()
            }
        }

        ClassSet { set }
    }

    pub fn from_regex(regex: &Regex) -> ClassSet {
        let mut classes = vec![];
        collect_classes(&mut classes, regex);

        Self::from_classes(&classes)
    }

    pub fn from_vector(vector: &RegexVector) -> ClassSet {
        let mut classes = vec![];
        for e in &vector.exprs {
            collect_classes(&mut classes, e)
        }

        Self::from_classes(&classes)
    }

    pub fn classes<'a>(&'a self) -> impl Iterator<Item = &'a Class> + 'a {
        self.set.iter().filter(|c| c.is_empty().not())
    }

    /// Pick a char from each class in the set
    pub fn pick_chars<'a>(&'a self) -> impl Iterator<Item = char> + 'a {
        self.set.iter().map(|s| s.pick())
    }
}

/// A regular expression vector
///
/// It behaves like alteration, but remembers the state of all regular expressions
/// allowing to asign an action that corresponds to a particular expression
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RegexVector {
    pub exprs: Vec<Regex>,
}

impl Derivative for RegexVector {
    fn derivative(&self, ch: char) -> Self {
        let exprs = self.exprs.iter().map(|e| e.derivative(ch)).collect();

        RegexVector { exprs }
    }

    fn class_set(&self) -> ClassSet {
        ClassSet::from_vector(self)
    }
}

impl RegexVector {
    /// Find the indices of all nullable expressions of the RegexVector
    pub fn nullables<'a>(&'a self) -> impl Iterator<Item = usize> + 'a {
        self.exprs
            .iter()
            .enumerate()
            .filter(|(_, e)| e.is_nullable())
            .map(|(i, _)| i)
    }
}
