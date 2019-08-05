//! ファイルやディレクトリのハッシュの生成。

use std::fmt;
use std::fs::File;
use std::io::{self, copy, Read, Seek, SeekFrom, Write};

use failure::Fail;
use hex::encode;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use tempfile::tempfile;

const BUFFER_SIZE: usize = 4096;

/// エントリのSHA3-256ハッシュID
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct HashID(String);

impl HashID {
    /// ハッシュ値の文字列表現を4-4-56文字に分割して返す。
    ///
    /// リポジトリでの保存先ディレクトリの階層化に使われる。
    pub fn parts(&self) -> (&str, &str, &str) {
        let s = self.0.as_str();
        (&s[0..4], &s[4..8], &s[8..])
    }

    /// 文字列表現への参照を返す。
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for HashID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// ファイルのハッシュ値を計算する。
///
/// ファイル`f`は一旦最後まで読み込まれた後、シーク位置が先頭に巻き戻される。
pub fn hash(f: &mut File) -> Result<HashID> {
    let mut hasher = Sha3_256::new();
    copy(f, &mut hasher)?;
    f.seek(SeekFrom::Start(0))?;

    Ok(HashID(encode(hasher.result())))
}

/// `r`から内容を一時ファイルにコピーしつつ、ハッシュ値を計算する。
pub fn hash_reader<R: Read>(mut r: R) -> Result<(HashID, File)> {
    let mut hasher = Sha3_256::new();
    let mut temp = tempfile()?;
    let mut buffer = [0u8; BUFFER_SIZE];

    loop {
        let read_size = r.read(&mut buffer)?;
        if read_size == 0 {
            break;
        }
        let bytes = &buffer[..read_size];

        hasher.write_all(&bytes)?;
        temp.write_all(&bytes)?;
    }

    temp.flush()?;
    temp.seek(SeekFrom::Start(0))?; // 読み込みに備えてファイル先頭に巻き戻しておく

    let hash = HashID(encode(hasher.result()));

    Ok((hash, temp))
}

#[allow(missing_docs)]
pub type Result<T> = std::result::Result<T, Error>;

/// ファイルシステムのスキャンで発生しうるエラー
#[derive(Debug, Fail)]
pub enum Error {
    /// 入出力エラーが発生した
    #[fail(display = "failed scan with IO error: {}", _0)]
    IO(#[fail(cause)] io::Error),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::IO(e)
    }
}
