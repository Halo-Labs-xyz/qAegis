#!/usr/bin/env python3
"""
QuantumAegis Whitepaper: Verifiable Quantum Coprocessing for On-Chain Security
Generates a comprehensive PDF whitepaper on the opportunity for verifiable quantum 
coprocessing integrated with blockchain solutions.
"""

try:
    from reportlab.lib.pagesizes import letter
    from reportlab.lib.units import inch
    from reportlab.lib import colors
    from reportlab.lib.styles import getSampleStyleSheet, ParagraphStyle
    from reportlab.platypus import (
        SimpleDocTemplate, Paragraph, Spacer, Table, TableStyle, 
        Image, PageBreak, KeepTogether, ListFlowable, ListItem
    )
    from reportlab.lib.enums import TA_CENTER, TA_LEFT, TA_RIGHT, TA_JUSTIFY
    from reportlab.lib.colors import HexColor
    from reportlab.pdfgen import canvas
    from datetime import datetime
    import os
    import numpy as np
    import matplotlib
    matplotlib.use('Agg')
    import matplotlib.pyplot as plt
    from matplotlib.patches import FancyBboxPatch, Circle, Arrow
    import matplotlib.patches as mpatches
except ImportError as e:
    print(f"Missing library: {e}")
    print("Install: pip install reportlab matplotlib numpy")
    exit(1)

# Color palette - quantum/security themed
PRIMARY = HexColor('#0A1628')      # Deep navy
ACCENT = HexColor('#00D4AA')       # Quantum teal
SECONDARY = HexColor('#7B68EE')    # Purple
WARNING = HexColor('#FF6B6B')      # Alert red
GOLD = HexColor('#FFD700')         # Gold
TEXT = HexColor('#1a1a2e')
DARK_TEXT = HexColor('#2c3e50')
LIGHT_BG = HexColor('#f8f9fa')
QUANTUM_BLUE = HexColor('#3498db')
SECURE_GREEN = HexColor('#27ae60')

def rgb(hexcolor):
    """Convert HexColor to matplotlib RGB tuple."""
    if hasattr(hexcolor, 'rgb'):
        return hexcolor.rgb()
    return (0.5, 0.5, 0.5)

# Matplotlib colors
ACCENT_RGB = rgb(ACCENT)
SECONDARY_RGB = rgb(SECONDARY)
WARNING_RGB = rgb(WARNING)
GOLD_RGB = rgb(GOLD)
QUANTUM_BLUE_RGB = rgb(QUANTUM_BLUE)
SECURE_GREEN_RGB = rgb(SECURE_GREEN)


def create_quantum_threat_timeline(output_path='temp_threat_timeline.png'):
    """Create quantum threat timeline visualization."""
    fig, ax = plt.subplots(figsize=(12, 6))
    
    # Timeline data
    years = [2024, 2028, 2032, 2036, 2040, 2045]
    threat_levels = [10, 25, 50, 75, 90, 99]
    qubit_counts = [1000, 5000, 50000, 500000, 2000000, 10000000]
    
    # Plot threat curve
    ax.fill_between(years, threat_levels, alpha=0.3, color=WARNING_RGB)
    ax.plot(years, threat_levels, marker='o', linewidth=3, color=WARNING_RGB, markersize=10)
    
    # Add milestones
    milestones = [
        (2024, 15, "NIST PQC\nStandards"),
        (2028, 30, "1K Logical\nQubits"),
        (2032, 55, "RSA-2048\nVulnerable"),
        (2036, 80, "ECDSA\nBroken"),
        (2040, 92, "Harvest Now\nDecrypt Now"),
    ]
    
    for year, y, label in milestones:
        ax.annotate(label, (year, y), textcoords="offset points", 
                   xytext=(0, 20), ha='center', fontsize=9, fontweight='bold',
                   bbox=dict(boxstyle='round,pad=0.3', facecolor='white', edgecolor='gray'))
    
    ax.set_xlabel('Year', fontsize=12, fontweight='bold')
    ax.set_ylabel('Cryptographic Risk Level (%)', fontsize=12, fontweight='bold')
    ax.set_title('Quantum Threat Timeline: Classical Cryptography at Risk', fontsize=14, fontweight='bold', pad=15)
    ax.set_ylim(0, 105)
    ax.set_xlim(2023, 2046)
    ax.grid(True, alpha=0.3, linestyle='--')
    ax.set_facecolor('#fafafa')
    
    # Add qubit scale on right axis
    ax2 = ax.twinx()
    ax2.plot(years, [np.log10(q) for q in qubit_counts], '--', color=QUANTUM_BLUE_RGB, linewidth=2, alpha=0.7)
    ax2.set_ylabel('Log₁₀(Logical Qubits)', fontsize=10, color=QUANTUM_BLUE_RGB)
    ax2.tick_params(axis='y', labelcolor=QUANTUM_BLUE_RGB)
    
    plt.tight_layout()
    plt.savefig(output_path, dpi=150, bbox_inches='tight', facecolor='white')
    plt.close()
    return output_path


