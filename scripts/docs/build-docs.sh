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
    python3 << 'PYTHON_SCRIPT'
import os
import markdown
from pathlib import Path

def convert_md_to_html(md_path):
    """Convert markdown file to HTML"""
    with open(md_path, 'r', encoding='utf-8') as f:
        md_content = f.read()
    
    # Extract title from first h1
    title = "Documentation"
    for line in md_content.split('\n'):
        if line.startswith('# '):
            title = line[2:].strip()
            break
    
    # Convert markdown to HTML
    html_content = markdown.markdown(
        md_content,
        extensions=['fenced_code', 'tables', 'codehilite']
    )
    
    # Create HTML page
    html_page = f"""<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{title}</title>
  <style>
    body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; max-width: 1200px; margin: 0 auto; padding: 2rem; line-height: 1.6; }}
    h1, h2, h3, h4, h5, h6 {{ color: #333; margin-top: 2rem; }}
    code {{ background: #f5f5f5; padding: 0.2em 0.4em; border-radius: 3px; font-family: 'Monaco', 'Courier New', monospace; }}
    pre {{ background: #f5f5f5; padding: 1rem; border-radius: 5px; overflow-x: auto; }}
    pre code {{ background: none; padding: 0; }}
    a {{ color: #0066cc; text-decoration: none; }}
    a:hover {{ text-decoration: underline; }}
    table {{ border-collapse: collapse; width: 100%; margin: 1rem 0; }}
    th, td {{ border: 1px solid #ddd; padding: 0.5rem; text-align: left; }}
    th {{ background: #f5f5f5; }}
    blockquote {{ border-left: 4px solid #ddd; padding-left: 1rem; margin-left: 0; color: #666; }}
    .mermaid {{ text-align: center; margin: 2rem 0; }}
  </style>
  <script src="https://cdn.jsdelivr.net/npm/mermaid/dist/mermaid.min.js"></script>
  <script>mermaid.initialize({{startOnLoad:true}});</script>
</head>
<body>
  <div class="markdown-body">
    {html_content}
  </div>
</body>
</html>"""
    
    return html_page

def process_directory(source_dir, output_dir):
    """Recursively convert all markdown files in directory"""
    source = Path(source_dir)
    output = Path(output_dir)
    
    for md_file in source.rglob("*.md"):
        rel_path = md_file.relative_to(source)
        html_file = output / rel_path.with_suffix('.html')
        html_file.parent.mkdir(parents=True, exist_ok=True)
        
        html_content = convert_md_to_html(md_file)
        html_file.write_text(html_content, encoding='utf-8')
        print(f"Converted: {md_file} -> {html_file}")

# Convert docs directory
if os.path.exists("docs"):
    process_directory("docs", "_site/docs")

# Convert root markdown files
root_files = ["README.md", "CONTRIBUTING.md", "DEPLOYMENT.md", "CHANGELOG.md"]
for md_file in root_files:
    if os.path.exists(md_file):
        html_content = convert_md_to_html(md_file)
        html_path = f"_site/{os.path.splitext(md_file)[0]}.html"
        with open(html_path, 'w', encoding='utf-8') as f:
            f.write(html_content)
        print(f"Converted: {md_file} -> {html_path}")
PYTHON_SCRIPT
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
