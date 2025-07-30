# RAG System in Rust

A high-performance Retrieval-Augmented Generation (RAG) system built in Rust that processes PDF documents and provides intelligent query responses using Google's Gemini LLM with precise citations.

## Features

- 🔍 **PDF Document Processing**: Automatically extracts and processes text from PDF files
- 🧠 **Semantic Search**: Uses advanced embeddings to find relevant document chunks
- 🤖 **Gemini Integration**: Leverages Google's Gemini LLM for intelligent responses
- 📖 **Precise Citations**: Provides exact text excerpts and document references
- ⚡ **High Performance**: Built in Rust for speed and reliability
- 🌐 **REST API**: Clean JSON API for easy integration

## Quick Start

### Prerequisites

- Rust 1.70+ installed
- Google Gemini API key (already configured in `.env`)

### Installation & Running

```bash
# Build the project
cargo build --release

# Run the RAG system
cargo run
```

The server will start on `http://127.0.0.1:8080`

## API Endpoints

### 1. Health Check
```http
GET /health
```
**Response:**
```json
{
  "status": "success",
  "message": "RAG System is running"
}
```

### 2. Query Documents
```http
POST /query
Content-Type: application/json
```
**Request:**
```json
{
  "query": "What are the key findings about economic growth?",
  "max_results": 5
}
```

**Response:**
```json
{
  "status": "success",
  "response": "Based on the analyzed documents, the key findings about economic growth indicate...",
  "citations": [
    {
      "document": "CHOTGDP23004V012223.pdf",
      "text_excerpt": "Economic growth indicators show a positive trend with GDP increasing by 3.2% annually...",
      "confidence_score": 0.85
    }
  ],
  "processing_time_ms": 1250
}
```

### 3. Get Document Information
```http
GET /documents
```
**Response:**
```json
{
  "status": "success",
  "documents": [
    {
      "id": "uuid-here",
      "filename": "BAJHLIP23020V012223.pdf",
      "chunks_count": 45,
      "content_length": 12500
    }
  ],
  "total_documents": 5
}
```

## Usage Examples

### Basic Query
```bash
curl -X POST http://127.0.0.1:8080/query \
  -H "Content-Type: application/json" \
  -d '{
    "query": "What are the main risk factors mentioned in the documents?",
    "max_results": 3
  }'
```

### Document Information
```bash
curl http://127.0.0.1:8080/documents
```

## Architecture

The system consists of several key components:

1. **Document Processor**: Extracts text from PDFs and creates semantic chunks
2. **Embedding Service**: Generates vector embeddings for semantic search
3. **Query Service**: Orchestrates the retrieval and generation process
4. **Gemini Service**: Interfaces with Google's Gemini LLM
5. **REST API**: Provides clean HTTP endpoints

## Configuration

The system uses environment variables defined in `.env`:

```env
GEMINI_API_KEY=your_gemini_api_key_here
```

## Document Processing

- **Chunk Size**: 500 characters with 50-character overlap
- **Embedding Model**: AllMiniLML6V2 for fast, accurate embeddings
- **Similarity**: Cosine similarity for chunk relevance scoring

## Performance

- **Concurrent Processing**: Utilizes Rust's async capabilities
- **Memory Efficient**: Streaming processing for large documents
- **Fast Retrieval**: Vector similarity search in milliseconds
- **Caching**: Embeddings are computed once and cached

## Error Handling

All API responses include proper HTTP status codes:

- `200`: Success
- `400`: Bad Request (invalid JSON or parameters)
- `500`: Internal Server Error

Error response format:
```json
{
  "status": "error",
  "error": "Detailed error message"
}
```

## Logs

The system provides detailed logging:
- Document processing progress
- Embedding generation status
- Query processing metrics
- Error diagnostics

Start the server and monitor logs for system status and performance metrics.
