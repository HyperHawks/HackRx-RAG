use serde::Serialize;

#[derive(Serialize)]
pub struct HackRxResponse {
    pub answers: Vec<String>,
}