def create_protocol_stack(output_path='temp_protocol_stack.png'):
    """Create protocol stack visualization."""
    fig, ax = plt.subplots(figsize=(10, 8))
    ax.set_xlim(0, 10)
    ax.set_ylim(0, 10)
    ax.axis('off')
    
    # Stack layers
    layers = [
        (1, 0.5, 8, 1.5, QUANTUM_BLUE_RGB, 'Quantum Coprocessor Layer\n• Circuit Simulation • Threat Assessment • Proof Generation'),
        (1, 2.2, 8, 1.5, SECONDARY_RGB, 'Verifiable Computation Layer\n• zkML Proofs • Poseidon2 Hashing • TEE Attestation'),
        (1, 3.9, 8, 1.5, ACCENT_RGB, 'Post-Quantum Cryptography Layer\n• ML-DSA-87 • SLH-DSA-256s • Hybrid Signatures'),
        (1, 5.6, 8, 1.5, SECURE_GREEN_RGB, 'Trusted Execution Environment\n• Intel TDX/SGX • AMD SEV • Encrypted Mempool'),
        (1, 7.3, 8, 1.5, GOLD_RGB, 'On-Chain Settlement Layer\n• OP Stack L2 • Smart Contracts • Verifier Registry'),
    ]
    
    for x, y, w, h, color, text in layers:
        rect = FancyBboxPatch((x, y), w, h, boxstyle="round,pad=0.02,rounding_size=0.2",
                              facecolor=color, edgecolor='black', linewidth=2, alpha=0.85)
        ax.add_patch(rect)
        ax.text(x + w/2, y + h/2, text, ha='center', va='center', 
               fontsize=10, fontweight='bold', color='white')
    
    # Arrows between layers
    for i in range(4):
        y_start = layers[i][1] + layers[i][3]
        y_end = layers[i+1][1]
        ax.annotate('', xy=(5, y_end), xytext=(5, y_start),
                   arrowprops=dict(arrowstyle='->', color='gray', lw=2))
    
    ax.set_title('QuantumAegis Protocol Stack', fontsize=14, fontweight='bold', pad=20)
    
    plt.tight_layout()
    plt.savefig(output_path, dpi=150, bbox_inches='tight', facecolor='white')
    plt.close()
    return output_path


def create_20year_roadmap(output_path='temp_roadmap.png'):
    """Create 20-year cryptographic primitives roadmap."""
    fig, ax = plt.subplots(figsize=(14, 8))
    
    # Timeline
    years = list(range(2025, 2046))
    
    # Primitives with start/end years and importance
    primitives = [
        ('PQC Signatures (ML-DSA, SLH-DSA)', 2024, 2045, ACCENT_RGB, 0),
        ('PQC Key Exchange (ML-KEM, HQC)', 2025, 2045, ACCENT_RGB, 1),
        ('Hybrid Classical+PQC', 2024, 2035, QUANTUM_BLUE_RGB, 2),
        ('zkML Verifiable AI', 2025, 2045, SECONDARY_RGB, 3),
        ('TEE Attestation Chains', 2024, 2040, SECURE_GREEN_RGB, 4),
        ('Homomorphic Encryption', 2028, 2045, GOLD_RGB, 5),
        ('Quantum-Safe ZK Proofs', 2026, 2045, SECONDARY_RGB, 6),
        ('Post-Quantum TLS', 2025, 2035, QUANTUM_BLUE_RGB, 7),
        ('PQ Smart Contract Wallets', 2026, 2040, ACCENT_RGB, 8),
        ('Decentralized Quantum Oracles', 2030, 2045, SECONDARY_RGB, 9),
        ('Quantum Random Beacons', 2028, 2045, QUANTUM_BLUE_RGB, 10),
        ('FHE+PQC Computation', 2032, 2045, GOLD_RGB, 11),
        ('Quantum Key Distribution', 2035, 2045, WARNING_RGB, 12),
    ]
    
    for name, start, end, color, idx in primitives:
        ax.barh(idx, end - start, left=start, height=0.6, color=color, alpha=0.8, edgecolor='black')
        ax.text(start + 0.3, idx, name, va='center', ha='left', fontsize=9, fontweight='bold', color='white')
    
    # Era markers
    eras = [
        (2024, 2028, 'NISQ Era', '#ffebee'),
        (2028, 2035, 'Early Fault-Tolerant', '#e3f2fd'),
        (2035, 2045, 'Cryptographic Transition', '#e8f5e9'),
    ]
    
    for start, end, name, color in eras:
        ax.axvspan(start, end, alpha=0.15, color=color)
        ax.text((start + end) / 2, 13.2, name, ha='center', va='bottom', fontsize=10, 
               fontweight='bold', style='italic')
    
    ax.set_xlabel('Year', fontsize=12, fontweight='bold')
    ax.set_ylabel('Cryptographic Primitive', fontsize=12, fontweight='bold')
    ax.set_title('20-Year Roadmap: Cryptographic Primitives Evolution', fontsize=14, fontweight='bold', pad=15)
    ax.set_xlim(2024, 2046)
    ax.set_ylim(-0.5, 14)
    ax.set_yticks([])
    ax.grid(True, alpha=0.3, axis='x', linestyle='--')
    ax.set_facecolor('#fafafa')
    
    plt.tight_layout()
    plt.savefig(output_path, dpi=150, bbox_inches='tight', facecolor='white')
    plt.close()
    return output_path


def create_verifiable_ai_diagram(output_path='temp_verifiable_ai.png'):
    """Create verifiable AI computation diagram."""
    fig, ax = plt.subplots(figsize=(12, 6))
    ax.set_xlim(0, 12)
    ax.set_ylim(0, 6)
    ax.axis('off')
    
    # Components
    boxes = [
        (0.5, 2, 2.5, 2, QUANTUM_BLUE_RGB, 'AI Model\n(Private Weights)'),
        (4, 2, 2.5, 2, SECONDARY_RGB, 'zkML Proof\nGeneration'),
        (7.5, 2, 2.5, 2, ACCENT_RGB, 'On-Chain\nVerifier'),
        (10.5, 2, 1.2, 2, SECURE_GREEN_RGB, 'Result'),
    ]
    
    for x, y, w, h, color, text in boxes:
        rect = FancyBboxPatch((x, y), w, h, boxstyle="round,pad=0.02,rounding_size=0.15",
                              facecolor=color, edgecolor='black', linewidth=2, alpha=0.85)
        ax.add_patch(rect)
        ax.text(x + w/2, y + h/2, text, ha='center', va='center', 
               fontsize=10, fontweight='bold', color='white')
    
    # Arrows
    arrows = [(3, 3), (6.5, 3), (10, 3)]
    for x, y in arrows:
        ax.annotate('', xy=(x + 0.8, y), xytext=(x, y),
                   arrowprops=dict(arrowstyle='->', color='gray', lw=2))
    
    # Labels above arrows
    labels = [
        (3.5, 4.3, 'Input + Computation'),
        (7, 4.3, 'Proof Verification'),
        (10.5, 4.3, 'Verified Output'),
    ]
    for x, y, text in labels:
        ax.text(x, y, text, ha='center', va='bottom', fontsize=9, fontweight='bold')
    
    # Privacy guarantees below
    privacy = [
        (1.75, 1.3, 'Model weights\nremain private'),
        (5.25, 1.3, 'Computation\nproof generated'),
        (8.75, 1.3, 'Proof verified\non-chain'),
    ]
    for x, y, text in privacy:
        ax.text(x, y, text, ha='center', va='top', fontsize=8, style='italic', color='gray')
    
    ax.set_title('Verifiable AI: Zero-Knowledge ML Proof Flow', fontsize=14, fontweight='bold', pad=15)
    
    plt.tight_layout()
    plt.savefig(output_path, dpi=150, bbox_inches='tight', facecolor='white')
    plt.close()
    return output_path


