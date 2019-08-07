use super::*;

#[test]
fn test_match_name_pattern() {
    let cases: Vec<(Vec<NamePatternPart>, Vec<(&'static str, bool)>)> = vec![
        (
            vec![NamePatternPart::Str("a".to_owned())],
            vec![
                ("", false),
                ("a", true),
                ("b", false),
                ("aa", false),
                ("ab", false),
            ],
        ),
        (
            vec![NamePatternPart::Str("ab".to_owned())],
            vec![
                ("", false),
                ("a", false),
                ("b", false),
                ("aa", false),
                ("ab", true),
                ("aba", false),
            ],
        ),
        (
            vec![
                NamePatternPart::Str("a".to_owned()),
                NamePatternPart::Str("b".to_owned()),
            ],
            vec![
                ("", false),
                ("a", false),
                ("b", false),
                ("aa", false),
                ("ab", true),
                ("aba", false),
            ],
        ),
        (
            vec![NamePatternPart::AnyChar],
            vec![("", false), ("a", true), ("b", true), ("aa", false)],
        ),
        (
            vec![
                NamePatternPart::Str("a".to_owned()),
                NamePatternPart::AnyChar,
                NamePatternPart::Str("a".to_owned()),
            ],
            vec![
                ("", false),
                ("a", false),
                ("aa", false),
                ("aaa", true),
                ("aba", true),
                ("baa", false),
                ("aaaa", false),
            ],
        ),
        (
            vec![NamePatternPart::AnyStr],
            vec![("", true), ("a", true), ("aa", true), ("aaa", true)],
        ),
        (
            vec![
                NamePatternPart::Str("a".to_owned()),
                NamePatternPart::AnyStr,
                NamePatternPart::Str("a".to_owned()),
            ],
            vec![
                ("", false),
                ("a", false),
                ("aa", true),
                ("aaa", true),
                ("aba", true),
                ("baa", false),
                ("aaaa", true),
                ("abca", true),
                ("aaaab", false),
            ],
        ),
        (
            vec![
                NamePatternPart::Str("a".to_owned()),
                NamePatternPart::AnyStr,
                NamePatternPart::AnyStr,
                NamePatternPart::Str("a".to_owned()),
            ],
            vec![
                ("", false),
                ("a", false),
                ("aa", true),
                ("aaa", true),
                ("aba", true),
                ("baa", false),
                ("aaaa", true),
                ("abca", true),
                ("aaaab", false),
            ],
        ),
    ];

    for (parts, strs) in cases {
        let pat = NamePattern::from_parts(parts.clone());

        for (s, to_be) in strs {
            let actual = pat.match_str(s);
            if to_be {
                assert!(actual, "pattern {:?} should match with {}", parts, s);
            } else {
                assert!(!actual, "pattern {:?} should not match with {}", parts, s);
            }
        }
    }
}
