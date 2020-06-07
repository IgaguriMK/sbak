//! ファイルやディレクトリのスキャンを行う。

use std::convert::{TryFrom, TryInto};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use log::{info, trace, warn};
use serde_json::to_writer;

use crate::core::entry::*;
use crate::core::hash::{self, hash as hash_file, hash_reader, HashID};
use crate::core::ignore::{self, IgnoreStack};
use crate::core::repo::{self, Bank};
use crate::core::timestamp;

/// 更新されたファイルやディレクトリをスキャンするスキャナ
#[derive(Debug)]
pub struct Scanner<'a> {
    bank: &'a Bank<'a>,
}

impl<'a> Scanner<'a> {
    /// 指定された`Bank`に保存する、デフォルト設定のスキャナを生成する
    pub fn new(bank: &'a Bank<'a>) -> Scanner {
        Scanner { bank }
    }

    /// Bankの対象ディレクトリをスキャンする
    pub fn scan(&self) -> Result<FsHash> {
        let path = self.bank.target_path();
        trace!("scan root path = {:?}", path);
        let last_id = self.bank.last_scan()?.map(|e| e.id().clone());
        trace!("last_scan root entry id = {:?}", last_id);
        let attr = convert_metadata(path, &fs::metadata(path)?)?;

        trace!("load ing bank ignore patterns");
        let ignore_patterns = self.bank.load_ignore_patterns()?;
        let ignore_stack = IgnoreStack::new(path, ignore_patterns);

        trace!("start scan root dir");
        let id = self.scan_dir(path, &ignore_stack, attr, last_id)?;

        Ok(id)
    }

    fn scan_node(
        &self,
        p: &Path,
        ignore_stack: &IgnoreStack,
        last_entry: Option<&FsHash>,
    ) -> Result<Option<FsHash>> {
        match self.scan_node_inner(p, ignore_stack, last_entry) {
            Ok(v) => Ok(v),
            Err(Error::IO(e)) => {
                if e.kind() == io::ErrorKind::PermissionDenied {
                    Ok(None)
                } else {
                    Err(Error::IO(e))
                }
            }
            Err(e) => Err(e),
        }
    }

    fn scan_node_inner(
        &self,
        p: &Path,
        ignore_stack: &IgnoreStack,
        last_entry: Option<&FsHash>,
    ) -> Result<Option<FsHash>> {
        info!("{:?}", p);
        let fs_meta = fs::symlink_metadata(p)?;
        let attr = convert_metadata(p, &fs_meta)?;
        trace!("{:?}: {:?}", p, attr);

        let file_type = fs_meta.file_type();
        if file_type.is_dir() {
            trace!("{:?} is dir.", p);
            let dir_hash = self.scan_dir(p, ignore_stack, attr, last_entry.map(|x| x.id()))?;
            Ok(Some(dir_hash))
        } else if file_type.is_file() {
            trace!("{:?} is file.", p);
            let old_hash = last_entry.and_then(|h| h.clone().try_into().ok());
            let file_hash = self.scan_file(p, attr, old_hash)?;
            Ok(Some(file_hash))
        } else if file_type.is_symlink() {
            let symlink_hash = self.scan_symlink(p, attr)?;
            Ok(Some(symlink_hash))
        } else {
            warn!("{:?} is not dir nor file", p);
            Ok(None)
        }
    }

