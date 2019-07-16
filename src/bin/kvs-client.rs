use clap::{Arg, App, AppSettings, SubCommand};
use std::process::exit;
use std::io::prelude::*;
use std::net::TcpStream;

fn main() -> Result<()>{
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .setting(AppSettings::DisableHelpSubcommand)
        // .setting(AppSettings::SubcommandRequiredElseHelp)
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

    // match matches.subcommand() {
    //     ("get", Some(matches)) => {
    //         // clap enforces KEY argument.
    //         let key = matches.value_of("KEY").unwrap();

    //         unimplemented!();
    //     }
    //     ("set", Some(matches)) => {
    //         // clap enforces KEY argument.
    //         let key = matches.value_of("KEY").unwrap();
    //         // clap enforces VALUE argument.
    //         let value = matches.value_of("VALUE").unwrap();

    //         unimplemented!();
    //     }
    //     ("rm", Some(matches)) => {
    //         // clap enforces KEY argument.
    //         let key = matches.value_of("KEY").unwrap();

    //         unimplemented!();
    //     }
    //     _ => unreachable!(),
    // }

    let mut stream = TcpStream::connect("[::1]:4000")?;

    Ok(())
}

type Result<T> = std::result::Result<T, ClientError>;

/// Error type for KvsClient.
#[derive(Debug)]
pub enum ClientError {
    KvStore(kvs::KvStoreError),
    Io(std::io::Error),
}

impl From<kvs::KvStoreError> for ClientError {
    fn from(err: kvs::KvStoreError) -> ClientError {
        ClientError::KvStore(err)
    }
}

impl From<std::io::Error> for ClientError {
    fn from(err: std::io::Error) -> ClientError {
        ClientError::Io(err)
    }
}
