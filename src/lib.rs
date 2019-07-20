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

mod store;

/// Bindings for sled database.
pub mod sled;

/// KvsEngine represents the storage interface used by KvsServer.
pub trait KvsEngine {
    /// Set the value for the given key.
    fn set(&mut self, key: String, value: String) -> Result<()>;
    /// Get the value of the given key.
    fn get(&mut self, key: String) -> Result<Option<String>>;
    /// Remove the value of the given key.
    fn remove(&mut self, key: String) -> Result<()>;
}

