use std::collections::BTreeMap;

use clap::{App, ArgMatches};

mod backup;

pub trait SubCmd {
    fn name(&self) -> &'static str;
    fn command_args(&self) -> App<'static, 'static>;
    fn exec(&self, matches: &ArgMatches) -> !;
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

    pub fn execute(&self, name: &str, matches: &ArgMatches) {
        if let Some(cmd) = self.table.get(name) {
            cmd.exec(matches);
        }
    }

    fn append(&mut self, subcmd: Box<dyn SubCmd>) {
        if let Some(exists) = self.table.insert(subcmd.name().to_owned(), subcmd) {
            panic!("registering duplecated subcommand: {}", exists.name());
        }
    }
}
