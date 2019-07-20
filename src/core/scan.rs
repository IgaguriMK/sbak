//! ファイルやディレクトリのスキャンを行う。

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTimeError, UNIX_EPOCH};

use failure::Fail;

use crate::core::fs_tree::*;

#[derive(Debug, Clone, Default)]
pub struct Scanner {}

impl Scanner {
    pub fn new() -> Scanner {
        Scanner {}
    }

    pub fn scan<P: AsRef<Path>>(&self, p: P) -> Result<FsEntry> {
        let p = p.as_ref();
        self.scan_i(p, p)
    }

    fn scan_i(&self, base: &Path, p: &Path) -> Result<FsEntry> {
        let fs_meta = fs::metadata(p)?;
        let attr = convert_metadata(strip_path(base, p)?, &fs_meta)?;
        if fs_meta.is_dir() {
            Ok(self.scan_dir(base, p, attr)?.into())
        } else if fs_meta.is_file() {
            Ok(FileEntry::new(attr).into())
        } else {
            panic!("{:?} is not dir nor file", p)
        }
    }

    fn scan_dir(&self, base: &Path, p: &Path, attr: Attributes) -> Result<DirEntry> {
        let mut entry = DirEntry::new(attr);

        for ch in fs::read_dir(p)? {
            let ch = ch?;
            entry.append(self.scan_i(base, &ch.path())?);
        }

        Ok(entry)
    }
}

fn convert_metadata(path: PathBuf, fs_meta: &fs::Metadata) -> Result<Attributes> {
    let readonly = fs_meta.permissions().readonly();
    let unix_time_u64 = fs_meta.modified()?.duration_since(UNIX_EPOCH)?.as_secs();
    let timestamp = Timestamp::from_unix_time(unix_time_u64);

    Ok(Attributes::new(path, readonly, timestamp))
}

fn strip_path(base: &Path, path: &Path) -> Result<PathBuf> {
    let p = path
        .strip_prefix(base)
        .map_err(|_| Error::OutOfBaseDir(path.to_owned()))?;
    Ok(p.to_owned())
}

type Result<T> = std::result::Result<T, Error>;

/// ファイルシステムのスキャンで発生しうるエラー
#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "failed scan with IO error: {}", _0)]
    IO(#[fail(cause)] io::Error),
    #[fail(display = "timestamp is older than UNIX epoch")]
    Timestamp,
    #[fail(display = "path is out of base path: {:?}", _0)]
    OutOfBaseDir(PathBuf),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::IO(e)
    }
}

impl From<SystemTimeError> for Error {
    fn from(_e: SystemTimeError) -> Error {
        Error::Timestamp
    }
}
