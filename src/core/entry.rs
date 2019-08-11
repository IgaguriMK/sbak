//!ファイルシステムのスキャン結果の表現

use std::convert::{TryFrom, TryInto};
use std::path::PathBuf;

use failure::Fail;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::core::hash::HashID;
use crate::core::timestamp::Timestamp;

/// ファイルシステムの1エントリの表現
pub trait Entry: Serialize + DeserializeOwned {
    /// ハッシュ値によるIDを返す
    fn id(&self) -> Option<HashID>;
    /// IDを設定する
    fn set_id(&mut self, id: HashID);
    /// このエントリの属性を返す
    fn attr(&self) -> &Attributes;
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
    /// シンボリックリンク
    #[serde(rename = "symlink")]
    Symlink(SymlinkEntry),
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
            FsEntry::Symlink(ref x) => x.id(),
        }
    }

    fn set_id(&mut self, id: HashID) {
        match self {
            FsEntry::Dir(ref mut x) => x.set_id(id),
            FsEntry::File(ref mut x) => x.set_id(id),
            FsEntry::Symlink(ref mut x) => x.set_id(id),
        }
    }

    fn attr(&self) -> &Attributes {
        match self {
            FsEntry::Dir(ref x) => x.attr(),
            FsEntry::File(ref x) => x.attr(),
            FsEntry::Symlink(ref x) => x.attr(),
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
    children: Vec<FsHash>,
}

impl DirEntry {
    /// 子エントリのイテレータを返す。
    pub fn children(&self) -> impl Iterator<Item = &FsHash> {
        self.children.iter()
    }

    /// 指定された名前の子エントリを取得する。
    pub fn find_child(&self, name: &str) -> Option<&FsHash> {
        for ch in &self.children {
            if ch.attr().name() == name {
                return Some(ch);
            }
        }
        None
    }

    /// 指定された名前の子ディレクトリエントリを取得する。
    pub fn find_dir(&self, name: &str) -> Option<&DirHash> {
        match self.find_child(name) {
            Some(FsHash::Dir(x)) => Some(x),
            _ => None,
        }
    }

    /// 指定された名前の子ファイルエントリを取得する。
    pub fn find_file(&self, name: &str) -> Option<&FileHash> {
        match self.find_child(name) {
            Some(FsHash::File(x)) => Some(x),
            _ => None,
        }
    }
}

impl Entry for DirEntry {
    fn id(&self) -> Option<HashID> {
        self.id.clone()
    }

    fn set_id(&mut self, id: HashID) {
        self.id = Some(id);
    }

    fn attr(&self) -> &Attributes {
        &self.attr
    }
}

/// [`DirEntry`](struct.DirEntry.html)のBuilder
pub struct DirEntryBuilder {
    attr: Attributes,
    children: Vec<FsHash>,
}

impl DirEntryBuilder {
    /// 新たなBuilderを生成する。
    pub fn new(attr: Attributes) -> DirEntryBuilder {
        DirEntryBuilder {
            attr,
            children: Vec::new(),
        }
    }

    /// 子エントリのIDを追加する。
    pub fn append(&mut self, ch: FsHash) {
        self.children.push(ch);
    }

    /// 正規化された[`DirEntry`](struct.DirEntry.html)を生成する。
    pub fn build(mut self) -> DirEntry {
        self.children.sort();

        DirEntry {
            id: None,
            attr: self.attr,
            children: self.children,
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

    fn attr(&self) -> &Attributes {
        &self.attr
    }
}

/// シンボリックリンクの各種情報
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymlinkEntry {
    #[serde(skip_serializing)]
    #[serde(default)]
    id: Option<HashID>,
    attr: Attributes,
    target: String,
    is_dir: bool,
}

impl SymlinkEntry {
    /// 新たなシンボリックリンクエントリを生成する。
    pub fn new(attr: Attributes, target: String, is_dir: bool) -> SymlinkEntry {
        SymlinkEntry {
            id: None,
            attr,
            target,
            is_dir,
        }
    }

    /// シンボリックリンクのターゲットパスを返す。
    pub fn target(&self) -> PathBuf {
        PathBuf::from(&self.target)
    }

    /// シンボリックリンクがディレクトリを指すかどうかを返す。
    pub fn is_dir(&self) -> bool {
        self.is_dir
    }
}

impl Entry for SymlinkEntry {
    fn id(&self) -> Option<HashID> {
        self.id.clone()
    }

    fn set_id(&mut self, id: HashID) {
        self.id = Some(id);
    }

    fn attr(&self) -> &Attributes {
        &self.attr
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

    /// エントリの名前を取得する。
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 読み込み専用かどうかを取得する。
    pub fn readonly(&self) -> bool {
        self.readonly
    }

    /// 更新日時を取得する。
    pub fn modified(&self) -> Timestamp {
        self.modified
    }
}

/// エントリのハッシュ値と属性
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[allow(missing_docs)]
#[serde(tag = "type")]
pub enum FsHash {
    #[serde(rename = "dir")]
    Dir(DirHash),
    #[serde(rename = "file")]
    File(FileHash),
    #[serde(rename = "symlink")]
    Symlink(SymlinkHash),
}

impl FsHash {
    /// ハッシュ値を取得する。
    pub fn id(&self) -> HashID {
        match self {
            FsHash::Dir(x) => x.id(),
            FsHash::File(x) => x.id(),
            FsHash::Symlink(x) => x.id(),
        }
    }

    /// Attributesを取得する。
    pub fn attr(&self) -> &Attributes {
        match self {
            FsHash::Dir(x) => x.attr(),
            FsHash::File(x) => x.attr(),
            FsHash::Symlink(x) => x.attr(),
        }
    }
}

impl TryFrom<DirEntry> for FsHash {
    type Error = NoIdError;

    fn try_from(e: DirEntry) -> Result<Self, Self::Error> {
        e.try_into().map(FsHash::Dir)
    }
}

impl TryFrom<FileEntry> for FsHash {
    type Error = NoIdError;

    fn try_from(e: FileEntry) -> Result<Self, Self::Error> {
        e.try_into().map(FsHash::File)
    }
}

impl TryFrom<SymlinkEntry> for FsHash {
    type Error = NoIdError;

    fn try_from(e: SymlinkEntry) -> Result<Self, Self::Error> {
        e.try_into().map(FsHash::Symlink)
    }
}

impl From<DirHash> for FsHash {
    fn from(x: DirHash) -> FsHash {
        FsHash::Dir(x)
    }
}

impl From<FileHash> for FsHash {
    fn from(x: FileHash) -> FsHash {
        FsHash::File(x)
    }
}

impl From<SymlinkHash> for FsHash {
    fn from(x: SymlinkHash) -> FsHash {
        FsHash::Symlink(x)
    }
}

/// ディレクトリを表すディレクトリの子エントリ
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DirHash {
    attr: Attributes,
    id: HashID,
}

impl DirHash {
    /// ハッシュ値を取得する。
    pub fn id(&self) -> HashID {
        self.id.clone()
    }

    /// Attributesを取得する。
    pub fn attr(&self) -> &Attributes {
        &self.attr
    }
}

impl TryFrom<DirEntry> for DirHash {
    type Error = NoIdError;

    fn try_from(e: DirEntry) -> Result<Self, Self::Error> {
        if let Some(id) = e.id() {
            Ok(DirHash { attr: e.attr, id })
        } else {
            Err(NoIdError::NoId)
        }
    }
}

impl TryFrom<FsHash> for DirHash {
    type Error = MismatchHashType;

    fn try_from(h: FsHash) -> Result<Self, Self::Error> {
        match h {
            FsHash::Dir(x) => Ok(x),
            h => Err(MismatchHashType(h)),
        }
    }
}

/// ファイルを表すディレクトリの子エントリ
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FileHash {
    attr: Attributes,
    id: HashID,
}

impl FileHash {
    /// ハッシュ値を取得する。
    pub fn id(&self) -> HashID {
        self.id.clone()
    }

    /// Attributesを取得する。
    pub fn attr(&self) -> &Attributes {
        &self.attr
    }
}

impl TryFrom<FileEntry> for FileHash {
    type Error = NoIdError;

    fn try_from(e: FileEntry) -> Result<Self, Self::Error> {
        if let Some(id) = e.id() {
            Ok(FileHash { attr: e.attr, id })
        } else {
            Err(NoIdError::NoId)
        }
    }
}

impl TryFrom<FsHash> for FileHash {
    type Error = MismatchHashType;

    fn try_from(h: FsHash) -> Result<Self, Self::Error> {
        match h {
            FsHash::File(x) => Ok(x),
            h => Err(MismatchHashType(h)),
        }
    }
}

/// シンボリックリンクを表すディレクトリの子エントリ
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SymlinkHash {
    attr: Attributes,
    id: HashID,
}

impl SymlinkHash {
    /// ハッシュ値を取得する。
    pub fn id(&self) -> HashID {
        self.id.clone()
    }

    /// Attributesを取得する。
    pub fn attr(&self) -> &Attributes {
        &self.attr
    }
}

impl TryFrom<SymlinkEntry> for SymlinkHash {
    type Error = NoIdError;

    fn try_from(e: SymlinkEntry) -> Result<Self, Self::Error> {
        if let Some(id) = e.id() {
            Ok(SymlinkHash { attr: e.attr, id })
        } else {
            Err(NoIdError::NoId)
        }
    }
}

impl TryFrom<FsHash> for SymlinkHash {
    type Error = MismatchHashType;

    fn try_from(h: FsHash) -> Result<Self, Self::Error> {
        match h {
            FsHash::Symlink(x) => Ok(x),
            h => Err(MismatchHashType(h)),
        }
    }
}

/// エントリの[`FsHash`](struct.FsHash.html)への変換で発生しうるエラー
#[derive(Debug, Fail)]
pub enum NoIdError {
    /// IDが未設定である
    #[fail(display = "entry id isn't calculated")]
    NoId,
}

/// [`FsHash`](struct.FsHash.html)から[`DirHash`](struct.DirHash.html)や[`FileHash`](struct.FileHash.html)への変換で発生しうるエラー
#[derive(Debug, Fail)]
#[fail(display = "mismatch hash type")]
pub struct MismatchHashType(FsHash);
