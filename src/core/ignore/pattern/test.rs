use std::path::PathBuf;

use super::super::EntryPath;
use super::*;

#[test]
fn test_matches() {
    let patterns = parse(
        "
*.txt
*.mp4
dir/
/rdir/
!note.txt
a/sample.png
a/**/sample.jpg
!a/x.mp4
#xxx"
            .as_bytes(),
    )
    .unwrap();

    let root = PathBuf::from("/d");

    let cases = vec![
        (Match::Parent, "/d/a", false),
        (Match::Ignored, "/d/a.txt", false),
        (Match::Ignored, "/d/a.mp4", false),
        (Match::Ignored, "/d/a/a.mp4", false),
        (Match::Parent, "/d/dir", false),
        (Match::Ignored, "/d/dir", true),
        (Match::Ignored, "/d/a/b/dir", true),
        (Match::Parent, "/d/rdir", false),
        (Match::Ignored, "/d/rdir", true),
        (Match::Parent, "/d/a/rdir", true),
        (Match::Allowed, "/d/note.txt", false),
        (Match::Parent, "/d/sample.png", false),
        (Match::Ignored, "/d/a/sample.png", false),
        (Match::Parent, "/d/a/a/sample.png", false),
        (Match::Parent, "/d/sample.jpg", false),
        (Match::Ignored, "/d/a/sample.jpg", false),
        (Match::Ignored, "/d/a/a/sample.jpg", false),
        (Match::Allowed, "/d/a/x.mp4", false),
        (Match::Parent, "/d/#xxx", false),
    ];

    for (to_be, path_str, is_dir) in cases {
        eprintln!();
        let path = PathBuf::from(path_str);
        let entry_path = EntryPath::from_path(&root, &path, is_dir).unwrap();
        let actual = patterns.matches(&entry_path);

        assert_eq!(to_be, actual, "path = {}, is_dir = {}", path_str, is_dir);
    }
}

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
