#!/usr/bin/env python3
"""Inject mermaid.js into Rust documentation HTML files"""

import os
import re
from pathlib import Path

MERMAID_SCRIPT = '''<script src="https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.min.js"></script>
<script>
  mermaid.initialize({ startOnLoad: true, theme: 'default', securityLevel: 'loose' });
  mermaid.run();
</script>'''

MERMAID_STYLE = '''<style>
  .mermaid { text-align: center; margin: 2rem 0; background: #fff; padding: 1rem; border-radius: 5px; min-height: 2rem; }
</style>'''

def inject_mermaid_into_html(html_path):
    """Inject mermaid.js and styles into a Rust doc HTML file"""
    with open(html_path, 'r', encoding='utf-8') as f:
        html_content = f.read()
    
    # Check if mermaid is already injected
    if 'mermaid@10' in html_content:
        return False
    
    # Convert mermaid code blocks to divs if present
    # Rust docs use <pre class="language-mermaid"><code>...</code></pre> or similar
    html_content = re.sub(
        r'<pre[^>]*class="[^"]*mermaid[^"]*"[^>]*><code[^>]*>(.*?)</code></pre>',
        lambda m: f'<div class="mermaid">\n{m.group(1).replace("&lt;", "<").replace("&gt;", ">").replace("&amp;", "&")}\n</div>',
        html_content,
        flags=re.DOTALL | re.IGNORECASE
    )
    
    # Also handle <pre><code class="language-mermaid">...</code></pre>
    html_content = re.sub(
        r'<pre><code[^>]*class="[^"]*mermaid[^"]*"[^>]*>(.*?)</code></pre>',
        lambda m: f'<div class="mermaid">\n{m.group(1).replace("&lt;", "<").replace("&gt;", ">").replace("&amp;", "&")}\n</div>',
        html_content,
        flags=re.DOTALL | re.IGNORECASE
    )
    
    # Inject mermaid style in <head>
    if '<head>' in html_content and MERMAID_STYLE not in html_content:
        # Insert style before </head>
        html_content = html_content.replace('</head>', f'{MERMAID_STYLE}</head>', 1)
    
    # Inject mermaid script before </body>
    if '</body>' in html_content:
        html_content = html_content.replace('</body>', f'{MERMAID_SCRIPT}</body>', 1)
    elif '</html>' in html_content:
        # If no </body>, inject before </html>
        html_content = html_content.replace('</html>', f'{MERMAID_SCRIPT}</html>', 1)
    
    with open(html_path, 'w', encoding='utf-8') as f:
        f.write(html_content)
    
    return True

def process_rust_docs(doc_dir):
    """Process all HTML files in Rust documentation directory"""
    doc_path = Path(doc_dir)
    if not doc_path.exists():
        print(f"Rust docs directory not found: {doc_dir}")
        return
    
    html_files = list(doc_path.rglob("*.html"))
    if not html_files:
        print(f"No HTML files found in {doc_dir}")
        return
    
    processed = 0
    for html_file in html_files:
        if inject_mermaid_into_html(html_file):
            processed += 1
    
    print(f"Injected mermaid.js into {processed} Rust documentation files")

if __name__ == "__main__":
    import sys
    
    # Default to _site/rust if no argument provided
    rust_doc_dir = sys.argv[1] if len(sys.argv) > 1 else "_site/rust"
    
    process_rust_docs(rust_doc_dir)
