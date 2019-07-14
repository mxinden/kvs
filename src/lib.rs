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
        c: std::io::Error,
        /// Name of the file.
        name: String,
    },

    /// Failure when opening temporary file.
    #[fail(display = "failed to open temporary file")]
    OpenTempFileFailure {
        /// Underlying io Error.
        #[cause]
        c: std::io::Error,
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
    indexed_log_file: IndexedLogFile,
}

impl KvStore {
    /// Create new KvStore from file.
    pub fn open(path: &std::path::Path) -> Result<KvStore> {
        let log_file = IndexedLogFile::new(path)?;

        let kvs = KvStore {
            indexed_log_file: log_file,
        };

        Ok(kvs)
    }

    /// Returns the value for the given key.
    pub fn get(&mut self, k: String) -> Result<Option<String>> {
        let value = self.indexed_log_file.get(k);

        // Don't error when key is not found.
        if let Err(KvStoreError::KeyNotFound) = value {
            return Ok(None);
        }

        value
    }

    /// Sets the value for the given key.
    pub fn set(&mut self, k: String, v: String) -> Result<()> {
        self.indexed_log_file.set(k, v)
    }

    /// Removes the value of the given key.
    pub fn remove(&mut self, k: String) -> Result<()> {
        self.indexed_log_file.remove(k)
    }

    fn compact_log(&mut self) -> Result<()> {
        let mut tmp_file = tempfile::tempfile()
            .map_err(|c| KvStoreError::OpenTempFileFailure { c })?;

        Ok(())
    }
}

type Offset = u64;

struct IndexedLogFile {
    log_file: LogFile,
    index: std::collections::HashMap<String, Offset>,
}

impl IndexedLogFile {
    fn new(path: &std::path::Path) -> Result<Self> {
        let log_file = LogFile::new(path)?;

        let index = std::collections::HashMap::new();

        let mut indexed_log_file = IndexedLogFile{
            log_file,
            index,
        };

        indexed_log_file.build_index()?;

        Ok(indexed_log_file)
    }

    fn get(&mut self, key: String) -> Result<Option<String>> {
        let offset = self.index.get(&key)
            .ok_or_else(|| KvStoreError::KeyNotFound)?;

        if let Some(cmd) =  self.log_file.read_cmd(*offset as Offset)? {
            Ok(cmd.value())
        } else {
            Ok(None)
        }
    }

    fn set(&mut self, k: String, v: String) -> Result<()> {
        let offset = self.log_file.write_cmd(Command::Set{k: k.clone(),v})?;

        self.index.insert(k, offset);

        Ok(())
    }

    fn remove(&mut self, k: String) -> Result<()> {
        match self.index.get(&k) {
            Some(_v) => {}
            None => return Err(KvStoreError::KeyNotFound),
        };

        let cmd = Command::Remove { k: k.clone() };

        let offset = self.log_file.write_cmd(cmd)?;

        self.index.insert(k, offset);

        Ok(())
    }

    fn build_index(&mut self) -> Result<()>  {
        let mut offset: Offset = 0;

        let reader = self.log_file.get_reader(offset)?;

        let mut stream = serde_json::Deserializer::from_reader(reader)
            .into_iter::<Command>();

        while let Some(cmd) = stream.next() {
            let cmd = cmd.map_err(|c| KvStoreError::DeserializationFailure { c })?;

            self.index.insert(cmd.key(), offset);

            offset = stream.byte_offset() as Offset;
        }

        Ok(())
    }
}

// LogFile represents a database log file on disk.
struct LogFile {
    reader: std::io::BufReader<std::fs::File>,
    // TODO: How about a buffered writer that we can flush once after
    // compaction?
    file: std::fs::File,
    // Position within the file.
    position: Offset,
}

impl LogFile {
    fn new(path: &std::path::Path) -> Result<LogFile> {
        let path = path.join("db");

        let mut write_file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(path.clone())
            .map_err(|c| KvStoreError::OpenFileFailure {
                c,
                name: path.display().to_string(),
            })?;

        // Get end of file.
        let position = write_file
            .seek(std::io::SeekFrom::End(0))
            .map_err(|c| KvStoreError::SeekFileFailure { c })?;

        let read_file = std::fs::OpenOptions::new()
            .read(true)
            .open(path.clone())
            .map_err(|c| KvStoreError::OpenFileFailure {
                c,
                name: path.display().to_string(),
            })?;
        let reader = std::io::BufReader::new(read_file);

        return Ok(LogFile{
            reader: reader,
            file: write_file,
            position,
        })
    }

    fn write_cmd(&mut self, cmd: Command) -> Result<Offset> {
        let offset = self.position;

        let serialized =
            serde_json::to_string(&cmd).map_err(|c| KvStoreError::SerializationFailure { c })?;

        self.position = self.file.write(serialized.as_bytes())
            .map(|p| p as Offset)
            .map_err(|c| KvStoreError::WriteToFileFailure {
                c,
            })?;

        Ok(offset)
    }

    // TODO: Call this iter()?
    fn get_reader(&mut self, offset: Offset) -> Result<&mut std::io::BufReader<std::fs::File>>{
        self.reader
            .seek(std::io::SeekFrom::Start(offset))
            .map_err(|c| KvStoreError::SeekFileFailure { c })?;

        Ok(&mut self.reader)
    }

    fn read_cmd(&mut self, offset: Offset) -> Result<Option<Command>>  {
        self.reader
            .seek(std::io::SeekFrom::Start(offset ))
            .map_err(|c| KvStoreError::SeekFileFailure { c })?;

        let mut stream = serde_json::Deserializer::from_reader(&mut self.reader)
            .into_iter::<Command>();

        if let Some(cmd) = stream.next() {
            let cmd = cmd.map_err(|c| KvStoreError::DeserializationFailure { c })?;

            Ok(Some(cmd))
        } else {
            Ok(None)
        }
    }
}
