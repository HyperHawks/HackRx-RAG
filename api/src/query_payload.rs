use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct QueryPayload {
    pub query: String,
    pub pdf_url: Option<String>, // New optional field for PDF URL
}
