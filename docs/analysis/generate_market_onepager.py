#!/usr/bin/env python3
"""
One-Pager: Verifiable Quantum Security Market Opportunity
Concise, data-driven article on market landscape and forecasts.
"""

from reportlab.lib.pagesizes import letter
from reportlab.lib.units import inch
from reportlab.lib import colors
from reportlab.lib.styles import getSampleStyleSheet, ParagraphStyle
from reportlab.platypus import (
    SimpleDocTemplate, Paragraph, Spacer, Table, TableStyle, 
    Image, KeepTogether, HRFlowable
)
from reportlab.lib.enums import TA_CENTER, TA_LEFT, TA_JUSTIFY
from reportlab.lib.colors import HexColor
from datetime import datetime
import os
import numpy as np
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt

# Colors
PRIMARY = HexColor('#0A1628')
ACCENT = HexColor('#00D4AA')
SECONDARY = HexColor('#7B68EE')
WARNING = HexColor('#FF6B6B')
GOLD = HexColor('#FFD700')
DARK_TEXT = HexColor('#2c3e50')
LIGHT_BG = HexColor('#f8f9fa')

def rgb(h):
    return h.rgb() if hasattr(h, 'rgb') else (0.5, 0.5, 0.5)

def create_market_chart(output_path='temp_market.png'):
    """Create combined market projection chart."""
    fig, ax = plt.subplots(figsize=(10, 4))
    
    years = [2025, 2026, 2028, 2030, 2032, 2035]
    
    # Market data (in billions USD)
    pqc = [0.42, 0.7, 1.5, 2.84, 5.5, 13.3]
    blockchain_sec = [3.0, 4.5, 8.0, 15.0, 20.0, 23.4]
    quantum_cyber = [2.0, 2.8, 5.0, 8.5, 14.0, 24.2]
    
    x = np.arange(len(years))
    width = 0.25
    
    bars1 = ax.bar(x - width, pqc, width, label='Post-Quantum Cryptography', color=rgb(ACCENT), alpha=0.85, edgecolor='black')
    bars2 = ax.bar(x, blockchain_sec, width, label='Blockchain Security', color=rgb(SECONDARY), alpha=0.85, edgecolor='black')
    bars3 = ax.bar(x + width, quantum_cyber, width, label='Quantum Cybersecurity', color=rgb(GOLD), alpha=0.85, edgecolor='black')
    
    ax.set_xlabel('Year', fontsize=11, fontweight='bold')
    ax.set_ylabel('Market Size (USD Billions)', fontsize=11, fontweight='bold')
    ax.set_title('Security Infrastructure Markets: 2025-2035 Projections', fontsize=13, fontweight='bold', pad=10)
    ax.set_xticks(x)
    ax.set_xticklabels(years)
    ax.legend(loc='upper left', fontsize=9)
    ax.grid(True, alpha=0.3, axis='y', linestyle='--')
    ax.set_facecolor('#fafafa')
    ax.set_ylim(0, 30)
    
    # Add total annotation
    total_2035 = pqc[-1] + blockchain_sec[-1] + quantum_cyber[-1]
    ax.annotate(f'Total TAM 2035:\n${total_2035:.0f}B+', xy=(5, 26), fontsize=11, 
               fontweight='bold', ha='center', bbox=dict(boxstyle='round', facecolor='white', edgecolor='gray'))
    
    plt.tight_layout()
    plt.savefig(output_path, dpi=150, bbox_inches='tight', facecolor='white')
    plt.close()
    return output_path


