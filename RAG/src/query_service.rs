use crate::models::*;
use crate::embedding_service::EmbeddingService;
use crate::gemini_service::GeminiService;
use anyhow::Result;
use std::sync::Arc;

pub struct QueryService {
    embedding_service: Arc<EmbeddingService>,
    gemini_service: Arc<GeminiService>,
}

impl QueryService {
    pub fn new(embedding_service: Arc<EmbeddingService>, gemini_service: Arc<GeminiService>) -> Self {
        Self {
            embedding_service,
            gemini_service,
        }
    }

    pub async fn query(&self, query: &str, documents: &[Document], max_results: usize) -> Result<QueryResponse> {
        let start_time = std::time::Instant::now();

        // Generate query embedding
        let query_embedding = self.embedding_service.embed_query(query).await?;

        // Find relevant chunks
        let relevant_chunks = self.find_relevant_chunks(&query_embedding, documents, max_results)?;

        // Generate response using Gemini
        let response = self.gemini_service
            .generate_response(query, &relevant_chunks, documents)
            .await?;

        // Create citations
        let citations = self.create_citations(&relevant_chunks, documents);

        let processing_time = start_time.elapsed().as_millis();

        Ok(QueryResponse {
            status: "success".to_string(),
            response,
            citations,
            processing_time_ms: processing_time,
        })
    }

    fn find_relevant_chunks(
        &self,
        query_embedding: &[f32],
        documents: &[Document],
        max_results: usize,
    ) -> Result<Vec<DocumentChunk>> {
        let mut chunk_scores: Vec<(DocumentChunk, f32)> = Vec::new();

        for document in documents {
            for chunk in &document.chunks {
                if let Some(chunk_embedding) = &chunk.embedding {
                    let similarity = self.embedding_service
                        .calculate_similarity(query_embedding, chunk_embedding);
                    chunk_scores.push((chunk.clone(), similarity));
                }
            }
        }

        // Sort by similarity score (highest first)
        chunk_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Take top results
        let relevant_chunks: Vec<DocumentChunk> = chunk_scores
            .into_iter()
            .take(max_results)
            .map(|(chunk, _)| chunk)
            .collect();

        log::info!("Found {} relevant chunks", relevant_chunks.len());
        Ok(relevant_chunks)
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
                });
            }
        }

        citations
    }
}
