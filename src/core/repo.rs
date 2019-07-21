use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use failure::Fail;
use serde::{Deserialize, Serialize};
use serde_json::{self, to_writer};

use crate::core::hash::HashID;
use crate::core::timestamp::Timestamp;

#[derive(Debug)]
pub struct Repository {
    path: PathBuf,
    objects_dir: PathBuf,
    banks_dir: PathBuf,
}

impl Repository {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Repository, Error> {
        let repo = Repository::new(path);

        check_path(&repo.path, "repository directory")?;
        check_path(&repo.objects_dir, "/object")?;
        check_path(&repo.banks_dir, "/banks")?;

        Ok(repo)
    }

    pub fn open_or_create<P: AsRef<Path>>(path: P) -> Result<Repository, Error> {
        let repo = Repository::new(path);

        ensure_dir(&repo.path)?;
        ensure_dir(&repo.objects_dir)?;
        ensure_dir(&repo.banks_dir)?;

        Ok(repo)
    }

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

#[derive(Debug)]
pub struct Bank<'a> {
    repo: &'a Repository,
    path: PathBuf,
}

impl<'a> Bank<'a> {
    fn new(repo: &'a Repository, path: PathBuf) -> Bank<'a> {
        Bank { repo, path }
    }

    pub fn save_object(&self, id: HashID, temp: fs::File) -> Result<(), io::Error> {
        self.repo.save_object(id, temp)
    }

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

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "failed scan with IO error: {}", _0)]
    IO(#[fail(cause)] io::Error),

    #[fail(display = "failed parse: {}", _0)]
    Serde(#[fail(cause)] serde_json::Error),

    #[fail(display = "repository isn't complete: {} is {}", _0, _1)]
    IncompleteRepo(&'static str, &'static str),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::IO(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Error {
        Error::Serde(e)
    }
}