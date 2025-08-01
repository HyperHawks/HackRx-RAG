# HackRx-RAG Docker Setup

This document explains how to build and run the HackRx-RAG application using Docker.

## Prerequisites

- Docker (version 20.10 or later)
- Docker Compose (optional, but recommended)
- At least 2GB of free disk space for the Docker image

## Quick Start

### 1. Clone and Setup

```bash
git clone <your-repo-url>
cd HackRx-RAG
```

### 2. Configure Environment Variables

Copy the environment template and edit it with your API keys:

```bash
cp .env.template .env
# Edit .env file with your actual API keys
```

### 3. Build and Run

Using the provided script (recommended):

```bash
# Make script executable (if not already)
chmod +x docker.sh

# Build and run
./docker.sh run
```

Or manually:

```bash
# Build the image
docker build -t hackrx-rag:latest .

# Run with docker-compose
docker-compose up -d
```

### 4. Access the Application

The API will be available at: `http://localhost:8000`

Health check endpoint: `http://localhost:8000/health`

## Docker Commands

### Using the Management Script

The `docker.sh` script provides convenient commands:

```bash
./docker.sh build      # Build the Docker image
./docker.sh run        # Build and run with docker-compose
./docker.sh run-docker # Build and run with docker directly
./docker.sh stop       # Stop all containers
./docker.sh logs       # Show container logs
./docker.sh restart    # Restart containers
./docker.sh help       # Show help
```

### Manual Docker Commands

#### Build the image:
```bash
docker build -t hackrx-rag:latest .
```

#### Run the container:
```bash
docker run -d \
  --name hackrx-rag \
  -p 8000:8000 \
  --env-file .env \
  -v "$(pwd)/RAG:/app/data:ro" \
  hackrx-rag:latest
```

#### View logs:
```bash
docker logs -f hackrx-rag
```

#### Stop the container:
```bash
docker stop hackrx-rag
docker rm hackrx-rag
```

## Docker Compose

The `docker-compose.yml` file provides an easy way to manage the application:

```bash
# Start the application
docker-compose up -d

# View logs
docker-compose logs -f

# Stop the application
docker-compose down

# Restart the application
docker-compose restart
```

## Environment Variables

The following environment variables can be configured:

| Variable | Description | Required |
|----------|-------------|----------|
| `GEMINI_API_KEY` | Google Gemini API key | Yes |
| `OPENAI_API_KEY` | OpenAI API key (if used) | Optional |
| `RUST_LOG` | Logging level (debug, info, warn, error) | No (default: info) |

## File Structure

```
HackRx-RAG/
├── Dockerfile              # Docker build configuration
├── docker-compose.yml      # Docker Compose configuration
├── docker.sh              # Management script
├── .dockerignore          # Files to exclude from Docker build
├── .env.template          # Environment variables template
├── Cargo.toml             # Workspace configuration
├── api/                   # API server code
│   ├── Cargo.toml
│   └── src/
└── RAG/                   # RAG library code
    ├── Cargo.toml
    ├── *.pdf              # PDF documents for RAG
    └── src/
```

## Troubleshooting

### Common Issues

1. **Port already in use:**
   ```bash
   # Check what's using port 8000
   lsof -i :8000
   # Or change the port in docker-compose.yml
   ```

2. **Permission denied errors:**
   ```bash
   # Make sure docker.sh is executable
   chmod +x docker.sh
   ```

3. **API keys not working:**
   - Ensure your `.env` file has the correct API keys
   - Check that the `.env` file is in the project root
   - Verify the API keys are valid

4. **Container won't start:**
   ```bash
   # Check container logs
   docker logs hackrx-rag
   # Or with docker-compose
   docker-compose logs hackrx-rag-api
   ```

### Health Check

The application includes a health check endpoint at `/health`. You can test it:

```bash
curl http://localhost:8000/health
```

Should return: `OK`

### Viewing Logs

Real-time logs:
```bash
# With docker-compose
docker-compose logs -f hackrx-rag-api

# With docker directly
docker logs -f hackrx-rag
```

## Production Deployment

For production deployment, consider:

1. **Use a reverse proxy** (nginx, traefik) for SSL termination and load balancing
2. **Set up proper logging** with log aggregation
3. **Configure resource limits** in docker-compose.yml
4. **Use Docker secrets** for API keys instead of environment files
5. **Set up monitoring** and health checks
6. **Use multi-stage builds** for smaller images (already configured)

### Resource Limits Example

Add to docker-compose.yml:

```yaml
services:
  hackrx-rag-api:
    # ... other configuration
    deploy:
      resources:
        limits:
          memory: 1G
          cpus: '0.5'
        reservations:
          memory: 512M
          cpus: '0.25'
```

## Development

For development with Docker:

1. **Mount source code** for live reloading:
   ```yaml
   volumes:
     - ./api:/usr/src/app/api
     - ./RAG:/usr/src/app/RAG
   ```

2. **Use development profile** in Cargo.toml

3. **Enable debug logging**:
   ```bash
   export RUST_LOG=debug
   ```
