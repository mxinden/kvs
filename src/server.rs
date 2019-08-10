use crate::network::{Req, Resp, SuccResp};
use log::error;
use std::fs;
use std::io::Read;
use std::io::Write;
use std::net::{TcpListener, TcpStream};

/// Represents a database server instance, wrapping a datastore, accepting
/// incoming connections.
#[derive(Clone)]
pub struct Server<E, P>
where
    E: crate::KvsEngine + Sync + std::panic::RefUnwindSafe + std::panic::UnwindSafe,
    P: crate::thread_pool::ThreadPool,
{
    db: E,
    pool: P,
}

impl<E, P> Server<E, P>
where
    E: crate::KvsEngine + Sync + std::panic::RefUnwindSafe + std::panic::UnwindSafe,
    P: crate::thread_pool::ThreadPool,
{
    /// Construct a new server.
    pub fn new(db_path: &std::path::Path, threads: u32) -> Server<E, P> {
        let db = <E>::open(db_path).unwrap();
        let pool = <P>::new(threads).unwrap();

        Server { db, pool }
    }

    /// Listen on the given address for incoming requests.
    pub fn listen(&self, addr: String) -> Result<()> {
        let listener = TcpListener::bind(addr)?;

        for stream in listener.incoming() {
            let stream = stream?;
            let db = self.db.clone();
            self.pool.spawn(move || match handle(stream, db) {
                Ok(()) => {}
                Err(e) => error!("failed to handle stream: {:?}", e),
            })
        }

        Ok(())
    }
}

fn handle<E>(mut stream: TcpStream, db: E) -> Result<()>
where
    E: crate::KvsEngine + Sync + std::panic::RefUnwindSafe,
{
    let mut req_stream = serde_json::Deserializer::from_reader(&stream).into_iter::<Req>();

    let req = req_stream
        .next()
        .ok_or_else(|| ServerError::ClosedStream)??;

    let resp: Resp = match req {
        Req::Get(k) => db.get(k).map(|v| SuccResp::Get(v)),
        Req::Set(k, v) => db.set(k, v).map(|()| SuccResp::Set),
        Req::Remove(k) => db.remove(k).map(|()| SuccResp::Remove),
    }
    .map_err(|e| crate::network::Error::Server(e.to_string()));

    let serialized = serde_json::to_string(&resp)?;

    stream.write_all(serialized.as_bytes())?;

    Ok(())
}

type Result<T> = std::result::Result<T, ServerError>;

/// Error type for KvsServer.
#[derive(Debug)]
pub enum ServerError {
    /// KvStore error wrapper.
    KvStore(crate::KvStoreError),
    /// Closed stream error.
    ClosedStream,
    /// Io error wrapper.
    Io(std::io::Error),
    /// Serde json error wrapper.
    SerdeJson(serde_json::error::Error),
}

impl From<crate::KvStoreError> for ServerError {
    fn from(err: crate::KvStoreError) -> ServerError {
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
