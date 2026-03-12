#!/usr/bin/env python3
"""
Generate a beautiful one-page PDF summary of HYPE DCF analysis.

Requires: pip install reportlab
"""

try:
    from reportlab.lib.pagesizes import letter, A4
except ImportError:
    print("Error: reportlab not installed.")
    print("Install with: pip install reportlab")
    exit(1)
from reportlab.lib.units import inch
from reportlab.lib import colors
from reportlab.lib.styles import getSampleStyleSheet, ParagraphStyle
from reportlab.platypus import SimpleDocTemplate, Paragraph, Spacer, Table, TableStyle, PageBreak
from reportlab.lib.enums import TA_CENTER, TA_LEFT, TA_RIGHT
from reportlab.pdfgen import canvas
from reportlab.lib.colors import HexColor
from datetime import datetime
import os

# Colors - Hyperliquid theme (green/teal)
PRIMARY_COLOR = HexColor('#00D4AA')  # Hyperliquid green
ACCENT_COLOR = HexColor('#7B68EE')   # Purple accent
TEXT_COLOR = HexColor('#1a1a2e')     # Dark blue
LIGHT_BG = HexColor('#f8f9fa')
DARK_TEXT = HexColor('#2c3e50')

def create_summary_pdf(output_path='HYPE_DCF_Summary.pdf'):
    """Create a beautiful one-page PDF summary."""
    
    # Create document
    doc = SimpleDocTemplate(
        output_path,
        pagesize=letter,
        rightMargin=0.75*inch,
        leftMargin=0.75*inch,
        topMargin=0.5*inch,
        bottomMargin=0.5*inch
    )
    
    # Container for the 'Flowable' objects
    elements = []
    
    # Define custom styles
    styles = getSampleStyleSheet()
    
    # Title style
    title_style = ParagraphStyle(
        'CustomTitle',
        parent=styles['Heading1'],
        fontSize=32,
        textColor=PRIMARY_COLOR,
        spaceAfter=12,
        alignment=TA_CENTER,
        fontName='Helvetica-Bold'
    )
    
    # Subtitle style
    subtitle_style = ParagraphStyle(
        'CustomSubtitle',
        parent=styles['Normal'],
        fontSize=14,
        textColor=DARK_TEXT,
        spaceAfter=20,
        alignment=TA_CENTER,
        fontName='Helvetica'
    )
    
    # Section header style
    section_style = ParagraphStyle(
        'SectionHeader',
        parent=styles['Heading2'],
        fontSize=16,
        textColor=PRIMARY_COLOR,
        spaceAfter=8,
        spaceBefore=12,
        fontName='Helvetica-Bold'
    )
    
    # Key metric style
    metric_style = ParagraphStyle(
        'Metric',
        parent=styles['Normal'],
        fontSize=11,
        textColor=DARK_TEXT,
        spaceAfter=4,
        fontName='Helvetica'
    )
    
    # Value style (bold, larger)
    value_style = ParagraphStyle(
        'Value',
        parent=styles['Normal'],
        fontSize=18,
        textColor=PRIMARY_COLOR,
        spaceAfter=8,
        fontName='Helvetica-Bold'
    )
    
    # Body text style
    body_style = ParagraphStyle(
        'Body',
        parent=styles['Normal'],
        fontSize=10,
        textColor=DARK_TEXT,
        spaceAfter=6,
        leading=14,
        fontName='Helvetica'
    )
    
    # URL style
    url_style = ParagraphStyle(
        'URL',
        parent=styles['Normal'],
        fontSize=11,
        textColor=ACCENT_COLOR,
        spaceAfter=12,
        alignment=TA_CENTER,
        fontName='Helvetica-Bold'
    )
    
    # Title
    elements.append(Paragraph("HYPE Token DCF Valuation", title_style))
    elements.append(Paragraph("Investment-Grade Analysis & Price Projections", subtitle_style))
    elements.append(Spacer(1, 0.2*inch))
    
    # Key Metrics Section
    elements.append(Paragraph("Key Metrics", section_style))
    
    # Create metrics table
    metrics_data = [
        ['Current Price', '$25.05', 'Market Cap', '$8.93B'],
        ['2025 Revenue', '$845.6M', 'Daily Volume', '$3.57B'],
        ['Market Share', '70%', 'TVL', '$4.48B'],
        ['DAU', '41,280', 'Open Interest', '$8.44B'],
    ]
    
    metrics_table = Table(metrics_data, colWidths=[2*inch, 1.5*inch, 2*inch, 1.5*inch])
    metrics_table.setStyle(TableStyle([
        ('BACKGROUND', (0, 0), (-1, -1), LIGHT_BG),
        ('TEXTCOLOR', (0, 0), (-1, -1), DARK_TEXT),
        ('ALIGN', (0, 0), (-1, -1), 'LEFT'),
        ('ALIGN', (1, 0), (1, -1), 'RIGHT'),
        ('ALIGN', (3, 0), (3, -1), 'RIGHT'),
        ('FONTNAME', (0, 0), (-1, -1), 'Helvetica'),
        ('FONTNAME', (1, 0), (1, -1), 'Helvetica-Bold'),
        ('FONTNAME', (3, 0), (3, -1), 'Helvetica-Bold'),
        ('FONTSIZE', (0, 0), (-1, -1), 10),
        ('FONTSIZE', (1, 0), (1, -1), 12),
        ('FONTSIZE', (3, 0), (3, -1), 12),
        ('TEXTCOLOR', (1, 0), (1, -1), PRIMARY_COLOR),
        ('TEXTCOLOR', (3, 0), (3, -1), PRIMARY_COLOR),
        ('BOTTOMPADDING', (0, 0), (-1, -1), 8),
        ('TOPPADDING', (0, 0), (-1, -1), 8),
        ('GRID', (0, 0), (-1, -1), 0.5, colors.grey),
    ]))
    elements.append(metrics_table)
    elements.append(Spacer(1, 0.15*inch))
    
    # Valuation Summary
    elements.append(Paragraph("Valuation Summary", section_style))
    
    valuation_data = [
        ['Scenario', 'Enterprise Value', 'Price Target', '10-Year CAGR'],
        ['Bear', '$5-8B', '$15-25', '8-12%'],
        ['Base', '$12-18B', '$35-50', '15-25%'],
        ['Bull', '$25-40B', '$70-110', '25-35%'],
    ]
    
    valuation_table = Table(valuation_data, colWidths=[1.2*inch, 1.8*inch, 1.5*inch, 1.5*inch])
    valuation_table.setStyle(TableStyle([
        ('BACKGROUND', (0, 0), (-1, 0), PRIMARY_COLOR),
        ('TEXTCOLOR', (0, 0), (-1, 0), colors.white),
        ('BACKGROUND', (0, 1), (-1, -1), LIGHT_BG),
        ('TEXTCOLOR', (0, 1), (-1, -1), DARK_TEXT),
        ('ALIGN', (0, 0), (-1, -1), 'CENTER'),
        ('FONTNAME', (0, 0), (-1, 0), 'Helvetica-Bold'),
        ('FONTNAME', (0, 1), (-1, -1), 'Helvetica'),
        ('FONTSIZE', (0, 0), (-1, -1), 10),
        ('BOTTOMPADDING', (0, 0), (-1, -1), 10),
        ('TOPPADDING', (0, 0), (-1, -1), 10),
        ('GRID', (0, 0), (-1, -1), 1, colors.grey),
        ('ROWBACKGROUNDS', (0, 1), (-1, -1), [LIGHT_BG, colors.white]),
    ]))
    elements.append(valuation_table)
    elements.append(Spacer(1, 0.15*inch))
    
    # Timeline Projections
    elements.append(Paragraph("10-Year Price Trajectory (Base Case)", section_style))
    
    timeline_data = [
        ['Horizon', 'Price Target', 'Upside'],
        ['2026 (1 year)', '$30-40', '+20-60%'],
        ['2028 (3 years)', '$50-70', '+100-180%'],
        ['2030 (5 years)', '$80-120', '+220-380%'],
        ['2035 (10 years)', '$150-250', '+500-900%'],
    ]
    
    timeline_table = Table(timeline_data, colWidths=[2*inch, 2*inch, 2*inch])
    timeline_table.setStyle(TableStyle([
        ('BACKGROUND', (0, 0), (-1, 0), ACCENT_COLOR),
        ('TEXTCOLOR', (0, 0), (-1, 0), colors.white),
        ('BACKGROUND', (0, 1), (-1, -1), LIGHT_BG),
        ('TEXTCOLOR', (0, 1), (-1, -1), DARK_TEXT),
        ('ALIGN', (0, 0), (-1, -1), 'CENTER'),
        ('FONTNAME', (0, 0), (-1, 0), 'Helvetica-Bold'),
        ('FONTNAME', (0, 1), (-1, -1), 'Helvetica'),
        ('FONTSIZE', (0, 0), (-1, -1), 10),
        ('BOTTOMPADDING', (0, 0), (-1, -1), 8),
        ('TOPPADDING', (0, 0), (-1, -1), 8),
        ('GRID', (0, 0), (-1, -1), 0.5, colors.grey),
        ('TEXTCOLOR', (2, 1), (2, -1), PRIMARY_COLOR),
        ('FONTNAME', (2, 1), (2, -1), 'Helvetica-Bold'),
    ]))
    elements.append(timeline_table)
    elements.append(Spacer(1, 0.15*inch))
    
    # Key Findings
    elements.append(Paragraph("Key Findings", section_style))
    
    findings = [
        "• HYPE trades at 6.2x P/S vs 16x for dYdX, indicating potential upside",
        "• Protocol generates $845M+ annual revenue with 70% perpetual market share",
        "• Base case implies 15-25% CAGR over 10 years with significant price appreciation",
        "• Deflationary tokenomics (2% annual burns) and ecosystem premium (15%) support valuation",
        "• HyperEVM expansion and spot trading growth provide additional catalysts"
    ]
    
    for finding in findings:
        elements.append(Paragraph(finding, body_style))
    
    elements.append(Spacer(1, 0.15*inch))
    
    # Methodology Note
    elements.append(Paragraph("Methodology", section_style))
    methodology_text = """
    This analysis uses a Discounted Cash Flow (DCF) model valuing HYPE as equity with claim on full protocol value. 
    Revenue is projected with growth rates declining to terminal growth. Enterprise value includes ecosystem premium 
    (15%) and deflationary supply effects. Discount rates: 15-22% (crypto-appropriate risk premiums).
    """
    elements.append(Paragraph(methodology_text.strip(), body_style))
    elements.append(Spacer(1, 0.15*inch))
    
    # Call to Action
    elements.append(Spacer(1, 0.1*inch))
    cta_text = "For detailed analysis, interactive models, and full methodology:"
    elements.append(Paragraph(cta_text, body_style))
    elements.append(Spacer(1, 0.05*inch))
    
    # Try to get GitHub URL from git config
    github_url = "https://github.com/YOUR_USERNAME/qAegis/tree/main/docs/analysis"
    try:
        import subprocess
        result = subprocess.run(
            ['git', 'config', '--get', 'remote.origin.url'],
            capture_output=True,
            text=True,
            cwd=os.path.dirname(os.path.abspath(__file__))
        )
        if result.returncode == 0:
            remote_url = result.stdout.strip()
            # Convert SSH to HTTPS if needed
            if remote_url.startswith('git@'):
                remote_url = remote_url.replace('git@github.com:', 'https://github.com/')
                remote_url = remote_url.replace('.git', '')
            elif remote_url.startswith('https://'):
                remote_url = remote_url.replace('.git', '')
            github_url = f"{remote_url}/tree/main/docs/analysis"
    except:
        pass  # Use default if git not available
    
    elements.append(Paragraph(f"🔗 {github_url}", url_style))
    
    elements.append(Spacer(1, 0.1*inch))
    
    # Footer
    footer_text = f"Generated: {datetime.now().strftime('%B %Y')} | Not Financial Advice | DYOR"
    footer_style = ParagraphStyle(
        'Footer',
        parent=styles['Normal'],
        fontSize=8,
        textColor=colors.grey,
        alignment=TA_CENTER,
        fontName='Helvetica-Oblique'
    )
    elements.append(Paragraph(footer_text, footer_style))
    
    # Build PDF
    doc.build(elements)
    print(f"✓ Summary PDF created: {output_path}")

if __name__ == "__main__":
    # Update GitHub URL if needed
    create_summary_pdf('HYPE_DCF_Summary.pdf')
    print("\nTo customize GitHub URL, edit the script and update 'github_url' variable.")
