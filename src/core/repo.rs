//! バックアップ先となるリポジトリの操作

use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use failure::Fail;
use log::trace;
use serde::{Deserialize, Serialize};
use serde_json::{self, from_reader, to_writer};

use crate::core::entry::DirEntry;
use crate::core::hash::{self, HashID};
use crate::core::ignore::pattern::{self, load_patterns, Patterns};
use crate::core::timestamp::Timestamp;

const BANK_CONFIG_FILE: &str = "config.json";
const HISTORY_SUFFIX: &str = ".history.json";

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
    pub fn open_bank<'a>(&'a self, name: &str) -> Result<Bank<'a>, Error> {
        let bank_dir = self.bank_path(name);

        let f = fs::File::open(&bank_dir.join(BANK_CONFIG_FILE))?;
        let config = from_reader(f)?;

        Ok(Bank::new(self, bank_dir, config))
    }

    /// 全ての[`Bank`](struct.Bank.html)を開くイテレータを取得する。
    ///
    /// 要素の順序はBankの名前の辞書順になる。
    pub fn open_all_banks(&self) -> Result<Banks, Error> {
        let mut names = Vec::<String>::new();

        for dir_entry in self.banks_dir.read_dir()? {
            let name = dir_entry?
                .file_name()
                .into_string()
                .map_err(Error::InvalidFileName)?;
            names.push(name);
        }

        names.sort();
        names.reverse();

        Ok(Banks { repo: self, names })
    }

    /// 指定された名前の[`Bank`](struct.Bank.html)を作成する。
    pub fn create_bank<P: AsRef<Path>>(&self, name: &str, target_path: P) -> Result<(), Error> {
        let bank_dir = self.bank_path(name);

        let target_path = target_path.as_ref().canonicalize()?;
        if !target_path.is_dir() {
            return Err(Error::InvalidInput(format!(
                "target path '{:?}' isn't directory.",
                target_path
            )));
        }
        let bank_config = BankConfig { target_path };

        let bank = Bank::new(self, bank_dir, bank_config);
        bank.create()?;
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

/// Bank一覧のイテレータ
pub struct Banks<'a> {
    repo: &'a Repository,
    names: Vec<String>,
}

impl<'a> Iterator for Banks<'a> {
    type Item = Result<Bank<'a>, Error>;

    fn next(&mut self) -> Option<Result<Bank<'a>, Error>> {
        if let Some(name) = self.names.pop() {
            Some(self.repo.open_bank(&name))
        } else {
            None
        }
    }
}

/// バックアップ元に対応した履歴の保存先を表す型
#[derive(Debug)]
pub struct Bank<'a> {
    repo: &'a Repository,
    path: PathBuf,
    config: BankConfig,
}

