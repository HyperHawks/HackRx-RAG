use crate::query_payload::QueryPayload;
use crate::rag_response::RagResponse;

use std::process::Command;
use std::io::{self, ErrorKind, Read};
use axum::http::StatusCode;
use axum::Json;
use tokio::io::AsyncWriteExt;
use tempfile::NamedTempFile;

use unicode_segmentation::UnicodeSegmentation;
use tiktoken_rs::{cl100k_base, CoreBPE};

// This struct will hold the extracted text along with metadata
#[derive(Debug, serde::Serialize)]
pub struct TextChunk {
    pub content: String,
    pub doc_id: String,
    pub page_number: Option<u32>,
    pub start_char_index: usize,
    pub end_char_index: usize,
}

// Function to extract text using pdftotext (No change)
pub async fn extract_text_from_pdf_with_pdftotext(file_path: &str) -> Result<String, io::Error> {
    let output = Command::new("pdftotext")
        .arg(file_path)
        .arg("-") // Output to stdout
        .output()
        .await?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        let error_message = String::from_utf8_lossy(&output.stderr);
        eprintln!("pdftotext error: {}", error_message);
        Err(io::Error::new(
            ErrorKind::Other,
            format!("pdftotext failed: {}", error_message)
        ))
    }
}

// Sentence segmentation function (No change)
fn segment_text_into_sentences(text: &str) -> Vec<String> {
    text.unicode_sentences()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

// --- REFINED: Intelligent Chunking with Token-based limits and Overlap ---
pub fn create_chunks_token_based(
    indexed_sentences: Vec<IndexedSentence>,
    doc_id: &str,
    tokenizer: &CoreBPE,
    max_chunk_tokens: usize,
    overlap_tokens: usize,
) -> Vec<TextChunk> {
    let mut chunks: Vec<TextChunk> = Vec::new();
    let mut current_chunk_sentences_buffer: Vec<IndexedSentence> = Vec::new();
    let mut current_chunk_tokens = 0;

    for sentence in indexed_sentences {
        let sentence_tokens = tokenizer.encode_ordinary(&sentence.content).len();

        if current_chunk_tokens + sentence_tokens > max_chunk_tokens && !current_chunk_sentences_buffer.is_empty() {
            let chunk_content = current_chunk_sentences_buffer.iter()
                .map(|s| s.content.as_str())
                .collect::<Vec<&str>>()
                .join(" ");

            let start_idx = current_chunk_sentences_buffer.first().unwrap().start_char_index;
            let end_idx = start_idx + chunk_content.len() - 1;

            chunks.push(TextChunk {
                content: chunk_content,
                doc_id: doc_id.to_string(),
                page_number: None,
                start_char_index: start_idx,
                end_char_index: end_idx,
            });

            let mut new_buffer: Vec<IndexedSentence> = Vec::new();
            let mut new_buffer_tokens = 0;

            for s in current_chunk_sentences_buffer.iter().rev() {
                let s_tokens = tokenizer.encode_ordinary(&s.content).len();
                if new_buffer_tokens + s_tokens > overlap_tokens {
                    break;
                }
                new_buffer.insert(0, s.clone());
                new_buffer_tokens += s_tokens;
            }

            current_chunk_sentences_buffer = new_buffer;
            current_chunk_tokens = new_buffer_tokens;
        }

        current_chunk_sentences_buffer.push(sentence);
        current_chunk_tokens += sentence_tokens;
    }

    if !current_chunk_sentences_buffer.is_empty() {
        let chunk_content = current_chunk_sentences_buffer.iter()
            .map(|s| s.content.as_str())
            .collect::<Vec<&str>>()
            .join(" ");

        let start_idx = current_chunk_sentences_buffer.first().unwrap().start_char_index;
        let end_idx = start_idx + chunk_content.len() - 1;

        chunks.push(TextChunk {
            content: chunk_content,
            doc_id: doc_id.to_string(),
            page_number: None,
            start_char_index: start_idx,
            end_char_index: end_idx,
        });
    }

    chunks
}

// Helper struct to keep track of sentence content and its original start index
#[derive(Debug, Clone)]
struct IndexedSentence {
    content: String,
    start_char_index: usize,
}

// Function to segment text into sentences, preserving their original start character index
fn segment_text_into_indexed_sentences(text: &str) -> Vec<IndexedSentence> {
    let mut indexed_sentences = Vec::new();
    let mut current_char_index = 0;

    for sentence in text.unicode_sentences() {
        let trimmed_sentence = sentence.trim();
        if !trimmed_sentence.is_empty() {
            indexed_sentences.push(IndexedSentence {
                content: trimmed_sentence.to_string(),
                start_char_index: current_char_index + (sentence.len() - trimmed_sentence.len()),
            });
        }
        current_char_index += sentence.len();
    }
    indexed_sentences
}

pub async fn handle_query_with_pdf_url(
    Json(payload): Json<QueryPayload>,
) -> Result<Json<RagResponse>, (StatusCode, String)> {
    // Clone user_query early if process_rag_query needs its own copy
    let user_query = payload.query.clone(); // Clone here

    let mut extracted_text_for_rag = String::new();

    let bpe = cl100k_base().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to load tokenizer: {}", e)))?;

    if let Some(pdf_url) = payload.pdf_url {
        println!("Attempting to download PDF from: {}", pdf_url);
        let response = reqwest::get(&pdf_url).await
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("Failed to download PDF: {}", e)))?;

        let pdf_bytes = response.bytes().await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to read PDF bytes: {}", e)))?;

        let mut temp_file = NamedTempFile::new()
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create temp file: {}", e)))?;
        let temp_path = temp_file.path().to_path_buf();

        temp_file.write_all(&pdf_bytes).await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to write to temp file: {}", e)))?;
        temp_file.flush().await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to flush temp file: {}", e)))?;

        let doc_identifier = pdf_url.split('/').last().unwrap_or("unknown_url_doc").to_string();
        let pdf_text = extract_text_from_pdf_with_pdftotext(temp_path.to_str().unwrap()).await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("PDF text extraction failed: {}", e)))?;

        let indexed_sentences = segment_text_into_indexed_sentences(&pdf_text);

        const MAX_CHUNK_TOKENS: usize = 700;
        const OVERLAP_TOKENS: usize = 100;

        let chunks = create_chunks_token_based(indexed_sentences, &doc_identifier, &bpe, MAX_CHUNK_TOKENS, OVERLAP_TOKENS);

        extracted_text_for_rag = chunks.iter()
            .map(|c| c.content.clone())
            .collect::<Vec<String>>()
            .join("\n\n");

        let encoded_context_tokens = bpe.encode_ordinary(&extracted_text_for_rag);
        let max_llm_context_tokens = 4096 - bpe.encode_ordinary(&user_query).len() - 50;
        if encoded_context_tokens.len() > max_llm_context_tokens {
            let truncated_context_tokens = encoded_context_tokens[0..max_llm_context_tokens].to_vec();
            extracted_text_for_rag = bpe.decode(truncated_context_tokens)
                .unwrap_or_else(|_| "Context truncated due to token limit.".to_string());
            println!("Context truncated to {} tokens.", bpe.encode_ordinary(&extracted_text_for_rag).len());
        }
    }

    // Now, pass the cloned `user_query` and `extracted_text_for_rag`
    match process_rag_query(user_query, extracted_text_for_rag).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// Changed the signature to accept String for user_query and file_context
