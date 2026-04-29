//! TF-IDF vectorization utilities for text similarity calculations
//!
//! This module provides TF-IDF (Term Frequency-Inverse Document Frequency) vectorization
//! for converting text documents into numerical feature vectors. Used by the Deduce engine
//! for label deduction based on text similarity.
//!
//! Features:
//! - Tokenization with lowercase normalization
//! - Unigram and bigram generation
//! - English stop word removal
//! - Vocabulary limiting (top 1000 terms)
//! - TF-IDF scoring
//! - Cosine similarity calculation

use std::collections::{HashMap, HashSet};

/// TF-IDF vectorizer for converting text documents to feature vectors
///
/// The vectorizer supports:
/// - Unigrams and bigrams (ngram_range: 1-2)
/// - English stop word removal
/// - Vocabulary size limit (max_features: 1000)
/// - fit_transform() for training and transforming
/// - transform() for transforming new documents
pub struct TfidfVectorizer {
    /// Maximum number of features (vocabulary size)
    max_features: usize,
    /// N-gram range (min, max) - default (1, 2) for unigrams and bigrams
    ngram_range: (usize, usize),
    /// Set of English stop words to remove
    stop_words: HashSet<String>,
    /// Vocabulary mapping term -> index
    vocabulary: HashMap<String, usize>,
    /// IDF scores for each term in vocabulary
    idf_scores: Vec<f64>,
}

impl TfidfVectorizer {
    /// Create a new TF-IDF vectorizer with default settings
    ///
    /// Default configuration:
    /// - max_features: 1000
    /// - ngram_range: (1, 2) - unigrams and bigrams
    /// - stop_words: English stop words
    pub fn new() -> Self {
        Self {
            max_features: 1000,
            ngram_range: (1, 2),
            stop_words: Self::load_english_stop_words(),
            vocabulary: HashMap::new(),
            idf_scores: Vec::new(),
        }
    }

    /// Fit the vectorizer on documents and transform them to TF-IDF vectors
    ///
    /// This method:
    /// 1. Tokenizes all documents
    /// 2. Generates n-grams
    /// 3. Builds vocabulary (limited to max_features)
    /// 4. Computes IDF scores
    /// 5. Transforms documents to TF-IDF vectors
    ///
    /// # Arguments
    /// * `documents` - Slice of text documents
    ///
    /// # Returns
    /// Vector of TF-IDF feature vectors (one per document)
    pub fn fit_transform(&mut self, documents: &[String]) -> Vec<Vec<f64>> {
        // Step 1: Tokenize all documents
        let tokenized_docs: Vec<Vec<String>> = documents
            .iter()
            .map(|doc| {
                let tokens = self.tokenize(doc);
                self.generate_ngrams(&tokens)
            })
            .collect();

        // Step 2: Build vocabulary from all terms
        let mut term_doc_freq: HashMap<String, usize> = HashMap::new();
        for doc_tokens in &tokenized_docs {
            let unique_terms: HashSet<_> = doc_tokens.iter().collect();
            for term in unique_terms {
                *term_doc_freq.entry(term.clone()).or_insert(0) += 1;
            }
        }

        // Step 3: Select top max_features terms by document frequency
        let mut term_freq_vec: Vec<_> = term_doc_freq.into_iter().collect();
        term_freq_vec.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by frequency descending
        
        let selected_terms: Vec<String> = term_freq_vec
            .into_iter()
            .take(self.max_features)
            .map(|(term, _)| term)
            .collect();

        // Step 4: Build vocabulary mapping
        self.vocabulary = selected_terms
            .iter()
            .enumerate()
            .map(|(idx, term)| (term.clone(), idx))
            .collect();

        // Step 5: Compute IDF scores
        let num_docs = documents.len() as f64;
        self.idf_scores = vec![0.0; self.vocabulary.len()];
        
        for doc_tokens in &tokenized_docs {
            let unique_terms: HashSet<_> = doc_tokens.iter().collect();
            for term in unique_terms {
                if let Some(&idx) = self.vocabulary.get(term) {
                    self.idf_scores[idx] += 1.0;
                }
            }
        }

        // IDF = log(N / df) where N is total docs, df is document frequency
        for score in &mut self.idf_scores {
            if *score > 0.0 {
                *score = (num_docs / *score).ln();
            }
        }

        // Step 6: Transform documents to TF-IDF vectors
        self.transform(documents)
    }