    fn scan_dir(
        &self,
        p: &Path,
        ignore_stack: &IgnoreStack,
        attr: Attributes,
        last_id: Option<HashID>,
    ) -> Result<FsHash> {
        trace!("scan dir {:?}", p);
        let old_entry = if let Some(ref id) = last_id {
            trace!("dir has last_id = {}", id);
            self.bank.load_entry(id)?
        } else {
            trace!("dir has no last_id");
            DirEntryBuilder::new(attr.clone()).build()
        };

        let current_stack = ignore_stack.child(attr.name().to_owned())?;
        trace!("IGNORE STACK = {:?}", current_stack);

        let mut builder = DirEntryBuilder::new(attr);

        trace!("start scan dir children: {:?}", p);
        for ch in fs::read_dir(p)? {
            let ch = ch?;
            let name = ch
                .file_name()
                .into_string()
                .map_err(|_| Error::NameIsInvalidUnicode(ch.path()))?;
            trace!("child name = {}", name);

            let fs_meta = fs::metadata(p)?;
            if current_stack.ignored(&ch.path(), fs_meta.is_dir())? {
                trace!("ignore {:?}", ch.path());
                continue;
            }

            if let Some(ch_hash) =
                self.scan_node(&ch.path(), &current_stack, old_entry.find_child(&name))?
            {
                builder.append(ch_hash);
            }
        }
        trace!("finish scan dir children: {:?}", p);

        let mut entry = builder.build();

        trace!("start encode dir entry {:?}", p);
        let mut encoded = Vec::<u8>::new();
        to_writer(&mut encoded, &entry)?;

        trace!("start hash dir entry {:?}", p);
        let (id, temp) = hash_reader(encoded.as_slice())?;
        trace!("start save dir entry {:?} = {}", p, id);
        self.bank.save_object(&id, temp)?;
        trace!("dir entry saved {:?} = {}", p, id);

        entry.set_id(id);

        Ok(FsHash::try_from(entry).unwrap())
    }

    fn scan_file(
        &self,
        p: &Path,
        attr: Attributes,
        last_entry: Option<FileHash>,
    ) -> Result<FsHash> {
        trace!("scan file {:?}", p);

        if let Some(last_entry) = last_entry {
            if last_entry.attr().modified() == attr.modified() {
                trace!("skip scan file {:?}", p);
                return Ok(last_entry.into());
            }
        }

        let mut entry = FileEntry::new(attr);

        trace!("start scan file {:?}", p);
        let mut f = fs::File::open(p)?;
        let id = hash_file(&mut f)?;
        trace!("file hash {:?} = {}", p, id);
        trace!("start save file object {}", id);
        self.bank.save_object(&id, f)?;
        trace!("finish save file object {}", id);

        entry.set_id(id);

        Ok(FsHash::try_from(entry).unwrap())
    }

    fn scan_symlink(&self, p: &Path, attr: Attributes) -> Result<FsHash> {
        trace!("scan symlink {:?}", p);

        let target = fs::read_link(p)?;
        info!("symlink: {:?} => {:?}", p, target);
        let target_path_str = target
            .to_str()
            .ok_or_else(|| Error::NameIsInvalidUnicode(target.to_owned()))?
            .to_owned();

        let target_meta = fs::metadata(p)?;

        let mut entry = SymlinkEntry::new(attr, target_path_str, target_meta.is_dir());

        trace!("start encode dir entry {:?}", p);
        let mut encoded = Vec::<u8>::new();
        to_writer(&mut encoded, &entry)?;

        trace!("start hash symlink entry {:?}", p);
        let (id, temp) = hash_reader(encoded.as_slice())?;
        trace!("start save symlink entry {:?} = {}", p, id);
        self.bank.save_object(&id, temp)?;
        trace!("symlink entry saved {:?} = {}", p, id);

        entry.set_id(id);

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
type Result<T> = std::result::Result<T, Error>;

/// ファイルシステムのスキャンで発生しうるエラー
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// エントリのJSONへのエンコードの失敗
    #[error("failed parse FsEntry: {0}")]
    Encode(#[from] serde_json::Error),

    /// 除外判定に失敗した。
    #[error("failed load ignore patterns: {0}")]
    Ignore(#[from] ignore::Error),

    /// 入出力エラー
    #[error("failed scan with IO error: {0}")]
    IO(#[from] io::Error),

    /// 名前が空文字列である要素を発見した
    #[error("found empty name entry at {:?}", _0)]
    NameIsEmpty(PathBuf),

    /// パスがUnicodeで表現できない
    #[error("cannot convert file name to unicode {:?}", _0)]
    NameIsInvalidUnicode(PathBuf),

    /// リポジトリ操作エラーが発生
    #[error("{0}")]
    Repo(#[from] repo::Error),

    /// 対応範囲外のタイムスタンプを検出
    #[error("timestamp is invalid: {0}")]
    Timestamp(#[from] timestamp::Error),
}

impl From<hash::Error> for Error {
    fn from(e: hash::Error) -> Error {
        Error::IO(e.into())
    }
}
