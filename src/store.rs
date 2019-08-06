use crate::error::{Result, KvStoreError};
use crate::KvsEngine;
use serde::{Deserialize, Serialize};
use std::io::Seek;
use std::io::Write;
use std::sync::{Arc, Mutex};


/// KvStore stores values by their key.
///
/// # Example
///
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
///
#[derive(Clone)]
pub struct KvStore {
    indexed_log_file: Arc<Mutex<IndexedLogFile>>,
    // Needed later for compaction when replacing the old version by the
    // compacted one.
    path: std::path::PathBuf,
}

impl KvsEngine for KvStore {
    fn set(&self, key: String, value: String) -> Result<()> {
        KvStore::set(self, key, value)
    }
    fn get(&self, key: String) -> Result<Option<String>>{
        self.get(key)
    }
    fn remove(&self, key: String) -> Result<()> {
        self.remove(key)
    }
}

impl KvStore {
    /// Create new KvStore from file.
    pub fn open(path: &std::path::Path) -> Result<KvStore> {
        let log_file = IndexedLogFile::new(path)?;

        let kvs = KvStore {
            indexed_log_file: Arc::new(Mutex::new(log_file)),
            path: path.to_path_buf(),
        };

        Ok(kvs)
    }

    /// Returns the value for the given key.
    pub fn get(&self, k: String) -> Result<Option<String>> {
        let cmd = self.indexed_log_file.lock().unwrap().read(k);

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
    pub fn set(&self, k: String, v: String) -> Result<()> {
        self.indexed_log_file.lock().unwrap().write(Command::Set{k: k.clone(),v})?;

        if self.should_compact() {
            return self.compact_log();
        }

        Ok(())
    }

    /// Removes the value of the given key.
    pub fn remove(&self, k: String) -> Result<()> {
        let exists = self.indexed_log_file.lock().unwrap().read(k.clone())?;
        if exists.is_none() {
            return Err(KvStoreError::KeyNotFound);
        }

        self.indexed_log_file.lock().unwrap().write(Command::Remove { k: k.clone() })
    }

    fn should_compact(&self) -> bool {
        let index_size = self.indexed_log_file.lock().unwrap().index.len();

        let num_writes = self.indexed_log_file.lock().unwrap().log_file.num_writes;

        num_writes > 2 * index_size
    }

    fn compact_log(&self) -> Result<()> {
        // TODO: No reason to clone this thing except borrow checker.
        let old_index = self.indexed_log_file.lock().unwrap().index.clone();

        let tmp_folder = tempfile::tempdir()
            .map_err(|c| KvStoreError::OpenTmpDirFailure { c })?;

        let mut tmp_indexed_log = IndexedLogFile::new(tmp_folder.path())?;

        for (k, _offset) in old_index.iter() {
            let cmd = self.indexed_log_file.lock().unwrap().read(k.to_string())?;

            let cmd = cmd.ok_or_else(|| KvStoreError::KeyNotFound)?;

            tmp_indexed_log.write(cmd)?;
        }

        std::fs::rename(tmp_folder.path().join("db"), self.path.join("db"))
            .map_err(|c| KvStoreError::FileMoveFailure{
                c,
            })?;

        // TODO: This rebuilds the index again? We still have it in
        // tmp_indexed_log.index.
        *self.indexed_log_file.lock().unwrap() = IndexedLogFile::new(&self.path)?;

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

/// LogFile represents a database log file on disk.
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

        Ok(LogFile{
            reader,
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
            let cmd = cmd.map_err(|c| KvStoreError::DeserializationFailure { c })?;

            Ok(Some(cmd))
        } else {
            Ok(None)
        }
    }
}

#[derive(Serialize, Deserialize)]
enum Command {
    Set { k: String, v: String },
    Remove { k: String },
}

impl Command {
    fn key(&self) -> String {
        match self {
            Command::Set { k, .. } => k.to_string(),
            Command::Remove { k } => k.to_string(),
        }
    }

    fn value(&self) -> Option<String> {
        match self {
            Command::Set {  v, .. } => Some(v.to_string()),
            Command::Remove { .. } => None,
        }
    }
}

