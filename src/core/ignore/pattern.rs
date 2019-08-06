//! 除外ファイルのパターンを表す。

use failure::Fail;

#[cfg(test)]
mod test;

/// パターンのリストを表す。
pub struct Patterns {
    patterns: Vec<Pattern>,
}

/// 除外ファイルの1パターンを表す。
pub struct Pattern {
    parts: Vec<PatternPart>,
    allow: bool,
}

enum PatternPart {
    Normal(NamePattern),
}

struct NamePattern {
    parts: Vec<NamePatternPart>,
}

impl NamePattern {
    fn parse(pattern_str: &str) -> Result<NamePattern> {
        let mut parts = Vec::new();

        loop {}

        Ok(NamePattern { parts })
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
                let left_s = s.trim_start_matches(ps);
                return match_np(left_parts, left_s);
            } else {
                return false;
            }
        }
        NamePatternPart::AnyChar => {
            // マッチ対象文字列がない場合失敗が確定。
            if s.is_empty() {
                return false;
            }
            let left_s = trim_char(s);
            return match_np(left_parts, left_s);
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
            return false;
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
    return "";
}

enum NamePatternPart {
    Str(String),
    AnyChar,
    AnyStr,
}

type Result<T> = std::result::Result<T, Error>;

/// パターン操作で発生しうるエラー
#[derive(Debug, Fail)]
pub enum Error {
    /// パターン表現の文字列が不正である。
    #[fail(display = "invalid pattern string: {}", _0)]
    InvalidPattern(String),
}
