use clap::{App, AppSettings, Arg};
use env_logger;
use kvs::thread_pool::ThreadPool;
use log::error;

use std::io::Write;
use std::net::{TcpListener, TcpStream};

use std::fs;
use std::io::Read;

use kvs::network::{Req, Resp, SuccResp};

fn main() -> Result<(), kvs::server::ServerError> {
    env_logger::init();

    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .setting(AppSettings::DisableHelpSubcommand)
        .setting(AppSettings::VersionlessSubcommands)
        .arg(
            Arg::with_name("thread-pool")
                .takes_value(true)
                .long("thread-pool")
                .help("specify thread pool strategy")
                .possible_values(&vec!["shared", "naive", "rayon"])
                .default_value("shared"),
        )
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

    match matches.value_of("thread-pool").unwrap() {
        "shared" => {
            kvs::server::Server::<kvs::KvStore, kvs::thread_pool::SharedQueueThreadPool>::new(
                std::path::Path::new("./"),
                10,
            )
            .listen(addr.to_string())
        }
        "rayon" => kvs::server::Server::<kvs::KvStore, kvs::thread_pool::RayonThreadPool>::new(
            std::path::Path::new("./"),
            10,
        )
        .listen(addr.to_string()),
        _ => unimplemented!(),
    }
}
