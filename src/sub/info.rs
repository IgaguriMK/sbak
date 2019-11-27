use std::io;
use std::process::exit;

use clap::{App, Arg, ArgMatches, SubCommand};
use failure::Fail;
use log::{debug, error, info, trace, warn};

use super::SubCmd;

use crate::config::Config;
use crate::version::version;

pub fn new() -> Box<dyn SubCmd> {
    Box::new(Info::new())
}

pub struct Info();

impl Info {
    pub fn new() -> Info {
        Info()
    }

    fn wrapped_exec(&self, matches: &ArgMatches, config: Config) -> Result<()> {
        println!("Version:");
        println!("    {}", version(10));
        println!();

        println!("Config:");
        config.show();
        println!();

        if matches.is_present("log_test") {
            error!("Error log");
            warn!("Warn log");
            info!("info log");
            debug!("Debug log");
            trace!("Trace log");
        }

        Ok(())
    }
}

impl SubCmd for Info {
    fn name(&self) -> &'static str {
        "info"
    }

    fn command_args(&self) -> App {
        SubCommand::with_name(self.name())
            .about("show informations")
            .arg(Arg::with_name("log_test").long("log-test"))
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
    #[fail(display = "failed scan with IO error: {}", _0)]
    IO(#[fail(cause)] io::Error),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::IO(e)
    }
}
