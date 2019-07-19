use clap::{App, crate_name,  crate_authors, crate_description};
use failure::Fail;

use  sbak::sub::{Error as SubCmdError, sub_commands};
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
            return Ok(subs.execute(subcmd_name, matches)?);
        }
    }

    println!("main command");

    Ok(())
}

#[derive(Debug, Fail)]
enum Error {
    #[fail(display = "failed execute subcommand: {}", cause)]
    SubCmd{
        #[fail(cause)]
        cause: SubCmdError,
    },
}

impl From<SubCmdError> for Error {
    fn from(cause: SubCmdError) -> Error {
        Error::SubCmd{cause}
    }
}