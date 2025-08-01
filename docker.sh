#!/bin/bash

# HackRx-RAG Docker Build and Run Script

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_message() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    print_error "Docker is not installed. Please install Docker and try again."
    exit 1
fi

# Check if docker-compose is installed
if ! command -v docker-compose &> /dev/null; then
    print_warning "docker-compose is not installed. Using 'docker compose' instead."
    COMPOSE_CMD="docker compose"
else
    COMPOSE_CMD="docker-compose"
fi

# Function to build the Docker image
build_image() {
    print_message "Building HackRx-RAG Docker image..."
    docker build -t hackrx-rag:latest .
    print_message "Docker image built successfully!"
}

# Function to run with docker-compose
run_compose() {
    print_message "Starting HackRx-RAG with docker-compose..."
    if [ ! -f .env ]; then
        print_warning ".env file not found. Creating from template..."
        cp .env.template .env
        print_warning "Please edit .env file with your actual API keys before running!"
        return 1
    fi
    $COMPOSE_CMD up -d
    print_message "HackRx-RAG is running at http://localhost:8000"
}

# Function to run the container directly
run_container() {
    print_message "Running HackRx-RAG container..."
    docker run -d \
        --name hackrx-rag \
        -p 8000:8000 \
        --env-file .env \
        -v "$(pwd)/RAG:/app/data:ro" \
        hackrx-rag:latest
    print_message "HackRx-RAG is running at http://localhost:8000"
}

# Function to stop containers
stop_containers() {
    print_message "Stopping HackRx-RAG containers..."
    $COMPOSE_CMD down 2>/dev/null || docker stop hackrx-rag 2>/dev/null || true
    docker rm hackrx-rag 2>/dev/null || true
    print_message "Containers stopped."
}

# Function to show logs
show_logs() {
    if $COMPOSE_CMD ps | grep -q hackrx-rag-api; then
        $COMPOSE_CMD logs -f hackrx-rag-api
    elif docker ps | grep -q hackrx-rag; then
        docker logs -f hackrx-rag
    else
        print_error "No running HackRx-RAG containers found."
    fi
}

# Function to show help
show_help() {
    echo "HackRx-RAG Docker Management Script"
    echo ""
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  build       Build the Docker image"
    echo "  run         Run with docker-compose (recommended)"
    echo "  run-docker  Run with docker directly"
    echo "  stop        Stop all containers"
    echo "  logs        Show container logs"
    echo "  restart     Stop and restart containers"
    echo "  help        Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 build          # Build the image"
    echo "  $0 run            # Start with docker-compose"
    echo "  $0 logs           # View logs"
    echo "  $0 stop           # Stop containers"
}

# Main script logic
case "${1:-help}" in
    build)
        build_image
        ;;
    run)
        build_image
        run_compose
        ;;
    run-docker)
        build_image
        run_container
        ;;
    stop)
        stop_containers
        ;;
    logs)
        show_logs
        ;;
    restart)
        stop_containers
        sleep 2
        run_compose
        ;;
    help|--help|-h)
        show_help
        ;;
    *)
        print_error "Unknown command: $1"
        show_help
        exit 1
        ;;
esac
