#!/bin/bash

echo "🚀 Testing RAG System..."

# Wait for server to start
sleep 3

# Test health endpoint
echo "📋 Testing health check..."
curl -s http://127.0.0.1:8080/health | jq .

echo -e "\n📚 Getting document information..."
curl -s http://127.0.0.1:8080/documents | jq .

echo -e "\n🔍 Testing query..."
curl -s -X POST http://127.0.0.1:8080/query \
  -H "Content-Type: application/json" \
  -d '{
    "query": "What are the key financial metrics and performance indicators mentioned in these documents?",
    "max_results": 3
  }' | jq .

echo -e "\n✅ Test completed!"