impl<'a> Bank<'a> {
    fn new(repo: &'a Repository, path: PathBuf, config: BankConfig) -> Bank<'a> {
        Bank { repo, path, config }
    }

    /// ファイルを指定された`id`のオブジェクトとして保存する。
    pub fn save_object(&self, id: &HashID, file: fs::File) -> Result<(), io::Error> {
        self.repo.save_object(id, file)
    }

    /// スキャン結果のエンティティのIDを`timestamp`時点での履歴として保存する。
    pub fn save_history(&self, id: HashID, timestamp: Timestamp) -> Result<(), io::Error> {
        let history_dir = self.history_dir();
        trace!("history_dir = {:?}", history_dir);
        ensure_dir(&history_dir)?;

        let last_scan = History { id, timestamp };
        trace!("history entry = {:?}", last_scan);

        let history_file = history_dir.join(&last_scan.file_name());
        trace!("history_file = {:?}", history_file);
        let f = fs::File::create(&history_file)?;
        to_writer(f, &last_scan)?;
        trace!("finish save history file");

        let last_scan_file = self.last_scan_file();
        trace!("last_scan_file = {:?}", last_scan_file);
        let f = fs::File::create(&last_scan_file)?;
        to_writer(f, &last_scan)?;
        trace!("finish save last_scan");

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

    /// 履歴の一覧を得る。
    ///
    /// 古い順にソートされて返される。
    pub fn histories(&self) -> Result<Vec<History>, Error> {
        let mut res = Vec::<History>::new();

        for file in self.history_dir().read_dir()? {
            let file = file?;
            let name = file
                .file_name()
                .into_string()
                .map_err(Error::InvalidFileName)?;

            if name.ends_with(HISTORY_SUFFIX) {
                let f = fs::File::open(file.path())?;
                let history: History = from_reader(f)?;
                res.push(history);
            }
        }

        res.sort();
        Ok(res)
    }

    /// 指定されたハッシュ値のプレフィックスを持つ履歴の一覧を返す。
    pub fn find_hash(&self, hash_prefix: &str) -> Result<Vec<History>, Error> {
        let mut res = Vec::new();

        for h in self.histories()? {
            if h.id().as_str().starts_with(hash_prefix) {
                res.push(h);
            }
        }

        Ok(res)
    }

    /// バックアップ対象ディレクトリのパスを取得する。
    pub fn target_path(&self) -> &Path {
        &self.config.target_path
    }

    /// `Bank`で指定されている除外リストを読み込む。
    pub fn load_ignore_patterns(&self) -> Result<Patterns, Error> {
        let path = self.ignore_file();

        if path.exists() {
            Ok(load_patterns(path)?)
        } else {
            Ok(Patterns::default())
        }
    }

    fn create(&self) -> Result<(), Error> {
        ensure_dir(&self.path)?;
        ensure_dir(&self.history_dir())?;

        let f = fs::File::create(&self.path.join(BANK_CONFIG_FILE))?;
        to_writer(f, &self.config)?;

        Ok(())
    }

    fn history_dir(&self) -> PathBuf {
        self.path.join("history")
    }

    fn last_scan_file(&self) -> PathBuf {
        self.path.join("last_scan.json")
    }

    fn ignore_file(&self) -> PathBuf {
        self.path.join("ignore")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BankConfig {
    target_path: PathBuf,
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

    fn file_name(&self) -> String {
        format!("{}{}", self.timestamp.into_unix_epoch(), HISTORY_SUFFIX)
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
    /// エントリのハッシュ値が一致しない
    #[fail(display = "object not exists: {}", _0)]
    BrokenObject {
        /// 期待されるID値
        to_be: HashID,
        /// 実際に得られたID値
        actual: HashID,
    },

    /// 指定されたエントリが存在しない
    #[fail(display = "object not exists: {}", _0)]
    EntryNotFound(HashID),

    /// 除外リストの読み込みに失敗した
    #[fail(display = "failed load ignore patterns: {}", _0)]
    IgnorePattern(#[fail(cause)] pattern::ParseError),

    /// リポジトリが不完全な状態である
    #[fail(display = "repository isn't complete: {} is {}", _0, _1)]
    IncompleteRepo(&'static str, &'static str),

    /// パスがUnicodeで表現できない
    #[fail(display = "invalid file name {:?}", _0)]
    InvalidFileName(OsString),

    /// 入力が不正である。
    #[fail(display = "invalid input: {}", _0)]
    InvalidInput(String),

    /// 入出力エラーが発生した
    #[fail(display = "failed scan with IO error: {}", _0)]
    IO(#[fail(cause)] io::Error),

    /// JSONのパースに失敗した
    #[fail(display = "failed parse entry: {}", _0)]
    Parse(#[fail(cause)] serde_json::Error),
}

impl From<pattern::ParseError> for Error {
    fn from(e: pattern::ParseError) -> Error {
        Error::IgnorePattern(e)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::IO(e)
    }
}

impl From<hash::Error> for Error {
    fn from(e: hash::Error) -> Error {
        match e {
            hash::Error::IO(e) => Error::IO(e),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Error {
        Error::Parse(e)
    }
}
