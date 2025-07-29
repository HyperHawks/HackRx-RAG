mod models;
mod document_processor;
mod embedding_service;
mod gemini_service;
mod query_service;

use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer, Result as ActixResult};
use anyhow::Result;
use document_processor::DocumentProcessor;
use embedding_service::EmbeddingService;
use gemini_service::GeminiService;
use models::*;
use query_service::QueryService;
use std::sync::Arc;
use tokio::sync::RwLock;

type AppState = web::Data<Arc<RwLock<Vec<Document>>>>;

struct Services {
    query_service: Arc<QueryService>,
}

async fn health_check() -> ActixResult<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "message": "RAG System is running"
    })))
}

async fn query_documents(
    query_req: web::Json<QueryRequest>,
    data: AppState,
    services: web::Data<Services>,
) -> ActixResult<HttpResponse> {
    let documents = data.read().await;
    let max_results = query_req.max_results.unwrap_or(5);

    match services.query_service.query(&query_req.query, &documents, max_results).await {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(e) => {
            log::error!("Query error: {}", e);
            Ok(HttpResponse::InternalServerError().json(ErrorResponse {
                status: "error".to_string(),
                error: format!("Query processing failed: {}", e),
            }))
        }
    }
}

async fn get_documents_info(data: AppState) -> ActixResult<HttpResponse> {
    let documents = data.read().await;
    let info: Vec<_> = documents
        .iter()
        .map(|d| serde_json::json!({
            "id": d.id,
            "filename": d.filename,
            "chunks_count": d.chunks.len(),
            "content_length": d.content.len()
        }))
        .collect();

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "documents": info,
        "total_documents": documents.len()
    })))
}

async fn initialize_system() -> Result<(Arc<RwLock<Vec<Document>>>, Services)> {
    // Load environment variables
    dotenv::dotenv().ok();
    env_logger::init();

    log::info!("Initializing RAG System...");

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

    log::info!("RAG System initialized successfully!");

    let documents_state = Arc::new(RwLock::new(documents));
    let services = Services { query_service };

    Ok((documents_state, services))
}

#[actix_web::main]
async fn main() -> Result<()> {
    let (documents_state, services) = initialize_system().await?;

    log::info!("Starting web server on http://127.0.0.1:8080");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(documents_state.clone()))
            .app_data(web::Data::new(services.query_service.clone()))
            .app_data(web::Data::new(Services {
                query_service: services.query_service.clone(),
            }))
            .route("/health", web::get().to(health_check))
            .route("/query", web::post().to(query_documents))
            .route("/documents", web::get().to(get_documents_info))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await?;

    Ok(())
}
