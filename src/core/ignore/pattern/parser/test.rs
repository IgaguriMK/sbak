use super::super::{NamePattern as NP, NamePatternPart as NPP, Pattern as P, PatternPart as PP};
use super::*;

#[test]
fn str_test() {
    let mut chs = r"\/".chars();

    assert_eq!(chs.next(), Some('\\'));
    assert_eq!(chs.next(), Some('/'));
    assert_eq!(chs.next(), None);
}

#[test]
fn test_parse_pattern_success() {
    let a = PP::Normal(NP::from_parts(vec![NPP::s("a")]));
    let b = PP::Normal(NP::from_parts(vec![NPP::s("b")]));
    let a_s = PP::Normal(NP::from_parts(vec![NPP::s("a/")]));
    let any_txt = PP::Normal(NP::from_parts(vec![NPP::AnyStr, NPP::s(".txt")]));
    let any = PP::Normal(NP::from_parts(vec![NPP::AnyStr]));

    let cases = vec![
        // パーツが1つ
        ("a", P::from_parts(false, true, false, vec![a.clone()])),
        ("/a", P::from_parts(false, false, false, vec![a.clone()])),
        ("!a", P::from_parts(true, true, false, vec![a.clone()])),
        ("!/a", P::from_parts(true, false, false, vec![a.clone()])),
        ("a/", P::from_parts(false, true, true, vec![a.clone()])),
        ("/a/", P::from_parts(false, false, true, vec![a.clone()])),
        ("!a/", P::from_parts(true, true, true, vec![a.clone()])),
        ("!/a/", P::from_parts(true, false, true, vec![a.clone()])),
        // パーツが複数
        (
            "a/a",
            P::from_parts(false, false, false, vec![a.clone(), a.clone()]),
        ),
        (
            "/a/a",
            P::from_parts(false, false, false, vec![a.clone(), a.clone()]),
        ),
        (
            "!a/a",
            P::from_parts(true, false, false, vec![a.clone(), a.clone()]),
        ),
        (
            "!/a/a",
            P::from_parts(true, false, false, vec![a.clone(), a.clone()]),
        ),
        (
            "a/a/",
            P::from_parts(false, false, true, vec![a.clone(), a.clone()]),
        ),
        (
            "/a/a/",
            P::from_parts(false, false, true, vec![a.clone(), a.clone()]),
        ),
        (
            "!a/a/",
            P::from_parts(true, false, true, vec![a.clone(), a.clone()]),
        ),
        (
            "!/a/a/",
            P::from_parts(true, false, true, vec![a.clone(), a.clone()]),
        ),
        // 特殊文字を含む
        (
            r"a\//a",
            P::from_parts(false, false, false, vec![a_s.clone(), a.clone()]),
        ),
        (
            "a/*/b",
            P::from_parts(false, false, false, vec![a.clone(), any.clone(), b.clone()]),
        ),
        (
            "a/**/b",
            P::from_parts(false, false, false, vec![a.clone(), PP::AnyPath, b.clone()]),
        ),
        (
            "a/*.txt",
            P::from_parts(false, false, false, vec![a.clone(), any_txt.clone()]),
        ),
    ];

    for (pat_str, to_be) in cases {
        let actual = parse_pattern(pat_str).unwrap();

        assert_eq!(
            actual, to_be,
            "\n  '{}' should be parsed to\n    {:?},\n  but\n    {:?}",
            pat_str, to_be, actual
        );
    }
}

#[test]
fn test_parse_name_pattern_success() {
    let cases = vec![
        ("a", NP::from_parts(vec![NPP::s("a")])),
        ("aa", NP::from_parts(vec![NPP::s("aa")])),
        (r"a\?", NP::from_parts(vec![NPP::s("a?")])), // 正規化されるケース
        (r"a\?\?", NP::from_parts(vec![NPP::s("a??")])), // 正規化されるケース
        (r"a\*", NP::from_parts(vec![NPP::s("a*")])), // 正規化されるケース
        ("a?", NP::from_parts(vec![NPP::s("a"), NPP::AnyChar])),
        ("a*", NP::from_parts(vec![NPP::s("a"), NPP::AnyStr])),
        ("a**", NP::from_parts(vec![NPP::s("a"), NPP::AnyStr])), // 正規化されるケース
        ("?*", NP::from_parts(vec![NPP::AnyChar, NPP::AnyStr])),
    ];

    for (pat_str, to_be) in cases {
        let actual = parse_name_pattern(pat_str).unwrap();

        assert_eq!(
            actual, to_be,
            "\n    '{}' should be parsed to {:?}, but {:?}",
            pat_str, to_be, actual
        );
    }
}

#[test]
fn test_parse_name_pattern_fails() {
    let cases = vec![r"\", r"\\\", r"\a"];

    for pat_str in cases {
        let _ = parse_name_pattern(pat_str).unwrap_err();
    }
}
