use crate::models::*;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;

pub struct EmbeddingService {
    vocabulary: Arc<HashMap<String, usize>>,
    idf_scores: Arc<HashMap<String, f32>>,
}

impl EmbeddingService {
    pub async fn new() -> Result<Self> {
        log::info!("Initializing embedding service...");
        
        Ok(Self {
            vocabulary: Arc::new(HashMap::new()),
            idf_scores: Arc::new(HashMap::new()),
        })
    }

    pub async fn generate_embeddings(&self, documents: &mut Vec<Document>) -> Result<()> {
        log::info!("Generating embeddings for all document chunks...");
        
        // Build vocabulary from all chunks
        let mut word_counts: HashMap<String, usize> = HashMap::new();
        let mut doc_frequencies: HashMap<String, usize> = HashMap::new();
        let total_docs = documents.iter().map(|d| d.chunks.len()).sum::<usize>();
        
        // First pass: build vocabulary and document frequencies
        for document in documents.iter() {
            for chunk in &document.chunks {
                let words = self.tokenize(&chunk.content);
                let unique_words: std::collections::HashSet<_> = words.iter().collect();
                
                for word in &words {
                    *word_counts.entry(word.clone()).or_insert(0) += 1;
                }
                
                for word in unique_words {
                    *doc_frequencies.entry(word.clone()).or_insert(0) += 1;
                }
            }
        }
        
        // Calculate IDF scores
        let idf_scores: HashMap<String, f32> = doc_frequencies
            .iter()
            .map(|(word, df)| {
                let idf = (total_docs as f32 / *df as f32).ln();
                (word.clone(), idf)
            })
            .collect();
        
        // Build vocabulary (top 1000 words)
        let mut word_freq_pairs: Vec<_> = word_counts.iter().collect();
        word_freq_pairs.sort_by(|a, b| b.1.cmp(a.1));
        let vocabulary: HashMap<String, usize> = word_freq_pairs
            .into_iter()
            .take(1000)
            .enumerate()
            .map(|(idx, (word, _))| (word.clone(), idx))
            .collect();
        
        // Update self with vocabulary and IDF scores
        let vocabulary_arc = Arc::new(vocabulary);
        let idf_scores_arc = Arc::new(idf_scores);
        
        // Second pass: generate embeddings for each chunk
        for document in documents.iter_mut() {
            for chunk in document.chunks.iter_mut() {
                chunk.embedding = Some(self.create_tfidf_embedding(
                    &chunk.content,
                    &vocabulary_arc,
                    &idf_scores_arc,
                ));
            }
            log::info!("Generated embeddings for document: {}", document.filename);
        }
        
        Ok(())
    }

    pub async fn embed_query(&self, query: &str) -> Result<Vec<f32>> {
        // Use the same vocabulary for query embedding
        let embedding = self.create_tfidf_embedding(query, &self.vocabulary, &self.idf_scores);
        Ok(embedding)
    }

    fn create_tfidf_embedding(
        &self,
        text: &str,
        vocabulary: &HashMap<String, usize>,
        idf_scores: &HashMap<String, f32>,
    ) -> Vec<f32> {
        let mut embedding = vec![0.0; vocabulary.len().max(100)]; // Minimum 100 dimensions
        let words = self.tokenize(text);
        let word_counts = self.count_words(&words);
        let total_words = words.len() as f32;
        
        for (word, count) in word_counts {
            if let Some(&idx) = vocabulary.get(&word) {
                if idx < embedding.len() {
                    let tf = count as f32 / total_words;
                    let idf = idf_scores.get(&word).unwrap_or(&1.0);
                    embedding[idx] = tf * idf;
                }
            }
        }
        
        // Normalize the embedding
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for value in embedding.iter_mut() {
                *value /= norm;
            }
        }
        
        embedding
    }

    fn tokenize(&self, text: &str) -> Vec<String> {
        text.to_lowercase()
            .split_whitespace()
            .map(|word| {
                word.chars()
                    .filter(|c| c.is_alphanumeric())
                    .collect::<String>()
            })
            .filter(|word| word.len() > 2)
            .collect()
    }

    fn count_words(&self, words: &[String]) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for word in words {
            *counts.entry(word.clone()).or_insert(0) += 1;
        }
        counts
    }

    pub fn calculate_similarity(&self, embedding1: &[f32], embedding2: &[f32]) -> f32 {
        let min_len = embedding1.len().min(embedding2.len());
        
        let dot_product: f32 = embedding1[..min_len]
            .iter()
            .zip(embedding2[..min_len].iter())
            .map(|(a, b)| a * b)
            .sum();
        
        let norm1: f32 = embedding1[..min_len].iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm2: f32 = embedding2[..min_len].iter().map(|x| x * x).sum::<f32>().sqrt();
        
        if norm1 == 0.0 || norm2 == 0.0 {
            0.0
        } else {
            dot_product / (norm1 * norm2)
        }
    }
}
