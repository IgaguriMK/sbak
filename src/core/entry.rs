//!ファイルシステムのスキャン結果の表現

pub mod io;

use std::convert::TryFrom;
use std::path::{Path, PathBuf};

use failure::Fail;
use serde::{Deserialize, Serialize};

use crate::core::hash::HashID;
use crate::core::timestamp::Timestamp;

pub trait Entry {
    fn id(&self) -> Option<HashID>;
    fn set_id(&mut self, id: HashID);
    fn path(&self, parent: &Path) -> PathBuf;
}

/// ファイルシステムの1エントリの表現
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FsEntry {
    #[serde(rename = "d")]
    Dir(DirEntry),
    #[serde(rename = "f")]
    File(FileEntry),
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

impl Entry for FsEntry {
    fn id(&self) -> Option<HashID> {
        match self {
            FsEntry::Dir(ref x) => x.id(),
            FsEntry::File(ref x) => x.id(),
        }
    }

    fn set_id(&mut self, id: HashID) {
        match self {
            FsEntry::Dir(ref mut x) => x.set_id(id),
            FsEntry::File(ref mut x) => x.set_id(id),
        }
    }

    fn path(&self, parent: &Path) -> PathBuf {
        match self {
            FsEntry::Dir(ref x) => x.path(parent),
            FsEntry::File(ref x) => x.path(parent),
        }
    }
}

/// ディレクトリの各種情報
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirEntry {
    #[serde(rename = "id")]
    id: Option<HashID>,
    #[serde(rename = "a")]
    attr: Attributes,
    #[serde(rename = "ch")]
    childlen: Vec<FsHash>,
}

impl Entry for DirEntry {
    fn id(&self) -> Option<HashID> {
        self.id.clone()
    }

    fn set_id(&mut self, id: HashID) {
        self.id = Some(id);
    }

    fn path(&self, parent: &Path) -> PathBuf {
        parent.join(self.attr.name.to_owned())
    }
}

pub struct DirEntryBuilder {
    attr: Attributes,
    childlen: Vec<FsHash>,
}

impl DirEntryBuilder {
    pub fn new(attr: Attributes) -> DirEntryBuilder {
        DirEntryBuilder {
            attr,
            childlen: Vec::new(),
        }
    }

    pub fn append(&mut self, ch: FsHash) {
        self.childlen.push(ch);
    }

    pub fn build(mut self) -> DirEntry {
        self.childlen.sort();

        DirEntry {
            id: None,
            attr: self.attr,
            childlen: self.childlen,
        }
    }
}

/// ファイルの各種情報
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileEntry {
    #[serde(rename = "id")]
    id: Option<HashID>,
    #[serde(rename = "a")]
    attr: Attributes,
}

impl FileEntry {
    pub fn new(attr: Attributes) -> FileEntry {
        FileEntry { id: None, attr }
    }
}

impl Entry for FileEntry {
    fn id(&self) -> Option<HashID> {
        self.id.clone()
    }

    fn set_id(&mut self, id: HashID) {
        self.id = Some(id);
    }

    fn path(&self, parent: &Path) -> PathBuf {
        parent.join(self.attr.name.to_owned())
    }
}

/// ファイルやディレクトリの属性
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Attributes {
    #[serde(rename = "n")]
    name: String,
    #[serde(rename = "r")]
    readonly: bool,
    #[serde(rename = "mod")]
    modified: Timestamp,
}

impl Attributes {
    pub fn new(name: String, readonly: bool, modified: Timestamp) -> Attributes {
        Attributes {
            name,
            readonly,
            modified,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FsHash {
    #[serde(rename = "d")]
    Dir { attr: Attributes, id: HashID },
    #[serde(rename = "f")]
    File { attr: Attributes, id: HashID },
}

impl FsHash {
    pub fn id(&self) -> HashID {
        match self {
            FsHash::Dir { id, .. } => id.clone(),
            FsHash::File { id, .. } => id.clone(),
        }
    }
}

impl TryFrom<DirEntry> for FsHash {
    type Error = NoIdError;

    fn try_from(e: DirEntry) -> Result<Self, Self::Error> {
        if let Some(id) = e.id() {
            Ok(FsHash::Dir { attr: e.attr, id })
        } else {
            Err(NoIdError::NoId)
        }
    }
}

impl TryFrom<FileEntry> for FsHash {
    type Error = NoIdError;

    fn try_from(e: FileEntry) -> Result<Self, Self::Error> {
        if let Some(id) = e.id() {
            Ok(FsHash::File { attr: e.attr, id })
        } else {
            Err(NoIdError::NoId)
        }
    }
}

#[derive(Debug, Fail)]
pub enum NoIdError {
    #[fail(display = "entry id isn't calculated")]
    NoId,
}
