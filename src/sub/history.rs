use std::process::exit;

use clap::{App, Arg, ArgMatches, SubCommand};
use failure::Fail;

use super::SubCmd;

use crate::config::Config;
use crate::core::repo::{self, Repository};

pub fn new() -> Box<dyn SubCmd> {
    Box::new(History::new())
}

pub struct History();

impl History {
    pub fn new() -> History {
        History()
    }

    fn wrapped_exec(&self, matches: &ArgMatches, config: Config) -> Result<()> {
        let repository = Repository::open(
            config
                .repository_path()
                .ok_or(Error::NoValue("repository"))?,
        )?;

        let bank_name = matches.value_of("bank").unwrap();

        let show_count_str = matches.value_of("show_count").unwrap();
        let show_count: usize = if show_count_str == "all" {
            std::usize::MAX
        } else {
            show_count_str.parse().map_err(|_| {
                Error::InvalidCmdArg(format!(
                    "-n / --show-count '{}' is not number.",
                    show_count_str
                ))
            })?
        };

        let bank = repository.open_bank(bank_name)?;
        let mut histories = bank.histories()?;

        let l = histories.len();
        if l > show_count {
            let s = &histories[l - show_count..l];
            let mut tail_histories = Vec::with_capacity(show_count);
            tail_histories.extend_from_slice(s);
            histories = tail_histories;
        }

        for history in &histories {
            println!("{}    {}", history.timestamp(), history.id());
        }

        Ok(())
    }
}

impl SubCmd for History {
    fn name(&self) -> &'static str {
        "history"
    }

    fn command_args(&self) -> App {
        SubCommand::with_name(self.name())
            .about("show history")
            .arg(
                Arg::with_name("bank")
                    .short("b")
                    .long("bank")
                    .takes_value(true)
                    .required(true),
            )
            .arg(
                Arg::with_name("show_count")
                    .short("n")
                    .long("show-cownt")
                    .default_value("20"),
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
    #[fail(display = "Invalid command-line arguments: {}", _0)]
    InvalidCmdArg(String),

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
