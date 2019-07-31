use std::io;
use std::path::PathBuf;
use std::process::exit;

use clap::{App, ArgMatches, SubCommand};
use failure::Fail;

use super::SubCmd;

use crate::config::Config;
use crate::core::repo::{self, Repository};
use crate::core::scan::{self, Scanner};

pub fn new() -> Box<dyn SubCmd> {
    Box::new(Backup::new())
}

pub struct Backup();

impl Backup {
    pub fn new() -> Backup {
        Backup()
    }

    fn wrapped_exec(&self, _matches: &ArgMatches, _config: Config) -> Result<()> {
        let target_dir = PathBuf::from("./sample-target");

        let repo_dir = PathBuf::from("./sample-repo");
        let repo = Repository::open(&repo_dir)?;

        let bank = repo.open_bank("sample");
        let scanner = Scanner::new(bank);
        scanner.scan(target_dir)?;

        Ok(())
    }
}

impl SubCmd for Backup {
    fn name(&self) -> &'static str {
        "backup"
    }

    fn command_args(&self) -> App {
        SubCommand::with_name(self.name()).about("Backup files")
    }

    fn exec(&self, matches: &ArgMatches, config: Config) -> ! {
        match self.wrapped_exec(matches, config) {
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
    #[fail(display = "failed scan with IO error: {}", _0)]
    IO(#[fail(cause)] io::Error),

    #[fail(display = "file scan error: {}", _0)]
    Scan(#[fail(cause)] scan::Error),

    #[fail(display = "repository operation error: {}", _0)]
    Repo(#[fail(cause)] repo::Error),
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

impl From<repo::Error> for Error {
    fn from(e: repo::Error) -> Error {
        Error::Repo(e)
    }
}
