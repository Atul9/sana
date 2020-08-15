use sana_core::automata::{Automata, State};
use sana_core::regex::Regex;
use sana_core::Rule;

use std::convert::TryFrom;

#[test]
fn basic() {
    for test in regexps().into_iter() {
        let hir = regex_syntax::Parser::new().parse(test.0).unwrap();

        let regex = if let Ok(r) = Regex::try_from(hir) {
            r
        } else {
            continue;
        };

        let dfa = Rule {
            regex,
            priority: 0,
            action: 0,
        }
        .construct_dfa();

        if let Some(span) = test.2[0] {
            let input = if let Some(i) = test.1.get(span.0..span.1) {
                i
            } else {
                continue;
            };

            print!("{} at {}: ", test.0, input);

            let status = if dfa_match(&dfa, input) {
                "PASS"
            } else {
                panic!()
            };

            println!("{}", status)
        }
    }
}

fn dfa_match<T>(dfa: &Automata<T>, input: &str) -> bool {
    let mut state_ix = 0;
    for ch in input.chars() {
        if let Some(s) = dfa.transite(state_ix, ch) {
            state_ix = s
        } else {
            return false;
        };
    }

    matches!(dfa.get(state_ix), Some(&State::Action(_)))
}

pub fn regexps() -> Vec<(&'static str, &'static str, Vec<Option<(usize, usize)>>)> {
    vec![
        (r"abracadabra$", r"abracadabracadabra", vec![Some((7, 18))]),
        (r"a...b", r"abababbb", vec![Some((2, 7))]),
        (r"XXXXXX", r"..XXXXXX", vec![Some((2, 8))]),
        (r"\)", r"()", vec![Some((1, 2))]),
        (r"a]", r"a]a", vec![Some((0, 2))]),
        (r"\}", r"}", vec![Some((0, 1))]),
        (r"\]", r"]", vec![Some((0, 1))]),
        (r"]", r"]", vec![Some((0, 1))]),
        (r"^a", r"ax", vec![Some((0, 1))]),
        (r"\^a", r"a^a", vec![Some((1, 3))]),
        (r"a\^", r"a^", vec![Some((0, 2))]),
        (r"a$", r"aa", vec![Some((1, 2))]),
        (r"a\$", r"a$", vec![Some((0, 2))]),
        (r"^$", r"", vec![Some((0, 0))]),
        (r"$^", r"", vec![Some((0, 0))]),
        (r"a($)", r"aa", vec![Some((1, 2)), Some((2, 2))]),
        (r"a*(^a)", r"aa", vec![Some((0, 1)), Some((0, 1))]),
        (r"(..)*(...)*", r"a", vec![Some((0, 0))]),
        (r"(..)*(...)*", r"abcd", vec![Some((0, 4)), Some((2, 4))]),
        (
            r"(ab|a)(bc|c)",
            r"abc",
            vec![Some((0, 3)), Some((0, 2)), Some((2, 3))],
        ),
        (r"(ab)c|abc", r"abc", vec![Some((0, 3)), Some((0, 2))]),
        (r"a{0}b", r"ab", vec![Some((1, 2))]),
        (
            r"(a*)(b?)(b+)b{3}",
            r"aaabbbbbbb",
            vec![Some((0, 10)), Some((0, 3)), Some((3, 4)), Some((4, 7))],
        ),
        (
            r"(a*)(b{0,1})(b{1,})b{3}",
            r"aaabbbbbbb",
            vec![Some((0, 10)), Some((0, 3)), Some((3, 4)), Some((4, 7))],
        ),
        (
            r"((a|a)|a)",
            r"a",
            vec![Some((0, 1)), Some((0, 1)), Some((0, 1))],
        ),
        (
            r"(a*)(a|aa)",
            r"aaaa",
            vec![Some((0, 4)), Some((0, 3)), Some((3, 4))],
        ),
        (r"a*(a.|aa)", r"aaaa", vec![Some((0, 4)), Some((2, 4))]),
        (
            r"a(b)|c(d)|a(e)f",
            r"aef",
            vec![Some((0, 3)), None, None, Some((1, 2))],
        ),
        (r"(a|b)?.*", r"b", vec![Some((0, 1)), Some((0, 1))]),
        (r"(a|b)c|a(b|c)", r"ac", vec![Some((0, 2)), Some((0, 1))]),
        (
            r"(a|b)c|a(b|c)",
            r"ab",
            vec![Some((0, 2)), None, Some((1, 2))],
        ),
        (
            r"(a|b)*c|(a|ab)*c",
            r"abc",
            vec![Some((0, 3)), Some((1, 2))],
        ),
        (r"(a|b)*c|(a|ab)*c", r"xc", vec![Some((1, 2))]),
        (
            r"(.a|.b).*|.*(.a|.b)",
            r"xa",
            vec![Some((0, 2)), Some((0, 2))],
        ),
        (r"a?(ab|ba)ab", r"abab", vec![Some((0, 4)), Some((0, 2))]),
        (
            r"a?(ac{0}b|ba)ab",
            r"abab",
            vec![Some((0, 4)), Some((0, 2))],
        ),
        (r"ab|abab", r"abbabab", vec![Some((0, 2))]),
        (r"aba|bab|bba", r"baaabbbaba", vec![Some((5, 8))]),
        (r"aba|bab", r"baaabbbaba", vec![Some((6, 9))]),
        (
            r"(aa|aaa)*|(a|aaaaa)",
            r"aa",
            vec![Some((0, 2)), Some((0, 2))],
        ),
        (
            r"(a.|.a.)*|(a|.a...)",
            r"aa",
            vec![Some((0, 2)), Some((0, 2))],
        ),
        (r"ab|a", r"xabc", vec![Some((1, 3))]),
        (r"ab|a", r"xxabc", vec![Some((2, 4))]),
        (
            r"(?i)(?-u)(Ab|cD)*",
            r"aBcD",
            vec![Some((0, 4)), Some((2, 4))],
        ),
        (r"[^-]", r"--a", vec![Some((2, 3))]),
        (r"[a-]*", r"--a", vec![Some((0, 3))]),
        (r"[a-m-]*", r"--amoma--", vec![Some((0, 4))]),
        (
            r":::1:::0:|:::1:1:0:",
            r":::0:::1:::1:::0:",
            vec![Some((8, 17))],
        ),
        (
            r":::1:::0:|:::1:1:1:",
            r":::0:::1:::1:::0:",
            vec![Some((8, 17))],
        ),
        (r"[[:upper:]]", r"A", vec![Some((0, 1))]),
        (r"[[:lower:]]+", r"`az{", vec![Some((1, 3))]),
        (r"[[:upper:]]+", r"@AZ[", vec![Some((1, 3))]),
        ("\n", "\n", vec![Some((0, 1))]),
        ("\n", "\n", vec![Some((0, 1))]),
        (r"[^a]", "\n", vec![Some((0, 1))]),
        ("\na", "\na", vec![Some((0, 2))]),
        (
            r"(a)(b)(c)",
            r"abc",
            vec![Some((0, 3)), Some((0, 1)), Some((1, 2)), Some((2, 3))],
        ),
        (r"xxx", r"xxx", vec![Some((0, 3))]),
        (
            r"(^|[ (,;])((([Ff]eb[^ ]* *|0*2/|\* */?)0*[6-7]))([^0-9]|$)",
            r"feb 6,",
            vec![Some((0, 6))],
        ),
        (
            r"(^|[ (,;])((([Ff]eb[^ ]* *|0*2/|\* */?)0*[6-7]))([^0-9]|$)",
            r"2/7",
            vec![Some((0, 3))],
        ),
        (
            r"(^|[ (,;])((([Ff]eb[^ ]* *|0*2/|\* */?)0*[6-7]))([^0-9]|$)",
            r"feb 1,Feb 6",
            vec![Some((5, 11))],
        ),
        (
            r"((((((((((((((((((((((((((((((x))))))))))))))))))))))))))))))",
            r"x",
            vec![Some((0, 1)), Some((0, 1)), Some((0, 1))],
        ),
        (
            r"((((((((((((((((((((((((((((((x))))))))))))))))))))))))))))))*",
            r"xx",
            vec![Some((0, 2)), Some((1, 2)), Some((1, 2))],
        ),
        (
            r"a?(ab|ba)*",
            r"ababababababababababababababababababababababababababababababababababababababababa",
            vec![Some((0, 81)), Some((79, 81))],
        ),
        (
            r"abaa|abbaa|abbbaa|abbbbaa",
            r"ababbabbbabbbabbbbabbbbaa",
            vec![Some((18, 25))],
        ),
        (
            r"abaa|abbaa|abbbaa|abbbbaa",
            r"ababbabbbabbbabbbbabaa",
            vec![Some((18, 22))],
        ),
        (
            r"aaac|aabc|abac|abbc|baac|babc|bbac|bbbc",
            r"baaabbbabac",
            vec![Some((7, 11))],
        ),
        (r".*", r"", vec![Some((0, 2))]),
        (
            r"aaaa|bbbb|cccc|ddddd|eeeeee|fffffff|gggg|hhhh|iiiii|jjjjj|kkkkk|llll",
            r"XaaaXbbbXcccXdddXeeeXfffXgggXhhhXiiiXjjjXkkkXlllXcbaXaaaa",
            vec![Some((53, 57))],
        ),
        (r"a*a*a*a*a*b", r"aaaaaaaaab", vec![Some((0, 10))]),
        (r"^", r"", vec![Some((0, 0))]),
        (r"$", r"", vec![Some((0, 0))]),
        (r"^$", r"", vec![Some((0, 0))]),
        (r"^a$", r"a", vec![Some((0, 1))]),
        (r"abc", r"abc", vec![Some((0, 3))]),
        (r"abc", r"xabcy", vec![Some((1, 4))]),
        (r"abc", r"ababc", vec![Some((2, 5))]),
        (r"ab*c", r"abc", vec![Some((0, 3))]),
        (r"ab*bc", r"abc", vec![Some((0, 3))]),
        (r"ab*bc", r"abbc", vec![Some((0, 4))]),
        (r"ab*bc", r"abbbbc", vec![Some((0, 6))]),
        (r"ab+bc", r"abbc", vec![Some((0, 4))]),
        (r"ab+bc", r"abbbbc", vec![Some((0, 6))]),
        (r"ab?bc", r"abbc", vec![Some((0, 4))]),
        (r"ab?bc", r"abc", vec![Some((0, 3))]),
        (r"ab?c", r"abc", vec![Some((0, 3))]),
        (r"^abc$", r"abc", vec![Some((0, 3))]),
        (r"^abc", r"abcc", vec![Some((0, 3))]),
        (r"abc$", r"aabc", vec![Some((1, 4))]),
        (r"^", r"abc", vec![Some((0, 0))]),
        (r"$", r"abc", vec![Some((3, 3))]),
        (r"a.c", r"abc", vec![Some((0, 3))]),
        (r"a.c", r"axc", vec![Some((0, 3))]),
        (r"a.*c", r"axyzc", vec![Some((0, 5))]),
        (r"a[bc]d", r"abd", vec![Some((0, 3))]),
        (r"a[b-d]e", r"ace", vec![Some((0, 3))]),
        (r"a[b-d]", r"aac", vec![Some((1, 3))]),
        (r"a[-b]", r"a-", vec![Some((0, 2))]),
        (r"a[b-]", r"a-", vec![Some((0, 2))]),
        (r"a]", r"a]", vec![Some((0, 2))]),
        (r"a[]]b", r"a]b", vec![Some((0, 3))]),
        (r"a[^bc]d", r"aed", vec![Some((0, 3))]),
        (r"a[^-b]c", r"adc", vec![Some((0, 3))]),
        (r"a[^]b]c", r"adc", vec![Some((0, 3))]),
        (r"ab|cd", r"abc", vec![Some((0, 2))]),
        (r"ab|cd", r"abcd", vec![Some((0, 2))]),
        (r"a\(b", r"a(b", vec![Some((0, 3))]),
        (r"a\(*b", r"ab", vec![Some((0, 2))]),
        (r"a\(*b", r"a((b", vec![Some((0, 4))]),
        (
            r"((a))",
            r"abc",
            vec![Some((0, 1)), Some((0, 1)), Some((0, 1))],
        ),
        (
            r"(a)b(c)",
            r"abc",
            vec![Some((0, 3)), Some((0, 1)), Some((2, 3))],
        ),
        (r"a+b+c", r"aabbabc", vec![Some((4, 7))]),
        (r"a*", r"aaa", vec![Some((0, 3))]),
        (r"(a*)*", r"-", vec![Some((0, 0)), None]),
        (r"(a*)+", r"-", vec![Some((0, 0)), Some((0, 0))]),
        (r"(a*|b)*", r"-", vec![Some((0, 0)), None]),
        (r"(a+|b)*", r"ab", vec![Some((0, 2)), Some((1, 2))]),
        (r"(a+|b)+", r"ab", vec![Some((0, 2)), Some((1, 2))]),
        (r"(a+|b)?", r"ab", vec![Some((0, 1)), Some((0, 1))]),
        (r"[^ab]*", r"cde", vec![Some((0, 3))]),
        (r"(^)*", r"-", vec![Some((0, 0)), None]),
        (r"a*", r"", vec![Some((0, 0))]),
        (r"([abc])*d", r"abbbcd", vec![Some((0, 6)), Some((4, 5))]),
        (r"([abc])*bcd", r"abcd", vec![Some((0, 4)), Some((0, 1))]),
        (r"a|b|c|d|e", r"e", vec![Some((0, 1))]),
        (r"(a|b|c|d|e)f", r"ef", vec![Some((0, 2)), Some((0, 1))]),
        (r"((a*|b))*", r"-", vec![Some((0, 0)), None, None]),
        (r"abcd*efg", r"abcdefg", vec![Some((0, 7))]),
        (r"ab*", r"xabyabbbz", vec![Some((1, 3))]),
        (r"ab*", r"xayabbbz", vec![Some((1, 2))]),
        (r"(ab|cd)e", r"abcde", vec![Some((2, 5)), Some((2, 4))]),
        (r"[abhgefdc]ij", r"hij", vec![Some((0, 3))]),
        (r"(a|b)c*d", r"abcd", vec![Some((1, 4)), Some((1, 2))]),
        (r"(ab|ab*)bc", r"abc", vec![Some((0, 3)), Some((0, 1))]),
        (r"a([bc]*)c*", r"abc", vec![Some((0, 3)), Some((1, 3))]),
        (
            r"a([bc]*)(c*d)",
            r"abcd",
            vec![Some((0, 4)), Some((1, 3)), Some((3, 4))],
        ),
        (
            r"a([bc]+)(c*d)",
            r"abcd",
            vec![Some((0, 4)), Some((1, 3)), Some((3, 4))],
        ),
        (
            r"a([bc]*)(c+d)",
            r"abcd",
            vec![Some((0, 4)), Some((1, 2)), Some((2, 4))],
        ),
        (r"a[bcd]*dcdcde", r"adcdcde", vec![Some((0, 7))]),
        (r"(ab|a)b*c", r"abc", vec![Some((0, 3)), Some((0, 2))]),
        (
            r"((a)(b)c)(d)",
            r"abcd",
            vec![
                Some((0, 4)),
                Some((0, 3)),
                Some((0, 1)),
                Some((1, 2)),
                Some((3, 4)),
            ],
        ),
        (r"[A-Za-z_][A-Za-z0-9_]*", r"alpha", vec![Some((0, 5))]),
        (r"^a(bc+|b[eh])g|.h$", r"abh", vec![Some((1, 3))]),
        (
            r"(bc+d$|ef*g.|h?i(j|k))",
            r"effgz",
            vec![Some((0, 5)), Some((0, 5))],
        ),
        (
            r"(bc+d$|ef*g.|h?i(j|k))",
            r"ij",
            vec![Some((0, 2)), Some((0, 2)), Some((1, 2))],
        ),
        (
            r"(bc+d$|ef*g.|h?i(j|k))",
            r"reffgz",
            vec![Some((1, 6)), Some((1, 6))],
        ),
        (
            r"(((((((((a)))))))))",
            r"a",
            vec![
                Some((0, 1)),
                Some((0, 1)),
                Some((0, 1)),
                Some((0, 1)),
                Some((0, 1)),
                Some((0, 1)),
                Some((0, 1)),
                Some((0, 1)),
                Some((0, 1)),
                Some((0, 1)),
            ],
        ),
        (
            r"multiple words",
            r"multiple words yeah",
            vec![Some((0, 14))],
        ),
        (
            r"(.*)c(.*)",
            r"abcde",
            vec![Some((0, 5)), Some((0, 2)), Some((3, 5))],
        ),
        (r"abcd", r"abcd", vec![Some((0, 4))]),
        (r"a(bc)d", r"abcd", vec![Some((0, 4)), Some((1, 3))]),
        (r"a[-]?c", r"ac", vec![Some((0, 3))]),
        (
            r"M[ou]'?am+[ae]r .*([AEae]l[- ])?[GKQ]h?[aeu]+([dtz][dhz]?)+af[iy]",
            r"Muammar Qaddafi",
            vec![Some((0, 15)), None, Some((10, 12))],
        ),
        (
            r"M[ou]'?am+[ae]r .*([AEae]l[- ])?[GKQ]h?[aeu]+([dtz][dhz]?)+af[iy]",
            r"Mo'ammar Gadhafi",
            vec![Some((0, 16)), None, Some((11, 13))],
        ),
        (
            r"M[ou]'?am+[ae]r .*([AEae]l[- ])?[GKQ]h?[aeu]+([dtz][dhz]?)+af[iy]",
            r"Muammar Kaddafi",
            vec![Some((0, 15)), None, Some((10, 12))],
        ),
        (
            r"M[ou]'?am+[ae]r .*([AEae]l[- ])?[GKQ]h?[aeu]+([dtz][dhz]?)+af[iy]",
            r"Muammar Qadhafi",
            vec![Some((0, 15)), None, Some((10, 12))],
        ),
        (
            r"M[ou]'?am+[ae]r .*([AEae]l[- ])?[GKQ]h?[aeu]+([dtz][dhz]?)+af[iy]",
            r"Muammar Gadafi",
            vec![Some((0, 14)), None, Some((10, 11))],
        ),
        (
            r"M[ou]'?am+[ae]r .*([AEae]l[- ])?[GKQ]h?[aeu]+([dtz][dhz]?)+af[iy]",
            r"Mu'ammar Qadafi",
            vec![Some((0, 15)), None, Some((11, 12))],
        ),
        (
            r"M[ou]'?am+[ae]r .*([AEae]l[- ])?[GKQ]h?[aeu]+([dtz][dhz]?)+af[iy]",
            r"Moamar Gaddafi",
            vec![Some((0, 14)), None, Some((9, 11))],
        ),
        (
            r"M[ou]'?am+[ae]r .*([AEae]l[- ])?[GKQ]h?[aeu]+([dtz][dhz]?)+af[iy]",
            r"Mu'ammar Qadhdhafi",
            vec![Some((0, 18)), None, Some((13, 15))],
        ),
        (
            r"M[ou]'?am+[ae]r .*([AEae]l[- ])?[GKQ]h?[aeu]+([dtz][dhz]?)+af[iy]",
            r"Muammar Khaddafi",
            vec![Some((0, 16)), None, Some((11, 13))],
        ),
        (
            r"M[ou]'?am+[ae]r .*([AEae]l[- ])?[GKQ]h?[aeu]+([dtz][dhz]?)+af[iy]",
            r"Muammar Ghaddafy",
            vec![Some((0, 16)), None, Some((11, 13))],
        ),
        (
            r"M[ou]'?am+[ae]r .*([AEae]l[- ])?[GKQ]h?[aeu]+([dtz][dhz]?)+af[iy]",
            r"Muammar Ghadafi",
            vec![Some((0, 15)), None, Some((11, 12))],
        ),
        (
            r"M[ou]'?am+[ae]r .*([AEae]l[- ])?[GKQ]h?[aeu]+([dtz][dhz]?)+af[iy]",
            r"Muammar Ghaddafi",
            vec![Some((0, 16)), None, Some((11, 13))],
        ),
        (
            r"M[ou]'?am+[ae]r .*([AEae]l[- ])?[GKQ]h?[aeu]+([dtz][dhz]?)+af[iy]",
            r"Muamar Kaddafi",
            vec![Some((0, 14)), None, Some((9, 11))],
        ),
        (
            r"M[ou]'?am+[ae]r .*([AEae]l[- ])?[GKQ]h?[aeu]+([dtz][dhz]?)+af[iy]",
            r"Muammar Quathafi",
            vec![Some((0, 16)), None, Some((11, 13))],
        ),
        (
            r"M[ou]'?am+[ae]r .*([AEae]l[- ])?[GKQ]h?[aeu]+([dtz][dhz]?)+af[iy]",
            r"Muammar Gheddafi",
            vec![Some((0, 16)), None, Some((11, 13))],
        ),
        (
            r"M[ou]'?am+[ae]r .*([AEae]l[- ])?[GKQ]h?[aeu]+([dtz][dhz]?)+af[iy]",
            r"Moammar Khadafy",
            vec![Some((0, 15)), None, Some((11, 12))],
        ),
        (
            r"M[ou]'?am+[ae]r .*([AEae]l[- ])?[GKQ]h?[aeu]+([dtz][dhz]?)+af[iy]",
            r"Moammar Qudhafi",
            vec![Some((0, 15)), None, Some((10, 12))],
        ),
        (r"a+(b|c)*d+", r"aabcdd", vec![Some((0, 6)), Some((3, 4))]),
        (r"^.+$", r"vivi", vec![Some((0, 4))]),
        (r"^(.+)$", r"vivi", vec![Some((0, 4)), Some((0, 4))]),
        (
            r"^([^!.]+).att.com!(.+)$",
            r"gryphon.att.com!eby",
            vec![Some((0, 19)), Some((0, 7)), Some((16, 19))],
        ),
        (
            r"^([^!]+!)?([^!]+)$",
            r"bas",
            vec![Some((0, 3)), None, Some((0, 3))],
        ),
        (
            r"^([^!]+!)?([^!]+)$",
            r"bar!bas",
            vec![Some((0, 7)), Some((0, 4)), Some((4, 7))],
        ),
        (
            r"^([^!]+!)?([^!]+)$",
            r"foo!bas",
            vec![Some((0, 7)), Some((0, 4)), Some((4, 7))],
        ),
        (
            r"^.+!([^!]+!)([^!]+)$",
            r"foo!bar!bas",
            vec![Some((0, 11)), Some((4, 8)), Some((8, 11))],
        ),
        (
            r"((foo)|(bar))!bas",
            r"bar!bas",
            vec![Some((0, 7)), Some((0, 3)), None, Some((0, 3))],
        ),
        (
            r"((foo)|(bar))!bas",
            r"foo!bar!bas",
            vec![Some((4, 11)), Some((4, 7)), None, Some((4, 7))],
        ),
        (
            r"((foo)|(bar))!bas",
            r"foo!bas",
            vec![Some((0, 7)), Some((0, 3)), Some((0, 3))],
        ),
        (
            r"((foo)|bar)!bas",
            r"bar!bas",
            vec![Some((0, 7)), Some((0, 3))],
        ),
        (
            r"((foo)|bar)!bas",
            r"foo!bar!bas",
            vec![Some((4, 11)), Some((4, 7))],
        ),
        (
            r"((foo)|bar)!bas",
            r"foo!bas",
            vec![Some((0, 7)), Some((0, 3)), Some((0, 3))],
        ),
        (
            r"(foo|(bar))!bas",
            r"bar!bas",
            vec![Some((0, 7)), Some((0, 3)), Some((0, 3))],
        ),
        (
            r"(foo|(bar))!bas",
            r"foo!bar!bas",
            vec![Some((4, 11)), Some((4, 7)), Some((4, 7))],
        ),
        (
            r"(foo|(bar))!bas",
            r"foo!bas",
            vec![Some((0, 7)), Some((0, 3))],
        ),
        (
            r"(foo|bar)!bas",
            r"bar!bas",
            vec![Some((0, 7)), Some((0, 3))],
        ),
        (
            r"(foo|bar)!bas",
            r"foo!bar!bas",
            vec![Some((4, 11)), Some((4, 7))],
        ),
        (
            r"(foo|bar)!bas",
            r"foo!bas",
            vec![Some((0, 7)), Some((0, 3))],
        ),
        (
            r"^(([^!]+!)?([^!]+)|.+!([^!]+!)([^!]+))$",
            r"foo!bar!bas",
            vec![
                Some((0, 11)),
                Some((0, 11)),
                None,
                None,
                Some((4, 8)),
                Some((8, 11)),
            ],
        ),
        (
            r"^([^!]+!)?([^!]+)$|^.+!([^!]+!)([^!]+)$",
            r"bas",
            vec![Some((0, 3)), None, Some((0, 3))],
        ),
        (
            r"^([^!]+!)?([^!]+)$|^.+!([^!]+!)([^!]+)$",
            r"bar!bas",
            vec![Some((0, 7)), Some((0, 4)), Some((4, 7))],
        ),
        (
            r"^([^!]+!)?([^!]+)$|^.+!([^!]+!)([^!]+)$",
            r"foo!bar!bas",
            vec![Some((0, 11)), None, None, Some((4, 8)), Some((8, 11))],
        ),
        (
            r"^([^!]+!)?([^!]+)$|^.+!([^!]+!)([^!]+)$",
            r"foo!bas",
            vec![Some((0, 7)), Some((0, 4)), Some((4, 7))],
        ),
        (
            r"^(([^!]+!)?([^!]+)|.+!([^!]+!)([^!]+))$",
            r"bas",
            vec![Some((0, 3)), Some((0, 3)), None, Some((0, 3))],
        ),
        (
            r"^(([^!]+!)?([^!]+)|.+!([^!]+!)([^!]+))$",
            r"bar!bas",
            vec![Some((0, 7)), Some((0, 7)), Some((0, 4)), Some((4, 7))],
        ),
        (
            r"^(([^!]+!)?([^!]+)|.+!([^!]+!)([^!]+))$",
            r"foo!bar!bas",
            vec![
                Some((0, 11)),
                Some((0, 11)),
                None,
                None,
                Some((4, 8)),
                Some((8, 11)),
            ],
        ),
        (
            r"^(([^!]+!)?([^!]+)|.+!([^!]+!)([^!]+))$",
            r"foo!bas",
            vec![Some((0, 7)), Some((0, 7)), Some((0, 4)), Some((4, 7))],
        ),
        (r".*(/XXX).*", r"/XXX", vec![Some((0, 4)), Some((0, 4))]),
        (r".*(\\XXX).*", r"\XXX", vec![Some((0, 4)), Some((0, 4))]),
        (r"\\XXX", r"\XXX", vec![Some((0, 4))]),
        (r".*(/000).*", r"/000", vec![Some((0, 4)), Some((0, 4))]),
        (r".*(\\000).*", r"\000", vec![Some((0, 4)), Some((0, 4))]),
        (r"\\000", r"\000", vec![Some((0, 4))]),
    ]
}
