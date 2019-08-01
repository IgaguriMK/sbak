//! ファイルやディレクトリのスキャンを行う。

use std::convert::{TryFrom, TryInto};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use failure::Fail;

use serde_json::to_writer;

use crate::core::entry::*;
use crate::core::hash::{self, hash_reader, HashID};
use crate::core::repo::{self, Bank};
use crate::core::timestamp::{self, Timestamp};

/// 更新されたファイルやディレクトリをスキャンするスキャナ
#[derive(Debug)]
pub struct Scanner<'a> {
    bank: Bank<'a>,
}

impl<'a> Scanner<'a> {
    /// 指定された`Bank`に保存する、デフォルト設定のスキャナを生成する
    pub fn new(bank: Bank<'a>) -> Scanner {
        Scanner { bank }
    }

    /// 指定ディレクトリをスキャンする
    pub fn scan(&self) -> Result<()> {
        let scan_start = Timestamp::now()?;

        let path = self.bank.target_path();
        let last_id = self.bank.last_scan()?.map(|e| e.id().clone());
        let attr = convert_metadata(path, &fs::metadata(path)?)?;

        let id = self.scan_dir(path, attr, last_id)?;
        self.bank.save_history(id.id(), scan_start)?;

        Ok(())
    }

    fn scan_i(&self, p: &Path, last_entry: Option<&FsHash>) -> Result<FsHash> {
        eprintln!("{:?}", p);
        let fs_meta = fs::metadata(p)?;
        let attr = convert_metadata(p, &fs_meta)?;

        if fs_meta.is_dir() {
            Ok(self.scan_dir(p, attr, last_entry.map(|x| x.id()))?)
        } else if fs_meta.is_file() {
            let old_hash = last_entry.and_then(|h| h.clone().try_into().ok());
            let file_hash = self.scan_file(p, attr, old_hash)?;
            Ok(file_hash)
        } else {
            panic!("{:?} is not dir nor file", p)
        }
    }

    fn scan_dir(&self, p: &Path, attr: Attributes, last_id: Option<HashID>) -> Result<FsHash> {
        let old_entry = if let Some(ref id) = last_id {
            self.bank.load_dir_entry(id)?
        } else {
            DirEntryBuilder::new(attr.clone()).build()
        };

        let mut builder = DirEntryBuilder::new(attr);

        for ch in fs::read_dir(p)? {
            let ch = ch?;
            let name = ch
                .file_name()
                .into_string()
                .map_err(|_| Error::NameIsInvalidUnicode(ch.path().to_owned()))?;
            let ch_hash = self.scan_i(&ch.path(), old_entry.find_child(&name))?;
            builder.append(ch_hash);
        }

        let mut entry = builder.build();

        let mut encoded = Vec::<u8>::new();
        to_writer(&mut encoded, &entry)?;

        let (id, temp) = hash_reader(encoded.as_slice())?;
        self.bank.save_object(&id, temp)?;

        entry.set_id(id.clone());

        Ok(FsHash::try_from(entry).unwrap())
    }

    fn scan_file(
        &self,
        p: &Path,
        attr: Attributes,
        last_entry: Option<FileHash>,
    ) -> Result<FsHash> {
        if let Some(last_entry) = last_entry {
            if last_entry.attr().modified() == attr.modified() {
                eprintln!("skip.");
                return Ok(last_entry.into());
            }
        }

        let mut entry = FileEntry::new(attr);

        let f = fs::File::open(p)?;
        let (id, temp) = hash_reader(f)?;
        self.bank.save_object(&id, temp)?;

        entry.set_id(id.clone());

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

#[allow(missing_docs)]
pub type Result<T> = std::result::Result<T, Error>;

/// ファイルシステムのスキャンで発生しうるエラー
#[derive(Debug, Fail)]
pub enum Error {
    /// エントリのJSONへのエンコードの失敗
    #[fail(display = "failed parse FsEntry: {}", _0)]
    Encode(#[fail(cause)] serde_json::Error),

    /// 入出力エラー
    #[fail(display = "failed scan with IO error: {}", _0)]
    IO(#[fail(cause)] io::Error),

    /// 名前が空文字列である要素を発見した
    #[fail(display = "found empty name entry at {:?}", _0)]
    NameIsEmpty(PathBuf),

    /// パスがUnicodeで表現できない
    #[fail(display = "found empty name entry at {:?}", _0)]
    NameIsInvalidUnicode(PathBuf),

    /// リポジトリ操作エラーが発生
    #[fail(display = "{}", _0)]
    Repo(#[fail(cause)] repo::Error),

    /// 対応範囲外のタイムスタンプを検出
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

impl From<repo::Error> for Error {
    fn from(e: repo::Error) -> Error {
        Error::Repo(e)
    }
}
