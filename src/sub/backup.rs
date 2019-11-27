use std::io;
use std::process::exit;

use clap::{App, Arg, ArgMatches, SubCommand};
use failure::Fail;
use log::{error, info, trace};

use super::SubCmd;

use crate::config::Config;
use crate::core::repo::{self, Bank, Repository};
use crate::core::scan::{self, Scanner};
use crate::core::timestamp::{self, Timestamp};

pub fn new() -> Box<dyn SubCmd> {
    Box::new(Backup::new())
}

pub struct Backup();

impl Backup {
    pub fn new() -> Backup {
        Backup()
    }

    fn wrapped_exec(&self, matches: &ArgMatches, config: Config) -> Result<()> {
        let repo_path = matches
            .value_of("repo")
            .map(|s| s.parse().unwrap())
            .or_else(|| config.repository_path().map(|p| p.to_owned()))
            .ok_or_else(|| Error::Arg("no repository path"))?;
        let repo = Repository::open(&repo_path)?;

        if let Some(bank_name) = matches.value_of("bank") {
            let bank = repo.open_bank(bank_name)?;
            scan(bank)?;
        } else {
            for bank in repo.open_all_banks()? {
                let bank = bank?;
                scan(bank)?;
            }
        }

        Ok(())
    }
}

fn scan(bank: Bank) -> Result<()> {
    let scan_start = Timestamp::now()?;
    info!("scan start at {}", scan_start);

    let scanner = Scanner::new(&bank);
    let id = scanner.scan()?;

    trace!("start save history");
    bank.save_history(id.id(), scan_start)?;
    trace!("finish scan {:?}", bank.target_path());

    Ok(())
}

impl SubCmd for Backup {
    fn name(&self) -> &'static str {
        "backup"
    }

    fn command_args(&self) -> App {
        SubCommand::with_name(self.name())
            .about("Backup files")
            .arg(
                Arg::with_name("repo")
                    .long("repo")
                    .takes_value(true)
                    .help("Overwrite repository path"),
            )
            .arg(
                Arg::with_name("bank")
                    .short("b")
                    .long("bank")
                    .takes_value(true)
                    .help("Bank name"),
            )
    }

    fn exec(&self, matches: &ArgMatches, config: Config) -> ! {
        match self.wrapped_exec(matches, config) {
            Ok(()) => exit(0),
            Err(e) => {
                if cfg!(debug_assertions) {
                    error!("{:#?}", e);
                } else {
                    error!("{}", e);
                }
                exit(1)
            }
        }
    }
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "{}", _0)]
    Arg(&'static str),

    #[fail(display = "failed scan with IO error: {}", _0)]
    IO(#[fail(cause)] io::Error),

    #[fail(display = "file scan error: {}", _0)]
    Scan(#[fail(cause)] scan::Error),

    #[fail(display = "repository operation error: {}", _0)]
    Repo(#[fail(cause)] repo::Error),

    #[fail(display = "timestamp is older than UNIX epoch")]
    Timestamp,
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

impl From<timestamp::Error> for Error {
    fn from(_e: timestamp::Error) -> Error {
        Error::Timestamp
    }
}
