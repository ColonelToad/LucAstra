//! Vector search module using HNSW (Hierarchical Navigable Small World) algorithm.
//!
//! This module provides semantic search capabilities using vector embeddings,
//! replacing the simple TF-IDF keyword search with neural network-based similarity.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VectorError {
    #[error("index error: {0}")]
    IndexError(String),
    #[error("embedding dimension mismatch: expected {expected}, got {got}")]
    DimensionMismatch { expected: usize, got: usize },
    #[error("empty embeddings")]
    EmptyEmbeddings,
}

pub type VectorResult<T> = std::result::Result<T, VectorError>;

/// A document with its vector embedding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorDocument {
    pub id: usize,
    pub path: PathBuf,
    pub embedding: Vec<f32>,
    pub snippet: String,
}

/// Search result with similarity score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorSearchResult {
    pub path: PathBuf,
    pub score: f32,
    pub snippet: String,
}

/// Simple vector index using cosine similarity (naive implementation).
///
/// TODO: Replace with HNSW for better performance on large corpora.
/// Current implementation is O(n) for search, HNSW would be O(log n).
pub struct VectorIndex {
    documents: Vec<VectorDocument>,
    dimensions: Option<usize>,
    next_id: usize,
}

impl VectorIndex {
    pub fn new() -> Self {
        Self {
            documents: Vec::new(),
            dimensions: None,
            next_id: 0,
        }
    }

    /// Add a document with its embedding to the index.
    pub fn add_document(
        &mut self,
        path: PathBuf,
        embedding: Vec<f32>,
        snippet: String,
    ) -> VectorResult<usize> {
        if embedding.is_empty() {
            return Err(VectorError::EmptyEmbeddings);
        }

        // Validate dimension consistency
        if let Some(dims) = self.dimensions {
            if embedding.len() != dims {
                return Err(VectorError::DimensionMismatch {
                    expected: dims,
                    got: embedding.len(),
                });
            }
        } else {
            self.dimensions = Some(embedding.len());
        }

        let id = self.next_id;
        self.next_id += 1;

        self.documents.push(VectorDocument {
            id,
            path,
            embedding,
            snippet,
        });

        Ok(id)
    }

    /// Search for similar documents using cosine similarity.
    pub fn search(
        &self,
        query_embedding: &[f32],
        k: usize,
    ) -> VectorResult<Vec<VectorSearchResult>> {
        if query_embedding.is_empty() {
            return Err(VectorError::EmptyEmbeddings);
        }

        if let Some(dims) = self.dimensions {
            if query_embedding.len() != dims {
                return Err(VectorError::DimensionMismatch {
                    expected: dims,
                    got: query_embedding.len(),
                });
            }
        }

        let mut scored_docs: Vec<(f32, &VectorDocument)> = self
            .documents
            .iter()
            .map(|doc| {
                let similarity = cosine_similarity(&doc.embedding, query_embedding);
                (similarity, doc)
            })
            .collect();

        // Sort by similarity (descending)
        scored_docs.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        Ok(scored_docs
            .into_iter()
            .take(k)
            .map(|(score, doc)| VectorSearchResult {
                path: doc.path.clone(),
                score,
                snippet: doc.snippet.clone(),
            })
            .collect())
    }

    /// Get the number of indexed documents.
    pub fn len(&self) -> usize {
        self.documents.len()
    }

    /// Check if the index is empty.
    pub fn is_empty(&self) -> bool {
        self.documents.is_empty()
    }

    /// Get the embedding dimensions (None if no documents indexed).
    pub fn dimensions(&self) -> Option<usize> {
        self.dimensions
    }

    /// Clear all documents from the index.
    pub fn clear(&mut self) {
        self.documents.clear();
        self.dimensions = None;
        self.next_id = 0;
    }
}

impl Default for VectorIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute cosine similarity between two vectors.
/// Returns value in range [-1, 1], where 1 means identical direction.
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len(), "Vectors must have same length");

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot_product / (norm_a * norm_b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity_identical() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        assert!(cosine_similarity(&a, &b).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_opposite() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![-1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) + 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_vector_index_add_document() {
        let mut index = VectorIndex::new();
        let path = PathBuf::from("/test/doc1.txt");
        let embedding = vec![0.1, 0.2, 0.3];
        let snippet = "Test document".to_string();

        let id = index.add_document(path, embedding, snippet).unwrap();
        assert_eq!(id, 0);
        assert_eq!(index.len(), 1);
        assert_eq!(index.dimensions(), Some(3));
    }

    #[test]
    fn test_vector_index_dimension_mismatch() {
        let mut index = VectorIndex::new();
        index
            .add_document(
                PathBuf::from("/test/doc1.txt"),
                vec![0.1, 0.2, 0.3],
                "Doc 1".to_string(),
            )
            .unwrap();

        let result = index.add_document(
            PathBuf::from("/test/doc2.txt"),
            vec![0.1, 0.2], // Wrong dimension
            "Doc 2".to_string(),
        );

        assert!(matches!(result, Err(VectorError::DimensionMismatch { .. })));
    }

    #[test]
    fn test_vector_index_search() {
        let mut index = VectorIndex::new();

        index
            .add_document(
                PathBuf::from("/test/doc1.txt"),
                vec![1.0, 0.0, 0.0],
                "Document about X".to_string(),
            )
            .unwrap();

        index
            .add_document(
                PathBuf::from("/test/doc2.txt"),
                vec![0.0, 1.0, 0.0],
                "Document about Y".to_string(),
            )
            .unwrap();

        index
            .add_document(
                PathBuf::from("/test/doc3.txt"),
                vec![0.7, 0.7, 0.0],
                "Document about X and Y".to_string(),
            )
            .unwrap();

        // Query similar to doc1
        let query = vec![0.9, 0.1, 0.0];
        let results = index.search(&query, 2).unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].path, PathBuf::from("/test/doc1.txt"));
        assert!(results[0].score > 0.9);
    }

    #[test]
    fn test_vector_index_empty_embeddings() {
        let mut index = VectorIndex::new();
        let result =
            index.add_document(PathBuf::from("/test/doc.txt"), vec![], "Empty".to_string());
        assert!(matches!(result, Err(VectorError::EmptyEmbeddings)));
    }

    #[test]
    fn test_vector_index_clear() {
        let mut index = VectorIndex::new();
        index
            .add_document(
                PathBuf::from("/test/doc.txt"),
                vec![1.0, 0.0],
                "Test".to_string(),
            )
            .unwrap();

        assert_eq!(index.len(), 1);
        index.clear();
        assert_eq!(index.len(), 0);
        assert_eq!(index.dimensions(), None);
    }
}
