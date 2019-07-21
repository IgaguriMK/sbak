//! ファイルやディレクトリのハッシュの生成。

use std::fmt;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};

use failure::Fail;
use hex::encode;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use tempfile::tempfile;

const BUFFER_SIZE: usize = 4096;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct HashID(String);

impl HashID {
    pub fn parts(&self) -> (&str, &str, &str) {
        let s = self.0.as_str();
        (&s[0..4], &s[4..8], &s[8..])
    }
}

impl fmt::Display for HashID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub fn hash_reader<R: Read>(mut r: R) -> Result<(HashID, File)> {
    let mut hasher = Sha3_256::new();
    let mut temp = tempfile()?;
    let mut buffer = [0u8; BUFFER_SIZE];

    loop {
        let read_size = r.read(&mut buffer)?;
        if read_size == 0 {
            // End of input (due to BUFFER_SIZE > 0)
            break;
        }
        let bytes = &buffer[..read_size];

        hasher.write_all(&bytes)?;
        temp.write_all(&bytes)?;
    }

    temp.flush()?;
    temp.seek(SeekFrom::Start(0))?; // Seek to start of tempfile

    let hash = HashID(encode(hasher.result()));

    Ok((hash, temp))
}

pub type Result<T> = std::result::Result<T, Error>;

/// ファイルシステムのスキャンで発生しうるエラー
#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "failed scan with IO error: {}", _0)]
    IO(#[fail(cause)] io::Error),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::IO(e)
    }
}
