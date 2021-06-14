//! 保存されるデータモデルのモジュール

use std::fmt;

use chrono::{DateTime, Utc};
use hex::encode;
use serde::{Deserialize, Serialize};

/// リポジトリに保存されたデータのIDとなるハッシュ値
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct HashId(String);

impl HashId {
    /// ハッシュ値のバイト列から [`HashId`] を生成する。
    pub fn from_bytes(bs: &[u8]) -> HashId {
        HashId(encode(bs))
    }
}

impl fmt::Display for HashId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// バックアップの世代を表す
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BackupPoint {
    tree_id: HashId,
    started_at: DateTime<Utc>,
    finished_at: DateTime<Utc>,
    memo: Option<String>,
}

/// ファイルシステムの1つのノードを表す。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Node {
    data: NodeData,
}

/// フォルダ内のノード一覧を表す。
///
/// [`DirData::data_id()`] が指すファイルの中身はこの型である。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NodeList {
    nodes: Vec<Node>,
}

/// ファイルシステムのノードのデータを表す
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum NodeData {
    /// ファイル
    File(FileData),
    /// フォルダ
    Dir(DirData),
}

/// ファイルのデータを表す
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FileData {
    modified: DateTime<Utc>,
    data_id: HashId,
    size: u64,
}

impl FileData {
    /// データのID
    ///
    /// IDが指すデータの中身はファイルの中身である。
    pub fn data_id(&self) -> &HashId {
        &self.data_id
    }
}

/// フォルダのデータを表す
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DirData {
    data_id: HashId,
    node_count: u64,
}

impl DirData {
    /// データのID
    ///
    /// データの中身は [`NodeList`] である。
    pub fn data_id(&self) -> &HashId {
        &self.data_id
    }
}
