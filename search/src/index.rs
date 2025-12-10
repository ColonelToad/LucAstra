//! BM25 inverted index implementation.

use crate::tokenizer::Tokenizer;
use lucastra_core::Result;
use std::collections::{HashMap, HashSet};
use tracing::debug;

/// BM25 parameters.
const K1: f32 = 1.5;
const B: f32 = 0.75;

/// Inverted index for BM25 scoring.
pub struct BM25Index {
    /// Document ID → content tokens
    documents: HashMap<String, Vec<String>>,
    /// Term → set of document IDs
    term_docs: HashMap<String, HashSet<String>>,
    /// Term → document frequencies
    term_freqs: HashMap<String, HashMap<String, usize>>,
    /// Average document length
    avg_doc_len: f32,
}

impl BM25Index {
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
            term_docs: HashMap::new(),
            term_freqs: HashMap::new(),
            avg_doc_len: 0.0,
        }
    }

    /// Add a document to the index.
    pub fn add_document(&mut self, doc_id: &str, content: &str) -> Result<()> {
        let tokens = Tokenizer::tokenize(content);
        let tokens = Tokenizer::remove_stopwords(tokens);

        debug!("Adding document {} with {} tokens", doc_id, tokens.len());

        // Store document
        self.documents.insert(doc_id.to_string(), tokens.clone());

        // Update term statistics
        let mut term_count = HashMap::new();
        for token in &tokens {
            *term_count.entry(token.clone()).or_insert(0) += 1;

            self.term_docs
                .entry(token.clone())
                .or_insert_with(HashSet::new)
                .insert(doc_id.to_string());
        }

        for (term, count) in term_count {
            self.term_freqs
                .entry(term)
                .or_insert_with(HashMap::new)
                .insert(doc_id.to_string(), count);
        }

        // Recalculate average document length
        let total_len: usize = self.documents.values().map(|d| d.len()).sum();
        self.avg_doc_len = total_len as f32 / self.documents.len() as f32;

        Ok(())
    }

    /// Search for documents matching a query.
    pub fn search(&self, query: &str, top_k: usize) -> Result<Vec<(String, f32)>> {
        let tokens = Tokenizer::tokenize(query);
        let tokens = Tokenizer::remove_stopwords(tokens);

        if tokens.is_empty() {
            return Ok(Vec::new());
        }

        let mut scores: HashMap<String, f32> = HashMap::new();

        for token in tokens {
            if let Some(docs) = self.term_docs.get(&token) {
                let idf = self.idf(docs.len());

                for doc_id in docs {
                    let term_freq = self
                        .term_freqs
                        .get(&token)
                        .and_then(|m| m.get(doc_id))
                        .copied()
                        .unwrap_or(0) as f32;

                    let doc_len = self
                        .documents
                        .get(doc_id)
                        .map(|d| d.len() as f32)
                        .unwrap_or(0.0);

                    let bm25_score = self.bm25_score(
                        term_freq,
                        idf,
                        doc_len,
                        self.avg_doc_len,
                    );

                    *scores.entry(doc_id.clone()).or_insert(0.0) += bm25_score;
                }
            }
        }

        let mut ranked: Vec<_> = scores.into_iter().collect();
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        Ok(ranked.into_iter().take(top_k).collect())
    }

    /// Calculate IDF (inverse document frequency).
    fn idf(&self, doc_count: usize) -> f32 {
        let n = self.documents.len() as f32;
        ((n - doc_count as f32 + 0.5) / (doc_count as f32 + 0.5) + 1.0).ln()
    }

    /// Calculate BM25 score.
    fn bm25_score(&self, term_freq: f32, idf: f32, doc_len: f32, avg_doc_len: f32) -> f32 {
        let numerator = term_freq * (K1 + 1.0);
        let denominator = term_freq + K1 * (1.0 - B + B * (doc_len / avg_doc_len));
        idf * (numerator / denominator)
    }

    /// Clear the index.
    pub fn clear(&mut self) {
        self.documents.clear();
        self.term_docs.clear();
        self.term_freqs.clear();
        self.avg_doc_len = 0.0;
    }
}

impl Default for BM25Index {
    fn default() -> Self {
        Self::new()
    }
}
