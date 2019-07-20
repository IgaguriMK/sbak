use std::fs::File;
use std::io::{self, BufReader};
use std::path::Path;

use failure::Fail;
use serde_json::{self, from_reader, to_writer};

use super::FsEntry;

pub fn load_fs_tree<P: AsRef<Path>>(path: P) -> Result<FsEntry, LoadError> {
    let f = File::open(path)?;
    let r = BufReader::new(f);
    Ok(from_reader(r)?)
}

pub fn save_fs_tree<P: AsRef<Path>>(path: P, tree: &FsEntry) -> Result<(), SaveError> {
    let f = File::create(path)?;
    to_writer(f, tree)?;
    Ok(())
}

#[derive(Debug, Fail)]
pub enum LoadError {
    #[fail(display = "failed scan with IO error: {}", _0)]
    IO(#[fail(cause)] io::Error),
    #[fail(display = "failed parse FsEntry: {}", _0)]
    Parse(#[fail(cause)] serde_json::Error),
}

impl From<io::Error> for LoadError {
    fn from(e: io::Error) -> LoadError {
        LoadError::IO(e)
    }
}

impl From<serde_json::Error> for LoadError {
    fn from(e: serde_json::Error) -> LoadError {
        LoadError::Parse(e)
    }
}

#[derive(Debug, Fail)]
pub enum SaveError {
    #[fail(display = "failed scan with IO error: {}", _0)]
    IO(#[fail(cause)] io::Error),
    #[fail(display = "failed parse FsEntry: {}", _0)]
    Encode(#[fail(cause)] serde_json::Error),
}

impl From<io::Error> for SaveError {
    fn from(e: io::Error) -> SaveError {
        SaveError::IO(e)
    }
}

impl From<serde_json::Error> for SaveError {
    fn from(e: serde_json::Error) -> SaveError {
        SaveError::Encode(e)
    }
}
