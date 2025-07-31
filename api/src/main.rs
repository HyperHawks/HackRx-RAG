mod hackrx_request;
mod hackrx_response;
mod utils;

use axum::{extract::State, routing::post, Json, Router};
use std::sync::Arc;
use tokio::sync::RwLock;

use rag_system::{models::Document, RagLibrary};

use crate::{
    hackrx_request::HackRxRequest,
    hackrx_response::HackRxResponse,
    utils::handle_hackrx_run,
};

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

    let app = Router::new()
        .route("/hackrx/run", post(handle_hackrx_run))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000")
        .await
        .unwrap();
    println!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}