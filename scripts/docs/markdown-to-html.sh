#!/bin/bash
# Convert markdown files to HTML

set -e

convert_md_to_html() {
    local md_file="$1"
    local output_dir="$2"
    
    if [ ! -f "$md_file" ]; then
        return
    fi
    
    local rel_path="${md_file#$3/}"
    local html_file="${rel_path%.md}.html"
    local output_path="$output_dir/$html_file"
    local output_dir_path="$(dirname "$output_path")"
    
    mkdir -p "$output_dir_path"
    
    # Extract title from first h1 or use filename
    local title=$(head -n 1 "$md_file" | sed 's/^# *//' || basename "$md_file" .md)
    
    # Convert markdown to HTML using basic sed/awk or pandoc if available
    if command -v pandoc &> /dev/null; then
        pandoc "$md_file" -f markdown -t html5 --standalone \
            --metadata title="$title" \
            --css /style.css \
            -o "$output_path"
    else
        # Fallback: simple markdown to HTML conversion
        cat > "$output_path" << EOF
<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>$title</title>
  <link rel="stylesheet" href="/style.css">
  <script src="https://cdn.jsdelivr.net/npm/marked/marked.min.js"></script>
</head>
<body>
  <div class="markdown-body">
    <script>
      fetch('$rel_path')
        .then(r => r.text())
        .then(md => {
          document.body.innerHTML = marked.parse(md);
        });
    </script>
  </div>
</body>
</html>
EOF
        # Copy original markdown for client-side rendering
        mkdir -p "$(dirname "$output_dir/$rel_path")"
        cp "$md_file" "$output_dir/$rel_path"
    fi
}

# Better approach: use a simple markdown-to-HTML converter
convert_with_basic_parser() {
    local md_file="$1"
    local output_path="$2"
    local title="$3"
    local base_path="$4"
    
    # Read markdown and convert basic elements
    {
        echo "<!DOCTYPE html>"
        echo "<html>"
        echo "<head>"
        echo "  <meta charset=\"utf-8\">"
        echo "  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">"
        echo "  <title>$title</title>"
        echo "  <link rel=\"stylesheet\" href=\"$base_path/style.css\">"
        echo "  <style>"
        echo "    body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; max-width: 1200px; margin: 0 auto; padding: 2rem; line-height: 1.6; }"
        echo "    h1, h2, h3 { color: #333; margin-top: 2rem; }"
        echo "    code { background: #f5f5f5; padding: 0.2em 0.4em; border-radius: 3px; font-family: 'Monaco', 'Courier New', monospace; }"
        echo "    pre { background: #f5f5f5; padding: 1rem; border-radius: 5px; overflow-x: auto; }"
        echo "    pre code { background: none; padding: 0; }"
        echo "    a { color: #0066cc; }"
        echo "    table { border-collapse: collapse; width: 100%; margin: 1rem 0; }"
        echo "    th, td { border: 1px solid #ddd; padding: 0.5rem; text-align: left; }"
        echo "    th { background: #f5f5f5; }"
        echo "    blockquote { border-left: 4px solid #ddd; padding-left: 1rem; margin-left: 0; color: #666; }"
        echo "  </style>"
        echo "</head>"
        echo "<body>"
        echo "  <div class=\"markdown-body\">"
        
        # Convert markdown to HTML using sed/awk
        awk '
        BEGIN { in_code = 0; in_pre = 0; }
        /^```/ {
            if (in_pre) {
                print "</pre>"
                in_pre = 0
            } else {
                lang = $2
                print "<pre><code>"
                in_pre = 1
            }
            next
        }
        in_pre {
            gsub(/&/, "\\&amp;")
            gsub(/</, "\\&lt;")
            gsub(/>/, "\\&gt;")
            print
            next
        }
        /^# / { gsub(/^# /, "<h1>"); print $0 "</h1>"; next }
        /^## / { gsub(/^## /, "<h2>"); print $0 "</h2>"; next }
        /^### / { gsub(/^### /, "<h3>"); print $0 "</h3>"; next }
        /^#### / { gsub(/^#### /, "<h4>"); print $0 "</h4>"; next }
        /^\|.*\|/ {
            if (!table_started) {
                print "<table>"
                table_started = 1
            }
            gsub(/^\| /, "<tr><td>")
            gsub(/ \|/, "</td><td>")
            gsub(/ \|$/, "</td></tr>")
            print
            next
        }
        /^$/ {
            if (table_started) {
                print "</table>"
                table_started = 0
            }
            print "<br>"
            next
        }
        {
            # Inline code
            gsub(/`([^`]+)`/, "<code>\\1</code>")
            # Links [text](url)
            gsub(/\[([^\]]+)\]\(([^)]+)\)/, "<a href=\"\\2\">\\1</a>")
            # Bold **text**
            gsub(/\*\*([^*]+)\*\*/, "<strong>\\1</strong>")
            # Italic *text*
            gsub(/\*([^*]+)\*/, "<em>\\1</em>")
            print "<p>" $0 "</p>"
        }
        END {
            if (in_pre) print "</pre></code>"
            if (table_started) print "</table>"
        }
        ' "$md_file"
        
        echo "  </div>"
        echo "</body>"
        echo "</html>"
    } > "$output_path"
}

# Main conversion function
convert_markdown_files() {
    local source_dir="$1"
    local output_dir="$2"
    local base_path="$3"
    
    find "$source_dir" -name "*.md" -type f | while read -r md_file; do
        local rel_path="${md_file#$source_dir/}"
        local html_file="${rel_path%.md}.html"
        local output_path="$output_dir/$html_file"
        local output_dir_path="$(dirname "$output_path")"
        
        mkdir -p "$output_dir_path"
        
        # Extract title from first h1
        local title=$(grep -m 1 "^# " "$md_file" | sed 's/^# *//' || basename "$md_file" .md)
        
        if command -v pandoc &> /dev/null; then
            pandoc "$md_file" -f markdown -t html5 --standalone \
                --metadata title="$title" \
                --css "$base_path/style.css" \
                -o "$output_path"
        else
            convert_with_basic_parser "$md_file" "$output_path" "$title" "$base_path"
        fi
    done
}

if [ "$#" -lt 2 ]; then
    echo "Usage: $0 <source_dir> <output_dir> [base_path]"
    exit 1
fi

convert_markdown_files "$1" "$2" "${3:-..}"
