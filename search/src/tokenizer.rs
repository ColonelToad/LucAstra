//! Tokenizer for BM25 indexing.

/// Simple tokenizer that splits on whitespace and punctuation.
pub struct Tokenizer;

impl Tokenizer {
    pub fn tokenize(text: &str) -> Vec<String> {
        text.to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty() && s.len() > 2) // Filter stopwords by length
            .map(|s| s.to_string())
            .collect()
    }

    /// Check if a word is a common stopword.
    pub fn is_stopword(word: &str) -> bool {
        matches!(
            word,
            "the" | "a" | "an" | "and" | "or" | "is" | "in" | "at" | "to" | "for" | "of" | "on"
                | "with" | "by" | "from"
        )
    }

    /// Remove stopwords from token list.
    pub fn remove_stopwords(tokens: Vec<String>) -> Vec<String> {
        tokens
            .into_iter()
            .filter(|t| !Self::is_stopword(t))
            .collect()
    }
}
