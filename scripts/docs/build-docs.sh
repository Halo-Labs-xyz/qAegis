#!/bin/bash
# Build documentation for local preview

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

cd "$PROJECT_ROOT"

echo "Building Rust documentation..."
cd services/qrms
cargo doc --no-deps --all-features

echo "Preparing documentation site..."
cd "$PROJECT_ROOT"
mkdir -p _site

# Copy Rust documentation
if [ -d "services/qrms/target/doc" ]; then
    echo "Copying Rust documentation..."
    cp -r services/qrms/target/doc _site/rust
fi

# Convert markdown to HTML
echo "Converting markdown to HTML..."

# Check if markdown library is available
if python3 -c "import markdown" 2>/dev/null; then
    python3 scripts/docs/convert-md-to-html.py
else
    echo "Warning: Python markdown library not found. Install with: pip install markdown"
    echo "Copying markdown files as-is..."
    mkdir -p _site/docs
    cp -r docs/* _site/docs/ || true
    cp README.md _site/ || true
    cp CONTRIBUTING.md _site/ || true
    cp DEPLOYMENT.md _site/ || true
    cp CHANGELOG.md _site/ || true
fi

# Create index page
cat > _site/index.html << 'EOF'
<!DOCTYPE html>
<html>
<head>
  <title>QuantumAegis Documentation</title>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <style>
    body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; max-width: 1200px; margin: 0 auto; padding: 2rem; }
    h1 { color: #333; }
    .section { margin: 2rem 0; padding: 1rem; background: #f5f5f5; border-radius: 8px; }
    a { color: #0066cc; text-decoration: none; }
    a:hover { text-decoration: underline; }
    ul { list-style: none; padding-left: 0; }
    li { margin: 0.5rem 0; }
  </style>
</head>
<body>
  <h1>QuantumAegis Documentation</h1>
  <div class="section">
    <h2>Rust API Documentation</h2>
    <ul>
      <li><a href="rust/qrms/index.html">QRMS Rust API</a></li>
      <li><a href="rust/qrms/apqc/index.html">APQC Module</a></li>
      <li><a href="rust/qrms/qrm/index.html">QRM Module</a></li>
      <li><a href="rust/qrms/crypto/index.html">Crypto Module</a></li>
    </ul>
  </div>
  <div class="section">
    <h2>Architecture Documentation</h2>
    <ul>
      <li><a href="docs/architecture/README.html">Architecture Overview</a></li>
      <li><a href="docs/architecture/stack_architecture.html">Stack Architecture</a></li>
      <li><a href="docs/architecture/qrms_implementation.html">QRMS Implementation</a></li>
      <li><a href="docs/architecture/quantum_resistance_model.html">Quantum Resistance Model</a></li>
      <li><a href="docs/architecture/threat_taxonomy.html">Threat Taxonomy</a></li>
    </ul>
  </div>
  <div class="section">
    <h2>Project Documentation</h2>
    <ul>
      <li><a href="README.html">README</a></li>
      <li><a href="CONTRIBUTING.html">Contributing</a></li>
      <li><a href="DEPLOYMENT.html">Deployment</a></li>
      <li><a href="CHANGELOG.html">Changelog</a></li>
    </ul>
  </div>
</body>
</html>
EOF

echo "Documentation built in _site/"
echo "To preview locally, run: cd _site && python3 -m http.server 8000"
