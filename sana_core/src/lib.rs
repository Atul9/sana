//! This is the core of Sana.
//!
//! Specifically, this crate provides the following:
//!
//! - Extended regular expression derivatives
//! - DFA construction for a rule set
//! - IR generation from a DFA
//!
//! If you just want generate a lexer, use the main crate (`sana`) instead.

use automata::{Automata, State};
use regex::{Derivative, Regex, RegexVector};
use std::collections::{HashMap, VecDeque};

pub mod automata;
#[cfg(feature = "automata_dot")]
pub mod dot;
pub mod ir;
pub mod regex;

/// DFA construction error
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// Ambiguity error for rules with given indices
    ///
    /// The rules can match the same string but have the same precedence
    AmbiguityError(usize, usize),
}

/// A lexer rule
///
/// It usually corresponds to a token
#[derive(Debug, Clone, PartialEq)]
pub struct Rule<T> {
    pub regex: Regex,
    pub priority: usize,
    pub action: T,
}

impl<T: Clone> Rule<T> {
    /// Construct DFA using regular expression derivatives
    pub fn construct_dfa(&self) -> Automata<T> {
        let state = if self.regex.is_nullable() {
            State::Action(self.action.clone())
        } else {
            State::Normal
        };

        let mut automata = Automata::new(state);
        let mut queue = VecDeque::<Regex>::new();
        let mut stored = HashMap::<Regex, usize>::new();

        queue.push_back(self.regex.clone());
        stored.insert(self.regex.clone(), 0);

        while let Some(r) = queue.pop_front() {
            let from = *stored.get(&r).unwrap();
            let set = r.class_set();

            for class in set.classes() {
                let dr = r.derivative(class.pick());
                let to = if let Some(&i) = stored.get(&dr) {
                    i
                } else {
                    let i = stored.len();
                    let state = if dr.is_nullable() {
                        State::Action(self.action.clone())
                    } else {
                        State::Normal
                    };

                    queue.push_back(dr.clone());
                    stored.insert(dr, i);

                    automata.insert_state(state);

                    i
                };

                for range in class.ranges() {
                    automata.insert_edge(from, to, range)
                }
            }
        }

        automata
    }
}

/// A rule set is just a vector of rules
#[derive(Debug, Clone, PartialEq)]
pub struct RuleSet<T> {
    pub rules: Vec<Rule<T>>,
}

impl<T: Clone> RuleSet<T> {
    /// From all rules with index i ∈ rule_indices, return the rule
    /// with the higherst priority
    ///
    /// If there are more than one such rules, return the ambiguity error
    fn top_rule<I>(&self, mut rule_indices: I) -> Result<Option<&Rule<T>>, Error>
    where
        I: Iterator<Item = usize>,
    {
        let ix = if let Some(ix) = rule_indices.next() {
            ix
        } else {
            return Ok(None);
        };

        let (mut top_ix, mut top_prio) = (ix, self.rules[ix].priority);
        for i in rule_indices {
            use std::cmp::Ordering::*;

            let prio = self.rules[i].priority;
            match prio.cmp(&top_prio) {
                Less => (),
                Equal => return Err(Error::AmbiguityError(top_ix, i)),
                Greater => {
                    top_ix = i;
                    top_prio = prio
                }
            }
        }

        Ok(Some(&self.rules[top_ix]))
    }

    /// Construct a DFA from a rule set
    ///
    /// In the resulting DFA, the action of each action state is set to
    /// the action of the rule with the highest priority.
    ///
    /// If there's more than one rule with the same priority that matches
    /// the same input, then an ambiguity error is returned
    pub fn construct_dfa(&self) -> Result<Automata<T>, Error> {
        let vector = RegexVector {
            exprs: self.rules.iter().map(|r| r.regex.clone()).collect(),
        };
        let rule = self.top_rule(vector.nullables())?;
        let state = match rule {
            Some(rule) => State::Action(rule.action.clone()),
            _ => State::Normal,
        };

        let mut automata = Automata::new(state);
        let mut queue = VecDeque::new();
        let mut stored = HashMap::<_, usize>::new();

        queue.push_back(vector.clone());
        stored.insert(vector, 0);

        let mut current_ranges = vec![];

        while let Some(vec) = queue.pop_front() {
            let from = *stored.get(&vec).unwrap();
            let set = vec.class_set();

            for class in set.classes() {
                let dvec = vec.derivative(class.pick());
                let to = if let Some(&i) = stored.get(&dvec) {
                    i
                } else {
                    let i = stored.len();
                    let state = match self.top_rule(dvec.nullables())? {
                        Some(rule) => State::Action(rule.action.clone()),
                        _ => State::Normal,
                    };

                    queue.push_back(dvec.clone());
                    stored.insert(dvec, i);

                    automata.insert_state(state);

                    i
                };

                current_ranges.extend(class.ranges().map(|r| (from, to, r)));
            }

            current_ranges.sort();

            let mut current = None;
            for (from, to, range) in current_ranges.drain(..) {
                match &mut current {
                    Some((f, t, r)) => match range.concat(*r) {
                        Some(conc) if *f == from && *t == to => *r = conc,
                        _ => {
                            automata.insert_edge(*f, *t, *r);
                            current = Some((from, to, range))
                        }
                    },
                    _ => current = Some((from, to, range)),
                }
            }
            if let Some((from, to, range)) = current {
                automata.insert_edge(from, to, range)
            }
        }

        Ok(automata)
    }
}
