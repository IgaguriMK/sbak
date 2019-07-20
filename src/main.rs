use clap::{App, crate_name,  crate_authors, crate_description};
use failure::Fail;

use sbak::core::scan::{self, Scanner};
use sbak::sub::sub_commands;
use sbak::version::version;

fn main() {
    if let Err(e) = w_main() {
        eprintln!("{}", e);
        if cfg!(debug_assertions) {
            eprintln!("{:#?}", e);
        }
        std::process::exit(1);
    }
}


fn w_main() -> Result<(), Error> {
    let subs = sub_commands();

    let ver = version(8);
    let matches = App::new(crate_name!())
        .author(crate_authors!("\n"))
        .about(crate_description!())
        .version(ver.as_str())
        .subcommands(subs.arg_defs())
        .get_matches();

    if let (subcmd_name, Some(matches)) = matches.subcommand() {
        if subcmd_name != "" {
            subs.execute(subcmd_name, matches);
        }
    }

    let scanner = Scanner::new();
    let tree = scanner.scan("./sample-target")?;
    println!("{:#?}", tree);

    Ok(())
}

#[derive(Debug, Fail)]
enum Error {
    #[fail(display = "{}", _0)]
    Other(#[fail(cause)] Box<dyn Fail>),
}

impl From<scan::Error> for Error {
    fn from(cause: scan::Error) -> Error {
        Error::Other(Box::new(cause))
    }
}