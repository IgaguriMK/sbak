use clap::{App, crate_name};
use failure::Fail;

use  sbak::sub::{Error as SubCmdError, sub_commands};

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

    let matches = App::new(crate_name!())
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