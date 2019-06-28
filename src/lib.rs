#![deny(missing_docs)]

//! # KvStore
//! `KvStore` packages a key value store.

use std::collections::HashMap;

/// KvStore stores values by their key.
///
/// # Example
/// ``` rust
/// use kvs::KvStore;
///
/// let mut s = KvStore::new();
/// s.set("key1".to_owned(), "value1".to_owned());
/// assert_eq!(s.get("key1".to_owned()), Some("value1".to_owned()));
/// ```
#[derive(Default)]
pub struct KvStore {
    s: HashMap<String, String>,
}

impl KvStore {
    /// Returns a new key value store.
    pub fn new() -> KvStore {
        KvStore { s: HashMap::new() }
    }

    /// Returns the value for the given key.
    pub fn get(&self, k: String) -> Option<String> {
        self.s.get(&k).map(std::borrow::ToOwned::to_owned)
    }

    /// Sets the value for the given key.
    pub fn set(&mut self, k: String, v: String) {
        self.s.insert(k, v);
    }

    /// Removes the value of the given key.
    pub fn remove(&mut self, k: String) {
        self.s.remove(&k);
    }
}