def create_timeline_mini(output_path='temp_timeline_mini.png'):
    """Create compact threat timeline."""
    fig, ax = plt.subplots(figsize=(10, 2.5))
    
    # Timeline
    ax.axhline(y=0.5, color='gray', linewidth=3, alpha=0.3)
    
    milestones = [
        (2024, 'NIST PQC\nStandards', rgb(ACCENT)),
        (2026, 'Enterprise\nMigration', rgb(SECONDARY)),
        (2028, '1K Logical\nQubits', rgb(GOLD)),
        (2032, 'RSA/ECDSA\nVulnerable', rgb(WARNING)),
        (2035, 'CRQC\nDeployment', HexColor('#8B0000').rgb()),
    ]
    
    for year, label, color in milestones:
        ax.scatter(year, 0.5, s=200, color=color, zorder=5, edgecolor='black', linewidth=2)
        ax.annotate(label, (year, 0.5), textcoords="offset points", 
                   xytext=(0, 25), ha='center', fontsize=9, fontweight='bold')
        ax.annotate(str(year), (year, 0.5), textcoords="offset points", 
                   xytext=(0, -25), ha='center', fontsize=9)
    
    ax.set_xlim(2023, 2036)
    ax.set_ylim(0, 1)
    ax.axis('off')
    ax.set_title('Quantum Threat Timeline', fontsize=11, fontweight='bold', pad=5)
    
    plt.tight_layout()
    plt.savefig(output_path, dpi=150, bbox_inches='tight', facecolor='white')
    plt.close()
    return output_path


