//! ファイル除外パターンを扱う

pub mod pattern;

use std::ffi::OsString;
use std::path::{Component, Path, PathBuf};

use failure::Fail;

#[cfg(test)]
mod test;

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

type Result<T> = std::result::Result<T, Error>;

/// ファイルの除外判定で発生しうるエラー
#[derive(Debug, Fail)]
pub enum Error {
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

impl From<OsString> for Error {
    fn from(osstr: OsString) -> Error {
        Error::NotValidUnicode(osstr)
    }
}
