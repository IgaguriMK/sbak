//! ファイル除外パターンを扱う

pub mod pattern;

use std::ffi::OsString;
use std::io;
use std::path::{Component, Path, PathBuf};

use failure::Fail;

#[cfg(test)]
mod test;

/// エントリのパスを表す。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntryPath {
    parts: Vec<String>,
}

impl EntryPath {
    /// `EntryPath` を生成する。
    ///
    /// バックアップ対象の起点ディレクトリが`root`、エントリのパスが`entry`である。
    ///
    /// # Failures
    ///
    /// ファイルの絶対パスが取得できない場合、[`Error::IO`](enum.Error.html#variant.IO)を返す。
    ///
    /// `entry`が`root`の子でないかパスの処理中に`root`の外に出た場合、[`Error::NotChild`](enum.Error.html#variant.NotChild)を返す。
    pub fn new(root: &Path, entry: &Path) -> Result<EntryPath> {
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

        Ok(EntryPath { parts })
    }

    /// エントリのルートからの相対パスのパーツのリストを返す。
    pub fn parts(&self) -> &[String] {
        &self.parts
    }
}

type Result<T> = std::result::Result<T, Error>;

/// ファイルの除外判定で発生しうるエラー
#[derive(Debug, Fail)]
pub enum Error {
    /// 入出力エラーが発生した。
    #[fail(display = "failed with IO error: {}", _0)]
    IO(#[fail(cause)] io::Error),

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

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::IO(e)
    }
}

impl From<OsString> for Error {
    fn from(osstr: OsString) -> Error {
        Error::NotValidUnicode(osstr)
    }
}
