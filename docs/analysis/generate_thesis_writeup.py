#!/usr/bin/env python3
"""
Thesis Write-Up: Verifiable Quantum Security Infrastructure
Theory + practical applications for cofounder context.
"""

from reportlab.lib.pagesizes import letter
from reportlab.lib.units import inch
from reportlab.lib import colors
from reportlab.lib.styles import getSampleStyleSheet, ParagraphStyle
from reportlab.platypus import (
    SimpleDocTemplate, Paragraph, Spacer, Table, TableStyle, 
    PageBreak, HRFlowable
)
from reportlab.lib.enums import TA_CENTER, TA_LEFT, TA_JUSTIFY
from reportlab.lib.colors import HexColor
from datetime import datetime

PRIMARY = HexColor('#0A1628')
ACCENT = HexColor('#00D4AA')
SECONDARY = HexColor('#7B68EE')
DARK_TEXT = HexColor('#2c3e50')
LIGHT_BG = HexColor('#f8f9fa')


def create_thesis_writeup(output_path='Verifiable_Quantum_Security_Thesis.pdf'):
    """Generate detailed thesis write-up."""
    
    doc = SimpleDocTemplate(
        output_path,
        pagesize=letter,
        rightMargin=0.7*inch,
        leftMargin=0.7*inch,
        topMargin=0.6*inch,
        bottomMargin=0.5*inch
    )
    
    elements = []
    styles = getSampleStyleSheet()
    
    title_style = ParagraphStyle(
        'Title', parent=styles['Heading1'],
        fontSize=20, textColor=PRIMARY, spaceAfter=6,
        alignment=TA_CENTER, fontName='Helvetica-Bold'
    )
    
    section_style = ParagraphStyle(
        'Section', parent=styles['Heading2'],
        fontSize=13, textColor=PRIMARY, spaceAfter=8,
        spaceBefore=16, fontName='Helvetica-Bold'
    )
    
    body_style = ParagraphStyle(
        'Body', parent=styles['Normal'],
        fontSize=10, textColor=DARK_TEXT, spaceAfter=10,
        leading=14, alignment=TA_JUSTIFY, fontName='Helvetica'
    )
    
    # === TITLE ===
    elements.append(Paragraph("Verifiable Quantum Security Infrastructure", title_style))
    elements.append(Paragraph("A Thesis on Convergent Cryptographic Threats and the Infrastructure Required to Address Them", 
                             ParagraphStyle('Sub', fontSize=11, textColor=DARK_TEXT, alignment=TA_CENTER, spaceAfter=12)))
    elements.append(HRFlowable(width="100%", thickness=2, color=ACCENT, spaceAfter=16))
    
    # === SECTION 1: THE CRYPTOGRAPHIC FOUNDATION ===
    elements.append(Paragraph("1. The Cryptographic Foundation of Digital Infrastructure", section_style))
    
    s1p1 = """
    All digital security rests on mathematical assumptions. RSA assumes factoring large integers is hard. 
    Elliptic curve cryptography (ECDSA, BLS, EdDSA) assumes the discrete logarithm problem on elliptic 
    curves is hard. Hash functions (SHA-256, Keccak) assume finding collisions or preimages is hard. 
    These assumptions have held for decades against classical computers. They do not hold against 
    quantum computers.<super>[1]</super>
    """
    elements.append(Paragraph(s1p1.strip(), body_style))
    
    s1p2 = """
    Shor's algorithm, published in 1994, solves integer factorization and discrete logarithm in 
    polynomial time on a quantum computer.<super>[2]</super> This breaks RSA, ECDSA, BLS, Diffie-Hellman, and every 
    public-key cryptosystem currently deployed at scale. Grover's algorithm provides quadratic speedup 
    for search problems, halving the effective security of symmetric encryption and hash functions.<super>[3]</super> 
    AES-256 becomes AES-128 equivalent; SHA-256 becomes SHA-128 equivalent.
    """
    elements.append(Paragraph(s1p2.strip(), body_style))
    
    s1p3 = """
    The threat is not theoretical. IBM, Google, and others are building quantum computers with 
    increasing qubit counts and decreasing error rates. Google's Willow processor (105 qubits, 
    0.34% two-qubit error rate) demonstrated below-threshold error correction in 2024.<super>[4]</super> The trajectory 
    points to cryptographically relevant quantum computers (CRQC)—machines capable of breaking 
    RSA-2048 and ECDSA-256—within 10-15 years.<super>[5]</super> Conservative estimates place this at 2035-2040; 
    aggressive estimates at 2030-2032.
    """
    elements.append(Paragraph(s1p3.strip(), body_style))
    
    # === SECTION 2: THE BLOCKCHAIN VULNERABILITY ===
    elements.append(Paragraph("2. Blockchain's Unique Exposure", section_style))
    
    s2p1 = """
    Blockchain infrastructure has properties that amplify quantum risk beyond traditional systems.<super>[6]</super> 
    First, immutability: every transaction ever recorded remains on-chain forever. Bitcoin's genesis 
    block is still readable. This means "harvest now, decrypt later" (HNDL) attacks have permanent 
    targets.<super>[7]</super> An adversary can capture encrypted or signed transactions today, store them, and decrypt 
    or forge them once quantum capability arrives. The attack surface is the entire history of the chain.
    """
    elements.append(Paragraph(s2p1.strip(), body_style))
    
    s2p2 = """
    Second, key reuse: blockchain addresses derived from public keys are reused across transactions. 
    Once a public key is exposed (which happens on the first spend from an address), it becomes a 
    permanent target. Approximately $6B in Bitcoin sits in addresses with exposed public keys—funds 
    that will be immediately vulnerable when CRQC arrives.<super>[8]</super>
    """
    elements.append(Paragraph(s2p2.strip(), body_style))
    
    s2p3 = """
    Third, consensus dependencies: proof-of-stake systems rely on BLS signatures for validator 
    aggregation. A quantum adversary could forge validator signatures, manipulate consensus, and 
    rewrite chain history. The security model of Ethereum, Solana, and most modern L1s assumes 
    BLS signature unforgeability—an assumption that fails under Shor's algorithm.
    """
    elements.append(Paragraph(s2p3.strip(), body_style))
    
    s2p4 = """
    Fourth, smart contract rigidity: contracts deployed with hardcoded cryptographic assumptions 
    cannot be easily upgraded. A contract that verifies ECDSA signatures will continue to verify 
    ECDSA signatures even after ECDSA is broken. The $200B+ locked in DeFi protocols depends on 
    cryptographic assumptions embedded in immutable code.<super>[9]</super>
    """
    elements.append(Paragraph(s2p4.strip(), body_style))
    
    # === SECTION 3: THE AI ACCOUNTABILITY GAP ===
    elements.append(Paragraph("3. The Parallel Problem: AI Without Accountability", section_style))
    
    s3p1 = """
    A separate but convergent problem exists in AI systems. Machine learning models increasingly 
    make consequential decisions: credit scoring, fraud detection, medical diagnosis, trading 
    algorithms. These decisions lack cryptographic accountability. When a model outputs a prediction, 
    there is no way to verify that the claimed model produced the claimed output without access to 
    the model weights—information that providers refuse to disclose for competitive reasons.
    """
    elements.append(Paragraph(s3p1.strip(), body_style))
    
    s3p2 = """
    This creates a trust model incompatible with decentralized systems. Blockchain's core value 
    proposition is "don't trust, verify." AI's current model is "trust the provider." As AI agents 
    interact with on-chain systems—executing trades, managing treasuries, operating DAOs—this 
    incompatibility becomes untenable. An AI agent claiming to follow governance rules has no way 
    to prove it actually does.
    """
    elements.append(Paragraph(s3p2.strip(), body_style))
    
    s3p3 = """
    Zero-knowledge machine learning (zkML) solves this by generating cryptographic proofs of inference.<super>[10]</super> 
    A model produces output and a proof; the proof demonstrates correct execution without revealing 
    weights. The verifier confirms the proof on-chain. The model owner retains IP protection; the 
    user gains cryptographic certainty. This transforms AI from a black box into a verifiable 
    computation. Recent breakthroughs (Lagrange DeepProve, Polyhedra zkML) achieve 1000x speedups 
    in proof generation, making LLM verification practical.<super>[11]</super>
    """
    elements.append(Paragraph(s3p3.strip(), body_style))
    
    # === SECTION 4: THE INFRASTRUCTURE THESIS ===
    elements.append(Paragraph("4. The Infrastructure Thesis", section_style))
    
    s4p1 = """
    Quantum resistance is necessary but insufficient. Replacing ECDSA with ML-DSA addresses the 
    signature vulnerability but leaves the system fragmented. Different protocols will adopt 
    different algorithms at different times. Cross-chain bridges will face compatibility nightmares. 
    Wallets will struggle with key management across multiple cryptographic schemes. The migration 
    itself introduces new attack surfaces.
    """
    elements.append(Paragraph(s4p1.strip(), body_style))
    
    s4p2 = """
    The thesis is that infrastructure-layer solutions—not application-layer patches—capture the 
    value of this transition. The required infrastructure has four components:
    """
    elements.append(Paragraph(s4p2.strip(), body_style))
    
    s4p3 = """
    <b>Component 1: Threat Assessment Oracle.</b> Quantum capability is not binary; it evolves 
    continuously. A system that monitors quantum progress and translates it into actionable risk 
    scores enables automated responses. When risk crosses thresholds, key rotation triggers. When 
    new attacks emerge, mitigation deploys. The oracle simulates quantum attacks against current 
    cryptography and reports vulnerability levels. This is a data product—continuous, proprietary, 
    defensible through accumulated assessment history.
    """
    elements.append(Paragraph(s4p3.strip(), body_style))
    
    s4p4 = """
    <b>Component 2: Adaptive Cryptography Layer.</b> Rather than one-time migration, continuous 
    adaptation. Hybrid signatures (classical + post-quantum) provide defense-in-depth: if either 
    scheme is broken, the other remains secure.<super>[12]</super> Algorithm rotation based on threat levels ensures 
    the system responds to evolving attacks. Key management abstracts cryptographic complexity from 
    applications—they request "sign this," the layer handles algorithm selection. NIST standardized 
    ML-DSA (Dilithium), ML-KEM (Kyber), and SLH-DSA (SPHINCS+) in August 2024.<super>[13]</super>
    """
    elements.append(Paragraph(s4p4.strip(), body_style))
    
    s4p5 = """
    <b>Component 3: Verifiable Computation Environment.</b> Heavy computation—quantum simulations, 
    ML inference, complex proofs—cannot run on-chain economically. Off-chain execution with on-chain 
    verification is the pattern. Trusted execution environments (TEEs) provide hardware isolation; 
    the CPU itself enforces that code runs correctly and data remains private.<super>[14]</super> Intel TDX enables 
    VM-level isolation with multi-GB memory; Intel SGX provides process-level enclaves.<super>[15]</super> TEE attestation creates 
    a cryptographic proof of execution environment integrity. Combined with PQC signatures on outputs, 
    this creates end-to-end verifiable computation.
    """
    elements.append(Paragraph(s4p5.strip(), body_style))
    
    s4p6 = """
    <b>Component 4: On-Chain Settlement and Verification.</b> Results from off-chain computation 
    settle on-chain with minimal footprint. A rollup architecture—compute off-chain, prove on-chain—
    aligns with existing scaling solutions. The verification contracts are lightweight: check a 
    hash, verify a signature, store a result. The heavy lifting happens elsewhere; the chain provides 
    finality and immutability.
    """
    elements.append(Paragraph(s4p6.strip(), body_style))
    
    # === SECTION 5: THE MARKET ===
    elements.append(Paragraph("5. Market Quantification", section_style))
    
    s5p1 = """
    The addressable market spans multiple segments that historically operated independently but 
    converge under this infrastructure thesis. Market projections from MarketsandMarkets, Juniper 
    Research, and McKinsey indicate compound growth rates of 32-55% across segments:<super>[16][17][18]</super>
    """
    elements.append(Paragraph(s5p1.strip(), body_style))
    
    # THE ONE TABLE
    market_data = [
        ['Market Segment', '2025 Size', '2035 Projection', 'CAGR', 'Key Drivers'],
        ['Post-Quantum Cryptography', '$0.42B', '$13.3B', '46%', 'NIST standards, regulatory mandates, HNDL threat'],
        ['Blockchain Security', '$3.0B', '$23.4B', '38%', 'DeFi growth, smart contract exploits ($2.4B in 2024)'],
        ['Quantum Cybersecurity', '$2.0B', '$24.2B', '32%', 'Enterprise migration, government contracts'],
        ['Verifiable AI (zkML)', '$0.1B', '$8.0B', '55%+', 'AI accountability, compliance verification'],
        ['Combined Infrastructure TAM', '$5.5B', '$68.9B', '~35%', 'Convergent adoption across segments'],
    ]
    
    market_table = Table(market_data, colWidths=[1.7*inch, 0.8*inch, 0.9*inch, 0.6*inch, 2.4*inch])
    market_table.setStyle(TableStyle([
        ('BACKGROUND', (0, 0), (-1, 0), PRIMARY),
        ('TEXTCOLOR', (0, 0), (-1, 0), colors.white),
        ('BACKGROUND', (0, 1), (-1, 4), LIGHT_BG),
        ('BACKGROUND', (0, 5), (-1, 5), HexColor('#e8f5e9')),
        ('FONTNAME', (0, 0), (-1, 0), 'Helvetica-Bold'),
        ('FONTNAME', (0, 5), (-1, 5), 'Helvetica-Bold'),
        ('FONTSIZE', (0, 0), (-1, -1), 8),
        ('ALIGN', (1, 0), (3, -1), 'CENTER'),
        ('VALIGN', (0, 0), (-1, -1), 'MIDDLE'),
        ('BOTTOMPADDING', (0, 0), (-1, -1), 6),
        ('TOPPADDING', (0, 0), (-1, -1), 6),
        ('GRID', (0, 0), (-1, -1), 0.5, colors.grey),
    ]))
    elements.append(market_table)
    elements.append(Spacer(1, 0.12*inch))
    
    s5p2 = """
    The key insight is convergence. A financial institution needs PQC for regulatory compliance, 
    blockchain security for DeFi exposure, quantum cybersecurity for enterprise infrastructure, 
    and verifiable AI for model governance. These are not separate purchases—they're one infrastructure 
    decision. The vendor that provides integrated solutions across segments captures outsized value 
    versus point-solution competitors.
    """
    elements.append(Paragraph(s5p2.strip(), body_style))
    
    # === SECTION 6: EMERGING INFRASTRUCTURE PATTERNS ===
    elements.append(Paragraph("6. Emerging Infrastructure Patterns", section_style))
    
    s6_intro = """
    Three infrastructure patterns are converging with the quantum security thesis. Each represents 
    a distinct middleware opportunity; together they define the next-generation stack.
    """
    elements.append(Paragraph(s6_intro.strip(), body_style))
    
    s6_qml = """
    <b>Pattern 1: Quantum Machine Learning (QML).</b> Quantum computers excel at specific ML tasks: 
    kernel methods, optimization, sampling from complex distributions.<super>[19]</super> Variational quantum 
    eigensolvers (VQE) and quantum approximate optimization algorithms (QAOA) provide speedups 
    for portfolio optimization, drug discovery, and logistics.<super>[20]</super> The infrastructure requirement: 
    hybrid classical-quantum pipelines where classical preprocessing feeds quantum circuits, 
    quantum results return for classical post-processing. Current QML runs on NISQ devices with 
    high noise; error mitigation and result verification become critical. A QML coprocessor needs 
    the same proof infrastructure as threat assessment—execute off-chain, verify on-chain. The 
    convergence: zkML proofs for classical inference, quantum result attestation for QML inference, 
    same verification contracts, same trust model.
    """
    elements.append(Paragraph(s6_qml.strip(), body_style))
    
    s6_shared = """
    <b>Pattern 2: Shared Sequencing.</b> L2 rollups currently run isolated sequencers—single points 
    of failure, MEV extraction vectors, censorship risks.<super>[21]</super> Shared sequencing decouples ordering from 
    execution: a shared sequencer layer orders transactions across multiple rollups; individual 
    rollups execute against the ordered batch.<super>[22]</super> Benefits: atomic cross-rollup transactions, reduced 
    MEV (ordering commits before content reveals), censorship resistance through sequencer rotation. 
    The quantum angle: shared sequencers are high-value targets. Compromise one sequencer, manipulate 
    ordering across all connected rollups. PQC signing of sequence commitments, TEE-protected 
    sequencer execution, and quantum-resistant BFT consensus become requirements. The infrastructure 
    that secures a single rollup sequencer extends naturally to shared sequencing—same TEE attestation, 
    same PQC signatures, same threat monitoring, larger attack surface to protect.
    """
    elements.append(Paragraph(s6_shared.strip(), body_style))
    
    s6_copro = """
    <b>Pattern 3: Coprocessor Middleware Unbundling.</b> On-chain computation is expensive; off-chain 
    computation lacks trust. Coprocessors solve this: specialized off-chain compute with on-chain 
    verification.<super>[23]</super> The market is unbundling into discrete middleware layers: (a) proving markets 
    (Succinct, RISC Zero) generate proofs for arbitrary computation,<super>[24]</super> (b) proof aggregation networks 
    (zkVerify, Nebra) batch and verify proofs cheaply,<super>[25]</super> (c) data availability layers (EigenDA, Celestia) 
    store inputs/outputs,<super>[26]</super> (d) attestation networks verify execution environments. Each layer is 
    independently composable. A quantum coprocessor slots into this stack: it's another proof-generating 
    middleware. But it has unique properties—threat assessment requires continuous monitoring, not 
    one-shot computation; results feed back into system configuration (algorithm rotation); the data 
    is proprietary and accumulates value. The play is not "yet another coprocessor" but "the coprocessor 
    that other coprocessors need for quantum-safe operation."
    """
    elements.append(Paragraph(s6_copro.strip(), body_style))
    
    s6_integrate = """
    <b>Integration thesis:</b> These patterns compound. Shared sequencing needs quantum-resistant 
    ordering proofs. QML inference needs verifiable results that settle on-chain. Coprocessor 
    middlewares need PQC signatures on all outputs. A unified infrastructure layer—threat oracle + 
    adaptive crypto + TEE + verification contracts—serves all three patterns. Build once, deploy 
    across QML pipelines, shared sequencers, and coprocessor networks. The market fragments if 
    each pattern builds its own security stack; it consolidates if one infrastructure layer becomes 
    the standard.
    """
    elements.append(Paragraph(s6_integrate.strip(), body_style))
    
    # === SECTION 7: PRACTICAL APPLICATIONS ===
    elements.append(Paragraph("7. Practical Applications", section_style))
    
    s6p1 = """
    <b>Application 1: Quantum-Resistant Wallet Infrastructure.</b> Existing wallets use ECDSA 
    (secp256k1) for signing. Migration requires: (a) new key generation using PQC algorithms, 
    (b) signature verification contracts that accept PQC signatures, (c) migration paths that 
    don't require users to move funds to new addresses (see EIP-7702 for EOA code-setting). 
    The infrastructure layer provides key generation APIs, verification precompiles, and migration 
    tooling. Wallet developers integrate; users get quantum resistance transparently.
    """
    elements.append(Paragraph(s6p1.strip(), body_style))
    
    s6p2 = """
    <b>Application 2: Cross-Chain Bridge Security.</b> Bridges are the highest-value targets in 
    crypto—$2B+ stolen in 2022-2023. Most bridges use multi-signature schemes with ECDSA keys. 
    Quantum attacks on bridge signers could drain all locked funds. PQC bridge infrastructure 
    replaces vulnerable multi-sigs with quantum-resistant threshold signatures. TEE attestation 
    ensures signer nodes run correct code. Threat oracles monitor for quantum progress that could 
    compromise specific bridges.
    """
    elements.append(Paragraph(s6p2.strip(), body_style))
    
    s6p3 = """
    <b>Application 3: Verifiable AI Agents.</b> AI agents managing on-chain assets need cryptographic 
    constraints. A trading agent should prove it followed risk parameters before executing trades. 
    A DAO management agent should prove it consulted governance rules before proposing actions. 
    zkML infrastructure enables these proofs: the agent's inference generates a ZK proof; the proof 
    verifies on-chain before the transaction executes. Non-compliant inference cannot settle.
    """
    elements.append(Paragraph(s6p3.strip(), body_style))
    
    s6p4 = """
    <b>Application 4: Encrypted Mempool and MEV Protection.</b> Maximal extractable value (MEV) 
    costs users $500M+ annually through front-running and sandwich attacks. TEE-protected sequencers 
    enable encrypted mempools: transactions remain encrypted until ordering is committed. The 
    sequencer cannot see transaction contents; neither can MEV bots. This requires TEE infrastructure 
    with on-chain attestation verification—exactly the component described above.
    """
    elements.append(Paragraph(s6p4.strip(), body_style))
    
    s6p5 = """
    <b>Application 5: Compliance-Ready DeFi.</b> Institutional adoption of DeFi requires compliance 
    verification. Can a protocol prove it doesn't interact with sanctioned addresses without 
    revealing all user data? Can a lending protocol prove its risk models meet regulatory standards 
    without exposing proprietary algorithms? zkML + PQC infrastructure enables these proofs. 
    Compliance becomes cryptographically verifiable rather than trust-based.
    """
    elements.append(Paragraph(s6p5.strip(), body_style))
    
    # === SECTION 8: TIMING ===
    elements.append(Paragraph("8. Timing and Urgency", section_style))
    
    s7p1 = """
    The window for proactive migration is approximately 2025-2030. After 2030, timelines become 
    uncertain enough that reactive scrambling dominates. The asymmetry is severe: building 
    infrastructure takes years, but once CRQC arrives, attacks are immediate. A protocol that 
    starts PQC migration in 2032 when CRQC lands in 2035 has three years to rebuild its entire 
    cryptographic foundation—likely insufficient.
    """
    elements.append(Paragraph(s7p1.strip(), body_style))
    
    s7p2 = """
    NIST standardized ML-KEM, ML-DSA, and SLH-DSA in August 2024. The algorithms exist. The 
    libraries exist (pqcrypto, liboqs). What doesn't exist is integrated infrastructure that makes 
    adoption seamless. The opportunity is building that infrastructure layer before demand becomes 
    desperate. Early movers capture integration relationships; late movers compete on price for 
    commodity implementations.
    """
    elements.append(Paragraph(s7p2.strip(), body_style))
    
    # === SECTION 9: EXECUTION PATH ===
    elements.append(Paragraph("9. Execution Path", section_style))
    
    s8p1 = """
    Phase 1 establishes credibility: working L2 with real PQC signatures, functional threat 
    assessment, deployed contracts. This exists—QuantumAegis has operational testnet infrastructure 
    with ML-DSA-87 and SLH-DSA-256s signatures, a QVM oracle simulating Grover and Shor attacks, 
    and TEE sequencer architecture. The foundation is built.
    """
    elements.append(Paragraph(s8p1.strip(), body_style))
    
    s8p2 = """
    Phase 2 extends reach: API products for wallet developers, bridge operators, DeFi protocols. 
    The infrastructure becomes a service. Threat assessment as SaaS. PQC signature verification 
    as precompile. TEE attestation as middleware. Each integration creates switching costs; each 
    customer's data improves the threat model.
    """
    elements.append(Paragraph(s8p2.strip(), body_style))
    
    s8p3 = """
    Phase 3 captures convergence: zkML integration enables verifiable AI applications. The same 
    infrastructure that provides PQC and TEE provides proof verification. A protocol using 
    QuantumAegis for quantum resistance naturally uses it for AI verification—one vendor, one 
    integration, multiple capabilities.
    """
    elements.append(Paragraph(s8p3.strip(), body_style))
    
    s8p4 = """
    Phase 4 achieves defensibility: accumulated threat data enables ML-based risk prediction. 
    Historical assessment accuracy creates trust. Standards alignment (NIST, OP Stack roadmap, 
    Ethereum EIPs) creates legitimacy. The infrastructure becomes the default choice not because 
    it's the only option, but because switching costs exceed benefits.
    """
    elements.append(Paragraph(s8p4.strip(), body_style))
    
    # === CLOSING ===
    elements.append(Paragraph("10. Summary", section_style))
    
    closing = """
    Quantum computing breaks current cryptography. AI systems lack accountability. These problems 
    converge on infrastructure requirements: threat assessment, adaptive cryptography, verifiable 
    computation, on-chain settlement. The market grows from $5.5B to $69B over the next decade. 
    The window for building is now; the window for migration closes by 2030. Infrastructure-layer 
    solutions—not application patches—capture durable value. The thesis is not "quantum is coming" 
    (obvious) but "integrated infrastructure for quantum + AI + privacy is the correct abstraction 
    layer for this transition" (non-obvious, defensible, executable).
    """
    elements.append(Paragraph(closing.strip(), body_style))
    
    elements.append(Spacer(1, 0.15*inch))
    
    # === REFERENCES ===
    elements.append(HRFlowable(width="100%", thickness=1, color=ACCENT, spaceBefore=8))
    elements.append(Paragraph("References", section_style))
    
    ref_style = ParagraphStyle(
        'Reference', parent=body_style,
        fontSize=8, spaceAfter=3, leftIndent=18, firstLineIndent=-18, leading=10
    )
    
    references = [
        "[1] Bernstein, D.J. & Lange, T. (2017). Post-quantum cryptography. Nature 549, 188-194. https://doi.org/10.1038/nature23461",
        "[2] Shor, P. (1994). Algorithms for quantum computation: discrete logarithms and factoring. FOCS 1994. https://ieeexplore.ieee.org/document/365700",
        "[3] Grover, L. (1996). A fast quantum mechanical algorithm for database search. STOC 1996. https://arxiv.org/abs/quant-ph/9605043",
        "[4] Google Quantum AI. (2024). Willow processor specifications and error correction results. https://quantumai.google/hardware/datasheet/willow",
        "[5] Mosca, M. (2018). Cybersecurity in an era with quantum computers. IEEE Security & Privacy. https://doi.org/10.1109/MSP.2018.3761723",
        "[6] QRL Foundation. (2025). The Definitive Guide to Post-Quantum Blockchain Security. https://theqrl.org/the-definitive-guide-to-post-quantum-blockchain-security",
        "[7] NSA. (2021). Quantum Computing and Post-Quantum Cryptography FAQ. https://media.defense.gov/2021/Aug/04/2002821837/-1/-1/1/Quantum_FAQs_20210804.PDF",
        "[8] Deloitte. (2022). Quantum computers could crack Bitcoin by 2030. https://www2.deloitte.com/nl/nl/pages/innovatie/artikelen/quantum-computers-and-the-bitcoin-blockchain.html",
        "[9] DeFiLlama. (2025). Total Value Locked in DeFi protocols. https://defillama.com/",
        "[10] Kang, D. et al. (2024). Scaling Up Trustless DNN Inference with Zero-Knowledge Proofs. EuroSys 2024. https://ddkang.github.io/papers/2024/zkml-eurosys.pdf",
        "[11] Lagrange Labs. (2025). DeepProve: Verifiable AI at Scale. https://docs.lagrange.dev/deepprove/overview",
        "[12] IETF. (2025). Post-Quantum Cryptography for Engineers. https://datatracker.ietf.org/doc/html/draft-ietf-pquip-pqc-engineers",
        "[13] NIST. (2024). NIST Releases First 3 Finalized Post-Quantum Encryption Standards. https://nist.gov/news-events/news/2024/08/nist-releases-first-3-finalized-post-quantum-encryption-standards",
        "[14] Intel. (2024). Trust Domain Extensions (TDX) Architecture Specification. https://www.intel.com/content/www/us/en/developer/tools/trust-domain-extensions/documentation.html",
        "[15] Intel. (2024). Software Guard Extensions (SGX) Developer Guide. https://www.intel.com/content/www/us/en/developer/tools/software-guard-extensions/overview.html",
        "[16] MarketsandMarkets. (2025). Post-Quantum Cryptography Market Report. https://www.marketsandmarkets.com/Market-Reports/post-quantum-cryptography-market-126986626.html",
        "[17] Juniper Research. (2025). Post-quantum Cryptography Market 2026-2035. https://www.juniperresearch.com/research/iot-emerging-technology/iot-security/post-quantum-cryptography-research-report/",
        "[18] McKinsey. (2024). Quantum communication growth drivers: Cybersecurity and quantum computing. https://www.mckinsey.com/capabilities/tech-and-ai/our-insights/quantum-communication-growth-drivers-cybersecurity-and-quantum-computing",
        "[19] Schuld, M. & Petruccione, F. (2021). Machine Learning with Quantum Computers. Springer. https://doi.org/10.1007/978-3-030-83098-4",
        "[20] Farhi, E. et al. (2014). A Quantum Approximate Optimization Algorithm. arXiv:1411.4028. https://arxiv.org/abs/1411.4028",
        "[21] Flashbots. (2023). The Future of MEV. https://writings.flashbots.net/the-future-of-mev",
        "[22] Espresso Systems. (2024). Shared Sequencing: Decentralizing Rollup Sequencing. https://docs.espressosys.com/",
        "[23] Brevis. (2024). ZK Coprocessor Architecture. https://docs.brevis.network/",
        "[24] RISC Zero. (2024). zkVM: General Purpose Zero-Knowledge Virtual Machine. https://dev.risczero.com/",
        "[25] zkVerify. (2025). Modular Blockchain for ZK Proof Verification. https://zkverify.io/",
        "[26] Celestia. (2024). Data Availability Layer Documentation. https://docs.celestia.org/",
    ]
    
    for ref in references:
        elements.append(Paragraph(ref, ref_style))
    
    elements.append(Spacer(1, 0.15*inch))
    elements.append(HRFlowable(width="100%", thickness=1, color=colors.grey))
    
    footer = f"{datetime.now().strftime('%B %Y')}"
    footer_style = ParagraphStyle('Footer', fontSize=8, textColor=colors.grey, alignment=TA_CENTER)
    elements.append(Paragraph(footer, footer_style))
    
    doc.build(elements)
    print(f"Thesis write-up generated: {output_path}")
    return output_path


if __name__ == "__main__":
    import sys
    output = sys.argv[1] if len(sys.argv) > 1 else 'Verifiable_Quantum_Security_Thesis.pdf'
    create_thesis_writeup(output)
