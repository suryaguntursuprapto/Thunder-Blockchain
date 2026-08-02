// ---------------------------------------------------------------------------
//  Thunder Blockchain — RocksDB Storage Layer
// ---------------------------------------------------------------------------
//  A thin wrapper around `rocksdb` providing a clean key-value API.
// ---------------------------------------------------------------------------

use rocksdb::{Options, DB};


/// Persistent key-value storage backed by RocksDB.
pub struct Storage {
    db: DB,
}

impl Storage {
    /// Open (or create) a RocksDB database at the given path.
    pub fn new(path: &str) -> Self {
        std::fs::create_dir_all(path).expect("failed to create storage directory");
        
        let mut opts = Options::default();
        opts.create_if_missing(true);
        // Additional optimizations for high throughput can be added to `opts` here.

        let db = DB::open(&opts, path).expect("failed to open RocksDB");
        Self { db }
    }

    /// Store a key-value pair.
    pub fn put(&self, key: &[u8], value: &[u8]) {
        self.db
            .put(key, value)
            .expect("RocksDB put failed");
    }

    /// Retrieve the value for a key. Returns `None` if not found.
    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.db.get(key).unwrap_or(None)
    }

    /// Delete a key.
    pub fn delete(&self, key: &[u8]) {
        self.db
            .delete(key)
            .expect("RocksDB delete failed");
    }

    /// Check whether a key exists.
    pub fn contains(&self, key: &[u8]) -> bool {
        self.db.get(key).unwrap_or(None).is_some()
    }

    /// Force a flush to disk. (RocksDB flushes atomically)
    pub fn flush(&self) {
        self.db.flush().expect("RocksDB flush failed");
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_storage() -> Storage {
        let dir = std::env::temp_dir().join(format!("thunder_rocks_{}", rand::random::<u64>()));
        Storage::new(dir.to_str().unwrap())
    }

    #[test]
    fn test_put_get() {
        let store = temp_storage();
        store.put(b"key1", b"value1");
        assert_eq!(store.get(b"key1"), Some(b"value1".to_vec()));
    }

    #[test]
    fn test_get_nonexistent() {
        let store = temp_storage();
        assert_eq!(store.get(b"nonexistent"), None);
    }

    #[test]
    fn test_delete() {
        let store = temp_storage();
        store.put(b"key2", b"value2");
        store.delete(b"key2");
        assert_eq!(store.get(b"key2"), None);
    }

    #[test]
    fn test_overwrite() {
        let store = temp_storage();
        store.put(b"key3", b"old");
        store.put(b"key3", b"new");
        assert_eq!(store.get(b"key3"), Some(b"new".to_vec()));
    }

    #[test]
    fn test_contains() {
        let store = temp_storage();
        assert!(!store.contains(b"key4"));
        store.put(b"key4", b"val");
        assert!(store.contains(b"key4"));
    }
}
