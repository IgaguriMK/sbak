//! ファイル除外パターンを扱う

pub mod pattern;

use std::ffi::OsString;
use std::path::{Component, Path, PathBuf};

use pattern::{load_patterns, Match, Patterns};

use failure::Fail;

#[cfg(test)]
mod test;

const IGNORE_FILE: &str = ".sbakignore";

/// エントリのパスを表す。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntryPath {
    parts: Vec<String>,
    is_dir: bool,
}

impl EntryPath {
    /// `EntryPath` を生成する。
    ///
    /// バックアップ対象の起点ディレクトリが`root`、エントリのパスが`entry`である。
    ///
    /// # Failures
    ///
    /// `entry`が`root`の子でないかパスの処理中に`root`の外に出た場合、[`Error::NotChild`](enum.Error.html#variant.NotChild)を返す。
    pub fn from_path(root: &Path, entry: &Path, is_dir: bool) -> Result<EntryPath> {
        let relative = entry
            .strip_prefix(&root)
            .map_err(|_| Error::NotChild(entry.to_owned(), root.to_owned()))?;

        let mut parts = Vec::new();

        for c in relative.components() {
            match c {
                Component::Normal(p) => {
                    let s = p.to_owned().into_string()?;
                    parts.push(s);
                }
                Component::ParentDir => {
                    if parts.pop().is_none() {
                        return Err(Error::NotChild(
                            root.parent().map(|p| p.to_owned()).unwrap_or_default(),
                            root.to_owned(),
                        ));
                    }
                }
                c => {
                    return Err(Error::UnexpectedComponent(format!("{:?}", c)));
                }
            }
        }

        Ok(EntryPath { parts, is_dir })
    }

    /// エントリのルートからの相対パスのパーツのリストを返す。
    fn parts(&self) -> &[String] {
        &self.parts
    }
}

/// 除外判定の設定を親ディレクトリに遡るためのスタック。
pub struct IgnoreStack<'a> {
    root_path: PathBuf,
    parent: Option<&'a IgnoreStack<'a>>,
    current_patterns: Patterns,
}

impl<'a> IgnoreStack<'a> {
    /// 新たな除外パターンのスタックを作成する。
    pub fn new(root_path: &'a Path, bank_patterns: Patterns) -> IgnoreStack<'a> {
        IgnoreStack {
            root_path: root_path.to_owned(),
            parent: None,
            current_patterns: bank_patterns,
        }
    }

    /// ディレクトリ名を指定して子エントリ用の除外判定を生成する。
    pub fn child<'b>(&'a self, dir_name: String) -> Result<IgnoreStack<'b>>
    where
        'a: 'b,
    {
        // スタックの底の場合はBank由来の設定なので、パスに加えない。
        let root_path = if self.parent.is_some() {
            self.root_path.join(&dir_name)
        } else {
            self.root_path.clone()
        };

        // 除外設定を読み込み
        let ignore_file = self.root_path.join(IGNORE_FILE);
        let current_patterns = if ignore_file.exists() {
            load_patterns(&ignore_file)?
        } else {
            Patterns::default()
        };

        Ok(IgnoreStack {
            root_path,
            parent: Some(&self),
            current_patterns,
        })
    }

    /// 除外対象かどうかチェックする。
    pub fn ignored(&self, path: &Path, is_dir: bool) -> Result<bool> {
        let entry_path = EntryPath::from_path(&self.root_path, path, is_dir)?;

        match self.current_patterns.matches(&entry_path) {
            Match::Allowed => return Ok(false),
            Match::Ignored => return Ok(true),
            _ => {}
        }

        if let Some(parent) = self.parent {
            return parent.ignored(path, is_dir);
        }
        Ok(false)
    }
}

type Result<T> = std::result::Result<T, Error>;

/// ファイルの除外判定で発生しうるエラー
#[derive(Debug, Fail)]
pub enum Error {
    /// 除外リストの読み込みに失敗した
    #[fail(display = "failed load ignore patterns: {}", _0)]
    IgnorePattern(#[fail(cause)] pattern::ParseError),

    /// エントリのパスがバックアップ対象のルートの子ではない。
    #[fail(display = "invalid path: {:?} is not child of {:?} ", _0, _1)]
    NotChild(PathBuf, PathBuf),

    /// エントリのパスの一部が正しいUnicodeに変換できない。
    #[fail(display = "invalid path: contains non-unicode part {:?} ", _0)]
    NotValidUnicode(OsString),

    /// エントリのパスの一部にファイル名ではない部分がある。
    #[fail(display = "invalid path: contains non-normal part {} ", _0)]
    UnexpectedComponent(String),
}

impl From<pattern::ParseError> for Error {
    fn from(e: pattern::ParseError) -> Error {
        Error::IgnorePattern(e)
    }
}

impl From<OsString> for Error {
    fn from(osstr: OsString) -> Error {
        Error::NotValidUnicode(osstr)
    }
}
