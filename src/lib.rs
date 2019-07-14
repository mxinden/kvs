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
    OpenTmpDirFailure {
        /// Underlying io Error.
        #[cause]
        c: std::io::Error,
    },

    /// Failure when move file.
    #[fail(display = "failed to move file")]
    FileMoveFailure {
        /// Underlying io Error.
        #[cause]
        c: std::io::Error,
    },

    /// Failure when flush file.
    #[fail(display = "failed to flush file")]
    FileFlushFailure {
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
/// use tempfile::TempDir;
///
/// let temp_dir = TempDir::new().expect("unable to create temporary working directory");
/// let mut store = KvStore::open(temp_dir.path()).unwrap();
///
/// store.set("key1".to_owned(), "value1".to_owned()).unwrap();
/// store.set("key2".to_owned(), "value2".to_owned()).unwrap();
///
/// assert_eq!(store.get("key1".to_owned()).unwrap(), Some("value1".to_owned()));
/// assert_eq!(store.get("key2".to_owned()).unwrap(), Some("value2".to_owned()));
/// ```
pub struct KvStore {
    indexed_log_file: IndexedLogFile,
    // Needed later for compaction.
    path: std::path::PathBuf,
}

impl KvStore {
    /// Create new KvStore from file.
    pub fn open(path: &std::path::Path) -> Result<KvStore> {
        let log_file = IndexedLogFile::new(path)?;

        let kvs = KvStore {
            indexed_log_file: log_file,
            path: path.to_path_buf(),
        };

        Ok(kvs)
    }

    /// Returns the value for the given key.
    pub fn get(&mut self, k: String) -> Result<Option<String>> {
        let cmd = self.indexed_log_file.read(k);

        // Don't error when key is not found.
        if let Err(KvStoreError::KeyNotFound) = cmd {
            return Ok(None);
        }

        if let Some(cmd) =  cmd? {
            Ok(cmd.value())
        } else {
            Ok(None)
        }
    }

    /// Sets the value for the given key.
    pub fn set(&mut self, k: String, v: String) -> Result<()> {
        self.indexed_log_file.write(Command::Set{k: k.clone(),v})?;

        if self.should_compact() {
            return self.compact_log();
        }

        Ok(())
    }

    /// Removes the value of the given key.
    pub fn remove(&mut self, k: String) -> Result<()> {
        let exists = self.indexed_log_file.read(k.clone())?;
        if let None = exists {
            return Err(KvStoreError::KeyNotFound);
        }

        self.indexed_log_file.write(Command::Remove { k: k.clone() })
    }

    fn should_compact(&mut self) -> bool {
        let index_size = self.indexed_log_file.index.len();

        let num_writes = self.indexed_log_file.log_file.num_writes;

        num_writes > 2 * index_size
    }

    fn compact_log(&mut self) -> Result<()> {
        // TODO: No reason to clone this thing except borrow checker.
        let old_index = self.indexed_log_file.index.clone();

        let tmp_folder = tempfile::tempdir()
            .map_err(|c| KvStoreError::OpenTmpDirFailure { c })?;

        let mut tmp_indexed_log = IndexedLogFile::new(tmp_folder.path())?;

        for (k, _offset) in old_index.iter() {
            let cmd = self.indexed_log_file.read(k.to_string())?;

            let cmd = cmd.ok_or_else(|| KvStoreError::KeyNotFound)?;

            tmp_indexed_log.write(cmd)?;
        }

        std::fs::rename(tmp_folder.path().join("db"), self.path.join("db"))
            .map_err(|c| KvStoreError::FileMoveFailure{
                c,
            })?;

        // TODO: This rebuilds the index again? We still have it in
        // tmp_indexed_log.index.
        self.indexed_log_file = IndexedLogFile::new(&self.path)?;

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

    fn read(&mut self, key: String) -> Result<Option<Command>> {
        let offset = self.index.get(&key)
            .ok_or_else(|| KvStoreError::KeyNotFound)?;


        self.log_file.read_cmd(*offset as Offset)
    }

    fn write(&mut self, cmd: Command) -> Result<()> {
        let key = cmd.key();

        let offset = self.log_file.write_cmd(cmd)?;
        self.index.insert(key, offset);

        Ok(())
    }

    fn build_index(&mut self) -> Result<()>  {
        let mut log_size = 0;
        let mut offset: Offset = 0;

        let reader = self.log_file.get_reader(offset)?;

        let mut stream = serde_json::Deserializer::from_reader(reader)
            .into_iter::<Command>();

        while let Some(cmd) = stream.next() {
            log_size += 1;
            let cmd = cmd.map_err(|c| KvStoreError::DeserializationFailure { c })?;

            self.index.insert(cmd.key(), offset);

            offset = stream.byte_offset() as Offset;
        }

        self.log_file.num_writes = log_size;

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
    num_writes: usize,
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
            num_writes: 0,
        })
    }

    fn write_cmd(&mut self, cmd: Command) -> Result<Offset> {
        let offset = self.position;

        let serialized =
            serde_json::to_string(&cmd).map_err(|c| KvStoreError::SerializationFailure { c })?;

        self.position = offset + self.file.write(serialized.as_bytes())
            .map(|p| p as Offset)
            .map_err(|c| KvStoreError::WriteToFileFailure {
                c,
            })?;

        self.file.flush()
            .map_err(|c| KvStoreError::FileFlushFailure{c})?;

        self.num_writes += 1;

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
            if cmd.is_err() {
            }
            let cmd = cmd.map_err(|c| KvStoreError::DeserializationFailure { c })?;

            Ok(Some(cmd))
        } else {
            Ok(None)
        }
    }
}
