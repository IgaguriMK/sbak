use std::process::exit;

use clap::{App, Arg, ArgMatches, SubCommand};
use failure::Fail;

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

        if repository.bank_exists(name)? {
            println!("bank '{}' already exists.", name);
            return Ok(());
        }

        repository.create_bank(name)?;

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
    #[fail(display = "repository operation error: {}", _0)]
    Repo(#[fail(cause)] repo::Error),

    #[fail(display = "{}", _0)]
    Arg(&'static str),
}

impl From<repo::Error> for Error {
    fn from(e: repo::Error) -> Error {
        Error::Repo(e)
    }
}
