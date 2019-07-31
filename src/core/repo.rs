//! バックアップ先となるリポジトリの操作

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use failure::Fail;
use serde::{Deserialize, Serialize};
use serde_json::{self, from_reader, to_writer};

use crate::core::entry::DirEntry;
use crate::core::hash::{self, HashID};
use crate::core::timestamp::Timestamp;

/// バックアップ先となるリポジトリのディレクトリを管理する型。
///
/// リポジトリは共通のファイル本体を格納する`objects`ディレクトリと、`banks`以下にバックアップ元ごとに対応した[`Bank`](struct.Bank.html)を0個以上持つ。
#[derive(Debug)]
pub struct Repository {
    path: PathBuf,
    objects_dir: PathBuf,
    banks_dir: PathBuf,
}

impl Repository {
    /// 既存のリポジトリを開く。
    ///
    /// # Failures
    ///
    /// 必須のリポジトリとして必要なルートディレクトリ、`objects`ディレクトリ、`banks`ディレクトリのどれかが存在しないか書き込み不可能な場合、[`Error::IncompleteRepo`](enum.Error.html)を返す。
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Repository, Error> {
        // TODO: Readonlyでも読み込み用に開けるようにする。
        let repo = Repository::new(path);

        check_path(&repo.path, "repository directory")?;
        check_path(&repo.objects_dir, "/object")?;
        check_path(&repo.banks_dir, "/banks")?;

        Ok(repo)
    }

    /// リポジトリを生成する。
    ///
    /// # Failures
    ///
    /// 生成に失敗した場合、[`Error::IO`](enum.Error.html)を返す。
    pub fn create<P: AsRef<Path>>(path: P) -> Result<Repository, Error> {
        // TODO: 読み込み専用の場合エラーにする。
        let repo = Repository::new(path);

        ensure_dir(&repo.path)?;
        ensure_dir(&repo.objects_dir)?;
        ensure_dir(&repo.banks_dir)?;

        Ok(repo)
    }

    // ディレクトリの存在を保証するため、`new`は内部専用。
    fn new<P: AsRef<Path>>(path: P) -> Repository {
        let path = path.as_ref().to_owned();
        let objects_dir = path.join("objects");
        let banks_dir = path.join("banks");

        Repository {
            path,
            objects_dir,
            banks_dir,
        }
    }

    /// 指定された名前の[`Bank`](struct.Bank.html)を開く。
    pub fn open_bank<'a>(&'a self, name: &str) -> Bank<'a> {
        let bank_dir = self.bank_path(name);
        Bank::new(self, bank_dir)
    }

    /// 指定された名前の[`Bank`](struct.Bank.html)を作成する。
    pub fn create_bank(&self, name: &str) -> Result<(), Error> {
        let bank_dir = self.bank_path(name);
        let bank = Bank::new(self, bank_dir);
        bank.create_dir()?;
        Ok(())
    }

    /// 指定された名前のbankがあるかどうかチェックする。
    pub fn bank_exists(&self, name: &str) -> Result<bool, Error> {
        let bank_dir = self.bank_path(name);
        Ok(bank_dir.exists())
    }

    fn save_object(&self, id: &HashID, mut temp: fs::File) -> Result<(), io::Error> {
        let out_path = self.object_path(id);

        let out_dir = out_path.parent().unwrap();
        fs::create_dir_all(out_dir)?;

        let mut f = fs::File::create(&out_path)?;
        io::copy(&mut temp, &mut f)?;

        Ok(())
    }

    fn open_object(&self, id: &HashID) -> Result<fs::File, Error> {
        let obj_path = self.object_path(id);
        if !obj_path.exists() {
            return Err(Error::EntryNotFound(id.clone()));
        }

        let mut f = fs::File::open(&obj_path)?;
        let load_id = hash::hash(&mut f)?;
        if &load_id != id {
            return Err(Error::BrokenObject {
                to_be: id.clone(),
                actual: load_id,
            });
        }

        Ok(f)
    }

    fn object_path(&self, id: &HashID) -> PathBuf {
        let mut res = self.object_dir().to_owned();

        let (p0, p1, p2) = id.parts();
        res.push(p0);
        res.push(p1);
        res.push(p2);

        res
    }

    fn bank_path(&self, name: &str) -> PathBuf {
        self.banks_dir.join(name)
    }

    fn object_dir(&self) -> &Path {
        &self.objects_dir
    }
}

