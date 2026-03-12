#!/usr/bin/env python3
"""
QuantumAegis Execution Brief: How We Ship
Direct, no-filler document combining vision + market + execution.
"""

from reportlab.lib.pagesizes import letter
from reportlab.lib.units import inch
from reportlab.lib import colors
from reportlab.lib.styles import getSampleStyleSheet, ParagraphStyle
from reportlab.platypus import (
    SimpleDocTemplate, Paragraph, Spacer, Table, TableStyle, HRFlowable
)
from reportlab.lib.enums import TA_CENTER, TA_LEFT, TA_JUSTIFY
from reportlab.lib.colors import HexColor
from datetime import datetime

PRIMARY = HexColor('#0A1628')
ACCENT = HexColor('#00D4AA')
SECONDARY = HexColor('#7B68EE')
DARK_TEXT = HexColor('#2c3e50')
LIGHT_BG = HexColor('#f8f9fa')


def create_execution_brief(output_path='QuantumAegis_Execution_Brief.pdf'):
    """Generate execution brief."""
    
    doc = SimpleDocTemplate(
        output_path,
        pagesize=letter,
        rightMargin=0.6*inch,
        leftMargin=0.6*inch,
        topMargin=0.5*inch,
        bottomMargin=0.4*inch
    )
    
    elements = []
    styles = getSampleStyleSheet()
    
    title_style = ParagraphStyle(
        'Title', parent=styles['Heading1'],
        fontSize=22, textColor=PRIMARY, spaceAfter=4,
        alignment=TA_CENTER, fontName='Helvetica-Bold'
    )
    
    section_style = ParagraphStyle(
        'Section', parent=styles['Heading2'],
        fontSize=14, textColor=PRIMARY, spaceAfter=6,
        spaceBefore=14, fontName='Helvetica-Bold'
    )
    
    body_style = ParagraphStyle(
        'Body', parent=styles['Normal'],
        fontSize=10, textColor=DARK_TEXT, spaceAfter=6,
        leading=13, alignment=TA_JUSTIFY, fontName='Helvetica'
    )
    
    bold_body = ParagraphStyle(
        'BoldBody', parent=body_style,
        fontName='Helvetica-Bold'
    )
    
    bullet_style = ParagraphStyle(
        'Bullet', parent=body_style,
        leftIndent=15, bulletIndent=8, spaceAfter=4, fontSize=9.5
    )
    
    # === HEADER ===
    elements.append(Paragraph("QuantumAegis: Execution Brief", title_style))
    elements.append(HRFlowable(width="100%", thickness=2, color=ACCENT, spaceAfter=12))
    
    # === THE PROBLEM ===
    elements.append(Paragraph("The Problem", section_style))
    problem = """
    Blockchain infrastructure secures $3T+ in assets using cryptography that quantum computers 
    will break. RSA, ECDSA, BLS—all vulnerable to Shor's algorithm. Timeline: 10-15 years to 
    cryptographically relevant quantum computers. The "harvest now, decrypt later" attack is 
    already active. Every on-chain transaction is a permanent liability.
    """
    elements.append(Paragraph(problem.strip(), body_style))
    
    adjacent = """
    Adjacent problem: AI systems make financial decisions without cryptographic accountability. 
    No way to verify a model produced a claimed output without exposing proprietary weights. 
    Trust model incompatible with decentralized infrastructure.
    """
    elements.append(Paragraph(adjacent.strip(), body_style))
    
    # === THE MARKET ===
    elements.append(Paragraph("The Market", section_style))
    
    market_data = [
        ['Segment', '2025', '2035', 'CAGR'],
        ['Post-Quantum Cryptography', '$0.42B', '$13.3B', '46%'],
        ['Blockchain Security', '$3.0B', '$23.4B', '38%'],
        ['Quantum Cybersecurity', '$2.0B', '$24.2B', '32%'],
        ['Verifiable AI (zkML)', '$0.1B', '$8.0B', '55%'],
        ['Combined TAM', '$5.5B', '$68.9B', '~35%'],
    ]
    
    market_table = Table(market_data, colWidths=[2.5*inch, 1.2*inch, 1.2*inch, 0.9*inch])
    market_table.setStyle(TableStyle([
        ('BACKGROUND', (0, 0), (-1, 0), ACCENT),
        ('TEXTCOLOR', (0, 0), (-1, 0), colors.white),
        ('BACKGROUND', (0, 1), (-1, 4), LIGHT_BG),
        ('BACKGROUND', (0, 5), (-1, 5), HexColor('#e8f5e9')),
        ('FONTNAME', (0, 0), (-1, 0), 'Helvetica-Bold'),
        ('FONTNAME', (0, 5), (-1, 5), 'Helvetica-Bold'),
        ('FONTSIZE', (0, 0), (-1, -1), 9),
        ('ALIGN', (1, 0), (-1, -1), 'CENTER'),
        ('BOTTOMPADDING', (0, 0), (-1, -1), 6),
        ('GRID', (0, 0), (-1, -1), 0.5, colors.grey),
    ]))
    elements.append(market_table)
    elements.append(Spacer(1, 0.1*inch))
    
    # === WHAT WE BUILD ===
    elements.append(Paragraph("What We Build", section_style))
    
    elements.append(Paragraph("<b>Layer 1: Quantum Coprocessor</b>", bold_body))
    l1_points = [
        "Off-chain quantum circuit simulation (Google Cirq, Willow 105Q model)",
        "Threat assessment engine: Grover (symmetric) + Shor (asymmetric) attack modeling",
        "Risk scoring across 12 vulnerability categories",
        "Proof generation: Poseidon2 hashes + ML-DSA-87 signatures",
    ]
    for p in l1_points:
        elements.append(Paragraph(f"• {p}", bullet_style))
    
    elements.append(Paragraph("<b>Layer 2: Post-Quantum Cryptography</b>", bold_body))
    l2_points = [
        "ML-DSA-87 (Dilithium-5): Lattice-based signatures, NIST Level 5",
        "SLH-DSA-256s (SPHINCS+): Hash-based signatures, stateless",
        "Hybrid ECDSA+PQC: Backward compatibility during transition",
        "Automated algorithm rotation triggered by threat thresholds",
    ]
    for p in l2_points:
        elements.append(Paragraph(f"• {p}", bullet_style))
    
    elements.append(Paragraph("<b>Layer 3: TEE Sequencer</b>", bold_body))
    l3_points = [
        "Aegis-TEE: Intel TDX/SGX, AMD SEV enclave execution",
        "Encrypted mempool: Transactions hidden until ordering committed",
        "Hardware attestation chain verified on-chain",
        "Phala Network redundancy for failover",
    ]
    for p in l3_points:
        elements.append(Paragraph(f"• {p}", bullet_style))
    
    elements.append(Paragraph("<b>Layer 4: On-Chain Verification</b>", bold_body))
    l4_points = [
        "OP Stack L2 rollup (Chain ID 16584, Sepolia L1)",
        "PQCVerifier contract: ~48,000 gas per verification",
        "QRMSOracle: On-chain threat indicator registry",
        "Result immutability + audit trail",
    ]
    for p in l4_points:
        elements.append(Paragraph(f"• {p}", bullet_style))
    
    # === HOW WE SHIP ===
    elements.append(Paragraph("How We Ship", section_style))
    
    ship_data = [
        ['Phase', 'Deliverable', 'Status'],
        ['Foundation', 'OP Stack L2, QRMS service, contracts, dashboard', 'Complete'],
        ['Cryptography', 'ML-DSA-87, SLH-DSA-256s, hybrid signatures', 'Complete'],
        ['TEE Integration', 'Aegis-TEE sequencer, Phala redundancy', 'Complete'],
        ['QVM Oracle', 'Cirq simulation, Grover/Shor assessment', 'Complete'],
        ['Threat Intel', '12 categories, risk scoring, feeds', 'Simulated'],
        ['Quantum Coprocessing', 'Off-chain compute, on-chain verify', 'In Progress'],
        ['zkML Integration', 'Verifiable AI inference proofs', 'Planned'],
        ['Production', 'Mainnet, audits, monitoring', 'Q1 2027'],
    ]
    
    ship_table = Table(ship_data, colWidths=[1.8*inch, 3.2*inch, 1.2*inch])
    ship_table.setStyle(TableStyle([
        ('BACKGROUND', (0, 0), (-1, 0), PRIMARY),
        ('TEXTCOLOR', (0, 0), (-1, 0), colors.white),
        ('BACKGROUND', (0, 1), (-1, 4), HexColor('#e8f5e9')),
        ('BACKGROUND', (0, 5), (-1, 5), HexColor('#fff9e6')),
        ('BACKGROUND', (0, 6), (-1, 7), HexColor('#e3f2fd')),
        ('BACKGROUND', (0, 8), (-1, 8), LIGHT_BG),
        ('FONTNAME', (0, 0), (-1, 0), 'Helvetica-Bold'),
        ('FONTSIZE', (0, 0), (-1, -1), 9),
        ('ALIGN', (2, 0), (2, -1), 'CENTER'),
        ('BOTTOMPADDING', (0, 0), (-1, -1), 5),
        ('GRID', (0, 0), (-1, -1), 0.5, colors.grey),
    ]))
    elements.append(ship_table)
    elements.append(Spacer(1, 0.1*inch))
    
    # === COMPETITIVE POSITION ===
    elements.append(Paragraph("Competitive Position", section_style))
    
    position = """
    No existing L2 integrates quantum threat assessment with adaptive PQC and TEE sequencing. 
    Competitors address fragments: QRL provides quantum-resistant L1 (no EVM compatibility), 
    Phala offers TEE compute (no PQC), zkSync/Optimism provide rollups (classical crypto only). 
    QuantumAegis is the composable security layer—works alongside OP Stack's 10-year ECDSA 
    deprecation roadmap, integrates with Lean Ethereum (leanVM, leanSig), provides direct-to-consumer 
    threat APIs.
    """
    elements.append(Paragraph(position.strip(), body_style))
    
    # === MOAT ===
    elements.append(Paragraph("Defensibility", section_style))
    
    moat_points = [
        "<b>Integration depth</b>: Four-layer stack creates switching costs. Each layer reinforces the others.",
        "<b>Data flywheel</b>: Threat assessments improve with usage. First-mover captures training data for ML risk prediction.",
        "<b>Standards alignment</b>: NIST PQC, OP Stack roadmap, EIP-7702—we build on what's shipping, not theoretical.",
        "<b>Time asymmetry</b>: Migration takes years; we're shipping now. Late entrants face compressed timelines.",
    ]
    for p in moat_points:
        elements.append(Paragraph(f"• {p}", bullet_style))
    
    # === WHAT'S NEEDED ===
    elements.append(Paragraph("Execution Requirements", section_style))
    
    needs = """
    <b>Engineering</b>: Rust (coprocessor), Solidity (contracts), Go (OP Stack mods). 
    <b>Infrastructure</b>: TEE-capable cloud (Azure Confidential, GCP Confidential VMs), 
    quantum simulation capacity. <b>Integrations</b>: Wallet providers, bridge operators, 
    DeFi protocols for adoption. <b>Compliance</b>: Security audit (Trail of Bits tier), 
    regulatory engagement for PQC migration guidance.
    """
    elements.append(Paragraph(needs.strip(), body_style))
    
    # === BOTTOM LINE ===
    elements.append(Paragraph("Bottom Line", section_style))
    
    bottom = """
    $69B TAM by 2035. Quantum breaks current crypto. Migration takes years. We have working 
    infrastructure—L2 operational, PQC integrated, TEE sequencer complete, QVM oracle functional. 
    Gap to close: production hardening, zkML integration, mainnet deployment. The window for 
    proactive migration is finite. Ship now or scramble later.
    """
    elements.append(Paragraph(bottom.strip(), body_style))
    
    elements.append(Spacer(1, 0.15*inch))
    elements.append(HRFlowable(width="100%", thickness=1, color=colors.grey))
    
    footer = f"QuantumAegis | {datetime.now().strftime('%B %Y')} | github.com/quantumaegis"
    footer_style = ParagraphStyle('Footer', fontSize=8, textColor=colors.grey, alignment=TA_CENTER)
    elements.append(Paragraph(footer, footer_style))
    
    doc.build(elements)
    print(f"Execution brief generated: {output_path}")
    return output_path


if __name__ == "__main__":
    import sys
    output = sys.argv[1] if len(sys.argv) > 1 else 'QuantumAegis_Execution_Brief.pdf'
    create_execution_brief(output)
