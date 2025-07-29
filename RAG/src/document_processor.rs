use crate::models::*;
use anyhow::Result;
use pdf_extract::extract_text;
use regex::Regex;
use std::fs;
use std::path::Path;
use uuid::Uuid;

pub struct DocumentProcessor;

impl DocumentProcessor {
    pub fn new() -> Self {
        Self
    }

    pub async fn process_documents(&self, documents_dir: &str) -> Result<Vec<Document>> {
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
        let chunk_size = 500; // characters
        let overlap = 50; // characters overlap between chunks
        let mut chunks = Vec::new();
        
        // Clean and normalize text
        let cleaned_content = self.clean_text(content);
        let sentences = self.split_into_sentences(&cleaned_content);
        
        let mut current_chunk = String::new();
        let mut start_pos = 0;
        
        for sentence in sentences {
            if current_chunk.chars().count() + sentence.chars().count() > chunk_size && !current_chunk.is_empty() {
                // Create chunk
                let chunk = DocumentChunk {
                    id: Uuid::new_v4().to_string(),
                    content: current_chunk.trim().to_string(),
                    start_position: start_pos,
                    end_position: start_pos + current_chunk.chars().count(),
                    embedding: None,
                };
                chunks.push(chunk);
                
                // Start new chunk with overlap
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
        
        // Add the last chunk if it's not empty
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
        
        log::info!("Created {} chunks", chunks.len());
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
}