def create_threat_categories(output_path='temp_threat_cats.png'):
    """Create threat category visualization."""
    fig, ax = plt.subplots(figsize=(10, 8))
    
    categories = [
        ('Digital Signatures', 95, WARNING_RGB),
        ('Key Exchange', 90, WARNING_RGB),
        ('ZK Proof Forgery', 75, GOLD_RGB),
        ('Hash Reversal', 60, GOLD_RGB),
        ('Consensus Attacks', 55, GOLD_RGB),
        ('Cross-Chain Bridges', 70, GOLD_RGB),
        ('Network Layer', 45, QUANTUM_BLUE_RGB),
        ('Key Management', 80, WARNING_RGB),
        ('MEV/Ordering', 40, QUANTUM_BLUE_RGB),
        ('Smart Contracts', 65, GOLD_RGB),
        ('Side-Channel', 50, QUANTUM_BLUE_RGB),
        ('Migration Agility', 85, WARNING_RGB),
    ]
    
    names = [c[0] for c in categories]
    risks = [c[1] for c in categories]
    colors = [c[2] for c in categories]
    
    y_pos = np.arange(len(names))
    bars = ax.barh(y_pos, risks, color=colors, alpha=0.8, edgecolor='black')
    
    ax.set_yticks(y_pos)
    ax.set_yticklabels(names, fontsize=10)
    ax.set_xlabel('Quantum Vulnerability Score (0-100)', fontsize=12, fontweight='bold')
    ax.set_title('Blockchain Threat Categories: Quantum Vulnerability Assessment', fontsize=14, fontweight='bold', pad=15)
    ax.set_xlim(0, 105)
    ax.grid(True, alpha=0.3, axis='x', linestyle='--')
    ax.set_facecolor('#fafafa')
    
    # Add risk labels
    for i, (bar, risk) in enumerate(zip(bars, risks)):
        ax.text(risk + 1, i, f'{risk}%', va='center', ha='left', fontsize=9, fontweight='bold')
    
    # Legend
    legend_elements = [
        mpatches.Patch(facecolor=WARNING_RGB, edgecolor='black', label='Critical (>75)'),
        mpatches.Patch(facecolor=GOLD_RGB, edgecolor='black', label='High (50-75)'),
        mpatches.Patch(facecolor=QUANTUM_BLUE_RGB, edgecolor='black', label='Medium (<50)'),
    ]
    ax.legend(handles=legend_elements, loc='lower right', fontsize=9)
    
    plt.tight_layout()
    plt.savefig(output_path, dpi=150, bbox_inches='tight', facecolor='white')
    plt.close()
    return output_path


