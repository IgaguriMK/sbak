//!ファイルシステムのスキャン結果の表現

use std::convert::TryFrom;
use std::path::{Path, PathBuf};

use failure::Fail;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;

use crate::core::hash::HashID;
use crate::core::timestamp::Timestamp;

/// ファイルシステムの1エントリの表現
pub trait Entry: Serialize + DeserializeOwned {
    /// ハッシュ値によるIDを返す
    fn id(&self) -> Option<HashID>;
    /// IDを設定する
    fn set_id(&mut self, id: HashID);
    /// このエントリのパスを返す
    fn path(&self, parent: &Path) -> PathBuf;
}

/// エントリの実表現
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FsEntry {
    /// ディレクトリ
    #[serde(rename = "dir")]
    Dir(DirEntry),
    /// ファイル
    #[serde(rename = "file")]
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
    #[serde(skip_serializing)]
    #[serde(default)]
    id: Option<HashID>,
    attr: Attributes,
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

/// [`DirEntry`](struct.DirEntry.html)のBuilder
pub struct DirEntryBuilder {
    attr: Attributes,
    childlen: Vec<FsHash>,
}

impl DirEntryBuilder {
    /// 新たなBuilderを生成する。
    pub fn new(attr: Attributes) -> DirEntryBuilder {
        DirEntryBuilder {
            attr,
            childlen: Vec::new(),
        }
    }

    /// 子エントリのIDを追加する。
    pub fn append(&mut self, ch: FsHash) {
        self.childlen.push(ch);
    }

    /// 正規化された[`DirEntry`](struct.DirEntry.html)を取得する。
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
    #[serde(skip_serializing)]
    #[serde(default)]
    id: Option<HashID>,
    attr: Attributes,
}

impl FileEntry {
    /// 新たなファイルエントリを生成する。
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
    name: String,
    readonly: bool,
    modified: Timestamp,
}

impl Attributes {
    /// Attributesを生成する。
    pub fn new(name: String, readonly: bool, modified: Timestamp) -> Attributes {
        Attributes {
            name,
            readonly,
            modified,
        }
    }
}

/// エントリのハッシュ値と属性
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[allow(missing_docs)]
#[serde(tag="type")]
pub enum FsHash {
    #[serde(rename = "dir")]
    Dir { attr: Attributes, id: HashID },
    #[serde(rename = "file")]
    File { attr: Attributes, id: HashID },
}

impl FsHash {
    /// ハッシュ値を取得する
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

/// エントリの`FsHash`への変換で発生しうるエラー
#[derive(Debug, Fail)]
pub enum NoIdError {
    /// IDが未設定である
    #[fail(display = "entry id isn't calculated")]
    NoId,
}
