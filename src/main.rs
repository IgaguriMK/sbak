use std::process::exit;
use std::io::{stderr, Write};

use clap::{crate_authors, crate_description, crate_name, App};

use sbak::sub::sub_commands;
use sbak::version::version;

fn main() {
    let subs = sub_commands();

    let ver = version(8);
    let mut app = App::new(crate_name!())
        .author(crate_authors!("\n"))
        .about(crate_description!())
        .version(ver.as_str())
        .subcommands(subs.arg_defs());

    let mut help_str = Vec::<u8>::new();
    app.write_long_help(&mut help_str).unwrap();
    
    let matches = app.get_matches();

    if let (subcmd_name, Some(matches)) = matches.subcommand() {
        subs.execute(subcmd_name, matches); // 成功したらそのままプロセスを終了する
    }

    eprintln!("Need subcommand.");
    eprintln!();
    let mut out = stderr();
    out.write_all(&help_str).unwrap();
    eprintln!();
    exit(1);
}