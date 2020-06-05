use std::io::{stderr, Write};
use std::process::exit;

use anyhow::{Context, Result};
use clap::{crate_description, crate_name, App, Arg};
use log::{error, trace};

use sbak::config::{auto_load, load};
use sbak::smalllog;
use sbak::sub::sub_commands;
use sbak::version::version;

fn main() {
    smalllog::init();

    if let Err(e) = w_main() {
        error!("Error: {}", e);
        for c in e.chain().skip(1) {
            error!("    at: {}", c);
        }
        exit(1);
    }
}

fn w_main() -> Result<()> {
    let mut config = auto_load().context("loading aut-detected config file")?;
    config.apply_log();

    let subs = sub_commands();

    let ver = version(8);
    let mut app = App::new(crate_name!())
        .author("Igaguri <igagurimk@gmail.com>")
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
            Arg::with_name("log_level")
                .short("L")
                .long("log-level")
                .takes_value(true)
                .help("Log level"),
        )
        .subcommands(subs.arg_defs());

    let mut help_str = Vec::<u8>::new();
    app.write_long_help(&mut help_str).unwrap();

    let matches = app.get_matches();

    if let Some(extra_config_file) = matches.value_of("config") {
        let extra_config = load(extra_config_file)
            .with_context(|| format!("loading optional config file '{}'", extra_config_file))?;
        config = config.merged(&extra_config);
        config.apply_log();
    }

    if let Some(level_str) = matches.value_of("log_level") {
        config
            .set_log_level_str(level_str)
            .context("applying log level specified by command-line option")?;
        config.apply_log();
    }

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
