//! BM25-based full-text search for filesystem indexing.

pub mod index;
pub mod tokenizer;

pub use index::BM25Index;
pub use tokenizer::Tokenizer;

use lucastra_core::{command::SearchResult, Result};
use std::collections::HashMap;
use tracing::info;

/// Search service providing BM25-ranked document retrieval.
pub struct SearchService {
    index: BM25Index,
    documents: HashMap<String, String>, // path -> content
}

impl SearchService {
    pub fn new() -> Self {
        Self {
            index: BM25Index::new(),
            documents: HashMap::new(),
        }
    }

    /// Index a document (file) by path.
    pub fn index_document(&mut self, path: &str, content: &str) -> Result<()> {
        info!("Indexing document: {}", path);
        self.index.add_document(path, content)?;
        self.documents.insert(path.to_string(), content.to_string());
        Ok(())
    }

    /// Search for documents by query string.
    pub fn search(&self, query: &str, top_k: usize) -> Result<Vec<SearchResult>> {
        info!("Searching for: {}", query);
        let results = self.index.search(query, top_k)?;
        Ok(results
            .into_iter()
            .map(|(path, score)| {
                let snippet = self
                    .documents
                    .get(&path)
                    .map(|c| c.chars().take(200).collect::<String>())
                    .unwrap_or_else(|| "...".to_string());
                SearchResult { path, score, snippet }
            })
            .collect())
    }

    /// Clear all indexed documents.
    pub fn clear(&mut self) {
        self.index.clear();
        self.documents.clear();
    }

    /// Get document count.
    pub fn doc_count(&self) -> usize {
        self.documents.len()
    }
}

impl Default for SearchService {
    fn default() -> Self {
        Self::new()
    }
}
