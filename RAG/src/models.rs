use serde::{Deserialize, Serialize};

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
pub struct QueryRequest {
    pub query: String,
    pub max_results: Option<usize>,
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
pub struct GeminiRequest {
    pub contents: Vec<GeminiContent>,
    pub generation_config: Option<GeminiGenerationConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeminiContent {
    pub parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeminiPart {
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeminiGenerationConfig {
    pub temperature: f32,
    pub max_output_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeminiResponse {
    pub candidates: Vec<GeminiCandidate>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeminiCandidate {
    pub content: GeminiContent,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub status: String,
    pub error: String,
}
