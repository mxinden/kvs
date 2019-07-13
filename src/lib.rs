#![deny(missing_docs)]

//! # KvStore
//! `KvStore` packages a key value store.

use serde::{Deserialize, Serialize};
use std::io::Seek;
use std::io::Write;
#[macro_use]
extern crate failure_derive;

/// Error returned by the KvStore library.
#[derive(Debug, Fail)]
pub enum KvStoreError {
    /// Generic failure. TODO remove!
    #[fail(display = "generic failure")]
    GenericFailure,

    /// Failure when opening database file.
    #[fail(display = "failed to open file {}", name)]
    OpenFileFailure {
        /// Underlying io Error.
        #[cause]
        io_error: std::io::Error,
        /// Name of the file.
        name: String,
    },

    /// Failure when serializing input.
    #[fail(display = "failed to serialize input")]
    SerializationFailure {
        /// Underlying io Error.
        #[cause]
        c: serde_json::error::Error,
    },

    /// Failure when deserializing input.
    #[fail(display = "failed to deserialize input")]
    DeserializationFailure {
        /// Underlying io Error.
        #[cause]
        c: serde_json::error::Error,
    },

    /// Failure writing to file.
    #[fail(display = "failed to write to file")]
    WriteToFileFailure {
        /// Underlying io Error.
        #[cause]
        c: std::io::Error,
    },

    /// Failure seeking file.
    #[fail(display = "failed to seek file")]
    SeekFileFailure {
        /// Underlying io Error.
        #[cause]
        c: std::io::Error,
    },

    /// Failure finding key
    #[fail(display = "Key not found")]
    KeyNotFound,
}

/// Result type returned by the KvStore library.
pub type Result<T> = std::result::Result<T, KvStoreError>;

#[derive(Serialize, Deserialize)]
enum Command {
    Set { k: String, v: String },
    Remove { k: String },
}

impl Command {
    fn key(&self) -> String {
        match self {
            Command::Set { k, v: _} => k.to_string(),
            Command::Remove { k } => k.to_string(),
        }
    }

    fn value(&self) -> Option<String> {
        match self {
            Command::Set { k: _, v } => Some(v.to_string()),
            Command::Remove { k: _ } => None,
        }
    }
}

/// KvStore stores values by their key.
///
/// # Example
/// TODO: Update
/// ``` rust
/// use kvs::KvStore;
///
/// let mut s = KvStore::new();
/// s.set("key1".to_owned(), "value1".to_owned());
/// assert_eq!(s.get("key1".to_owned()), Some("value1".to_owned()));
/// ```
pub struct KvStore {
    reader: std::io::BufReader<std::fs::File>,
    file: std::fs::File,
    index: std::collections::HashMap<String, usize>,
}

impl KvStore {
    /// Create new KvStore from file.
    pub fn open(path: &std::path::Path) -> Result<KvStore> {
        let path = path.join("db");
        let write_file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(path.clone())
            .map_err(|e| KvStoreError::OpenFileFailure {
                io_error: e,
                name: path.display().to_string(),
            })?;

        let read_file = std::fs::OpenOptions::new()
            .read(true)
            .open(path.clone())
            .map_err(|e| KvStoreError::OpenFileFailure {
                io_error: e,
                name: path.display().to_string(),
            })?;

        let reader = std::io::BufReader::new(read_file);

        let mut kvs = KvStore {
            reader: reader,
            file: write_file,
            index: std::collections::HashMap::new(),
        };

        kvs.build_index()?;

        Ok(kvs)
    }

    fn build_index(&mut self) -> Result<()> {
        self.reader
            .seek(std::io::SeekFrom::Start(0))
            .map_err(|c| KvStoreError::SeekFileFailure { c })?;

        let mut stream =
            serde_json::Deserializer::from_reader(&mut self.reader).into_iter::<Command>();

        let mut offset = 0;

        while let Some(cmd) = stream.next() {
            let cmd = cmd.map_err(|c| KvStoreError::DeserializationFailure { c })?;

            self.index.insert(cmd.key(), offset);

            offset = stream.byte_offset();
        }

        Ok(())
    }

    /// Returns the value for the given key.
    pub fn get(&mut self, k: String) -> Result<Option<String>> {
        let offset = self.index.get(&k);

        if offset.is_none() {
            // We don't want to error when the key is not found.
            return Ok(None);
        }

        let offset = offset.unwrap();

        self.reader
            .seek(std::io::SeekFrom::Start(*offset as u64))
            .map_err(|c| KvStoreError::SeekFileFailure { c })?;

        let mut stream = serde_json::Deserializer::from_reader(&mut self.reader)
            .into_iter::<Command>();

        if let Some(cmd) = stream.next() {
            let cmd = cmd.map_err(|c| KvStoreError::DeserializationFailure { c })?;

            return Ok(cmd.value());
        }

        Ok(None)
    }

    /// Sets the value for the given key.
    pub fn set(&mut self, k: String, v: String) -> Result<()> {
        let cmd = Command::Set { k, v };

        self.cmd_to_file(cmd)
    }

    /// Removes the value of the given key.
    pub fn remove(&mut self, k: String) -> Result<()> {
        match self.index.get(&k) {
            Some(_v) => {}
            None => return Err(KvStoreError::KeyNotFound),
        };

        let cmd = Command::Remove { k };

        self.cmd_to_file(cmd)
    }

    fn cmd_to_file(&mut self, cmd: Command) -> Result<()> {
        // Seek to end of file.
        let position = self.reader
            .seek(std::io::SeekFrom::End(0))
            .map_err(|c| KvStoreError::SeekFileFailure { c })?;

        let serialized =
            serde_json::to_string(&cmd).map_err(|c| KvStoreError::SerializationFailure { c })?;

        writeln!(self.file, "{}", serialized).map_err(|c| KvStoreError::WriteToFileFailure { c })?;

        // Update index.
        self.index.insert(cmd.key(), position as usize);

        Ok(())
    }
}