    /// Transform documents to TF-IDF vectors using fitted vocabulary
    ///
    /// Must call fit_transform() first to build vocabulary and IDF scores.
    ///
    /// # Arguments
    /// * `documents` - Slice of text documents
    ///
    /// # Returns
    /// Vector of TF-IDF feature vectors (one per document)
    pub fn transform(&self, documents: &[String]) -> Vec<Vec<f64>> {
        documents
            .iter()
            .map(|doc| {
                let tokens = self.tokenize(doc);
                let ngrams = self.generate_ngrams(&tokens);
                let tf = self.compute_tf(&ngrams);
                
                // Create TF-IDF vector
                let mut tfidf_vec = vec![0.0; self.vocabulary.len()];
                for (term, tf_score) in tf {
                    if let Some(&idx) = self.vocabulary.get(&term) {
                        tfidf_vec[idx] = tf_score * self.idf_scores[idx];
                    }
                }
                
                tfidf_vec
            })
            .collect()
    }

    /// Tokenize text into words
    ///
    /// Process:
    /// 1. Convert to lowercase
    /// 2. Split on whitespace and punctuation
    /// 3. Remove stop words
    /// 4. Filter empty tokens
    ///
    /// # Arguments
    /// * `text` - Input text string
    ///
    /// # Returns
    /// Vector of tokens (words)
    fn tokenize(&self, text: &str) -> Vec<String> {
        text.to_lowercase()
            .split(|c: char| c.is_whitespace() || c.is_ascii_punctuation())
            .filter(|s| !s.is_empty())
            .filter(|s| !self.stop_words.contains(*s))
            .map(|s| s.to_string())
            .collect()
    }

    /// Generate n-grams from tokens
    ///
    /// Creates both unigrams and bigrams based on ngram_range.
    ///
    /// # Arguments
    /// * `tokens` - Slice of tokens
    ///
    /// # Returns
    /// Vector of n-grams (as strings)
    fn generate_ngrams(&self, tokens: &[String]) -> Vec<String> {
        let mut ngrams = Vec::new();

        // Generate n-grams for each size in range
        for n in self.ngram_range.0..=self.ngram_range.1 {
            if tokens.len() >= n {
                for i in 0..=(tokens.len() - n) {
                    let ngram = tokens[i..i + n].join(" ");
                    ngrams.push(ngram);
                }
            }
        }

        ngrams
    }

    /// Compute term frequency (TF) for tokens
    ///
    /// TF = count(term) / total_terms
    ///
    /// # Arguments
    /// * `tokens` - Slice of tokens
    ///
    /// # Returns
    /// HashMap mapping term -> TF score
    fn compute_tf(&self, tokens: &[String]) -> HashMap<String, f64> {
        let total_terms = tokens.len() as f64;
        let mut tf_map: HashMap<String, f64> = HashMap::new();

        for token in tokens {
            *tf_map.entry(token.clone()).or_insert(0.0) += 1.0;
        }

        // Normalize by total terms
        for value in tf_map.values_mut() {
            *value /= total_terms;
        }

        tf_map
    }

    /// Load English stop words
    ///
    /// Returns a set of common English stop words that should be filtered out
    /// during tokenization.
    ///
    /// # Returns
    /// HashSet of stop words
    fn load_english_stop_words() -> HashSet<String> {
        // Common English stop words
        let stop_words = vec![
            "a", "an", "and", "are", "as", "at", "be", "by", "for", "from",
            "has", "he", "in", "is", "it", "its", "of", "on", "that", "the",
            "to", "was", "will", "with", "the", "this", "but", "they", "have",
            "had", "what", "when", "where", "who", "which", "why", "how",
            "all", "each", "every", "both", "few", "more", "most", "other",
            "some", "such", "no", "nor", "not", "only", "own", "same", "so",
            "than", "too", "very", "can", "just", "should", "now",
        ];

        stop_words.iter().map(|&s| s.to_string()).collect()
    }
}

