#!/bin/bash
# Deployment script for Moltbot Secure

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_DIR"

echo "🚀 Deploying Moltbot Secure..."

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "❌ Docker is not running. Please start Docker and try again."
    exit 1
fi

# Check if config exists
if [ ! -f "moltbot-secure.toml" ]; then
    echo "⚠️  Configuration file not found. Creating from example..."
    cp moltbot-secure.toml.example moltbot-secure.toml
    echo "✅ Created moltbot-secure.toml. Please review and update it before deploying."
    read -p "Press Enter to continue or Ctrl+C to abort..."
fi

# Generate SSL certificates if they don't exist
if [ ! -f "nginx/ssl/cert.pem" ] || [ ! -f "nginx/ssl/key.pem" ]; then
    echo "🔐 Generating SSL certificates..."
    ./nginx/generate-ssl.sh
fi

# Create necessary directories
mkdir -p keys nginx/logs nginx/ssl

# Build and start services
echo "🏗️  Building Docker images..."
docker-compose build

echo "🚀 Starting services..."
docker-compose up -d

echo "⏳ Waiting for services to be healthy..."
sleep 5

# Check health
if curl -f http://localhost:8443/health > /dev/null 2>&1; then
    echo "✅ Service is healthy!"
else
    echo "⚠️  Service health check failed. Check logs with: docker-compose logs"
fi

echo ""
echo "📋 Deployment Summary:"
echo "  - Service: http://localhost:8443"
echo "  - HTTPS: https://localhost"
echo "  - Health: http://localhost:8443/health"
echo ""
echo "📝 Useful commands:"
echo "  - View logs: docker-compose logs -f"
echo "  - Stop services: docker-compose down"
echo "  - Restart services: docker-compose restart"
echo ""
echo "✅ Deployment complete!"
