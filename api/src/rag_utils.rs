use anyhow::Result;
use pdf_extract::extract_text;
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub filename: String,
    pub content: String,
    pub chunks: Vec<DocumentChunk>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentChunk {
    pub id: String,
    pub content: String,
    pub start_position: usize,
    pub end_position: usize,
    pub embedding: Option<Vec<f32>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryResponse {
    pub status: String,
    pub response: String,
    pub citations: Vec<Citation>,
    pub processing_time_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    pub document: String,
    pub text_excerpt: String,
    pub confidence_score: f32,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    generation_config: Option<GeminiGenerationConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiPart {
    text: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiGenerationConfig {
    temperature: f32,
    max_output_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiCandidate {
    content: GeminiContent,
}

#[derive(Debug)]
pub struct RagSystem {
    documents: Arc<RwLock<Vec<Document>>>,
    client: Client,
    api_key: String,
    vocabulary: HashMap<String, usize>,
    idf_scores: HashMap<String, f32>,
}

impl RagSystem {
    pub async fn new(documents_dir: &str) -> Result<Self> {
        log::info!("Initializing RAG System...");

        let api_key = env::var("GEMINI_API_KEY")
            .map_err(|_| anyhow::anyhow!("GEMINI_API_KEY environment variable not set"))?;

        let mut rag_system = Self {
            documents: Arc::new(RwLock::new(Vec::new())),
            client: Client::new(),
            api_key,
            vocabulary: HashMap::new(),
            idf_scores: HashMap::new(),
        };

        // Process documents
        let mut documents = rag_system.process_documents(documents_dir).await?;
        
        // Generate embeddings
        rag_system.generate_embeddings(&mut documents).await?;
        
        *rag_system.documents.write().await = documents;

        log::info!("RAG System initialized successfully!");
        Ok(rag_system)
    }

    pub async fn query(&self, query: &str, max_results: Option<usize>) -> Result<QueryResponse> {
        let start_time = std::time::Instant::now();
        let max_results = max_results.unwrap_or(5);

        // Generate query embedding
        let query_embedding = self.embed_query(query);

        // Find relevant chunks
        let documents = self.documents.read().await;
        let relevant_chunks = self.find_relevant_chunks(&query_embedding, &documents, max_results);

        // Generate response using Gemini
        let response = self.generate_response(query, &relevant_chunks, &documents).await?;

        // Create citations
        let citations = self.create_citations(&relevant_chunks, &documents);

        let processing_time = start_time.elapsed().as_millis();

        Ok(QueryResponse {
            status: "success".to_string(),
            response,
            citations,
            processing_time_ms: processing_time,
        })
    }

    async fn process_documents(&self, documents_dir: &str) -> Result<Vec<Document>> {
        let mut documents = Vec::new();
        let paths = fs::read_dir(documents_dir)?;

        for path in paths {
            let path = path?;
            let file_path = path.path();
            
            if let Some(extension) = file_path.extension() {
                if extension == "pdf" {
                    let doc = self.process_pdf(&file_path).await?;
                    documents.push(doc);
                }
            }
        }

        log::info!("Processed {} documents", documents.len());
        Ok(documents)
    }

    async fn process_pdf(&self, file_path: &Path) -> Result<Document> {
        let filename = file_path.file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();
        
        log::info!("Processing PDF: {}", filename);
        
        let content = extract_text(file_path)?;
        let chunks = self.create_chunks(&content);
        
        Ok(Document {
            id: Uuid::new_v4().to_string(),
            filename,
            content,
            chunks,
        })
    }

    fn create_chunks(&self, content: &str) -> Vec<DocumentChunk> {
        let chunk_size = 500;
        let overlap = 50;
        let mut chunks = Vec::new();
        
        let cleaned_content = self.clean_text(content);
        let sentences = self.split_into_sentences(&cleaned_content);
        
        let mut current_chunk = String::new();
        let mut start_pos = 0;
        
        for sentence in sentences {
            if current_chunk.chars().count() + sentence.chars().count() > chunk_size && !current_chunk.is_empty() {
                let chunk = DocumentChunk {
                    id: Uuid::new_v4().to_string(),
                    content: current_chunk.trim().to_string(),
                    start_position: start_pos,
                    end_position: start_pos + current_chunk.chars().count(),
                    embedding: None,
                };
                chunks.push(chunk);
                
                let overlap_text = if current_chunk.chars().count() > overlap {
                    current_chunk.chars().skip(current_chunk.chars().count() - overlap).collect::<String>()
                } else {
                    current_chunk.clone()
                };
                
                start_pos = start_pos + current_chunk.chars().count() - overlap_text.chars().count();
                current_chunk = overlap_text + " " + &sentence;
            } else {
                if !current_chunk.is_empty() {
                    current_chunk.push(' ');
                }
                current_chunk.push_str(&sentence);
            }
        }
        
        if !current_chunk.is_empty() {
            let chunk = DocumentChunk {
                id: Uuid::new_v4().to_string(),
                content: current_chunk.trim().to_string(),
                start_position: start_pos,
                end_position: start_pos + current_chunk.chars().count(),
                embedding: None,
            };
            chunks.push(chunk);
        }
        
        chunks
    }

    fn clean_text(&self, text: &str) -> String {
        let re_whitespace = Regex::new(r"\s+").unwrap();
        let re_special = Regex::new(r"[^\w\s.,!?;:()\-\[\]{}]").unwrap();
        
        let cleaned = re_special.replace_all(text, " ");
        let cleaned = re_whitespace.replace_all(&cleaned, " ");
        
        cleaned.trim().to_string()
    }

    fn split_into_sentences(&self, text: &str) -> Vec<String> {
        let re = Regex::new(r"[.!?]+\s+").unwrap();
        re.split(text).map(|s| s.to_string()).collect()
    }

    async fn generate_embeddings(&mut self, documents: &mut Vec<Document>) -> Result<()> {
        log::info!("Generating embeddings for all document chunks...");
        
        let mut word_counts: HashMap<String, usize> = HashMap::new();
        let mut doc_frequencies: HashMap<String, usize> = HashMap::new();
        let total_docs = documents.iter().map(|d| d.chunks.len()).sum::<usize>();
        
        // Build vocabulary and document frequencies
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
        
        // Build vocabulary
        let mut word_freq_pairs: Vec<_> = word_counts.iter().collect();
        word_freq_pairs.sort_by(|a, b| b.1.cmp(a.1));
        let vocabulary: HashMap<String, usize> = word_freq_pairs
            .into_iter()
            .take(1000)
            .enumerate()
            .map(|(idx, (word, _))| (word.clone(), idx))
            .collect();
        
        self.vocabulary = vocabulary;
        self.idf_scores = idf_scores;
        
        // Generate embeddings for each chunk
        for document in documents.iter_mut() {
            for chunk in document.chunks.iter_mut() {
                chunk.embedding = Some(self.create_tfidf_embedding(&chunk.content));
            }
        }
        
        Ok(())
    }

    fn embed_query(&self, query: &str) -> Vec<f32> {
        self.create_tfidf_embedding(query)
    }

    fn create_tfidf_embedding(&self, text: &str) -> Vec<f32> {
        let mut embedding = vec![0.0; self.vocabulary.len().max(100)];
        let words = self.tokenize(text);
        let word_counts = self.count_words(&words);
        let total_words = words.len() as f32;
        
        for (word, count) in word_counts {
            if let Some(&idx) = self.vocabulary.get(&word) {
                if idx < embedding.len() {
                    let tf = count as f32 / total_words;
                    let idf = self.idf_scores.get(&word).unwrap_or(&1.0);
                    embedding[idx] = tf * idf;
                }
            }
        }
        
        // Normalize
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

    fn calculate_similarity(&self, embedding1: &[f32], embedding2: &[f32]) -> f32 {
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

    fn find_relevant_chunks(&self, query_embedding: &[f32], documents: &[Document], max_results: usize) -> Vec<DocumentChunk> {
        let mut chunk_scores: Vec<(DocumentChunk, f32)> = Vec::new();

        for document in documents {
            for chunk in &document.chunks {
                if let Some(chunk_embedding) = &chunk.embedding {
                    let similarity = self.calculate_similarity(query_embedding, chunk_embedding);
                    chunk_scores.push((chunk.clone(), similarity));
                }
            }
        }

        chunk_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        chunk_scores
            .into_iter()
            .take(max_results)
            .map(|(chunk, _)| chunk)
            .collect()
    }

    async fn generate_response(&self, query: &str, relevant_chunks: &[DocumentChunk], documents: &[Document]) -> Result<String> {
        let context = self.build_context(relevant_chunks, documents);
        let prompt = self.build_prompt(query, &context);

        let request = GeminiRequest {
            contents: vec![GeminiContent {
                parts: vec![GeminiPart {
                    text: prompt,
                }],
            }],
            generation_config: Some(GeminiGenerationConfig {
                temperature: 0.3,
                max_output_tokens: 1000,
            }),
        };

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
            self.api_key
        );

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Gemini API error: {}", error_text));
        }

        let gemini_response: GeminiResponse = response.json().await?;
        
        let answer = gemini_response
            .candidates
            .first()
            .and_then(|c| c.content.parts.first())
            .map(|p| p.text.clone())
            .unwrap_or_else(|| "No response generated".to_string());

        Ok(answer)
    }

    fn build_context(&self, chunks: &[DocumentChunk], documents: &[Document]) -> String {
        let mut context = String::new();
        
        for chunk in chunks {
            if let Some(doc) = documents.iter().find(|d| d.chunks.iter().any(|c| c.id == chunk.id)) {
                context.push_str(&format!(
                    "Document: {}\nContent: {}\n\n",
                    doc.filename,
                    chunk.content
                ));
            }
        }
        
        context
    }

    fn build_prompt(&self, query: &str, context: &str) -> String {
        format!(
            r#"You are an expert assistant that answers questions based solely on the provided context documents. 

INSTRUCTIONS:
0. If the user asks general questions, politely answer if answerable, otherwise say you can only answer based on the provided context
1. If the question is related to policies, Answer the question using the information from the provided context
2. If you can't answer policies questions, say something like "I don't have enough information to answer that question"
2. Be concise but comprehensive
3. If you quote or reference specific information, indicate which document it came from
4. If the context doesn't contain enough information to answer the question, say so clearly
5. Do not add information not present in the context
6. Focus on accuracy and relevance

CONTEXT DOCUMENTS:
{context}

QUESTION: {query}

ANSWER :"#
        )
    }

    fn create_citations(&self, chunks: &[DocumentChunk], documents: &[Document]) -> Vec<Citation> {
        let mut citations = Vec::new();

        for chunk in chunks {
            if let Some(doc) = documents.iter().find(|d| d.chunks.iter().any(|c| c.id == chunk.id)) {
                let excerpt = if chunk.content.len() > 200 {
                    format!("{}...", &chunk.content[..200])
                } else {
                    chunk.content.clone()
                };

                citations.push(Citation {
                    document: doc.filename.clone(),
                    text_excerpt: excerpt,
                    confidence_score: 0.8,
                });
            }
        }

        citations
    }
}
