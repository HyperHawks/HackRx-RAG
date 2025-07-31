mod rag_utils;
mod utils;
mod query_payload;
mod rag_response;

use axum::{
    routing::post,
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use rag_utils::RagSystem;

static RAG_SYSTEM: OnceLock<RagSystem> = OnceLock::new();

use tokio::io::AsyncWriteExt;
use crate::utils::handle_query_with_pdf_url;

#[tokio::main]
async fn main() {
    // Initialize environment variables and logging
    dotenv::dotenv().ok();
    env_logger::init();

    // Initialize RAG system
    match RagSystem::new("../RAG").await {
        Ok(rag_system) => {
            RAG_SYSTEM.set(rag_system).unwrap();
            println!("RAG system initialized successfully");
        },
        Err(e) => {
            eprintln!("Failed to initialize RAG system: {}", e);
            std::process::exit(1);
        }
    }

    let app = Router::new()
        .route("/query", post(handle_query_with_pdf_url));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}