fn check_path(path: &Path, name: &'static str) -> Result<(), Error> {
    if !path.exists() {
        Err(Error::IncompleteRepo(name, "missing"))
    } else if fs::metadata(path)?.permissions().readonly() {
        Err(Error::IncompleteRepo(name, "read only"))
    } else {
        Ok(())
    }
}

/// バックアップ元に対応した履歴の保存先を表す型
#[derive(Debug)]
pub struct Bank<'a> {
    repo: &'a Repository,
    path: PathBuf,
}

impl<'a> Bank<'a> {
    fn new(repo: &'a Repository, path: PathBuf) -> Bank<'a> {
        Bank { repo, path }
    }

    /// ファイルを指定された`id`のオブジェクトとして保存する。
    pub fn save_object(&self, id: &HashID, file: fs::File) -> Result<(), io::Error> {
        self.repo.save_object(id, file)
    }

    /// スキャン結果のエンティティのIDを`timestamp`時点での履歴として保存する。
    pub fn save_history(&self, id: HashID, timestamp: Timestamp) -> Result<(), io::Error> {
        let history_dir = self.history_dir();
        ensure_dir(&history_dir)?;

        let last_scan = History { id, timestamp };

        let history_file = history_dir.join(&timestamp.to_string());
        let f = fs::File::create(&history_file)?;
        to_writer(f, &last_scan)?;

        let last_scan_file = self.last_scan_file();
        let f = fs::File::create(&last_scan_file)?;
        to_writer(f, &last_scan)?;

        Ok(())
    }

    /// 指定された時点でのBankのルートディレクトリのエントリを読み込む。
    pub fn load_root(&'a self, history: &History) -> Result<DirEntry, Error> {
        self.load_dir_entry(&history.id)
    }

    /// 指定された`id`のディレクトリエントリを読み込む。
    pub fn load_dir_entry(&'a self, id: &HashID) -> Result<DirEntry, Error> {
        let f = self.open_object(id)?;
        Ok(from_reader(f)?)
    }

    /// 指定された`id`のファイルを開く。
    ///
    /// 内部でファイルの整合性チェックが行われる。
    pub fn open_object(&self, id: &HashID) -> Result<fs::File, Error> {
        self.repo.open_object(id)
    }

    /// 最新の履歴を得る。
    ///
    /// 存在しない場合はNoneを返す。
    pub fn last_scan(&self) -> Result<Option<History>, Error> {
        let path = self.last_scan_file();

        if !path.exists() {
            return Ok(None);
        }

        let f = fs::File::open(&path)?;
        let history: History = from_reader(f)?;

        Ok(Some(history))
    }

    fn create_dir(&self) -> Result<(), Error> {
        ensure_dir(&self.path)?;
        ensure_dir(&self.history_dir())?;
        Ok(())
    }

    fn history_dir(&self) -> PathBuf {
        self.path.join("history")
    }

    fn last_scan_file(&self) -> PathBuf {
        self.path.join("last_scan")
    }
}

/// バックアップ履歴を表す
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct History {
    timestamp: Timestamp,
    id: HashID,
}

impl History {
    /// 履歴のルートのディレクトリエントリのIDを得る。
    pub fn id(&self) -> &HashID {
        &self.id
    }

    /// 履歴のバックアップ開始時刻のタイムスタンプを得る。
    pub fn timestamp(&self) -> Timestamp {
        self.timestamp
    }
}

fn ensure_dir(path: &Path) -> Result<(), io::Error> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

/// リポジトリ操作に関わるエラー
#[derive(Debug, Fail)]
pub enum Error {
    /// 入出力エラーが発生した
    #[fail(display = "failed scan with IO error: {}", _0)]
    IO(#[fail(cause)] io::Error),

    /// リポジトリが不完全な状態である
    #[fail(display = "repository isn't complete: {} is {}", _0, _1)]
    IncompleteRepo(&'static str, &'static str),

    /// 指定されたエントリが存在しない
    #[fail(display = "object not exists: {}", _0)]
    EntryNotFound(HashID),

    /// エントリのハッシュ値が一致しない
    #[fail(display = "object not exists: {}", _0)]
    BrokenObject {
        /// 期待されるID値
        to_be: HashID,
        /// 実際に得られたID値
        actual: HashID,
    },

    /// JSONのパースに失敗した
    #[fail(display = "failed parse entry: {}", _0)]
    Parse(#[fail(cause)] serde_json::Error),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::IO(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Error {
        Error::Parse(e)
    }
}

impl From<hash::Error> for Error {
    fn from(e: hash::Error) -> Error {
        match e {
            hash::Error::IO(e) => Error::IO(e),
        }
    }
}
