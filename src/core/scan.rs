//! ファイルやディレクトリのスキャンを行う。

use std::convert::{TryFrom, TryInto};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use failure::Fail;

use serde_json::to_writer;

use crate::core::entry::*;
use crate::core::hash::{self, hash_reader};
use crate::core::repo::Bank;
use crate::core::timestamp::{self, Timestamp};

#[derive(Debug)]
pub struct Scanner<'a> {
    last_scan: Timestamp,
    bank: Bank<'a>,
}

impl<'a> Scanner<'a> {
    pub fn new(bank: Bank<'a>) -> Scanner {
        Scanner {
            last_scan: Timestamp::default(),
            bank,
        }
    }

    pub fn set_last_scan(&mut self, timestamp: Timestamp) {
        self.last_scan = timestamp;
    }

    pub fn scan<P: AsRef<Path>>(self, path: P) -> Result<()> {
        let scan_start = Timestamp::now()?;

        let id = self.scan_i(path.as_ref())?;
        self.bank.save_history(id.id(), scan_start)?;

        Ok(())
    }

    fn scan_i(&self, p: &Path) -> Result<FsHash> {
        eprintln!("{:?}", p);
        let fs_meta = fs::metadata(p)?;
        let attr = convert_metadata(p, &fs_meta)?;

        if fs_meta.is_dir() {
            Ok(self.scan_dir(p, attr)?)
        } else if fs_meta.is_file() {
            Ok(self.scan_file(p, attr)?)
        } else {
            panic!("{:?} is not dir nor file", p)
        }
    }

    fn scan_dir(&self, p: &Path, attr: Attributes) -> Result<FsHash> {
        let mut builder = DirEntryBuilder::new(attr);

        for ch in fs::read_dir(p)? {
            let ch = ch?;
            let ch_hash = self.scan_i(&ch.path())?;
            builder.append(ch_hash);
        }

        let mut entry = builder.build();

        let mut encoded = Vec::<u8>::new();
        to_writer(&mut encoded, &entry)?;

        let (id, temp) = hash_reader(encoded.as_slice())?;
        entry.set_id(id.clone());

        self.bank.save_object(id, temp)?;

        Ok(FsHash::try_from(entry).unwrap())
    }

    fn scan_file(&self, p: &Path, attr: Attributes) -> Result<FsHash> {
        let mut entry = FileEntry::new(attr);

        let f = fs::File::open(p)?;
        let (id, temp) = hash_reader(f)?;
        entry.set_id(id.clone());

        self.bank.save_object(id, temp)?;

        Ok(FsHash::try_from(entry).unwrap())
    }
}

fn convert_metadata(path: &Path, fs_meta: &fs::Metadata) -> Result<Attributes> {
    if let Some(name) = path.file_name() {
        let readonly = fs_meta.permissions().readonly();
        let timestamp = fs_meta.modified()?.try_into()?;

        let name = name
            .to_str()
            .ok_or_else(|| Error::NameIsInvalidUnicode(path.to_owned()))?;

        Ok(Attributes::new(name.to_owned(), readonly, timestamp))
    } else {
        Err(Error::NameIsEmpty(path.to_owned()))
    }
}

pub type Result<T> = std::result::Result<T, Error>;

/// ファイルシステムのスキャンで発生しうるエラー
#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "failed parse FsEntry: {}", _0)]
    Encode(#[fail(cause)] serde_json::Error),

    #[fail(display = "failed scan with IO error: {}", _0)]
    IO(#[fail(cause)] io::Error),

    #[fail(display = "found empty name entry at {:?}", _0)]
    NameIsEmpty(PathBuf),

    #[fail(display = "found empty name entry at {:?}", _0)]
    NameIsInvalidUnicode(PathBuf),

    #[fail(display = "path is out of base path: {:?}", _0)]
    OutOfBaseDir(PathBuf),

    #[fail(display = "timestamp is older than UNIX epoch")]
    Timestamp,
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::IO(e)
    }
}

impl From<timestamp::Error> for Error {
    fn from(_e: timestamp::Error) -> Error {
        Error::Timestamp
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Error {
        Error::Encode(e)
    }
}

impl From<hash::Error> for Error {
    fn from(e: hash::Error) -> Error {
        match e {
            hash::Error::IO(e) => Error::IO(e),
        }
    }
}
