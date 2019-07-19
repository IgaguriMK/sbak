use clap::{App, ArgMatches, SubCommand};

use super::{SubCmd, Error};

pub fn new() -> Box<dyn SubCmd> {
    Box::new(Backup::new())
}

pub struct Backup();

impl Backup {
    pub fn new() -> Backup {
        Backup()
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

    fn exec(&self, _matches: &ArgMatches) -> Result<(), Error> {
        println!("subcommand {}", self.name());

        Ok(())
    }
}

