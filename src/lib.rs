#![deny(missing_docs)]

//! # KvStore
//! `KvStore` packages a key value store.

pub use error::{KvStoreError, Result};
pub use store::{KvStore};

#[macro_use]
extern crate failure_derive;

/// Errors thrown by KvStore.
pub mod error;

/// Types needed for client server network communication.
pub mod network;

/// Implementation of a basic thread pool.
pub mod thread_pool;

mod store;

/// Server implementation.
pub mod server;

/// KvsEngine represents the storage interface used by KvsServer.
pub trait KvsEngine: Clone + Send + 'static {
    /// Open a database.
    fn open(path: &std::path::Path) -> Result<Self>;
    /// Set the value for the given key.
    fn set(&self, key: String, value: String) -> Result<()>;
    /// Get the value of the given key.
    fn get(&self, key: String) -> Result<Option<String>>;
    /// Remove the value of the given key.
    fn remove(&self, key: String) -> Result<()>;
}

