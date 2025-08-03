mod hackrx_request;
mod hackrx_response;
mod utils;
mod auth;
mod query_payload;
mod rag_response;

use axum::{
    extract::State, 
    routing::{get, post}, 
    Json, Router,
    middleware,
    http::{StatusCode, Method},
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{CorsLayer, Any};
use serde::Serialize;

use rag_system::{models::Document, RagLibrary};

use crate::{
    hackrx_request::HackRxRequest,
    hackrx_response::HackRxResponse,
    utils::handle_hackrx_run,
    auth::{auth_middleware, generate_mock_token},
    query_payload::QueryPayload,
    rag_response::RagResponse,
};

// Health check handler
async fn health() -> &'static str {
    "OK"
}

// Login endpoint for generating mock tokens
#[derive(Serialize)]
struct LoginResponse {
    token: String,
    message: String,
}

#[derive(serde::Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

async fn login(Json(payload): Json<LoginRequest>) -> Result<Json<LoginResponse>, (StatusCode, String)> {
    // Mock authentication - in real app, verify credentials against database
    if payload.username.is_empty() || payload.password.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Username and password required".to_string()));
    }
    
    if payload.password.len() < 6 {
        return Err((StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()));
    }
    
    let token = generate_mock_token(&payload.username);
    
    Ok(Json(LoginResponse {
        token,
        message: "Login successful".to_string(),
    }))
}

// Protected endpoint to test authentication
async fn protected() -> &'static str {
    "This is a protected endpoint. You are authenticated!"
}

pub struct AppState {
    pub rag_library: Arc<RagLibrary>,
    pub documents: Arc<RwLock<Vec<Document>>>,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    env_logger::init();

    let (documents, rag_library) = RagLibrary::new().await.unwrap();

    let state = Arc::new(AppState {
        rag_library: Arc::new(rag_library),
        documents: Arc::new(RwLock::new(documents)),
    });

    // CORS configuration
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any)
        .allow_origin(Any);

    // Public routes (no authentication required)
    let public_routes = Router::new()
        .route("/health", get(health))
        .route("/login", post(login));

    // Protected routes (authentication required)
    let protected_routes = Router::new()
        .route("/hackrx/run", post(handle_hackrx_run))
        .route("/protected", get(protected))
        .layer(middleware::from_fn(auth_middleware))
        .with_state(state.clone());

    // Combine all routes
    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000")
        .await
        .unwrap();
    
    println!("🚀 Server starting on http://0.0.0.0:8000");
    println!("📋 Health check: http://0.0.0.0:8000/health");
    println!("🔐 Login endpoint: http://0.0.0.0:8000/login");
    println!("🛡️  Protected endpoints require Authorization: Bearer <token>");
    println!("   - POST /hackrx/run");
    println!("   - GET /protected");
    
    axum::serve(listener, app).await.unwrap();
}