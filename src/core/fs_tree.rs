//!ファイルシステムのスキャン結果の表現

pub mod io;

use std::path::{PathBuf, Path};

use serde::{Serialize, Deserialize};

/// 各種エントリに共通した属性取得操作のトレイト
pub trait Entry {
    /// スキャン原点となるパスからの相対パスを取得する。
    fn path(&self) -> &Path;
}

/// ファイルシステムの1エントリの表現
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FsEntry {
    Dir(DirEntry),
    File(FileEntry),
    // Symlink(SymlinkEntry) // TODO: シンボリックリンクを実装する
}

impl Entry for FsEntry {
    fn path(&self) -> &Path {
        match self {
            FsEntry::Dir(ref dir) => dir.path(),
            FsEntry::File(ref file) => file.path(),
        }
    }
}

impl From<DirEntry> for FsEntry {
    fn from(e: DirEntry) -> FsEntry {
        FsEntry::Dir(e)
    }
}

impl From<FileEntry> for FsEntry {
    fn from(e: FileEntry) -> FsEntry {
        FsEntry::File(e)
    }
}

/// ディレクトリの各種情報
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirEntry {
    path: PathBuf,
    meta: Metadata,
    childlen: Vec<FsEntry>,
}

impl DirEntry {
    pub fn new(path: PathBuf, meta: Metadata) -> DirEntry {
        DirEntry {
            path,
            meta,
            childlen: Vec::new(),
        }
    }

    pub fn append(&mut self, ch: FsEntry) {
        self.childlen.push(ch);
    }
}

impl Entry for DirEntry {
    fn path(&self) -> &Path {
        &self.path
    }
}

/// ファイルの各種情報
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileEntry {
    path: PathBuf,
    meta: Metadata,
}

impl FileEntry {
    pub fn new(path: PathBuf, meta: Metadata) -> FileEntry {
        FileEntry {
            path,
            meta,
        }
    }
}

impl Entry for FileEntry {
    fn path(&self) -> &Path {
        &self.path
    }
}

/// ファイルやディレクトリの属性
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Metadata {
    readonly: bool,
    modified: Timestamp,
}

impl Metadata {
    pub fn new(readonly: bool, modified: Timestamp) -> Metadata {
        Metadata {
            readonly,
            modified,
        }
    }
}

/// ファイルの更新日時
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Timestamp(u64);

impl Timestamp {
    pub fn from_unix_time(t: u64) -> Timestamp {
        Timestamp(t)
    }
}