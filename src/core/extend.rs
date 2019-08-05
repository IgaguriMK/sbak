//! ファイルを展開する

use std::collections::HashSet;
use std::convert::TryInto;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use failure::Fail;
use log::{info, trace};

use crate::core::entry::{DirEntry, Entry, FileHash, FsHash};
use crate::core::repo::{self, Bank, History};
use crate::core::timestamp::{self, Timestamp};

/// ファイルの展開を行う
#[derive(Debug)]
pub struct Extender<'a> {
    bank: &'a Bank<'a>,
    overwrite: bool,
    remove: bool,
}

impl<'a> Extender<'a> {
    /// 指定した[`Bank`](../repo/struct.Bank.html)からファイルを展開する`Extender`を生成する。
    pub fn new(bank: &'a Bank) -> Extender<'a> {
        Extender {
            bank,
            overwrite: false,
            remove: false,
        }
    }

    /// 上書きをするかどうかを設定する。
    pub fn allow_overwrite(&mut self, allow: bool) {
        self.overwrite = allow;
    }

    /// 削除をするかどうかを設定する。
    pub fn allow_remove(&mut self, allow: bool) {
        self.remove = allow;
    }

    /// 指定された`path`に`history`時点のファイルを展開する。
    pub fn extend<P: AsRef<Path>>(&self, target_path: P, history: &History) -> Result<()> {
        let path = target_path.as_ref();
        info!(
            "start extend to {:?} from {} {}",
            path,
            history.timestamp(),
            history.id()
        );
        let root_dir = self.bank.load_root(history)?;
        self.extend_dir(path, &root_dir)?;
        Ok(())
    }

    fn extend_dir(&self, path: &Path, dir_entry: &DirEntry) -> Result<()> {
        info!("extending directory {:?}", path);
        if !path.exists() {
            trace!("create dir {:?}", path);
            fs::create_dir(path)?;
        }

        if self.overwrite {
            // 内部にファイルを展開するために書き込みを許可
            let mut permission = fs::metadata(path)?.permissions();
            permission.set_readonly(false);
        }
        let mut exists = HashSet::<PathBuf>::new();

        for ch in dir_entry.children() {
            let attr = ch.attr();
            let ch_path = path.join(attr.name());

            match ch {
                FsHash::Dir(ref dir) => {
                    let ch_dir = self.bank.load_dir_entry(&dir.id())?;
                    self.extend_dir(&ch_path, &ch_dir)?;
                }
                FsHash::File(ref file) => {
                    self.extend_file(&ch_path, file)?;
                }
            }

            exists.insert(ch_path);
        }

        for ch in fs::read_dir(path)? {
            let ch = ch?;
            let ch_path = ch.path();
            if !exists.contains(&ch_path) {
                let typ = ch.file_type()?;

                if typ.is_dir() {
                    if self.remove {
                        info!("removing directory {:?}", ch_path);
                        fs::remove_dir_all(&ch_path)?;
                    } else {
                        info!("skip remove directory {:?}", ch_path);
                    }
                }
                if typ.is_file() {
                    if self.remove {
                        info!("removing file {:?}", ch_path);
                        fs::remove_file(&ch_path)?;
                    } else {
                        info!("skip remove file {:?}", ch_path);
                    }
                }
            }
        }

        if self.overwrite {
            let mut permission = fs::metadata(path)?.permissions();
            trace!(
                "overrite permission readonly={}",
                dir_entry.attr().readonly()
            );
            permission.set_readonly(dir_entry.attr().readonly());
        }

        info!("extended directory {:?}", path);
        Ok(())
    }

    fn extend_file(&self, path: &Path, file_hash: &FileHash) -> Result<()> {
        info!("extending file {:?}", path);
        let exists = path.exists();
        if exists && !self.overwrite {
            info!("skip existing file {:?}", path);
            return Ok(());
        }

        if exists {
            let meta = fs::metadata(path)?;
            let timestamp: Timestamp = meta.modified()?.try_into()?;

            trace!(
                "filesystem = {}, backup = {}",
                timestamp,
                file_hash.attr().modified()
            );
            if timestamp == file_hash.attr().modified() {
                info!("skip same timestamp: {:?}", path);
                return Ok(());
            }
        }

        info!("checking file checksum for {}", file_hash.id());
        let mut f = self.bank.open_object(&file_hash.id())?;
        let mut out = fs::File::create(path)?;
        info!("extracting file to {:?}", path);
        io::copy(&mut f, &mut out)?;

        if !exists || self.overwrite {
            let mut permission = fs::metadata(path)?.permissions();
            trace!(
                "overrite permission readonly={}",
                file_hash.attr().readonly()
            );
            permission.set_readonly(file_hash.attr().readonly());
        }

        Ok(())
    }
}

type Result<T> = std::result::Result<T, Error>;

/// ファイルシステムのスキャンで発生しうるエラー
#[derive(Debug, Fail)]
pub enum Error {
    /// 入出力エラー
    #[fail(display = "failed scan with IO error: {}", _0)]
    IO(#[fail(cause)] io::Error),

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

impl From<repo::Error> for Error {
    fn from(e: repo::Error) -> Error {
        Error::Repo(e)
    }
}

impl From<timestamp::Error> for Error {
    fn from(_e: timestamp::Error) -> Error {
        Error::Timestamp
    }
}
