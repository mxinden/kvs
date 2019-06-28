extern crate clap;

use clap::{Arg, App, AppSettings, SubCommand};
use std::process::exit;

fn main() {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .setting(AppSettings::DisableHelpSubcommand)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::VersionlessSubcommands)
        .subcommand(SubCommand::with_name("get")
                    .about("get value for given key")
                    .arg(Arg::with_name("KEY").required(true))
        )
        .subcommand(SubCommand::with_name("set")
                    .about("set key with the given value")
                    .arg(Arg::with_name("KEY").required(true))
                    .arg(Arg::with_name("VALUE").required(true))
        )
        .subcommand(SubCommand::with_name("rm")
                    .about("remove value for the given key")
                    .arg(Arg::with_name("KEY").required(true))
        )
        .get_matches();

    match matches.subcommand() {
        ("get", Some(_matches)) => {
            eprintln!("unimplemented");
            exit(1);
        }
        ("set", Some(_matches)) => {
            eprintln!("unimplemented");
            exit(1);
        }
        ("rm", Some(_matches)) => {
            eprintln!("unimplemented");
            exit(1);
        }
        _ => unreachable!(),
    }
}
