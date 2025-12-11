//! Embedding cache to avoid redundant API calls.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

pub type CacheResult<T> = Result<T, CacheError>;

/// Cached embedding entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry {
    text_hash: u64,
    embedding: Vec<f32>,
    model: String,
    timestamp: i64,
}

/// Simple disk-based embedding cache.
pub struct EmbeddingCache {
    cache_dir: PathBuf,
    memory_cache: HashMap<u64, Vec<f32>>,
    max_memory_entries: usize,
}

impl EmbeddingCache {
    pub fn new(cache_dir: PathBuf) -> CacheResult<Self> {
        fs::create_dir_all(&cache_dir)?;
        Ok(Self {
            cache_dir,
            memory_cache: HashMap::new(),
            max_memory_entries: 1000, // Keep 1000 most recent in memory
        })
    }

    /// Get cached embedding for text.
    pub fn get(&mut self, text: &str, model: &str) -> CacheResult<Option<Vec<f32>>> {
        let hash = Self::hash_text(text, model);

        // Check memory cache first
        if let Some(embedding) = self.memory_cache.get(&hash) {
            return Ok(Some(embedding.clone()));
        }

        // Check disk cache
        let cache_file = self.cache_dir.join(format!("{}.json", hash));
        if cache_file.exists() {
            let contents = fs::read_to_string(&cache_file)?;
            let entry: CacheEntry = serde_json::from_str(&contents)?;
            
            // Store in memory cache
            self.memory_cache.insert(hash, entry.embedding.clone());
            self.trim_memory_cache();
            
            return Ok(Some(entry.embedding));
        }

        Ok(None)
    }

    /// Store embedding in cache.
    pub fn put(&mut self, text: &str, model: &str, embedding: Vec<f32>) -> CacheResult<()> {
        let hash = Self::hash_text(text, model);

        // Store in memory
        self.memory_cache.insert(hash, embedding.clone());
        self.trim_memory_cache();

        // Store on disk
        let entry = CacheEntry {
            text_hash: hash,
            embedding,
            model: model.to_string(),
            timestamp: chrono::Utc::now().timestamp(),
        };

        let cache_file = self.cache_dir.join(format!("{}.json", hash));
        let json = serde_json::to_string(&entry)?;
        fs::write(cache_file, json)?;

        Ok(())
    }

    /// Clear old cache entries (older than `days` days).
    pub fn clear_old(&self, days: u64) -> CacheResult<usize> {
        let cutoff = chrono::Utc::now().timestamp() - (days as i64 * 86400);
        let mut removed = 0;

        for entry in fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            if let Ok(contents) = fs::read_to_string(entry.path()) {
                if let Ok(cache_entry) = serde_json::from_str::<CacheEntry>(&contents) {
                    if cache_entry.timestamp < cutoff {
                        fs::remove_file(entry.path())?;
                        removed += 1;
                    }
                }
            }
        }

        Ok(removed)
    }

    fn hash_text(text: &str, model: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        model.hash(&mut hasher);
        hasher.finish()
    }

    fn trim_memory_cache(&mut self) {
        if self.memory_cache.len() > self.max_memory_entries {
            // Remove oldest entries (simple: just clear half)
            let to_remove = self.memory_cache.len() - self.max_memory_entries;
            let keys: Vec<u64> = self.memory_cache.keys().take(to_remove).copied().collect();
            for key in keys {
                self.memory_cache.remove(&key);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cache_basic() {
        let temp_dir = TempDir::new().unwrap();
        let mut cache = EmbeddingCache::new(temp_dir.path().to_path_buf()).unwrap();

        let text = "Hello, world!";
        let model = "test-model";
        let embedding = vec![0.1, 0.2, 0.3];

        // Initially empty
        assert!(cache.get(text, model).unwrap().is_none());

        // Store and retrieve
        cache.put(text, model, embedding.clone()).unwrap();
        let retrieved = cache.get(text, model).unwrap().unwrap();
        assert_eq!(retrieved, embedding);
    }

    #[test]
    fn test_cache_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let embedding = vec![1.0, 2.0, 3.0];

        {
            let mut cache = EmbeddingCache::new(temp_dir.path().to_path_buf()).unwrap();
            cache.put("test", "model", embedding.clone()).unwrap();
        }

        // Create new cache instance (should load from disk)
        {
            let mut cache = EmbeddingCache::new(temp_dir.path().to_path_buf()).unwrap();
            let retrieved = cache.get("test", "model").unwrap().unwrap();
            assert_eq!(retrieved, embedding);
        }
    }
}
