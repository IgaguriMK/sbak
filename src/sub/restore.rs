use std::collections::HashSet;
use std::process::exit;

use clap::{App, Arg, ArgMatches, SubCommand};

use log::error;

use super::SubCmd;

use crate::config::Config;
use crate::core::extend::{self, Extender};
use crate::core::hash::HashID;
use crate::core::repo::{self, Repository};

pub fn new() -> Box<dyn SubCmd> {
    Box::new(Restore::new())
}

pub struct Restore();

impl Restore {
    pub fn new() -> Restore {
        Restore()
    }

    fn wrapped_exec(&self, matches: &ArgMatches, config: Config) -> Result<()> {
        let repo_path = matches
            .value_of("repo")
            .map(|s| s.parse().unwrap())
            .or_else(|| config.repository_path().map(|p| p.to_owned()))
            .ok_or_else(|| Error::Arg("no repository path"))?;

        let bank_name = matches.value_of("bank").unwrap();
        let target_path = matches.value_of("to").unwrap();

        let repo = Repository::open(&repo_path)?;
        let bank = repo.open_bank(bank_name)?;

        let mut extender = Extender::new(&bank);
        extender.allow_overwrite(matches.is_present("overwrite"));
        extender.allow_remove(matches.is_present("remove"));

        if let Some(hash_prefix) = matches.value_of("revision") {
            let histories = bank.find_hash(hash_prefix)?;
            if histories.is_empty() {
                eprintln!("Error: no histories with hash {}", hash_prefix);
                exit(1);
            }

            let hashes = histories
                .iter()
                .map(|h| h.id().clone())
                .collect::<HashSet<HashID>>();
            if hashes.len() > 1 {
                eprintln!("Error: multiple hash matched:");
                for h in hashes {
                    eprintln!("    {}", h);
                }
                exit(1);
            }

            let hist = histories.last().unwrap();
            extender.extend(target_path, hist)?;
        } else if let Some(hist) = bank.last_scan()? {
            extender.extend(target_path, &hist)?;
        } else {
            eprintln!("No scans in bank.");
            exit(1);
        }

        let symlinks = extender.symlinks();
        if matches.is_present("show_symlinks") {
            symlinks.show();
        }

        Ok(())
    }
}

impl SubCmd for Restore {
    fn name(&self) -> &'static str {
        "restore"
    }

    fn command_args(&self) -> App {
        SubCommand::with_name(self.name())
            .about("Restore files")
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
                    .required(true)
                    .help("Bank name"),
            )
            .arg(
                Arg::with_name("to")
                    .short("t")
                    .long("to")
                    .takes_value(true)
                    .required(true)
                    .help("Restore target"),
            )
            .arg(
                Arg::with_name("revision")
                    .short("r")
                    .long("revision")
                    .takes_value(true)
                    .help("Specify revision to restore"),
            )
            .arg(
                Arg::with_name("overwrite")
                    .short("O")
                    .long("overwrite")
                    .help("Overwrite existing files."),
            )
            .arg(
                Arg::with_name("remove")
                    .short("R")
                    .long("remove")
                    .help("Remove existing files if not contained in backup."),
            )
            .arg(
                Arg::with_name("show_symlinks")
                    .long("show-symlinks")
                    .help("Show symbolic link list"),
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

    #[error("failed extend: {0}")]
    Extend(#[from] extend::Error),

    #[error("repository operation error: {0}")]
    Repo(#[from] repo::Error),
}
