//!ファイルシステムのスキャン結果の表現

pub mod io;

use std::path::PathBuf;

use serde::{Deserialize, Serialize};


/// ファイルシステムの1エントリの表現
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FsEntry {
    #[serde(rename = "d")]
    Dir(DirEntry),
    #[serde(rename = "f")]
    File(FileEntry),
    // Symlink(SymlinkEntry) // TODO: シンボリックリンクを実装する
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
    #[serde(rename = "a")]
    attr: Attributes,
    #[serde(rename = "ch")]
    childlen: Vec<FsEntry>,
}

impl DirEntry {
    pub fn new(attr: Attributes) -> DirEntry {
        DirEntry {
            attr,
            childlen: Vec::new(),
        }
    }

    pub fn append(&mut self, ch: FsEntry) {
        self.childlen.push(ch);
    }
}

/// ファイルの各種情報
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileEntry {
    #[serde(rename = "a")]
    attr: Attributes,
}

impl FileEntry {
    pub fn new(attr: Attributes) -> FileEntry {
        FileEntry { attr }
    }
}

/// ファイルやディレクトリの属性
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attributes {
    #[serde(rename = "p")]
    path: PathBuf,
    #[serde(rename = "r")]
    readonly: bool,
    #[serde(rename = "mod")]
    modified: Timestamp,
}

impl Attributes {
    pub fn new(path: PathBuf, readonly: bool, modified: Timestamp) -> Attributes {
        Attributes {path,  readonly, modified }
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
