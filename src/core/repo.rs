//! バックアップ先となるリポジトリの操作

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use failure::Fail;
use serde::{Deserialize, Serialize};
use serde_json::{self, to_writer};

use crate::core::hash::HashID;
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

    /// 既存のリポジトリを開くか、存在しない場合生成する。
    /// 
    /// # Failures
    /// 
    /// ディレクトリが存在せず、生成も失敗した場合、[`Error::IO`](enum.Error.html)を返す。
    pub fn open_or_create<P: AsRef<Path>>(path: P) -> Result<Repository, Error> {
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
        let bank_dir = self.banks_dir.join(name);
        Bank::new(self, bank_dir)
    }

    fn save_object(&self, id: HashID, mut temp: fs::File) -> Result<(), io::Error> {
        let out_path = self.object_path(id);

        let out_dir = out_path.parent().unwrap();
        fs::create_dir_all(out_dir)?;

        let mut f = fs::File::create(&out_path)?;
        io::copy(&mut temp, &mut f)?;

        Ok(())
    }

    fn object_path(&self, id: HashID) -> PathBuf {
        let mut res = self.object_dir().to_owned();

        let (p0, p1, p2) = id.parts();
        res.push(p0);
        res.push(p1);
        res.push(p2);

        res
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
    pub fn save_object(&self, id: HashID, file: fs::File) -> Result<(), io::Error> {
        self.repo.save_object(id, file)
    }

    /// スキャン結果のエンティティのIDを`timestamp`時点での履歴として保存する。
    pub fn save_history(&self, id: HashID, timestamp: Timestamp) -> Result<(), io::Error> {
        let history_dir = self.history_dir();
        ensure_dir(&history_dir)?;

        let history_file = history_dir.join(&timestamp.to_string());
        fs::write(&history_file, &id.to_string())?;

        self.save_last_scan(&LastScan { id, timestamp })?;

        Ok(())
    }

    fn save_last_scan(&self, last_scan: &LastScan) -> Result<(), io::Error> {
        let last_scan_file = self.last_scan_file();

        let f = fs::File::create(&last_scan_file)?;
        to_writer(f, last_scan)?;

        Ok(())
    }

    fn history_dir(&self) -> PathBuf {
        self.path.join("history")
    }

    fn last_scan_file(&self) -> PathBuf {
        self.history_dir().join("last_scan")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
struct LastScan {
    timestamp: Timestamp,
    id: HashID,
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
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::IO(e)
    }
}