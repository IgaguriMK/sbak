use clap::{App, ArgMatches, SubCommand};
use std::path::PathBuf;
use std::process::exit;

use failure::Fail;

use super::SubCmd;

use crate::core::fs_tree::io::{load_fs_tree, save_fs_tree, LoadError, SaveError};
use crate::core::scan::{self, Scanner};

pub fn new() -> Box<dyn SubCmd> {
    Box::new(Backup::new())
}

pub struct Backup();

impl Backup {
    pub fn new() -> Backup {
        Backup()
    }

    fn wrapped_exec(&self, _matches: &ArgMatches) -> Result<(), Error> {
        let scanner = Scanner::new();
        let current_tree = scanner.scan("./sample-target")?;

        let save_path: PathBuf = "./last_scan.json".into();
        if save_path.exists() {
            let prev_tree = load_fs_tree(&save_path)?;

            if current_tree == prev_tree {
                println!("Not modified.");
            } else {
                println!("Detect modified.");
            }
        } else {
            println!("No previous save.")
        }

        save_fs_tree(&save_path, &current_tree)?;

        Ok(())
    }
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
                std::process::exit(1)
            }
        }
    }
}

#[derive(Debug, Fail)]
pub enum Error {
    // #[fail(display = "Found invalid command-line argument: {}", msg)]
    // InvalidArg { msg: &'static str },
    #[fail(display = "file scan error: {}", _0)]
    Scan(#[fail(cause)] scan::Error),
    #[fail(display = "failed load last scan data: {}", _0)]
    Load(#[fail(cause)] LoadError),
    #[fail(display = "failed save scan data: {}", _0)]
    Save(#[fail(cause)] SaveError),
}

impl From<scan::Error> for Error {
    fn from(e: scan::Error) -> Error {
        Error::Scan(e)
    }
}

impl From<LoadError> for Error {
    fn from(e: LoadError) -> Error {
        Error::Load(e)
    }
}

impl From<SaveError> for Error {
    fn from(e: SaveError) -> Error {
        Error::Save(e)
    }
}
