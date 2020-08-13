use sana_core::{Rule, RuleSet};
use sana_core::regex::Regex;
use sana_core::ir::{Ir, Vm, VmResult};

use std::convert::TryFrom;

fn compile(rules: &[(&str, &'static str, usize)]) -> Ir<&'static str> {
    let rules: Vec<_> = rules.iter()
        .map(|(regex, act, prio)|  {
            let hir = regex_syntax::Parser::new()
                .parse(regex).unwrap();
            let regex = Regex::try_from(hir).unwrap();

            Rule {
                regex,
                priority: *prio,
                action: *act
            }
        })
        .collect();

    let ruleset = RuleSet { rules };
    let dfa = ruleset.construct_dfa().unwrap();

    Ir::from_automata(dfa)
}

#[test]
fn doc_example() {
    let ir = compile(doc_tokens());
    let ir = ir.flatten();

    let input = "let answer = 42;";
    let mut vm = Vm::new(&ir, input);

    let gold = &[
        (3, Some("Let")),
        (1, Some("Whitespace")),
        (6, Some("Ident")),
        (1, Some("Whitespace")),
        (1, Some("Equals")),
        (1, Some("Whitespace")),
        (2, Some("Integer")),
        (1, Some("Semicolon")),
        (0, None),
    ];

    let mut pos = 0;
    for &g in gold {
        let res = vm.run();
        let gold =
            if let Some(act) = g.1 {
                let start = pos;
                pos = start + g.0;

                VmResult::Action {
                    start,
                    end: pos,
                    action: act
                }
            }
            else {
                VmResult::Eoi
            };

        assert_eq!(gold, res);
    }
}

pub fn doc_tokens() -> &'static [(&'static str, &'static str, usize)] {
    &[
        ("[a-zA-Z_][a-zA-Z0-9_]*", "Ident", 0),
        ("[0-9]+", "Integer", 0),

        ("let", "Let", 1),
        ("=", "Equals", 0),
        (";", "Semicolon", 0),

        ("[ \t\r\n]+", "Whitespace", 0),
    ]
}
