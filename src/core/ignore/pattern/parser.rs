use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::path::Path;

use super::*;

/// 除外パターンファイルを読み込む。
pub fn load_patterns<P: AsRef<Path>>(path: P) -> Result<Patterns> {
    let f = File::open(path)?;
    parse(f)
}

/// パターンのリストをパースする。
///
/// `#`で始まる行はコメントとみなす。
/// 空行は無視される。
pub fn parse<R: Read>(r: R) -> Result<Patterns> {
    let r = BufReader::new(r);

    let mut patterns = Vec::new();

    for line in r.lines() {
        let line = line?;

        // コメント行をスキップ
        if line.trim_start().starts_with('#') {
            continue;
        }
        // 空行をスキップ
        if line.is_empty() {
            continue;
        }

        let pat = parse_pattern(&line)?;
        patterns.push(pat);
    }

    Ok(Patterns::new(patterns))
}

fn parse_pattern(mut input: &str) -> Result<Pattern> {
    let allow = input.starts_with('!');
    if allow {
        input = input.split_at(1).1;
    }

    let mut cascade = !input.starts_with('/');
    if !cascade {
        input = input.split_at(1).1;
    }

    let dir_only = input.ends_with('/');
    if dir_only {
        input = input.split_at(input.len() - 1).0;
    }

    let mut parts = Vec::new();
    for part_str in pattern_split(input)? {
        if part_str == "**" {
            parts.push(PatternPart::AnyPath);
        } else {
            let name_pattern = parse_name_pattern(&part_str)?;
            parts.push(PatternPart::Normal(name_pattern));
        }
    }

    // パートが2つ以上なら子に伝播しない。
    if parts.len() >= 2 {
        cascade = false;
    }

    Ok(Pattern::from_parts(allow, cascade, dir_only, parts))
}

fn pattern_split(mut input: &str) -> Result<Vec<String>> {
    let mut res = Vec::new();

    while !input.is_empty() {
        let mut part = String::new();

        'part: while !input.is_empty() {
            while let Some((_, left)) = trim_if_match(input, r"\/") {
                part.push('/');
                input = left;
            }

            while let Some((ch, left)) = trim_char(input) {
                // エスケープシーケンスの先頭なのでパースし直す。
                if ch == '\\' {
                    continue 'part;
                }

                input = left;
                if ch == '/' {
                    break 'part;
                }

                part.push(ch)
            }
        }
        res.push(part);
    }
    Ok(res)
}

fn parse_name_pattern(mut input: &str) -> Result<NamePattern> {
    // NamePattern::from_parts で正規化されるので、パース時点では細切れになっていて問題ない。
    let orig_input = input;
    let mut res = Vec::new();

    'parse: while !input.is_empty() {
        // エスケープシーケンスを処理
        for escape_pat in &[r"\\", r"\?", r"\*"] {
            if let Some((p, left)) = trim_if_match(input, escape_pat) {
                res.push(NamePatternPart::Str(p.split_at(1).1.to_owned()));
                input = left;
                continue 'parse;
            }
        }

        // 不正なエスケープシーケンスがある。
        if input.starts_with('\\') {
            return Err(Error::InvalidPattern(orig_input.to_owned()));
        }

        // 1文字を処理
        let mut res_str = String::new();
        let mut next: Option<NamePatternPart> = None;
        while let Some((ch, left)) = trim_char(input) {
            // エスケープシーケンスの先頭なのでパースし直す。
            if ch == '\\' {
                break;
            }

            // 特殊文字
            if ch == '?' {
                next = Some(NamePatternPart::AnyChar);
                input = left;
                break;
            }
            if ch == '*' {
                next = Some(NamePatternPart::AnyStr);
                input = left;
                break;
            }

            // 通常の文字
            res_str.push(ch);
            input = left;
        }
        if !res_str.is_empty() {
            res.push(NamePatternPart::Str(res_str));
        }
        if let Some(part) = next {
            res.push(part);
        }
    }

    Ok(NamePattern::from_parts(res))
}

fn trim_if_match<'a>(s: &'a str, pat: &'a str) -> Option<(&'a str, &'a str)> {
    if s.starts_with(pat) {
        Some((pat, s.split_at(pat.len()).1))
    } else {
        None
    }
}

fn trim_char(s: &str) -> Option<(char, &str)> {
    if s.is_empty() {
        return None;
    }

    let mut i = 1usize;
    while i < s.len() {
        if s.is_char_boundary(i) {
            // 文字区切りが見つかったらそこで切断して返す。
            let (ch_str, left_s) = s.split_at(i);
            return Some((ch_str.chars().next().unwrap(), left_s));
        }
        i += 1;
    }
    Some((s.chars().next().unwrap(), ""))
}

type Result<T> = std::result::Result<T, Error>;

/// パターンのパースで発生しうるエラー
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// パターン表現の文字列が不正である。
    #[error("invalid pattern string: {0}")]
    InvalidPattern(String),

    /// 入出力エラー
    #[error("failed scan with IO error: {0}")]
    IO(#[from] io::Error),
}

#[cfg(test)]
mod test {
    use super::super::{
        NamePattern as NP, NamePatternPart as NPP, Pattern as P, PatternPart as PP,
    };
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
}
