// This file now serves as a library for the RAG functionality
// The actual server is now in the ../api folder

pub mod models;
pub mod document_processor;
pub mod embedding_service;
pub mod gemini_service;
pub mod query_service;

use anyhow::Result;
use document_processor::DocumentProcessor;
use embedding_service::EmbeddingService;
use gemini_service::GeminiService;
use models::*;
use query_service::QueryService;
use std::sync::Arc;

pub struct RagLibrary {
    pub query_service: Arc<QueryService>,
}

impl RagLibrary {
    pub async fn new() -> Result<(Vec<Document>, Self)> {
        // Load environment variables
        dotenv::dotenv().ok();
        env_logger::init();

        log::info!("Initializing RAG Library...");

        // Initialize services
        let embedding_service = Arc::new(EmbeddingService::new().await?);
        let gemini_service = Arc::new(GeminiService::new()?);
        let query_service = Arc::new(QueryService::new(
            embedding_service.clone(),
            gemini_service,
        ));

        // Process documents
        let document_processor = DocumentProcessor::new();
        let mut documents = document_processor.process_documents(".").await?;

        // Generate embeddings
        embedding_service.generate_embeddings(&mut documents).await?;

        log::info!("RAG Library initialized successfully!");

        let library = RagLibrary { query_service };

        Ok((documents, library))
    }
}

// This main function is now primarily for testing the library
#[tokio::main]
async fn main() -> Result<()> {
    println!("RAG Library - Use this as a library in the main API server");
    println!("Run the server from ../api instead");
    
    Ok(())
}
