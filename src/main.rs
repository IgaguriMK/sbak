use std::io::{stderr, Write};
use std::process::exit;

use clap::{crate_authors, crate_description, crate_name, App, Arg};
use failure::Fail;
use log::trace;

use sbak::config::{self, auto_load, load, VerboseLevel};
use sbak::sub::sub_commands;
use sbak::version::version;

fn main() {
    if let Err(e) = w_main() {
        eprintln!("{}", e);
        exit(1);
    }
}

fn w_main() -> Result<()> {
    let mut config = auto_load()?;

    let subs = sub_commands();

    let ver = version(8);
    let mut app = App::new(crate_name!())
        .author(crate_authors!("\n"))
        .about(crate_description!())
        .version(ver.as_str())
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .takes_value(true)
                .help("Extra config file"),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .multiple(true)
                .help("Verbosity level"),
        )
        .subcommands(subs.arg_defs());

    let mut help_str = Vec::<u8>::new();
    app.write_long_help(&mut help_str).unwrap();

    let matches = app.get_matches();

    let cmd_verbose = matches.occurrences_of("verbose") as usize;
    if cmd_verbose > 0 {
        config.set_verbose(VerboseLevel::from_v_count(cmd_verbose));
        config.apply_verbose();
    }

    if let Some(extra_config_file) = matches.value_of("config") {
        let extra_config = load(extra_config_file)?;
        config = config.merged(&extra_config);
    }

    config.apply_verbose();

    trace!("config = {:?}", config);

    if let (subcmd_name, Some(matches)) = matches.subcommand() {
        // 成功したらそのままプロセスを終了する
        subs.execute(subcmd_name, matches, config);
    }

    let mut out = stderr();
    out.write_all(&help_str).unwrap();
    eprintln!();
    exit(1);
}

type Result<T> = std::result::Result<T, Error>;

/// 設定ファイルの読み込みで発生しうるエラー
#[derive(Debug, Fail)]
pub enum Error {
    /// Config読み込みエラー
    #[fail(display = "failed load config: {}", _0)]
    Config(#[fail(cause)] config::Error),
}

impl From<config::Error> for Error {
    fn from(e: config::Error) -> Error {
        Error::Config(e)
    }
}
