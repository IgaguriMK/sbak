use std::collections::BTreeMap;

use clap::{App, ArgMatches};
use failure::Fail;

mod backup;

pub trait SubCmd {
    fn name(&self) -> &'static str;
    fn command_args(&self) -> App<'static, 'static>;
    fn exec(&self, matches: &ArgMatches) -> Result<(), Error>;
}

pub fn sub_commands() -> SubCommandSet {
    let mut set = SubCommandSet::new();

    set.append(backup::new());

    set
}

pub struct SubCommandSet {
    table: BTreeMap<String, Box<dyn SubCmd>>,
}

impl<'a> SubCommandSet {
    fn new() -> SubCommandSet {
        SubCommandSet {
            table: BTreeMap::new(),
        }
    }

    pub fn arg_defs(&'a self) -> impl Iterator<Item = App<'a, 'a>> {
        self.table.iter().map(|(_, c)| c.command_args())
    }

    pub fn execute(&self, name: &str, matches: &ArgMatches) -> Result<(), Error> {
        let cmd = self.table.get(name).unwrap();
        cmd.exec(matches)
    }

    fn append(&mut self, subcmd: Box<dyn SubCmd>) {
        self.table.insert(subcmd.name().to_owned(), subcmd);
    }
}

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Found invalid command-line argument: {}", msg)]
    InvalidArg { msg: &'static str },
}
