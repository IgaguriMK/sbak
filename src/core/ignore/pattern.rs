//! 除外ファイルのパターンを表す。

use failure::Fail;

#[cfg(test)]
mod test;

/// パターンのリストを表す。
#[derive(Debug, Clone)]
pub struct Patterns {
    patterns: Vec<Pattern>,
}

/// 除外ファイルの1パターンを表す。
#[derive(Debug, Clone)]
pub struct Pattern {
    parts: Vec<PatternPart>,
    allow: bool,
}

#[derive(Debug, Clone)]
enum PatternPart {
    Normal(NamePattern),
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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
