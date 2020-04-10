use std::process::exit;

use clap::{App, Arg, ArgMatches, SubCommand};
use failure::Fail;
use log::error;

use super::SubCmd;

use crate::config::Config;
use crate::core::repo::{self, Repository};

pub fn new() -> Box<dyn SubCmd> {
    Box::new(List::new())
}

pub struct List();

impl List {
    pub fn new() -> List {
        List()
    }

    fn wrapped_exec(&self, matches: &ArgMatches, config: Config) -> Result<()> {
        let repository = Repository::open(
            config
                .repository_path()
                .ok_or(Error::NoValue("repository"))?,
        )?;

        let utc = matches.is_present("utc");

        for bank in repository.open_all_banks()? {
            let bank = bank?;

            println!("{}", bank.name());
            if let Some(h) = bank.last_scan()? {
                if utc {
                    println!("last backup at {:#}", h.timestamp());
                } else {
                    println!("    {}", h.timestamp());
                }
            } else {
                println!("    No backups");
            }
        }

        Ok(())
    }
}

impl SubCmd for List {
    fn name(&self) -> &'static str {
        "list"
    }

    fn command_args(&self) -> App {
        SubCommand::with_name(self.name())
            .about("show buckets")
            .arg(
                Arg::with_name("utc")
                    .short("u")
                    .long("utc")
                    .help("show time in UTC"),
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
    #[fail(display = "no config value: {}", _0)]
    NoValue(&'static str),

    #[fail(display = "repository operation error: {}", _0)]
    Repo(#[fail(cause)] repo::Error),
}

impl From<repo::Error> for Error {
    fn from(e: repo::Error) -> Error {
        Error::Repo(e)
    }
}
