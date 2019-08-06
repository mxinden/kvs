use clap::{App, AppSettings, Arg};
use env_logger;
use log::error;

use std::io::Write;
use std::net::{TcpListener, TcpStream};

use std::fs;
use std::io::Read;

use kvs::network::{Req, Resp, SuccResp};

fn main() -> Result<()> {
    env_logger::init();

    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .setting(AppSettings::DisableHelpSubcommand)
        .setting(AppSettings::VersionlessSubcommands)
        .arg(
            Arg::with_name("addr")
                .long("addr")
                .takes_value(true)
                .help("specify the address to listen on")
                .default_value("[::1]:4000"),
        )
        .get_matches();

    error!(env!("CARGO_PKG_VERSION"));

    let addr = matches.value_of("addr").unwrap();
    error!("Listening on '{}'.", addr);

    let mut handler = Handler {
        db: Box::new(kvs::KvStore::open(std::path::Path::new("./"))?),
    };

    listen(addr.to_string(), &mut handler)?;

    Ok(())
}

struct Handler {
    db: Box<kvs::KvsEngine>,
}

impl Handler {
    fn handle(&mut self, stream: &mut TcpStream) -> Result<()> {
        let mut req_stream =
            serde_json::Deserializer::from_reader(stream.try_clone().unwrap()).into_iter::<Req>();

        let req = req_stream
            .next()
            .ok_or_else(|| ServerError::ClosedStream)??;

        let resp: Resp = match req {
            Req::Get(k) => self.db.get(k).map(|v| SuccResp::Get(v)),
            Req::Set(k, v) => self.db.set(k, v).map(|()| SuccResp::Set),
            Req::Remove(k) => self.db.remove(k).map(|()| SuccResp::Remove),
        }
        .map_err(|e| kvs::network::Error::Server(e.to_string()));

        let serialized = serde_json::to_string(&resp)?;

        stream.write_all(serialized.as_bytes())?;

        Ok(())
    }
}

fn listen(addr: String, handler: &mut Handler) -> Result<()> {
    let listener = TcpListener::bind(addr)?;

    for stream in listener.incoming() {
        handler.handle(&mut stream?)?;
    }

    Ok(())
}

type Result<T> = std::result::Result<T, ServerError>;

/// Error type for KvsServer.
#[derive(Debug)]
pub enum ServerError {
    KvStore(kvs::KvStoreError),
    ClosedStream,
    Io(std::io::Error),
    SerdeJson(serde_json::error::Error),
    EngineMissMatch {
        previous_engine: String,
        current_engine: String,
    },
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

impl From<serde_json::error::Error> for ServerError {
    fn from(err: serde_json::error::Error) -> ServerError {
        ServerError::SerdeJson(err)
    }
}

fn check_and_persist_engine(engine: String) -> Result<()> {
    let file_name = "./.engine".to_string();
    match fs::File::open(file_name.clone()) {
        Ok(mut f) => {
            let mut persisted_engine = String::new();

            f.read_to_string(&mut persisted_engine)?;

            if engine != persisted_engine {
                return Err(ServerError::EngineMissMatch {
                    previous_engine: persisted_engine,
                    current_engine: engine,
                });
            }

            return Ok(());
        }
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => {
                let mut file = fs::File::create(file_name)?;
                file.write_all(engine.as_bytes())?;
                Ok(())
            }
            _ => return Err(ServerError::Io(e)),
        },
    }
}
