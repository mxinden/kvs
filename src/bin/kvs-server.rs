use clap::{App, AppSettings, Arg, SubCommand};
use std::net::{TcpListener, TcpStream};
use std::process::exit;
use log::{info, warn};
use env_logger;

fn main() -> Result<()> {
    fn open_store() -> kvs::error::Result<kvs::KvStore>  {
        kvs::KvStore::open(std::path::Path::new("./"))
    };

    env_logger::init();

    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .setting(AppSettings::DisableHelpSubcommand)
        .setting(AppSettings::VersionlessSubcommands)
        .arg(
            Arg::with_name("engine")
                .takes_value(true)
                .long("engine")
                .help("specify database engine")
                .possible_values(&vec!["kvs", "sled"])
                .default_value("kvs"),
        )
        .arg(
            Arg::with_name("addr")
                .long("addr")
                .takes_value(true)
                .help("specify the address to listen on")
                .default_value("[::1]:4000"),
        )
        .get_matches();

    let addr = matches.value_of("addr").unwrap();
    info!("Listening on '{}'.", addr);

    let engine = matches.value_of("engine").unwrap();
    info!("Using engine '{}'.", engine);

    let handler = if engine == "kvs" {
        Handler {
            db: Box::new(open_store()?),
        }
    } else {
        unimplemented!();
    };

    listen(addr.to_string(), handler)?;

    Ok(())
}

struct Handler {
    db: Box<kvs::KvsEngine>,
}

impl Handler {
    fn handle(&self, stream: TcpStream) {
        info!("We got a Tcp stream");
    }
}

fn listen(addr: String, handler: Handler) -> std::io::Result<()> {
    // TODO: Remove unwrap.
    let listener = TcpListener::bind(addr).unwrap();

    for stream in listener.incoming() {
        handler.handle(stream?);
    }

    Ok(())
}

type Result<T> = std::result::Result<T, ServerError>;

/// Error type for KvsServer.
#[derive(Debug)]
pub enum ServerError {
    KvStore(kvs::KvStoreError),
    Io(std::io::Error),
}

impl From<kvs::KvStoreError> for ServerError {
    fn from(err: kvs::KvStoreError) -> ServerError {
        ServerError::KvStore(err)
    }
}

impl From<std::io::Error> for ServerError {
    fn from(err: std::io::Error) -> ServerError {
        ServerError::Io(err)
    }
}