impl Default for TfidfVectorizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate cosine similarity between two vectors
///
/// Cosine similarity = dot(a, b) / (||a|| * ||b||)
///
/// Returns value in range [-1, 1]:
/// - 1.0: Identical direction
/// - 0.0: Orthogonal (no similarity)
/// - -1.0: Opposite direction
///
/// # Arguments
/// * `vec1` - First vector
/// * `vec2` - Second vector
///
/// # Returns
/// Cosine similarity score
pub fn cosine_similarity(vec1: &[f64], vec2: &[f64]) -> f64 {
    let dot_product: f64 = vec1.iter().zip(vec2.iter()).map(|(a, b)| a * b).sum();
    let norm1: f64 = vec1.iter().map(|x| x.powi(2)).sum::<f64>().sqrt();
    let norm2: f64 = vec2.iter().map(|x| x.powi(2)).sum::<f64>().sqrt();

    if norm1 == 0.0 || norm2 == 0.0 {
        return 0.0; // No similarity if either vector is zero
    }

    dot_product / (norm1 * norm2)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // Unit Tests
    // ============================================================================

    #[test]
    fn test_tokenize_basic() {
        let vectorizer = TfidfVectorizer::new();
        let text = "Hello world! This is a test.";
        let tokens = vectorizer.tokenize(text);
        
        // Should be lowercase, no punctuation, no stop words
        assert!(tokens.contains(&"hello".to_string()));
        assert!(tokens.contains(&"world".to_string()));
        assert!(tokens.contains(&"test".to_string()));
        
        // Stop words should be removed
        assert!(!tokens.contains(&"this".to_string()));
        assert!(!tokens.contains(&"is".to_string()));
        assert!(!tokens.contains(&"a".to_string()));
    }

    #[test]
    fn test_tokenize_removes_stop_words() {
        let vectorizer = TfidfVectorizer::new();
        let text = "the quick brown fox";
        let tokens = vectorizer.tokenize(text);
        
        assert!(tokens.contains(&"quick".to_string()));
        assert!(tokens.contains(&"brown".to_string()));
        assert!(tokens.contains(&"fox".to_string()));
        assert!(!tokens.contains(&"the".to_string())); // Stop word removed
    }

    #[test]
    fn test_generate_ngrams_unigrams() {
        let vectorizer = TfidfVectorizer::new();
        let tokens = vec!["hello".to_string(), "world".to_string()];
        let ngrams = vectorizer.generate_ngrams(&tokens);
        
        // Should contain unigrams
        assert!(ngrams.contains(&"hello".to_string()));
        assert!(ngrams.contains(&"world".to_string()));
    }

    #[test]
    fn test_generate_ngrams_bigrams() {
        let vectorizer = TfidfVectorizer::new();
        let tokens = vec!["hello".to_string(), "world".to_string(), "test".to_string()];
        let ngrams = vectorizer.generate_ngrams(&tokens);
        
        // Should contain bigrams
        assert!(ngrams.contains(&"hello world".to_string()));
        assert!(ngrams.contains(&"world test".to_string()));
    }

    #[test]
    fn test_compute_tf() {
        let vectorizer = TfidfVectorizer::new();
        let tokens = vec![
            "hello".to_string(),
            "world".to_string(),
            "hello".to_string(),
        ];
        let tf = vectorizer.compute_tf(&tokens);
        
        // "hello" appears 2/3 times
        assert!((tf.get("hello").unwrap() - 2.0/3.0).abs() < 1e-10);
        // "world" appears 1/3 times
        assert!((tf.get("world").unwrap() - 1.0/3.0).abs() < 1e-10);
    }

    #[test]
    fn test_fit_transform_basic() {
        let mut vectorizer = TfidfVectorizer::new();
        let documents = vec![
            "hello world".to_string(),
            "hello test".to_string(),
            "world test".to_string(),
        ];
        
        let vectors = vectorizer.fit_transform(&documents);
        
        // Should have 3 vectors (one per document)
        assert_eq!(vectors.len(), 3);
        
        // Each vector should have same length (vocabulary size)
        let vocab_size = vectors[0].len();
        assert!(vocab_size > 0);
        assert_eq!(vectors[1].len(), vocab_size);
        assert_eq!(vectors[2].len(), vocab_size);
    }

    #[test]
    fn test_fit_transform_vocabulary_limit() {
        let mut vectorizer = TfidfVectorizer::new();
        
        // Create documents with more than 1000 unique terms
        let mut documents = Vec::new();
        for i in 0..1500 {
            documents.push(format!("term{}", i));
        }
        
        let vectors = vectorizer.fit_transform(&documents);
        
        // Vocabulary should be limited to max_features (1000)
        assert_eq!(vectors[0].len(), 1000);
    }

    #[test]
    fn test_transform_after_fit() {
        let mut vectorizer = TfidfVectorizer::new();
        let train_docs = vec![
            "hello world".to_string(),
            "hello test".to_string(),
        ];
        
        vectorizer.fit_transform(&train_docs);
        
        // Transform new document
        let new_docs = vec!["hello world test".to_string()];
        let vectors = vectorizer.transform(&new_docs);
        
        assert_eq!(vectors.len(), 1);
        assert!(vectors[0].len() > 0);
    }

    #[test]
    fn test_cosine_similarity_identical() {
        let vec1 = vec![1.0, 2.0, 3.0];
        let vec2 = vec![1.0, 2.0, 3.0];
        
        let similarity = cosine_similarity(&vec1, &vec2);
        assert!((similarity - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let vec1 = vec![1.0, 0.0];
        let vec2 = vec![0.0, 1.0];
        
        let similarity = cosine_similarity(&vec1, &vec2);
        assert!((similarity - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_cosine_similarity_opposite() {
        let vec1 = vec![1.0, 2.0];
        let vec2 = vec![-1.0, -2.0];
        
        let similarity = cosine_similarity(&vec1, &vec2);
        assert!((similarity - (-1.0)).abs() < 1e-10);
    }

    #[test]
    fn test_cosine_similarity_zero_vector() {
        let vec1 = vec![0.0, 0.0];
        let vec2 = vec![1.0, 2.0];
        
        let similarity = cosine_similarity(&vec1, &vec2);
        assert!((similarity - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_cosine_similarity_scaled_vectors() {
        let vec1 = vec![1.0, 2.0, 3.0];
        let vec2 = vec![2.0, 4.0, 6.0];
        
        // Scaled vectors have same direction, similarity = 1.0
        let similarity = cosine_similarity(&vec1, &vec2);
        assert!((similarity - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_tfidf_empty_document() {
        let mut vectorizer = TfidfVectorizer::new();
        let documents = vec!["".to_string()];
        
        let vectors = vectorizer.fit_transform(&documents);
        assert_eq!(vectors.len(), 1);
        // Empty document should produce zero vector
        assert_eq!(vectors[0].len(), 0);
    }

    #[test]
    fn test_tfidf_single_word_documents() {
        let mut vectorizer = TfidfVectorizer::new();
        let documents = vec![
            "apple".to_string(),
            "banana".to_string(),
            "apple".to_string(),
        ];
        
        let vectors = vectorizer.fit_transform(&documents);
        assert_eq!(vectors.len(), 3);
        
        // First and third documents should be similar (both "apple")
        let sim = cosine_similarity(&vectors[0], &vectors[2]);
        assert!((sim - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_stop_words_loaded() {
        let vectorizer = TfidfVectorizer::new();
        
        // Check that common stop words are present
        assert!(vectorizer.stop_words.contains("the"));
        assert!(vectorizer.stop_words.contains("a"));
        assert!(vectorizer.stop_words.contains("an"));
        assert!(vectorizer.stop_words.contains("and"));
        assert!(vectorizer.stop_words.contains("is"));
    }

    #[test]
    fn test_ngram_range() {
        let vectorizer = TfidfVectorizer::new();
        assert_eq!(vectorizer.ngram_range, (1, 2));
    }

    #[test]
    fn test_max_features() {
        let vectorizer = TfidfVectorizer::new();
        assert_eq!(vectorizer.max_features, 1000);
    }
}
