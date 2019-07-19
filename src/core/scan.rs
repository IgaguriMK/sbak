//! ファイルやディレクトリのスキャンを行う。

use std::fs;
use std::io;
use std::path::Path;

use failure::Fail;

use crate::core::fs_tree::*;

#[derive(Debug, Clone, Default)]
pub struct Scanner {
}

impl Scanner {
    pub fn new() -> Scanner {
        Scanner{}
    }

    pub fn scan<P: AsRef<Path>>(&self, p: P) -> Result<FsEntry> {
        let meta = fs::metadata(&p)?;
        
        if meta.is_dir() {
            Ok(self.scan_dir(p)?.into())
        } else if meta.is_file() {
            Ok(self.scan_file(p)?.into())
        } else {
            panic!("{:?} is not dir nor file", p.as_ref())
        }
    }

    fn scan_dir<P: AsRef<Path>>(&self, p: P) -> Result<DirEntry> {
        let mut entry = DirEntry::new(&p);

        for ch in fs::read_dir(p)? {
            let ch = ch?;
            entry.append(self.scan(ch.path())?);
        }

        Ok(entry)
    }

    fn scan_file<P: AsRef<Path>>(&self, p: P) -> Result<FileEntry> {
        Ok(FileEntry::new(p))
    }    
}

type Result<T> = std::result::Result<T, Error>;

/// ファイルシステムのスキャンで発生しうるエラー
#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display="failed scan with IO error: {}", _0)]
    IO(#[fail(cause)] io::Error),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::IO(e)
    }
}