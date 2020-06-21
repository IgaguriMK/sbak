use std::io;
use std::process::exit;

use clap::{App, Arg, ArgMatches, SubCommand};

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
                crate::util::dump_error(e);
                exit(1)
            }
        }
    }
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Arg(&'static str),

    #[error("failed scan with IO error: {0}")]
    IO(#[from] io::Error),

    #[error("file scan error: {0}")]
    Scan(#[from] scan::Error),

    #[error("repository operation error: {0}")]
    Repo(#[from] repo::Error),

    #[error("timestamp is invalid: {0}")]
    Timestamp(#[from] timestamp::Error),
}
