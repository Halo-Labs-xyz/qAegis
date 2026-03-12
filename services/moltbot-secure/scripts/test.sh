#!/bin/bash
# Test script for Moltbot Secure

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_DIR"

echo "🧪 Running tests for Moltbot Secure..."

# Run Rust tests
echo "Running Rust unit and integration tests..."
cargo test --lib --tests -- --nocapture

echo ""
echo "✅ All tests passed!"
