use std::process::exit;

use clap::{App, Arg, ArgMatches, SubCommand};

use log::error;

use super::super::SubCmd;

use crate::config::Config;
use crate::core::repo::{self, Repository};

pub fn new() -> Box<dyn SubCmd> {
    Box::new(Bank::new())
}

pub struct Bank();

impl Bank {
    pub fn new() -> Bank {
        Bank()
    }

    fn wrapped_exec(&self, matches: &ArgMatches, config: Config) -> Result<()> {
        let repo_path = matches
            .value_of("repo")
            .map(|s| s.parse().unwrap())
            .or_else(|| config.repository_path().map(|p| p.to_owned()))
            .ok_or_else(|| Error::Arg("no repository path"))?;

        let repository = Repository::open(&repo_path)?;
        let name = matches.value_of("name").unwrap();
        let path = matches.value_of("path").unwrap();

        if repository.bank_exists(name)? {
            println!("bank '{}' already exists.", name);
            return Ok(());
        }

        repository.create_bank(name, path)?;

        Ok(())
    }
}

impl SubCmd for Bank {
    fn name(&self) -> &'static str {
        "bank"
    }

    fn command_args(&self) -> App {
        SubCommand::with_name(self.name())
            .about("create or initialize bank")
            .arg(
                Arg::with_name("repo")
                    .long("repo")
                    .takes_value(true)
                    .help("Overwrite repository path"),
            )
            .arg(
                Arg::with_name("name")
                    .short("n")
                    .long("name")
                    .takes_value(true)
                    .required(true)
                    .help("Bank name"),
            )
            .arg(
                Arg::with_name("path")
                    .short("p")
                    .long("path")
                    .takes_value(true)
                    .required(true)
                    .help("Target path"),
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

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("repository operation error: {0}")]
    Repo(#[from] repo::Error),

    #[error("{0}")]
    Arg(&'static str),
}
