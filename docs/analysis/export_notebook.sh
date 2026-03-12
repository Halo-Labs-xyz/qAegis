#!/bin/bash
# Export script for HYPE DCF Model notebook
# Ensures proper rendering of plots and outputs

NOTEBOOK="hype_dcf_model.ipynb"
OUTPUT_DIR="exports"

# Create output directory
mkdir -p "$OUTPUT_DIR"

echo "Exporting notebook to HTML..."
# Export to HTML with all outputs
jupyter nbconvert --to html "$NOTEBOOK" \
    --output-dir="$OUTPUT_DIR" \
    --ExecutePreprocessor.enabled=True \
    --ExecutePreprocessor.timeout=600 \
    --TemplateExporter.exclude_input=False \
    --HTMLExporter.template=classic

echo "HTML export complete: $OUTPUT_DIR/${NOTEBOOK%.ipynb}.html"

echo ""
echo "For PDF export (requires LaTeX):"
echo "jupyter nbconvert --to pdf $NOTEBOOK --output-dir=$OUTPUT_DIR"
echo ""
echo "Note: PDF export may not render interactive Plotly charts."
echo "Consider using HTML export and converting to PDF with a browser."
