use clap::{App, ArgMatches, SubCommand};
use std::process::exit;

use failure::Fail;

use super::SubCmd;

pub fn new() -> Box<dyn SubCmd> {
    Box::new(Backup::new())
}

pub struct Backup();

impl Backup {
    pub fn new() -> Backup {
        Backup()
    }

    fn wrapped_exec(&self, _matches: &ArgMatches) -> Result<(), Error> {
        println!("subcommand {}", self.name());

        Ok(())
    }
}

impl SubCmd for Backup {
    fn name(&self) -> &'static str {
        "backup"
    }

    fn command_args(&self) -> App<'static, 'static> {
        SubCommand::with_name(self.name())
            .about("Backup files")
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
    #[fail(display = "dummy error")]
    _Other,
}
