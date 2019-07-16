#![deny(missing_docs)]

//! # KvStore
//! `KvStore` packages a key value store.

pub use error::{KvStoreError, Result};
pub use store::{KvStore};

#[macro_use]
extern crate failure_derive;

/// Errors thrown by KvStore.
pub mod error;

mod store;

/// KvsEngine represents the storage interface used by KvsServer.
pub trait KvsEngine {}