// And changed the return type to Result<RagResponse, String>
pub async fn process_rag_query(user_query: String, file_context: String) -> Result<RagResponse, String> {
    println!("Received query for RAG: {}", user_query);
    println!("File context provided: {}", !file_context.is_empty());

    let mut all_context_for_llm = String::new();
    let mut response_context_snippets: Vec<String> = Vec::new();

    // 1. Incorporate file context if available
    if !file_context.is_empty() {
        all_context_for_llm.push_str("### PROVIDED DOCUMENT CONTEXT:\n");
        all_context_for_llm.push_str(&file_context);
        all_context_for_llm.push_str("\n\n");
        response_context_snippets.push(format!("Context from uploaded file (first {} chars): {}", file_context.len().min(200), &file_context[0..file_context.len().min(200)]));
        if file_context.len() > 200 { response_context_snippets.push("... (truncated)".to_string()); }
    } else {
        // Add general dummy context if no file is provided
        all_context_for_llm.push_str("### GENERAL KNOWLEDGE BASE CONTEXT:\n");
        all_context_for_llm.push_str("General information about Rust programming language is available.\n");
        all_context_for_llm.push_str("Policies often cover terms like 'deductible', 'premium', 'claim process', and 'coverage limits'.\n\n");
        response_context_snippets.push("General knowledge context used.".to_string());
    }

    // 2. Construct the prompt for the LLM (this is still dummy for now)
    let llm_prompt = format!(
        "{}\n\n### USER QUESTION:\n{}\n\n### ANSWER:",
        all_context_for_llm,
        user_query // Use user_query directly
    );

    println!("Full LLM Prompt (first 500 chars):\n{}", &llm_prompt[0..llm_prompt.len().min(500)]);

    // --- Placeholder LLM Call Logic ---
    let dummy_answer = if !file_context.is_empty() {
        format!(
            "Based on the provided document context and your question about '{}', here is the synthesized answer: [LLM would generate answer here using the text from the PDF]. For instance, if you asked about fire damage, the document states: 'Fire damage to the insured property is covered up to a maximum of INR 10,00,000, as detailed in section 4.1.2.'",
            user_query
        )
    } else {
        format!(
            "Based on general knowledge, the answer to your question about '{}' is: Rust is a systems programming language focused on safety, performance, and concurrency. No specific document context was provided.",
            user_query
        )
    };
    
    Ok(RagResponse {
        answer: dummy_answer,
        context_snippets: response_context_snippets,
    })
}