def create_whitepaper_pdf(output_path='QuantumAegis_Whitepaper.pdf'):
    """Generate the complete whitepaper PDF."""
    
    print("Generating diagrams...")
    threat_timeline = create_quantum_threat_timeline()
    protocol_stack = create_protocol_stack()
    roadmap = create_20year_roadmap()
    verifiable_ai = create_verifiable_ai_diagram()
    threat_cats = create_threat_categories()
    
    doc = SimpleDocTemplate(
        output_path,
        pagesize=letter,
        rightMargin=0.75*inch,
        leftMargin=0.75*inch,
        topMargin=0.6*inch,
        bottomMargin=0.5*inch
    )
    
    elements = []
    styles = getSampleStyleSheet()
    
    # Styles
    title_style = ParagraphStyle(
        'Title', parent=styles['Heading1'],
        fontSize=26, textColor=PRIMARY, spaceAfter=8,
        alignment=TA_CENTER, fontName='Helvetica-Bold'
    )
    
    subtitle_style = ParagraphStyle(
        'Subtitle', parent=styles['Normal'],
        fontSize=14, textColor=DARK_TEXT, spaceAfter=25,
        alignment=TA_CENTER, fontName='Helvetica'
    )
    
    section_style = ParagraphStyle(
        'Section', parent=styles['Heading2'],
        fontSize=16, textColor=PRIMARY, spaceAfter=10,
        spaceBefore=18, fontName='Helvetica-Bold'
    )
    
    subsection_style = ParagraphStyle(
        'Subsection', parent=styles['Heading3'],
        fontSize=13, textColor=DARK_TEXT, spaceAfter=8,
        spaceBefore=12, fontName='Helvetica-Bold'
    )
    
    body_style = ParagraphStyle(
        'Body', parent=styles['Normal'],
        fontSize=10, textColor=DARK_TEXT, spaceAfter=8,
        leading=14, alignment=TA_JUSTIFY, fontName='Helvetica'
    )
    
    bullet_style = ParagraphStyle(
        'Bullet', parent=body_style,
        leftIndent=20, bulletIndent=10, spaceAfter=6
    )
    
    quote_style = ParagraphStyle(
        'Quote', parent=body_style,
        leftIndent=30, rightIndent=30,
        textColor=HexColor('#555555'),
        fontName='Helvetica-Oblique'
    )
    
    # === TITLE PAGE ===
    elements.append(Spacer(1, 1.5*inch))
    elements.append(Paragraph("Verifiable Quantum Coprocessing", title_style))
    elements.append(Paragraph("for On-Chain Security", title_style))
    elements.append(Spacer(1, 0.3*inch))
    elements.append(Paragraph(
        "A Technical Framework for Post-Quantum Cryptography, Verifiable AI,<br/>"
        "and Privacy-Preserving Computation in Blockchain Infrastructure",
        subtitle_style
    ))
    elements.append(Spacer(1, 0.5*inch))
    
    # Version info
    version_style = ParagraphStyle(
        'Version', parent=styles['Normal'],
        fontSize=10, textColor=colors.grey, alignment=TA_CENTER
    )
    elements.append(Paragraph(f"Version 1.0 | {datetime.now().strftime('%B %Y')}", version_style))
    elements.append(Paragraph("QuantumAegis Protocol", version_style))
    elements.append(PageBreak())
    
    # === ABSTRACT ===
    elements.append(Paragraph("Abstract", section_style))
    abstract = """
    Quantum computing poses an existential threat to blockchain cryptography. Current systems rely on 
    elliptic curve cryptography (ECDSA, BLS) and RSA, all vulnerable to Shor's algorithm. The threat 
    extends beyond key compromise: zero-knowledge proofs, consensus mechanisms, bridges, and the 
    entire security model face obsolescence. This paper presents a framework for verifiable quantum 
    coprocessing—off-chain quantum circuit simulation with on-chain proof verification—integrated with 
    post-quantum cryptography (PQC), trusted execution environments (TEE), and zero-knowledge machine 
    learning (zkML). The framework addresses not just quantum resistance, but the broader requirement 
    for verifiable computation in an era where AI decisions demand cryptographic accountability. We 
    outline a 20-year roadmap of cryptographic primitives essential for blockchain survival and 
    position quantum coprocessing as the foundational layer for this transition.
    """
    elements.append(Paragraph(abstract.strip(), body_style))
    elements.append(Spacer(1, 0.2*inch))
    
    # === SECTION 1: THE PROBLEM ===
    elements.append(Paragraph("1. The Quantum Threat to Blockchain", section_style))
    
    elements.append(Paragraph("1.1 Current Cryptographic Dependencies", subsection_style))
    crypto_text = """
    Modern blockchain infrastructure depends on three cryptographic assumptions: the hardness of 
    discrete logarithm problems (ECDSA/secp256k1), the difficulty of factoring large integers (RSA), 
    and collision resistance of hash functions (SHA-256, Keccak). Quantum computers, specifically 
    Shor's algorithm, solve discrete log and factoring problems in polynomial time. Grover's algorithm 
    provides quadratic speedup against hash functions, halving their effective security.
    """
    elements.append(Paragraph(crypto_text.strip(), body_style))
    
    elements.append(Paragraph("1.2 Attack Surface Analysis", subsection_style))
    elements.append(Image(threat_cats, width=6.2*inch, height=5*inch))
    elements.append(Spacer(1, 0.1*inch))
    
    attack_text = """
    The vulnerability assessment reveals 12 distinct threat categories. Critical vulnerabilities 
    (>75% risk) include digital signatures, key exchange, and key management—the foundation of 
    blockchain identity. High vulnerabilities (50-75%) span ZK proofs, smart contracts, bridges, 
    and consensus mechanisms. The "harvest now, decrypt later" (HNDL) strategy compounds risk: 
    adversaries can capture encrypted transactions today and decrypt them when quantum capability 
    arrives. Blockchain transactions are permanently recorded; past anonymity becomes future exposure.
    """
    elements.append(Paragraph(attack_text.strip(), body_style))
    
    elements.append(Paragraph("1.3 Timeline Projections", subsection_style))
    elements.append(Image(threat_timeline, width=6.5*inch, height=3.5*inch))
    elements.append(Spacer(1, 0.1*inch))
    
    timeline_text = """
    Conservative estimates place cryptographically relevant quantum computers (CRQC) at 2035-2040. 
    The transition period presents asymmetric risk: migration requires years while attacks can be 
    immediate. NIST standardized post-quantum algorithms (ML-KEM, ML-DSA, SLH-DSA) in August 2024, 
    but adoption across blockchain ecosystems remains nascent. The window for proactive migration 
    is narrowing.
    """
    elements.append(Paragraph(timeline_text.strip(), body_style))
    elements.append(PageBreak())
    
    # === SECTION 2: QUANTUM RESISTANCE IS PREREQUISITE ===
    elements.append(Paragraph("2. Quantum Resistance: Necessary but Insufficient", section_style))
    
    elements.append(Paragraph("2.1 The PQC Foundation", subsection_style))
    pqc_text = """
    Post-quantum cryptography provides the security foundation, but migration alone is insufficient. 
    NIST's standardized algorithms—ML-DSA (Dilithium) for signatures, ML-KEM (Kyber) for key 
    encapsulation, and SLH-DSA (SPHINCS+) for stateless hash-based signatures—address the immediate 
    cryptographic vulnerability. The algorithms are based on lattice problems and hash functions 
    believed resistant to both classical and quantum attacks.
    """
    elements.append(Paragraph(pqc_text.strip(), body_style))
    
    # PQC comparison table
    pqc_data = [
        ['Algorithm', 'Type', 'Public Key', 'Signature', 'Security Level'],
        ['ML-DSA-87', 'Lattice', '2,592 bytes', '4,595 bytes', 'NIST Level 5'],
        ['SLH-DSA-256s', 'Hash-based', '64 bytes', '29,792 bytes', 'NIST Level 5'],
        ['ECDSA (current)', 'Elliptic Curve', '33 bytes', '72 bytes', 'Broken by Shor'],
    ]
    
    pqc_table = Table(pqc_data, colWidths=[1.4*inch, 1.1*inch, 1.1*inch, 1.1*inch, 1.3*inch])
    pqc_table.setStyle(TableStyle([
        ('BACKGROUND', (0, 0), (-1, 0), ACCENT),
        ('TEXTCOLOR', (0, 0), (-1, 0), colors.white),
        ('BACKGROUND', (0, 1), (-1, 2), LIGHT_BG),
        ('BACKGROUND', (0, 3), (-1, 3), HexColor('#ffebee')),
        ('TEXTCOLOR', (0, 1), (-1, -1), DARK_TEXT),
        ('ALIGN', (0, 0), (-1, -1), 'CENTER'),
        ('FONTNAME', (0, 0), (-1, 0), 'Helvetica-Bold'),
        ('FONTSIZE', (0, 0), (-1, -1), 9),
        ('BOTTOMPADDING', (0, 0), (-1, -1), 8),
        ('GRID', (0, 0), (-1, -1), 0.5, colors.grey),
    ]))
    elements.append(pqc_table)
    elements.append(Spacer(1, 0.15*inch))
    
    elements.append(Paragraph("2.2 Beyond Algorithm Replacement", subsection_style))
    beyond_text = """
    PQC solves cryptographic vulnerability at the primitive level. The challenge extends to:
    """
    elements.append(Paragraph(beyond_text.strip(), body_style))
    
    beyond_points = [
        "<b>Signature aggregation</b>: BLS signatures enable efficient multi-party signing; lattice-based schemes lack equivalent aggregation, impacting validator economics.",
        "<b>Gas costs</b>: PQC signatures are 50-100x larger than ECDSA, increasing transaction costs and reducing throughput.",
        "<b>Key management</b>: Existing wallet infrastructure, hardware security modules, and custody solutions require redesign.",
        "<b>Protocol upgrades</b>: Smart contracts with hardcoded cryptographic assumptions need migration paths.",
        "<b>Cross-chain bridges</b>: Bridge security relies on multi-signature schemes vulnerable to key extraction.",
    ]
    for point in beyond_points:
        elements.append(Paragraph(f"• {point}", bullet_style))
    
    elements.append(Paragraph("2.3 The Verifiability Requirement", subsection_style))
    verify_text = """
    Quantum-safe cryptography secures data; verifiable computation ensures correctness. As 
    systems incorporate complex off-chain computation—AI inference, quantum simulations, 
    optimization algorithms—cryptographic proofs of execution become essential. Zero-knowledge 
    proofs currently rely on elliptic curve pairings (Groth16, PLONK) or FRI-based systems 
    (STARKs); the former requires quantum-safe replacement, the latter needs performance 
    optimization for practical deployment.
    """
    elements.append(Paragraph(verify_text.strip(), body_style))
    elements.append(PageBreak())
    
    # === SECTION 3: VERIFIABLE AI ===
    elements.append(Paragraph("3. The Verifiable AI Imperative", section_style))
    
    elements.append(Paragraph("3.1 AI in Financial Infrastructure", subsection_style))
    ai_text = """
    AI systems increasingly determine financial outcomes: credit scoring, fraud detection, 
    trading algorithms, risk assessment. These decisions lack cryptographic accountability. 
    A model's output cannot be independently verified without access to weights and architecture—
    information providers refuse to disclose. The result is a trust model incompatible with 
    decentralized systems.
    """
    elements.append(Paragraph(ai_text.strip(), body_style))
    
    elements.append(Paragraph("3.2 Zero-Knowledge Machine Learning (zkML)", subsection_style))
    elements.append(Image(verifiable_ai, width=6.5*inch, height=3.2*inch))
    elements.append(Spacer(1, 0.1*inch))
    
    zkml_text = """
    zkML enables cryptographic proofs that a model produced a specific output without revealing 
    model weights or intermediate computations. The prover generates a ZK proof alongside inference; 
    verifiers confirm correctness without knowledge of the model. Current implementations face 
    computational overhead—150x for simple models, impractical for large language models. Hardware 
    acceleration and algorithmic improvements are reducing this gap.
    """
    elements.append(Paragraph(zkml_text.strip(), body_style))
    
    elements.append(Paragraph("3.3 Applications and Requirements", subsection_style))
    app_points = [
        "<b>Compliance verification</b>: Prove AI doesn't use prohibited attributes (race, gender) without revealing decision logic.",
        "<b>ML-as-a-Service accountability</b>: Cloud providers prove model execution integrity.",
        "<b>Autonomous agents</b>: On-chain verification of AI agent decisions.",
        "<b>Oracle integrity</b>: Data feeds include proofs of computation.",
        "<b>Reputation systems</b>: Verifiable inference history without data exposure.",
    ]
    for point in app_points:
        elements.append(Paragraph(f"• {point}", bullet_style))
    
    elements.append(Paragraph("3.4 Privacy Requirements", subsection_style))
    privacy_text = """
    Verifiable AI creates a privacy paradox: verification requires transparency, but competitive 
    advantage demands secrecy. The solution combines multiple primitives: zkML protects model 
    weights, homomorphic encryption protects input data, TEEs provide hardware isolation, and 
    PQC ensures long-term security of all components. No single primitive suffices; the complete 
    stack requires integration.
    """
    elements.append(Paragraph(privacy_text.strip(), body_style))
    elements.append(PageBreak())
    
    # === SECTION 4: QUANTUM COPROCESSING ===
    elements.append(Paragraph("4. Verifiable Quantum Coprocessing", section_style))
    
    elements.append(Paragraph("4.1 Architecture Overview", subsection_style))
    elements.append(Image(protocol_stack, width=6*inch, height=4.8*inch))
    elements.append(Spacer(1, 0.1*inch))
    
    arch_text = """
    Quantum coprocessing separates heavy computation from lightweight verification. Off-chain 
    coprocessors execute quantum circuit simulations, threat assessments, and cryptographic 
    analysis. Results include cryptographic proofs verified on-chain with minimal gas expenditure. 
    The architecture mirrors rollup design: compute off-chain, prove on-chain.
    """
    elements.append(Paragraph(arch_text.strip(), body_style))
    
    elements.append(Paragraph("4.2 Coprocessor Functions", subsection_style))
    coproc_text = """
    The quantum coprocessor provides four core capabilities:
    """
    elements.append(Paragraph(coproc_text.strip(), body_style))
    
    func_data = [
        ['Function', 'Input', 'Output', 'Proof Type'],
        ['Circuit Simulation', 'Quantum circuit spec', 'State vector, measurements', 'Poseidon2 + PQC sig'],
        ['Threat Assessment', 'Algorithm, key size', 'Risk score, qubit estimate', 'ZK + TEE attestation'],
        ['Key Generation', 'Entropy source', 'PQC keypairs', 'TEE attestation'],
        ['Proof Generation', 'Computation result', 'ZK proof', 'Recursive proof'],
    ]
    
    func_table = Table(func_data, colWidths=[1.3*inch, 1.5*inch, 1.6*inch, 1.5*inch])
    func_table.setStyle(TableStyle([
        ('BACKGROUND', (0, 0), (-1, 0), SECONDARY),
        ('TEXTCOLOR', (0, 0), (-1, 0), colors.white),
        ('BACKGROUND', (0, 1), (-1, -1), LIGHT_BG),
        ('TEXTCOLOR', (0, 1), (-1, -1), DARK_TEXT),
        ('ALIGN', (0, 0), (-1, -1), 'CENTER'),
        ('FONTNAME', (0, 0), (-1, 0), 'Helvetica-Bold'),
        ('FONTSIZE', (0, 0), (-1, -1), 9),
        ('BOTTOMPADDING', (0, 0), (-1, -1), 8),
        ('GRID', (0, 0), (-1, -1), 0.5, colors.grey),
    ]))
    elements.append(func_table)
    elements.append(Spacer(1, 0.15*inch))
    
    elements.append(Paragraph("4.3 Proof System Design", subsection_style))
    proof_text = """
    Verification combines multiple proof mechanisms for defense-in-depth:
    """
    elements.append(Paragraph(proof_text.strip(), body_style))
    
    proof_points = [
        "<b>Poseidon2 hashing</b>: ZK-friendly hash function for efficient circuit verification. ~5,000 gas on-chain.",
        "<b>PQC signatures</b>: ML-DSA-87 signs coprocessor outputs. ~15,000 gas verification.",
        "<b>TEE attestation</b>: Hardware attestation from TDX/SGX/SEV enclaves proves execution environment integrity.",
        "<b>ZK proofs</b>: STARK-based proofs for complex computations; recursive composition for scalability.",
    ]
    for point in proof_points:
        elements.append(Paragraph(f"• {point}", bullet_style))
    
    elements.append(Paragraph("4.4 On-Chain Verification", subsection_style))
    onchain_text = """
    The on-chain verifier contract validates coprocessor results with minimal state:
    """
    elements.append(Paragraph(onchain_text.strip(), body_style))
    
    verify_data = [
        ['Operation', 'Gas Cost', 'Security Guarantee'],
        ['Poseidon2 verification', '~5,000', 'Hash integrity'],
        ['PQC signature check', '~15,000', 'Authenticity + non-repudiation'],
        ['TEE attestation check', '~8,000', 'Execution environment'],
        ['Result storage', '~20,000', 'Immutable record'],
        ['Total per verification', '~48,000', 'Multi-layer security'],
    ]
    
    verify_table = Table(verify_data, colWidths=[2*inch, 1.3*inch, 2.5*inch])
    verify_table.setStyle(TableStyle([
        ('BACKGROUND', (0, 0), (-1, 0), ACCENT),
        ('TEXTCOLOR', (0, 0), (-1, 0), colors.white),
        ('BACKGROUND', (0, 1), (-1, -2), LIGHT_BG),
        ('BACKGROUND', (0, -1), (-1, -1), HexColor('#e8f5e9')),
        ('TEXTCOLOR', (0, 1), (-1, -1), DARK_TEXT),
        ('ALIGN', (0, 0), (-1, -1), 'CENTER'),
        ('FONTNAME', (0, 0), (-1, 0), 'Helvetica-Bold'),
        ('FONTSIZE', (0, 0), (-1, -1), 9),
        ('BOTTOMPADDING', (0, 0), (-1, -1), 8),
        ('GRID', (0, 0), (-1, -1), 0.5, colors.grey),
    ]))
    elements.append(verify_table)
    elements.append(PageBreak())
    
    # === SECTION 5: TEE INTEGRATION ===
    elements.append(Paragraph("5. Trusted Execution Environment Layer", section_style))
    
    elements.append(Paragraph("5.1 Hardware Security Foundation", subsection_style))
    tee_text = """
    Trusted Execution Environments provide hardware-enforced isolation for sensitive computation. 
    Three technologies dominate: Intel SGX (process-level enclaves), Intel TDX (VM-level trust 
    domains), and AMD SEV (encrypted virtual machines). TEEs protect against privileged software 
    attacks, including malicious hypervisors and compromised operating systems.
    """
    elements.append(Paragraph(tee_text.strip(), body_style))
    
    tee_data = [
        ['Technology', 'Isolation Level', 'Memory Limit', 'Use Case'],
        ['Intel SGX', 'Process enclave', '256MB EPC', 'Key management, signing'],
        ['Intel TDX', 'Virtual machine', 'Multi-GB', 'Full coprocessor execution'],
        ['AMD SEV', 'Virtual machine', 'Multi-GB', 'Alternative cloud deployment'],
    ]
    
    tee_table = Table(tee_data, colWidths=[1.3*inch, 1.4*inch, 1.3*inch, 2*inch])
    tee_table.setStyle(TableStyle([
        ('BACKGROUND', (0, 0), (-1, 0), SECURE_GREEN),
        ('TEXTCOLOR', (0, 0), (-1, 0), colors.white),
        ('BACKGROUND', (0, 1), (-1, -1), LIGHT_BG),
        ('TEXTCOLOR', (0, 1), (-1, -1), DARK_TEXT),
        ('ALIGN', (0, 0), (-1, -1), 'CENTER'),
        ('FONTNAME', (0, 0), (-1, 0), 'Helvetica-Bold'),
        ('FONTSIZE', (0, 0), (-1, -1), 9),
        ('BOTTOMPADDING', (0, 0), (-1, -1), 8),
        ('GRID', (0, 0), (-1, -1), 0.5, colors.grey),
    ]))
    elements.append(tee_table)
    elements.append(Spacer(1, 0.15*inch))
    
    elements.append(Paragraph("5.2 Attestation Chains", subsection_style))
    attest_text = """
    Remote attestation verifies TEE integrity before trusting computation. The attestation chain 
    links hardware root-of-trust to on-chain verification: CPU generates attestation quote signed 
    by manufacturer key; quote includes measurement of enclave code; on-chain contract validates 
    quote signature and code hash. Compromise requires hardware-level attack.
    """
    elements.append(Paragraph(attest_text.strip(), body_style))
    
    elements.append(Paragraph("5.3 Encrypted Mempool", subsection_style))
    mempool_text = """
    TEE-protected sequencers enable encrypted mempools: transactions remain encrypted until 
    ordering is committed, eliminating MEV extraction through transaction reordering. The 
    sequencer receives encrypted transactions, orders them within the TEE, and reveals ordering 
    only after commitment. Front-running and sandwich attacks become computationally infeasible.
    """
    elements.append(Paragraph(mempool_text.strip(), body_style))
    
    elements.append(Paragraph("5.4 Redundancy and Failover", subsection_style))
    redundancy_text = """
    Single-TEE architectures create availability risk. Multi-TEE redundancy—Phala Network 
    provides decentralized SGX workers—enables failover without security degradation. State 
    migration between TEEs uses PQC-signed checkpoints, ensuring continuity across hardware 
    failures or maintenance windows.
    """
    elements.append(Paragraph(redundancy_text.strip(), body_style))
    elements.append(PageBreak())
    
    # === SECTION 6: 20-YEAR ROADMAP ===
    elements.append(Paragraph("6. Twenty-Year Cryptographic Primitives Roadmap", section_style))
    
    elements.append(Image(roadmap, width=7*inch, height=4*inch))
    elements.append(Spacer(1, 0.15*inch))
    
    elements.append(Paragraph("6.1 NISQ Era (2024-2028)", subsection_style))
    nisq_text = """
    The Noisy Intermediate-Scale Quantum era prioritizes migration preparation:
    """
    elements.append(Paragraph(nisq_text.strip(), body_style))
    
    nisq_points = [
        "<b>PQC signature deployment</b>: ML-DSA and SLH-DSA integration across wallet infrastructure.",
        "<b>Hybrid schemes</b>: Classical + PQC dual signatures provide defense-in-depth during transition.",
        "<b>Post-quantum TLS</b>: Network layer upgrade for encrypted communication channels.",
        "<b>TEE attestation standardization</b>: Cross-platform attestation verification contracts.",
        "<b>zkML foundations</b>: Proof-of-concept verifiable inference for simple models.",
    ]
    for point in nisq_points:
        elements.append(Paragraph(f"• {point}", bullet_style))
    
    elements.append(Paragraph("6.2 Early Fault-Tolerant Era (2028-2035)", subsection_style))
    ft_text = """
    Logical qubit demonstrations accelerate migration urgency:
    """
    elements.append(Paragraph(ft_text.strip(), body_style))
    
    ft_points = [
        "<b>Full PQC migration</b>: ECDSA deprecation begins; hybrid schemes become mandatory.",
        "<b>Quantum-safe ZK proofs</b>: STARK-based systems replace elliptic curve constructions.",
        "<b>Decentralized quantum oracles</b>: Network of quantum coprocessors for threat assessment.",
        "<b>Quantum random beacons</b>: Verifiable randomness from quantum sources.",
        "<b>PQ smart contract wallets</b>: Account abstraction with native PQC support.",
    ]
    for point in ft_points:
        elements.append(Paragraph(f"• {point}", bullet_style))
    
    elements.append(Paragraph("6.3 Cryptographic Transition Era (2035-2045)", subsection_style))
    trans_text = """
    CRQC deployment forces completion of transition:
    """
    elements.append(Paragraph(trans_text.strip(), body_style))
    
    trans_points = [
        "<b>Homomorphic encryption + PQC</b>: Compute on encrypted data with quantum-safe keys.",
        "<b>Quantum key distribution</b>: Hardware QKD integration for high-security channels.",
        "<b>Algorithm rotation automation</b>: Smart contracts trigger key migration based on threat indicators.",
        "<b>Full verifiable AI</b>: zkML for large language models becomes practical.",
        "<b>Post-classical consensus</b>: Consensus mechanisms designed for quantum network assumptions.",
    ]
    for point in trans_points:
        elements.append(Paragraph(f"• {point}", bullet_style))
    elements.append(PageBreak())
    
    # === SECTION 7: IMPLEMENTATION ===
    elements.append(Paragraph("7. QuantumAegis Implementation", section_style))
    
    elements.append(Paragraph("7.1 Current Deployment", subsection_style))
    impl_text = """
    QuantumAegis implements the described architecture as an OP Stack L2 rollup with integrated 
    quantum coprocessing:
    """
    elements.append(Paragraph(impl_text.strip(), body_style))
    
    impl_data = [
        ['Component', 'Status', 'Technology'],
        ['OP Stack L2', 'Operational', 'Chain ID 16584, Sepolia L1'],
        ['QVM Oracle', 'Complete', 'Google Cirq, Willow 105Q simulation'],
        ['PQC Signatures', 'Integrated', 'ML-DSA-87, SLH-DSA-256s (pqcrypto)'],
        ['Hybrid Signatures', 'Integrated', 'ECDSA + PQC dual signing'],
        ['TEE Sequencer', 'Complete', 'TDX/SEV/SGX, Phala redundancy'],
        ['Threat Monitoring', 'Operational', '12 categories, risk scoring'],
        ['On-chain Verifier', 'Deployed', 'PQCVerifier, QRMSOracle contracts'],
    ]
    
    impl_table = Table(impl_data, colWidths=[1.6*inch, 1.2*inch, 3*inch])
    impl_table.setStyle(TableStyle([
        ('BACKGROUND', (0, 0), (-1, 0), PRIMARY),
        ('TEXTCOLOR', (0, 0), (-1, 0), colors.white),
        ('BACKGROUND', (0, 1), (-1, -1), LIGHT_BG),
        ('TEXTCOLOR', (0, 1), (-1, -1), DARK_TEXT),
        ('ALIGN', (0, 0), (-1, -1), 'LEFT'),
        ('FONTNAME', (0, 0), (-1, 0), 'Helvetica-Bold'),
        ('FONTSIZE', (0, 0), (-1, -1), 9),
        ('BOTTOMPADDING', (0, 0), (-1, -1), 8),
        ('GRID', (0, 0), (-1, -1), 0.5, colors.grey),
    ]))
    elements.append(impl_table)
    elements.append(Spacer(1, 0.15*inch))
    
    elements.append(Paragraph("7.2 Threat Assessment Flow", subsection_style))
    flow_text = """
    1. Request submitted via REST API or on-chain transaction<br/>
    2. QVM oracle simulates quantum attack (Grover for symmetric, Shor for asymmetric)<br/>
    3. Threat assessment generated with qubit estimate and time-to-break<br/>
    4. Result signed with ML-DSA-87 + Poseidon2 hash<br/>
    5. On-chain verifier validates proof and stores result<br/>
    6. APQC layer triggers algorithm rotation if risk threshold exceeded
    """
    elements.append(Paragraph(flow_text.strip(), body_style))
    
    elements.append(Paragraph("7.3 Integration Points", subsection_style))
    integration_points = [
        "<b>Lean Ethereum</b>: leanVM execution, leanSig PQC signatures, leanMultisig aggregation.",
        "<b>EIP-7702</b>: Smart account migration with PQC key generation.",
        "<b>OP Stack</b>: Alignment with 10-year ECDSA deprecation roadmap.",
        "<b>Poseidon2</b>: ZK-friendly hashing for efficient on-chain verification.",
    ]
    for point in integration_points:
        elements.append(Paragraph(f"• {point}", bullet_style))
    elements.append(PageBreak())
    
    # === SECTION 8: CONCLUSION ===
    elements.append(Paragraph("8. Conclusion", section_style))
    
    conclusion_text = """
    Blockchain infrastructure faces convergent challenges: quantum computing threatens 
    cryptographic foundations, AI systems require verifiable computation, and users demand 
    privacy without sacrificing accountability. No single technology addresses all requirements. 
    The solution lies in composable security layers: post-quantum cryptography for long-term 
    security, zero-knowledge proofs for verifiable computation, trusted execution environments 
    for hardware isolation, and quantum coprocessing for proactive threat assessment.
    """
    elements.append(Paragraph(conclusion_text.strip(), body_style))
    elements.append(Spacer(1, 0.1*inch))
    
    conclusion2_text = """
    Quantum resistance is the prerequisite, not the destination. The next 20 years require 
    systematic evolution of cryptographic primitives—from hybrid signatures today to fully 
    homomorphic computation tomorrow. Projects that treat PQC migration as a checkbox exercise 
    will find themselves perpetually unprepared. The framework presented here—verifiable quantum 
    coprocessing with on-chain verification—provides the foundation for continuous adaptation 
    to emerging threats and capabilities.
    """
    elements.append(Paragraph(conclusion2_text.strip(), body_style))
    elements.append(Spacer(1, 0.1*inch))
    
    conclusion3_text = """
    The window for proactive transition is finite. Harvest-now-decrypt-later attacks are already 
    occurring. Every transaction recorded today becomes a future liability if cryptographic 
    migration lags quantum capability. The cost of early adoption is overhead; the cost of delayed 
    adoption is catastrophic failure. Verifiable quantum coprocessing enables continuous monitoring 
    and automated response, transforming reactive migration into proactive defense.
    """
    elements.append(Paragraph(conclusion3_text.strip(), body_style))
    elements.append(Spacer(1, 0.3*inch))
    
    # === REFERENCES ===
    elements.append(Paragraph("References", section_style))
    
    refs = [
        "[1] NIST. Post-Quantum Cryptography Standards. FIPS 203, 204, 205. August 2024.",
        "[2] Shor, P. Algorithms for Quantum Computation. FOCS 1994.",
        "[3] Grover, L. A Fast Quantum Mechanical Algorithm for Database Search. STOC 1996.",
        "[4] Google Quantum AI. Willow Processor Specifications. 2024.",
        "[5] OP Labs. OP Stack Specification. https://specs.optimism.io/",
        "[6] Polyhedra Network. zkML: Verifiable AI. 2025.",
        "[7] Intel. Trust Domain Extensions (TDX) Specification. 2024.",
        "[8] QRL Foundation. Post-Quantum Blockchain Security. 2025.",
        "[9] Ethereum Foundation. EIP-7702: Set EOA Code. 2024.",
        "[10] Grassi et al. Poseidon2: A Faster Version of the Poseidon Hash Function. IACR ePrint 2023/323.",
    ]
    
    ref_style = ParagraphStyle(
        'Reference', parent=body_style,
        fontSize=9, spaceAfter=4, leftIndent=20, firstLineIndent=-20
    )
    for ref in refs:
        elements.append(Paragraph(ref, ref_style))
    
    elements.append(Spacer(1, 0.3*inch))
    
    # Footer
    footer_text = f"QuantumAegis Protocol | {datetime.now().strftime('%B %Y')} | https://github.com/quantumaegis"
    footer_style = ParagraphStyle(
        'Footer', parent=styles['Normal'],
        fontSize=8, textColor=colors.grey, alignment=TA_CENTER
    )
    elements.append(Paragraph(footer_text, footer_style))
    
    # Build PDF
    doc.build(elements)
    
    # Cleanup temp files
    for f in [threat_timeline, protocol_stack, roadmap, verifiable_ai, threat_cats]:
        try:
            os.remove(f)
        except:
            pass
    
    print(f"Whitepaper generated: {output_path}")
    return output_path


if __name__ == "__main__":
    import sys
    output = sys.argv[1] if len(sys.argv) > 1 else 'QuantumAegis_Whitepaper.pdf'
    create_whitepaper_pdf(output)
