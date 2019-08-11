//! 除外ファイルのパターンを表す。

mod parser;

use super::EntryPath;

#[cfg(test)]
mod test;

pub use parser::{load_patterns, parse, Error as ParseError};

/// パターンのリストを表す。
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Patterns {
    patterns: Vec<Pattern>,
}

impl Patterns {
    fn new(patterns: Vec<Pattern>) -> Patterns {
        Patterns { patterns }
    }

    /// エントリがパターンにマッチするか検査する。
    pub fn matches(&self, entry_path: &EntryPath) -> Match {
        for pat in self.patterns.iter().rev() {
            match pat.matches(entry_path) {
                Match::Allowed => return Match::Allowed,
                Match::Ignored => return Match::Ignored,
                _ => {}
            }
        }
        Match::Parent
    }
}

/// 除外ファイルの1パターンを表す。
#[derive(Debug, Clone, PartialEq)]
pub struct Pattern {
    parts: Vec<PatternPart>,
    allow: bool,
    dir_only: bool,
}

impl Pattern {
    /// エントリがパターンにマッチするか検査する。
    pub fn matches(&self, entry_path: &EntryPath) -> Match {
        if self.dir_only && !entry_path.is_dir {
            return Match::Parent;
        }

        if match_path(&self.parts, entry_path.parts()) {
            if self.allow {
                Match::Allowed
            } else {
                Match::Ignored
            }
        } else {
            Match::Parent
        }
    }

    fn from_parts(
        allow: bool,
        cascade: bool,
        dir_only: bool,
        mut parts: Vec<PatternPart>,
    ) -> Pattern {
        let mut normalized_parts = Vec::with_capacity(parts.len());

        if cascade {
            normalized_parts.push(PatternPart::AnyPath);
        }

        // 先頭の要素から取り出しながら正規化処理をする。
        parts.reverse();
        while parts.len() >= 2 {
            let current = parts.pop().unwrap();
            let next = parts.pop().unwrap();
            match (current, next) {
                // ワイルドカードが連続
                (PatternPart::AnyPath, PatternPart::AnyPath) => {
                    parts.push(PatternPart::AnyPath);
                }
                // 既に正規
                (current, next) => {
                    normalized_parts.push(current);
                    parts.push(next);
                }
            }
        }
        if let Some(last_part) = parts.pop() {
            normalized_parts.push(last_part);
        }

        Pattern {
            parts: normalized_parts,
            allow,
            dir_only,
        }
    }
}

fn match_path(parts: &[PatternPart], path: &[String]) -> bool {
    if parts.is_empty() {
        return path.is_empty();
    }

    let p = &parts[0];
    let left_parts = &parts[1..];

    match p {
        PatternPart::Normal(pat) => {
            if let Some(ref s) = path.first() {
                if !pat.match_str(s) {
                    return false;
                }
                let left_path = &path[1..];
                return match_path(left_parts, left_path);
            }
            false
        }
        PatternPart::AnyPath => {
            for drop_cnt in 0..path.len() {
                let left_path = &path[drop_cnt..];
                if match_path(left_parts, left_path) {
                    return true;
                }
            }
            false
        }
    }
}

/// 除外パターンのマッチ結果を表す。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Match {
    /// 明示的に除外対象に指定されている。
    Ignored,
    /// 親ディレクトリの除外設定に従う。
    Parent,
    /// 明示的に許可されている。
    Allowed,
}

#[derive(Debug, Clone, PartialEq)]
enum PatternPart {
    Normal(NamePattern),
    AnyPath,
}

#[derive(Debug, Clone, PartialEq)]
struct NamePattern {
    parts: Vec<NamePatternPart>,
}

impl NamePattern {
    // 正規化処理をするので`new`ではなく`from_parts`という名前
    fn from_parts(mut parts: Vec<NamePatternPart>) -> NamePattern {
        let mut normalized_parts = Vec::with_capacity(parts.len());

        // 先頭の要素から取り出しながら正規化処理をする。
        parts.reverse();
        while parts.len() >= 2 {
            let current = parts.pop().unwrap();
            let next = parts.pop().unwrap();
            match (current, next) {
                // 文字列指定が連続
                (NamePatternPart::Str(mut s1), NamePatternPart::Str(s2)) => {
                    s1.push_str(&s2);
                    parts.push(NamePatternPart::Str(s1)); // 入力列に戻して正規化続行
                }
                // ワイルドカードが連続
                (NamePatternPart::AnyStr, NamePatternPart::AnyStr) => {
                    parts.push(NamePatternPart::AnyStr);
                }
                // 既に正規
                (current, next) => {
                    normalized_parts.push(current);
                    parts.push(next);
                }
            }
        }
        if let Some(last_part) = parts.pop() {
            normalized_parts.push(last_part);
        }

        NamePattern {
            parts: normalized_parts,
        }
    }

    fn match_str(&self, s: &str) -> bool {
        match_np(&self.parts, s)
    }
}

fn match_np(parts: &[NamePatternPart], s: &str) -> bool {
    if parts.is_empty() {
        // マッチすべきパートがなく、その時点でマッチ対象文字列が空であることがマッチ成功の条件。
        return s.is_empty();
    }

    let p = &parts[0];
    let left_parts = &parts[1..];

    match p {
        NamePatternPart::Str(ref ps) => {
            // 単純な文字列へのマッチ
            if s.starts_with(ps) {
                // trim_start_matches は複数回取り除いてしまうので不可。
                let (_, left_s) = s.split_at(ps.len());
                match_np(left_parts, left_s)
            } else {
                false
            }
        }
        NamePatternPart::AnyChar => {
            // マッチ対象文字列がない場合失敗が確定。
            if s.is_empty() {
                return false;
            }
            let left_s = trim_char(s);
            match_np(left_parts, left_s)
        }
        NamePatternPart::AnyStr => {
            // 最後のパーツなら自明にマッチ成功
            if left_parts.is_empty() {
                return true;
            }

            let mut left_s = s;
            while !left_s.is_empty() {
                // 成功パターン
                if match_np(left_parts, left_s) {
                    return true;
                }

                // 1文字消費してバックトラック
                left_s = trim_char(left_s);
            }
            // まだパターンのパーツがあり、全てのケースで失敗したので失敗。
            false
        }
    }
}

fn trim_char(s: &str) -> &str {
    let mut i = 1usize;
    while i < s.len() {
        if s.is_char_boundary(i) {
            // 文字区切りが見つかったらそこで切断して返す。
            let (_, left_s) = s.split_at(i);
            return left_s;
        }
        i += 1;
    }
    // 文字列が空でなく文字区切りが見つからないので、残っているのは1文字であるから、空文字列を返す。
    ""
}

#[derive(Debug, Clone, PartialEq)]
enum NamePatternPart {
    Str(String),
    AnyChar,
    AnyStr,
}

impl NamePatternPart {
    #[cfg(test)]
    fn s(s: &str) -> NamePatternPart {
        NamePatternPart::Str(s.to_owned())
    }
}
