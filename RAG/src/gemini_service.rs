use crate::models::*;
use anyhow::Result;
use reqwest::Client;
use std::env;

pub struct GeminiService {
    client: Client,
    api_key: String,
}

impl GeminiService {
    pub fn new() -> Result<Self> {
        let api_key = env::var("GEMINI_API_KEY")
            .map_err(|_| anyhow::anyhow!("GEMINI_API_KEY environment variable not set"))?;

        Ok(Self {
            client: Client::new(),
            api_key,
        })
    }

    pub async fn generate_response(&self, query: &str, relevant_chunks: &[DocumentChunk], documents: &[Document]) -> Result<String> {
        let context = self.build_context(relevant_chunks, documents);
        let prompt = self.build_prompt(query, &context);

        let request = GeminiRequest {
            contents: vec![GeminiContent {
                parts: vec![GeminiPart {
                    text: prompt,
                }],
            }],
            generation_config: Some(GeminiGenerationConfig {
                temperature: 0.3,
                max_output_tokens: 1000,
            }),
        };

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
            self.api_key
        );

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Gemini API error: {}", error_text));
        }

        let gemini_response: GeminiResponse = response.json().await?;
        
        let answer = gemini_response
            .candidates
            .first()
            .and_then(|c| c.content.parts.first())
            .map(|p| p.text.clone())
            .unwrap_or_else(|| "No response generated".to_string());

        Ok(answer)
    }

    fn build_context(&self, chunks: &[DocumentChunk], documents: &[Document]) -> String {
        let mut context = String::new();
        
        for chunk in chunks {
            // Find the document this chunk belongs to
            if let Some(doc) = documents.iter().find(|d| d.chunks.iter().any(|c| c.id == chunk.id)) {
                context.push_str(&format!(
                    "Document: {}\nContent: {}\n\n",
                    doc.filename,
                    chunk.content
                ));
            }
        }
        
        context
    }

    fn build_prompt(&self, query: &str, context: &str) -> String {
        format!(
            r#"You are an expert assistant that answers questions based solely on the provided context documents. 

INSTRUCTIONS:
1. Answer the question using ONLY the information from the provided context
2. Be concise but comprehensive
3. If you quote or reference specific information, indicate which document it came from
4. If the context doesn't contain enough information to answer the question, say so clearly
5. Do not add information not present in the context
6. Focus on accuracy and relevance
7. If user provides info such as M or F the user is specifying it's gender for example: 46M, knee surgery, Pune, 3-month policy means 46 year old male asking if knee surgery is covered or not he is from pune and has 3 months policy

CONTEXT DOCUMENTS:
{context}

QUESTION: {query}

ANSWER (be specific and cite sources):"#
        )
    }
}
