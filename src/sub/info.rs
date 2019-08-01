use std::io;
use std::process::exit;

use clap::{App, ArgMatches, SubCommand};
use failure::Fail;

use super::SubCmd;

use crate::config::Config;
use crate::version::{version, GIT_HASH_LEN};

pub fn new() -> Box<dyn SubCmd> {
    Box::new(Info::new())
}

pub struct Info();

impl Info {
    pub fn new() -> Info {
        Info()
    }

    fn wrapped_exec(&self, _matches: &ArgMatches, config: Config) -> Result<()> {
        println!("Version:");
        println!("    {}", version(GIT_HASH_LEN));
        println!();

        println!("Config:");
        config.show("    ");
        println!();

        Ok(())
    }
}

impl SubCmd for Info {
    fn name(&self) -> &'static str {
        "info"
    }

    fn command_args(&self) -> App {
        SubCommand::with_name(self.name()).about("show informations")
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
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::IO(e)
    }
}