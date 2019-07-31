mod bank;
mod repo;

use std::io::stderr;
use std::process::exit;

use clap::{App, ArgMatches, SubCommand};

use super::{SubCmd, SubCommandSet};

use crate::config::Config;

pub fn new() -> Box<dyn SubCmd> {
    Box::new(Init::new())
}

pub struct Init(SubCommandSet);

impl Init {
    pub fn new() -> Init {
        let mut subs: SubCommandSet = SubCommandSet::new();

        subs.append(bank::new());
        subs.append(repo::new());

        Init(subs)
    }
}

impl SubCmd for Init {
    fn name(&self) -> &'static str {
        "init"
    }

    fn command_args(&self) -> App {
        SubCommand::with_name(self.name())
            .about("create or initialize repository/bank")
            .subcommands(self.0.arg_defs())
    }

    fn exec(&self, matches: &ArgMatches, config: Config) -> ! {
        if let (subcmd_name, Some(matches)) = matches.subcommand() {
            self.0.execute(subcmd_name, matches, config); // 成功したらそのままプロセスを終了する
            exit(0)
        }

        let mut out = stderr();
        self.command_args().write_long_help(&mut out).unwrap();
        eprintln!();
        exit(1);
    }
}
