use std::process::exit;

use clap::{App, Arg, ArgMatches, SubCommand};
use failure::Fail;

use super::super::SubCmd;

use crate::config::Config;
use crate::core::repo::{self, Repository};

pub fn new() -> Box<dyn SubCmd> {
    Box::new(Repo::new())
}

pub struct Repo();

impl Repo {
    pub fn new() -> Repo {
        Repo()
    }

    fn wrapped_exec(&self, matches: &ArgMatches, _config: Config) -> Result<()> {
        let path = matches.value_of("path").unwrap();

        let _ = Repository::create(path)?;

        Ok(())
    }
}

impl SubCmd for Repo {
    fn name(&self) -> &'static str {
        "repo"
    }

    fn command_args(&self) -> App {
        SubCommand::with_name(self.name())
            .about("create or initialize repository")
            .arg(
                Arg::with_name("path")
                    .short("p")
                    .long("path")
                    .required(true),
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
}

impl From<repo::Error> for Error {
    fn from(e: repo::Error) -> Error {
        Error::Repo(e)
    }
}
