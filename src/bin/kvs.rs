use clap::{Arg, App, AppSettings, SubCommand};
use std::process::exit;
use kvs::{Result};

fn main() -> Result<()>{
    fn open_store() -> Result<kvs::KvStore>  {
        kvs::KvStore::open(std::path::Path::new("./"))
    };

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
        ("get", Some(matches)) => {
            // clap enforces KEY argument.
            let key = matches.value_of("KEY").unwrap();

            let mut store = open_store()?;

            match store.get(key.to_string())? {
                Some(v) => println!("{}", v),
                None => println!("{}", "Key not found"),
            };

            Ok(())
        }
        ("set", Some(matches)) => {
            // clap enforces KEY argument.
            let key = matches.value_of("KEY").unwrap();
            // clap enforces VALUE argument.
            let value = matches.value_of("VALUE").unwrap();

            let mut store = open_store()?;

            store.set(key.to_string(), value.to_string())
        }
        ("rm", Some(matches)) => {
            // clap enforces KEY argument.
            let key = matches.value_of("KEY").unwrap();

            let mut store = open_store()?;

            match store.remove(key.to_string()) {
                Ok(()) => Ok(()),
                Err(kvs::KvStoreError::KeyNotFound) => {
                    println!("Key not found");
                    exit(1);
                },
                Err(e) => return Err(e),
            }
        }
        _ => unreachable!(),
    }
}
