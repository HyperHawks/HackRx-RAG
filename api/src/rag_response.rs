use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct RagResponse {
    pub answer: String,
    pub context_snippets: Vec<String>,
}