use dot::{Edges, GraphWalk, Id, LabelText, Labeller, Nodes};

use crate::automata::{Automata, CharRange, State};
use std::borrow::Cow;

type Edge = (usize, CharRange, usize);

impl<'a, T> GraphWalk<'a, usize, Edge> for Automata<T> {
    fn nodes(&'a self) -> Nodes<'a, usize> {
        (0..self.states.len()).collect()
    }

    fn edges(&'a self) -> Edges<'a, Edge> {
        let vec = self
            .edges
            .iter()
            .map(|(&k, &v)| (k.0, k.1, v))
            .collect::<Vec<_>>();

        Cow::Owned(vec)
    }

    fn source(&'a self, edge: &Edge) -> usize {
        edge.0
    }

    fn target(&'a self, edge: &Edge) -> usize {
        edge.2
    }
}

impl<'a, T> Labeller<'a, usize, Edge> for Automata<T> {
    fn graph_id(&'a self) -> Id<'a> {
        Id::new("dfa").unwrap()
    }

    fn node_id(&'a self, n: &usize) -> Id<'a> {
        Id::new(format!("s{}", n)).unwrap()
    }

    fn edge_label(&'a self, e: &Edge) -> LabelText<'a> {
        let class = e.1;

        let label = match (class.start, class.end) {
            ('\0', '\u{10ffff}') => "*".to_string(),
            (a, b) if a == b => format!("{}", a),
            (a, b) => format!("{}..{}", a, b),
        };

        LabelText::label(label)
    }

    fn node_shape(&'a self, node: &usize) -> Option<LabelText<'a>> {
        match self.get(*node) {
            Some(State::Normal) => Some(LabelText::label("circle")),
            Some(State::Action(_)) => Some(LabelText::label("doublecircle")),
            _ => None,
        }
    }
}
