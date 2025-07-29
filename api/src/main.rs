use axum::{
    routing::post,
    http::StatusCode,
    Json, Router,
};
use serde::{Deserialize, Serialize};
// use srijan_lib

#[derive(Deserialize, Serialize)]
pub struct QueryPayload {
    pub query: String,
//     file path
}

#[derive(Deserialize, Serialize)]
pub struct RagResponse {
    pub answer: String,
    pub context_snippets: Vec<String>,
}

async fn process_rag_query(payload: QueryPayload) -> Result<RagResponse, String> {
    println!("Received query for RAG: {}", payload.query);

    // srijan_lib.chunk();

    let dummy_context = vec![
        format!("Information related to '{}' from Document A", payload.query),
        "Another relevant snippet from Document B.".to_string(),
    ];

    let dummy_answer = format!("Based on my knowledge base, the answer to your question about '{}' is: Rust is a systems programming language focused on safety, performance, and concurrency.", payload.query);

    Ok(RagResponse {
        answer: dummy_answer,
        context_snippets: dummy_context,
    })
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
    let app = Router::new()
        .route("/query", post(handle_query));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}