use std::io;
use std::fs;
use std::process::exit;
use std::path::{Path, PathBuf};

use clap::{App, ArgMatches, SubCommand};
use failure::Fail;

use super::SubCmd;

use crate::core::scan::{self, Scanner};

pub fn new() -> Box<dyn SubCmd> {
    Box::new(Backup::new())
}

pub struct Backup();

impl Backup {
    pub fn new() -> Backup {
        Backup()
    }

    fn wrapped_exec(&self, _matches: &ArgMatches) -> Result<()> {
        let target_dir = PathBuf::from("./sample-target");

        let repo_dir = PathBuf::from("./sample-repo");
        ensure_dir(&repo_dir)?;

        let object_dir = repo_dir.join("objects");
        ensure_dir(&object_dir)?;

        let scanner = Scanner::new("./sample-repo/objects");
        let (current_hash, recorded_at) = scanner.scan(target_dir)?;

        eprintln!("current_hash = {}", current_hash);
        eprintln!("recorded_at = {}", recorded_at);

        let history_dir = repo_dir.join("history");
        ensure_dir(&history_dir)?;

        let history_file = history_dir.join(&recorded_at.to_string());
        fs::write(&history_file, &current_hash.to_string())?;

        let last_scan_file = history_dir.join("last_scan");
        fs::write(&last_scan_file, &recorded_at.to_string())?;

        Ok(())
    }
}

fn ensure_dir(path: &Path) -> Result<()> {
    if ! path.exists() {
        fs::create_dir(path)?;
    }
    Ok(())
}

impl SubCmd for Backup {
    fn name(&self) -> &'static str {
        "backup"
    }

    fn command_args(&self) -> App<'static, 'static> {
        SubCommand::with_name(self.name()).about("Backup files")
    }

    fn exec(&self, matches: &ArgMatches) -> ! {
        match self.wrapped_exec(matches) {
            Ok(()) => exit(0),
            Err(e) => {
                eprintln!("{}", e);
                if cfg!(debug_assertions) {
                    eprintln!("{:#?}", e);
                }
                exit(1)
            }
        }
    }
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Fail)]
pub enum Error {
    // #[fail(display = "Found invalid command-line argument: {}", msg)]
    // InvalidArg { msg: &'static str },

    #[fail(display = "failed scan with IO error: {}", _0)]
    IO(#[fail(cause)] io::Error),

    #[fail(display = "file scan error: {}", _0)]
    Scan(#[fail(cause)] scan::Error),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::IO(e)
    }
}

impl From<scan::Error> for Error {
    fn from(e: scan::Error) -> Error {
        Error::Scan(e)
    }
}
