use serde::Deserialize;

#[derive(Deserialize)]
pub struct HackRxRequest {
    pub documents: String,
    pub questions: Vec<String>,
}