def create_onepager_pdf(output_path='Quantum_Security_Market_OnePager.pdf'):
    """Generate the one-pager PDF."""
    
    print("Generating charts...")
    market_chart = create_market_chart()
    timeline_chart = create_timeline_mini()
    
    doc = SimpleDocTemplate(
        output_path,
        pagesize=letter,
        rightMargin=0.5*inch,
        leftMargin=0.5*inch,
        topMargin=0.4*inch,
        bottomMargin=0.3*inch
    )
    
    elements = []
    styles = getSampleStyleSheet()
    
    # Styles - compact for one-pager
    title_style = ParagraphStyle(
        'Title', parent=styles['Heading1'],
        fontSize=20, textColor=PRIMARY, spaceAfter=2,
        alignment=TA_CENTER, fontName='Helvetica-Bold'
    )
    
    subtitle_style = ParagraphStyle(
        'Subtitle', parent=styles['Normal'],
        fontSize=10, textColor=DARK_TEXT, spaceAfter=8,
        alignment=TA_CENTER, fontName='Helvetica-Oblique'
    )
    
    section_style = ParagraphStyle(
        'Section', parent=styles['Heading2'],
        fontSize=12, textColor=PRIMARY, spaceAfter=4,
        spaceBefore=8, fontName='Helvetica-Bold'
    )
    
    body_style = ParagraphStyle(
        'Body', parent=styles['Normal'],
        fontSize=9, textColor=DARK_TEXT, spaceAfter=4,
        leading=11, alignment=TA_JUSTIFY, fontName='Helvetica'
    )
    
    bullet_style = ParagraphStyle(
        'Bullet', parent=body_style,
        leftIndent=12, bulletIndent=6, spaceAfter=2, fontSize=8.5
    )
    
    stat_style = ParagraphStyle(
        'Stat', parent=styles['Normal'],
        fontSize=9, textColor=ACCENT, fontName='Helvetica-Bold',
        alignment=TA_CENTER
    )
    
    # === HEADER ===
    elements.append(Paragraph("The $60B+ Verifiable Security Infrastructure Opportunity", title_style))
    elements.append(Paragraph(
        "Quantum-Resistant Cryptography, Verifiable AI, and On-Chain Security: 2025-2035 Market Landscape",
        subtitle_style
    ))
    elements.append(HRFlowable(width="100%", thickness=1, color=ACCENT, spaceAfter=6))
    
    # === THE SITUATION ===
    elements.append(Paragraph("The Convergent Threat", section_style))
    
    situation_text = """
    Three forces are reshaping cybersecurity infrastructure: <b>quantum computing</b> threatens all 
    public-key cryptography (RSA, ECDSA, BLS) within 10-15 years; <b>AI systems</b> make decisions 
    without cryptographic accountability; and <b>blockchain infrastructure</b> securing $3T+ in assets 
    relies on algorithms that quantum computers will break. The "harvest now, decrypt later" threat 
    is active—adversaries capture encrypted data today for future decryption. Every blockchain 
    transaction is permanently recorded; today's anonymity becomes tomorrow's exposure. NIST 
    standardized post-quantum algorithms (ML-DSA, ML-KEM, SLH-DSA) in August 2024, but adoption 
    across critical infrastructure remains below 5%.
    """
    elements.append(Paragraph(situation_text.strip(), body_style))
    
    # Timeline
    elements.append(Image(timeline_chart, width=7.2*inch, height=1.8*inch))
    
    # === WHAT MUST BE BUILT ===
    elements.append(Paragraph("Infrastructure Requirements: The Build List", section_style))
    
    # Two-column table for build requirements
    build_data = [
        ['On-Chain Infrastructure', 'Off-Chain Infrastructure'],
        [
            '• PQC signature verification precompiles\n'
            '• Quantum-safe ZK proof systems (STARKs)\n'
            '• Smart contract wallet migration (EIP-7702)\n'
            '• Cross-chain bridge PQC upgrades\n'
            '• Algorithm rotation automation',
            '• Quantum coprocessors for threat assessment\n'
            '• TEE attestation chains (TDX/SGX/SEV)\n'
            '• zkML proving infrastructure\n'
            '• Post-quantum TLS for all RPC endpoints\n'
            '• Hardware security module (HSM) upgrades'
        ],
        ['Cybersecurity Primitives', 'Enterprise Requirements'],
        [
            '• Hybrid classical+PQC signatures\n'
            '• Homomorphic encryption + PQC\n'
            '• Quantum random number generation\n'
            '• Decentralized key management\n'
            '• Encrypted mempool infrastructure',
            '• Compliance verification (zkML proofs)\n'
            '• Audit trail integrity (PQC-signed logs)\n'
            '• Identity systems migration\n'
            '• API gateway PQC termination\n'
            '• Legacy system bridging'
        ],
    ]
    
    build_table = Table(build_data, colWidths=[3.6*inch, 3.6*inch])
    build_table.setStyle(TableStyle([
        ('BACKGROUND', (0, 0), (-1, 0), ACCENT),
        ('BACKGROUND', (0, 2), (-1, 2), SECONDARY),
        ('TEXTCOLOR', (0, 0), (-1, 0), colors.white),
        ('TEXTCOLOR', (0, 2), (-1, 2), colors.white),
        ('BACKGROUND', (0, 1), (-1, 1), LIGHT_BG),
        ('BACKGROUND', (0, 3), (-1, 3), LIGHT_BG),
        ('TEXTCOLOR', (0, 1), (-1, 1), DARK_TEXT),
        ('TEXTCOLOR', (0, 3), (-1, 3), DARK_TEXT),
        ('ALIGN', (0, 0), (-1, -1), 'LEFT'),
        ('FONTNAME', (0, 0), (-1, 0), 'Helvetica-Bold'),
        ('FONTNAME', (0, 2), (-1, 2), 'Helvetica-Bold'),
        ('FONTSIZE', (0, 0), (-1, -1), 8),
        ('VALIGN', (0, 0), (-1, -1), 'TOP'),
        ('BOTTOMPADDING', (0, 0), (-1, -1), 6),
        ('TOPPADDING', (0, 0), (-1, -1), 6),
        ('GRID', (0, 0), (-1, -1), 0.5, colors.grey),
    ]))
    elements.append(build_table)
    elements.append(Spacer(1, 0.08*inch))
    
    # === MARKET PROJECTIONS ===
    elements.append(Paragraph("Market Size Projections: 2025-2035", section_style))
    elements.append(Image(market_chart, width=7.2*inch, height=2.8*inch))
    
    # Key stats table
    stats_data = [
        ['Market Segment', '2025', '2030', '2035', 'CAGR'],
        ['Post-Quantum Cryptography', '$0.42B', '$2.84B', '$13.3B', '46%'],
        ['Blockchain Security', '$3.0B', '$15.0B', '$23.4B', '38%'],
        ['Quantum Cybersecurity', '$2.0B', '$8.5B', '$24.2B', '32%'],
        ['zkML/Verifiable AI', '$0.1B', '$1.5B', '$8.0B', '55%+'],
        ['TOTAL ADDRESSABLE MARKET', '$5.5B', '$27.8B', '$68.9B', '~35%'],
    ]
    
    stats_table = Table(stats_data, colWidths=[2.2*inch, 1.0*inch, 1.0*inch, 1.0*inch, 0.8*inch])
    stats_table.setStyle(TableStyle([
        ('BACKGROUND', (0, 0), (-1, 0), PRIMARY),
        ('TEXTCOLOR', (0, 0), (-1, 0), colors.white),
        ('BACKGROUND', (0, 1), (-1, 4), LIGHT_BG),
        ('BACKGROUND', (0, 5), (-1, 5), HexColor('#e8f5e9')),
        ('TEXTCOLOR', (0, 1), (-1, -1), DARK_TEXT),
        ('ALIGN', (0, 0), (-1, -1), 'CENTER'),
        ('ALIGN', (0, 0), (0, -1), 'LEFT'),
        ('FONTNAME', (0, 0), (-1, 0), 'Helvetica-Bold'),
        ('FONTNAME', (0, 5), (-1, 5), 'Helvetica-Bold'),
        ('FONTSIZE', (0, 0), (-1, -1), 8),
        ('BOTTOMPADDING', (0, 0), (-1, -1), 5),
        ('TOPPADDING', (0, 0), (-1, -1), 5),
        ('GRID', (0, 0), (-1, -1), 0.5, colors.grey),
    ]))
    elements.append(stats_table)
    elements.append(Spacer(1, 0.08*inch))
    
    # === SECTOR DEPLOYMENT ===
    elements.append(Paragraph("Sector-Specific Deployment Opportunities", section_style))
    
    sector_text = """
    <b>Financial Services (40% of market)</b>: BFSI leads adoption due to regulatory pressure. 
    $2.36B lost to smart contract exploits in 2024 alone. PQC migration mandates emerging in EU/Singapore.
    <b>Government/Defense (25%)</b>: NSA memo mandates PQC transition by 2035. Classified systems 
    require quantum-safe encryption for "harvest now" threat mitigation.
    <b>Healthcare (15%)</b>: HIPAA compliance + long data retention = high HNDL exposure. 
    zkML enables AI diagnostics with verifiable compliance.
    <b>Critical Infrastructure (10%)</b>: Energy grids, telecom, transportation—SCADA/ICS systems 
    need retrofit paths. 20+ year equipment lifecycles complicate migration.
    <b>Enterprise IT (10%)</b>: API security, identity systems, cloud infrastructure. 
    Hybrid deployments bridge legacy systems during transition.
    """
    elements.append(Paragraph(sector_text.strip(), body_style))
    
    # === KEY INSIGHTS ===
    elements.append(Paragraph("Investment Implications", section_style))
    
    insights = [
        "<b>Timing asymmetry</b>: Migration requires 5-10 years; attacks can be instantaneous once CRQC arrives. Early movers capture integration revenue.",
        "<b>Infrastructure layer value</b>: PQC coprocessors, TEE attestation networks, and zkML proving markets are platform plays—not point solutions.",
        "<b>Regulatory tailwinds</b>: NIST standards (2024), EU Cyber Resilience Act, US Executive Order 14028 create compliance-driven demand.",
        "<b>Blockchain as forcing function</b>: $3T+ in on-chain assets with immutable transaction history creates existential migration pressure.",
        "<b>Verifiable AI as adjacent market</b>: zkML solves AI accountability—same cryptographic infrastructure, different application layer.",
    ]
    for insight in insights:
        elements.append(Paragraph(f"• {insight}", bullet_style))
    
    elements.append(Spacer(1, 0.06*inch))
    
    # === FOOTER ===
    elements.append(HRFlowable(width="100%", thickness=1, color=colors.grey, spaceBefore=4))
    
    footer_text = f"""
    <b>Sources</b>: MarketsandMarkets, Juniper Research, Grand View Research, McKinsey Quantum Communication Report, 
    Mordor Intelligence, Dimension Market Research. | <b>Generated</b>: {datetime.now().strftime('%B %Y')} | 
    <b>QuantumAegis Protocol</b>: github.com/quantumaegis
    """
    footer_style = ParagraphStyle(
        'Footer', parent=styles['Normal'],
        fontSize=7, textColor=colors.grey, alignment=TA_CENTER
    )
    elements.append(Paragraph(footer_text.strip(), footer_style))
    
    # Build PDF
    doc.build(elements)
    
    # Cleanup
    for f in [market_chart, timeline_chart]:
        try:
            os.remove(f)
        except:
            pass
    
    print(f"One-pager generated: {output_path}")
    return output_path


if __name__ == "__main__":
    import sys
    output = sys.argv[1] if len(sys.argv) > 1 else 'Quantum_Security_Market_OnePager.pdf'
    create_onepager_pdf(output)
