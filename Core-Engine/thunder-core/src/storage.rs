// ---------------------------------------------------------------------------
//  Thunder Blockchain — LevelDB Storage Layer
// ---------------------------------------------------------------------------
//  A thin wrapper around `rusty-leveldb` providing a clean key-value API.
// ---------------------------------------------------------------------------

use rusty_leveldb::{Options, DB};
use std::path::Path;

/// Persistent key-value storage backed by LevelDB.
pub struct Storage {
    db: DB,
}

impl Storage {
    /// Open (or create) a LevelDB database at the given path.
    pub fn new(path: &str) -> Self {
        let mut opts = Options::default();
        opts.create_if_missing = true;
        let db = DB::open(Path::new(path), opts).expect("failed to open LevelDB");
        Self { db }
    }

    /// Store a key-value pair.
    pub fn put(&mut self, key: &[u8], value: &[u8]) {
        self.db
            .put(key, value)
            .expect("LevelDB put failed");
    }

    /// Retrieve the value for a key.  Returns `None` if not found.
    pub fn get(&mut self, key: &[u8]) -> Option<Vec<u8>> {
        self.db.get(key)
    }

    /// Delete a key.
    pub fn delete(&mut self, key: &[u8]) {
        self.db
            .delete(key)
            .expect("LevelDB delete failed");
    }

    /// Check whether a key exists.
    pub fn contains(&mut self, key: &[u8]) -> bool {
        self.db.get(key).is_some()
    }

    /// Force a compaction / flush to disk.
    pub fn flush(&mut self) {
        self.db
            .flush()
            .expect("LevelDB flush failed");
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_storage() -> Storage {
        let dir = std::env::temp_dir().join(format!("thunder_db_test_{}", rand::random::<u64>()));
        Storage::new(dir.to_str().unwrap())
    }

    #[test]
    fn test_put_get() {
        let mut store = temp_storage();
        store.put(b"key1", b"value1");
        assert_eq!(store.get(b"key1"), Some(b"value1".to_vec()));
    }

    #[test]
    fn test_get_nonexistent() {
        let mut store = temp_storage();
        assert_eq!(store.get(b"nonexistent"), None);
    }

    #[test]
    fn test_delete() {
        let mut store = temp_storage();
        store.put(b"key2", b"value2");
        store.delete(b"key2");
        assert_eq!(store.get(b"key2"), None);
    }

    #[test]
    fn test_overwrite() {
        let mut store = temp_storage();
        store.put(b"key3", b"old");
        store.put(b"key3", b"new");
        assert_eq!(store.get(b"key3"), Some(b"new".to_vec()));
    }

    #[test]
    fn test_contains() {
        let mut store = temp_storage();
        assert!(!store.contains(b"key4"));
        store.put(b"key4", b"val");
        assert!(store.contains(b"key4"));
    }
}
