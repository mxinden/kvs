use crate::KvsEngine;
use sled::Db;

use crate::error::{KvStoreError, Result};

/// Adapter for sled database.
pub struct SledKvsEngine {
    tree: sled::Db,
}

impl SledKvsEngine {
    /// Open a sled database on the given path returning the SledKvsEngine
    /// adapter.
    pub fn open(path: &std::path::Path) -> Result<SledKvsEngine> {
        let db = Db::start_default(path)?;

        Ok(SledKvsEngine{
            tree: db,
        })
    }
}

impl KvsEngine for SledKvsEngine {
    /// Set the value for the given key.
    fn set(&mut self, key: String, value: String) -> Result<()> {
        let key = sled::IVec::from(key.as_bytes());
        let value = sled::IVec::from(value.as_bytes());
        self.tree.set(key, value).map(|_| ()).map_err(|e| KvStoreError::PageCache(e))?;
        // TODO: Don't just flush on each call.
        self.tree.flush().map(|_| ()).map_err(|e| KvStoreError::PageCache(e))
    }

    /// Get the value of the given key.
    fn get(&mut self, key: String) -> Result<Option<String>> {
        let ivec = sled::IVec::from(key.as_bytes());
        self.tree.get(ivec).map_err(|e| KvStoreError::PageCache(e)).map(|v| {
            v.map(|v| {
                let value: Vec<u8> = v.to_vec();
                // TODO: Handle unwrap.
                std::str::from_utf8(&value).unwrap().to_string()
            })
        })
    }

    /// Remove the value of the given key.
    fn remove(&mut self, key: String) -> Result<()> {
        let key = sled::IVec::from(key.as_bytes());
        self.tree.del(key)
            .map_err(|e| KvStoreError::PageCache(e))
            .and_then(|v| {
                match v {
                    Some(_) => Ok(()),
                    None => Err(KvStoreError::KeyNotFound),
                }
        })?;

        // TODO: Don't just flush on each call.
        self.tree.flush().map(|_| ()).map_err(|e| KvStoreError::PageCache(e))
    }
}
