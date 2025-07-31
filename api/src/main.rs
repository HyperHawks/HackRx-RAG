mod rag_utils;
mod utils;

use axum::{
    routing::post,
    http::StatusCode,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use rag_utils::RagSystem;

static RAG_SYSTEM: OnceLock<RagSystem> = OnceLock::new();

#[derive(Deserialize, Serialize)]
pub struct QueryPayload {
    pub query: String,
}

#[derive(Deserialize, Serialize)]
pub struct RagResponse {
    pub answer: String,
    pub context_snippets: Vec<String>,
}

async fn process_rag_query(payload: QueryPayload) -> Result<RagResponse, String> {
    println!("Received query for RAG: {}", payload.query);

    let rag_system = RAG_SYSTEM.get().ok_or("RAG system not initialized")?;
    
    match rag_system.query(&payload.query, None).await {
        Ok(query_response) => {
            let context_snippets = query_response
                .citations
                .iter()
                .map(|citation| format!("{}: {}", citation.document, citation.text_excerpt))
                .collect();

            Ok(RagResponse {
                answer: query_response.response,
                context_snippets,
            })
        },
        Err(e) => Err(format!("RAG query failed: {}", e)),
    }
}

async fn handle_query(
    Json(payload): Json<QueryPayload>,
) -> Result<Json<RagResponse>, (StatusCode, String)> {
    match process_rag_query(payload).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

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
        .route("/query", post(handle_query));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}