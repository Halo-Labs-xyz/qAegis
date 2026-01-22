#!/usr/bin/env python3
"""Convert markdown files to HTML for documentation site"""

import os
import sys
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

if __name__ == "__main__":
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
