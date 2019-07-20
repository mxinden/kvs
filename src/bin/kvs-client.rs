use clap::{Arg, App, AppSettings, SubCommand};
use std::process::exit;
use std::io::prelude::*;
use std::net::TcpStream;
use log::{info, warn};
use env_logger;

use kvs::network::{ Req, Resp, SuccResp};

fn main() -> Result<()>{
    env_logger::init();

    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .setting(AppSettings::DisableHelpSubcommand)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::VersionlessSubcommands)
        .arg(
            Arg::with_name("addr")
                .long("addr")
                .takes_value(true)
                .help("specify the address to listen on")
                .default_value("[::1]:4000")
                .global(true),
        )
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

    let addr = matches.value_of("addr").unwrap();
    info!("Connecting to '{}'.", addr);

    let mut stream = TcpStream::connect(addr.to_string())?;

    let req = match matches.subcommand() {
        ("get", Some(matches)) => {
            // clap enforces KEY argument.
            let key = matches.value_of("KEY").unwrap();

            Req::Get(key.to_string())
        }
        ("set", Some(matches)) => {
            // clap enforces KEY argument.
            let key = matches.value_of("KEY").unwrap();
            // clap enforces VALUE argument.
            let value = matches.value_of("VALUE").unwrap();

            Req::Set(key.to_string(), value.to_string())
        }
        ("rm", Some(matches)) => {
            // clap enforces KEY argument.
            let key = matches.value_of("KEY").unwrap();

            Req::Remove(key.to_string())
        }
        _ => unreachable!(),
    };

    let serialized = serde_json::to_string(&req)?;

    stream.write_all(serialized.as_bytes())?;

    let mut resp_stream =
        serde_json::Deserializer::from_reader(stream.try_clone().unwrap()).into_iter::<Resp>();

    let resp = resp_stream
        .next()
        .ok_or_else(|| ClientError::ClosedStream)??;

    let key_not_found = "Key not found".to_string();

    match resp {
        Ok(SuccResp::Get(v)) => {
            match v {
                None => println!("{}", key_not_found),
                Some(v) => println!("{}", v),
            }

            Ok(())
        },
        Ok(SuccResp::Set) | Ok(SuccResp::Remove) => {info!("success"); Ok(())},
        Err(kvs::network::Error::Server(e)) => {
            if e == key_not_found {
                eprintln!("{}", key_not_found)
            }

            Err(ClientError::NetworkError(kvs::network::Error::Server(e)))
        }
    }
}

type Result<T> = std::result::Result<T, ClientError>;

/// Error type for KvsClient.
#[derive(Debug)]
pub enum ClientError {
    KvStore(kvs::KvStoreError),
    ClosedStream,
    Io(std::io::Error),
    SerdeJson(serde_json::error::Error),
    NetworkError(kvs::network::Error),
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

impl From<serde_json::error::Error> for ClientError {
    fn from(err: serde_json::error::Error) -> ClientError {
        ClientError::SerdeJson(err)
    }
}

impl From<kvs::network::Error> for ClientError {
    fn from(err: kvs::network::Error) -> ClientError {
        ClientError::NetworkError(err)
    }
}
