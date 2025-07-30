use reqwest::Client;
use serde_json::json;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let base_url = "http://127.0.0.1:8080";

    println!("🔍 Testing RAG System Client");

    // Test health check
    println!("\n📋 Health Check:");
    let health_response = client
        .get(&format!("{}/health", base_url))
        .send()
        .await?;
    
    println!("Status: {}", health_response.status());
    let health_json: serde_json::Value = health_response.json().await?;
    println!("Response: {}", serde_json::to_string_pretty(&health_json)?);

    // Test document info
    println!("\n📚 Document Information:");
    let docs_response = client
        .get(&format!("{}/documents", base_url))
        .send()
        .await?;
    
    let docs_json: serde_json::Value = docs_response.json().await?;
    println!("Response: {}", serde_json::to_string_pretty(&docs_json)?);

    // Test query
    println!("\n🔍 Query Test:");
    let query_payload = json!({
        "query": "What are the main topics and findings discussed in these financial documents?",
        "max_results": 3
    });

    let query_response = client
        .post(&format!("{}/query", base_url))
        .header("Content-Type", "application/json")
        .json(&query_payload)
        .send()
        .await?;

    println!("Status: {}", query_response.status());
    let query_json: serde_json::Value = query_response.json().await?;
    println!("Response: {}", serde_json::to_string_pretty(&query_json)?);

    println!("\n✅ Client test completed!");
    Ok(())
}
