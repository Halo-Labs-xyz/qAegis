# Quantum Network Testbeds and Demonstrations

## Global Testbed Infrastructure

```mermaid
graph TB
    subgraph North_America["North America"]
        DARPA["DARPA QuANET"]
        NIST["NIST NG-QNet"]
        DOE["DOE Quantum Networks"]
        ANL["Argonne National Lab"]
    end
    
    subgraph Europe["Europe"]
        QIA["Quantum Internet Alliance"]
        KIT["KIT Karlsruhe"]
        BT["BT/Toshiba UK"]
        OPENQKD["OpenQKD"]
    end
    
    subgraph Asia["Asia"]
        USTC["USTC China Network"]
        MICIUS["Micius Satellite"]
        NICT["NICT Japan"]
        KRISS["KRISS Korea"]
    end
    
    DARPA -.->|"Collaboration"| QIA
    QIA -.->|"Standards"| USTC
    MICIUS -->|"Satellite Link"| Europe
    MICIUS -->|"Satellite Link"| North_America
```

## Beijing-Shanghai Backbone

```mermaid
flowchart LR
    subgraph Beijing["Beijing Node"]
        B_QKD["QKD System"]
        B_Trust["Trusted Node"]
    end
    
    subgraph Jinan["Jinan Node"]
        J_QKD["QKD System"]
        J_Trust["Trusted Node"]
    end
    
    subgraph Hefei["Hefei Node"]
        H_QKD["QKD System"]
        H_Trust["Trusted Node"]
    end
    
    subgraph Shanghai["Shanghai Node"]
        S_QKD["QKD System"]
        S_Trust["Trusted Node"]
    end
    
    Beijing -->|"~500km"| Jinan
    Jinan -->|"~500km"| Hefei
    Hefei -->|"~500km"| Shanghai
    
    Beijing -.->|"Micius Satellite"| Shanghai
```

**Specifications**:
- Total distance: 2,000+ km
- 32 trusted relay nodes
- Key rate: ~10 kbps average
- Operational since 2017
- Extended to 4,600 km network (2021)

## DARPA QuANET Architecture

```mermaid
flowchart TB
    subgraph Control["Control Plane"]
        SDN["SDN Controller"]
        QNM["Quantum Network Manager"]
        KMS["Key Management System"]
    end
    
    subgraph Data["Data Plane"]
        QN1["Quantum Node 1"]
        QN2["Quantum Node 2"]
        QN3["Quantum Node 3"]
        CR["Classical Router"]
    end
    
    subgraph Apps["Applications"]
        QKD["QKD Service"]
        ENT["Entanglement Service"]
        SEC["Secure Comm"]
    end
    
    SDN --> QNM
    QNM --> KMS
    
    QNM -->|"Control"| QN1
    QNM -->|"Control"| QN2
    QNM -->|"Control"| QN3
    
    QN1 ===|"Quantum Channel"| QN2
    QN2 ===|"Quantum Channel"| QN3
    QN1 ---|"Classical"| CR
    QN2 ---|"Classical"| CR
    QN3 ---|"Classical"| CR
    
    Apps --> Control
```

**Program Goals**:
- Hybrid quantum-classical networking
- IP-compatible quantum protocols
- Real-world deployment validation
- Metrics: throughput, latency, security

## European Quantum Communication Infrastructure

```mermaid
flowchart TB
    subgraph Satellite["Space Segment"]
        SAT["QKD Satellite"]
    end
    
    subgraph Ground["Ground Segment"]
        subgraph DE["Germany"]
            DE1["Berlin Hub"]
            DE2["Munich Hub"]
        end
        
        subgraph FR["France"]
            FR1["Paris Hub"]
        end
        
        subgraph NL["Netherlands"]
            NL1["Delft Hub"]
        end
        
        subgraph ES["Spain"]
            ES1["Madrid Hub"]
        end
    end
    
    SAT -.-> DE1
    SAT -.-> FR1
    SAT -.-> NL1
    SAT -.-> ES1
    
    DE1 ===|"Fiber QKD"| DE2
    DE1 ===|"Fiber QKD"| NL1
    FR1 ===|"Fiber QKD"| DE1
    FR1 ===|"Fiber QKD"| ES1
```

## Quantum Internet Alliance Prototype

```mermaid
flowchart LR
    subgraph Processing["Processing Nodes"]
        PN1["NV Center Node"]
        PN2["NV Center Node"]
    end
    
    subgraph Repeater["Quantum Repeater"]
        QM["Quantum Memory"]
        BSM["Bell State Measurement"]
    end
    
    subgraph Photonic["Photonic Clients"]
        PC1["Photonic Client"]
        PC2["Photonic Client"]
    end
    
    PN1 -->|"Photon"| Repeater
    PN2 -->|"Photon"| Repeater
    QM --> BSM
    BSM -->|"Heralding"| PN1
    BSM -->|"Heralding"| PN2
    
    PC1 -.->|"User Interface"| PN1
    PC2 -.->|"User Interface"| PN2
```

**Components**:
- NV center processing nodes (TU Delft)
- Photonic clients for user access
- Software stack for protocol execution
- Classical control infrastructure

## Demonstration Milestones

```mermaid
timeline
    title Quantum Networking Milestones
    
    2017 : Beijing-Shanghai backbone operational
         : First intercontinental QKD (China-Austria)
    
    2020 : 600 km twin-field QKD demonstrated
         : Entanglement over 1200 km satellite
    
    2022 : 1000 km fiber QKD without trusted nodes
         : Device-independent QKD demonstrated
    
    2023 : Multi-node entanglement distribution
         : Commercial QKD networks deployed
    
    2024 : 830 km fiber QKD record
         : Q-Chip IP-compatible quantum
    
    2025 : Quantum repeater prototypes
         : Pan-European QCI rollout
```

## Performance Metrics Comparison

| Testbed | Distance | Key Rate | Technology | Status |
|---------|----------|----------|------------|--------|
| Beijing-Shanghai | 2000 km | 10 kbps | Trusted nodes | Operational |
| Micius Satellite | 7600 km | 0.1 kbps | Satellite QKD | Operational |
| NIST NG-QNet | 100 km | 1 Mbps | Direct fiber | Development |
| QuTech Delft | 25 km | Variable | NV centers | Research |
| DARPA QuANET | Variable | TBD | Hybrid | Prototype |

## Hardware Technology Stack

```mermaid
flowchart TB
    subgraph Sources["Single Photon Sources"]
        SPDC["SPDC"]
        QD["Quantum Dots"]
        NV["NV Centers"]
    end
    
    subgraph Detectors["Single Photon Detectors"]
        SNSPD["SNSPD<br/>η>90%, <1 Hz DCR"]
        APD["InGaAs APD<br/>η~25%, 1 kHz DCR"]
    end
    
    subgraph Memory["Quantum Memories"]
        Atomic["Atomic Ensemble"]
        Solid["Solid State<br/>(NV, RE ions)"]
        Cavity["Cavity QED"]
    end
    
    subgraph Channel["Quantum Channels"]
        Fiber["Telecom Fiber<br/>0.2 dB/km @ 1550nm"]
        FSO["Free Space<br/>Atmosphere dependent"]
        Sat["Satellite<br/>~30 dB total loss"]
    end
    
    Sources --> Channel
    Channel --> Detectors
    Memory --> Sources
    Memory --> Detectors
```

## PhotonSync Technology

```mermaid
flowchart LR
    subgraph Stabilization["Phase/Frequency Stabilization"]
        LASER["Reference Laser"]
        PLL["Phase Lock Loop"]
        COMP["Active Compensation"]
    end
    
    subgraph Fiber["Standard Telecom Fiber"]
        F1["Segment 1"]
        F2["Segment 2"]
        FN["Segment N"]
    end
    
    subgraph QKD["QKD System"]
        ALICE["Alice"]
        BOB["Bob"]
    end
    
    LASER --> PLL
    PLL --> COMP
    COMP --> Fiber
    
    ALICE -->|"Quantum Signal"| F1
    F1 --> F2
    F2 --> FN
    FN -->|"Quantum Signal"| BOB
    
    COMP -.->|"Feedback"| F1
    COMP -.->|"Feedback"| F2
    COMP -.->|"Feedback"| FN
```

**Capability**:
- Converts standard fiber to quantum-grade
- Active phase drift compensation
- Enables TF-QKD over 1000+ km
- Real-time feedback system

## Simulation Tools Architecture

```mermaid
flowchart TB
    subgraph Tools["Simulation Frameworks"]
        NS["NetSquid<br/>(TU Delft)"]
        SQ["SimulaQron<br/>(Application level)"]
        SEQ["SeQUeNCe<br/>(Protocol stack)"]
        QNS["qns-3<br/>(ns-3 module)"]
    end
    
    subgraph Abstraction["Abstraction Levels"]
        PHY["Physical Layer<br/>Photon simulation"]
        LINK["Link Layer<br/>Entanglement generation"]
        NET["Network Layer<br/>Routing protocols"]
        APP["Application Layer<br/>QKD, distributed QC"]
    end
    
    NS --> PHY
    NS --> LINK
    SQ --> APP
    SEQ --> LINK
    SEQ --> NET
    SEQ --> APP
    QNS --> NET
```

## Recent Experimental Advances (2025-2026)

### Teleported Quantum Gates
- **Paper**: arXiv:2601.04848
- **Achievement**: Unconditionally teleported gates between remote solid-state qubit registers
- **Significance**: First demonstration of deterministic remote gate operations

### Deterministic Entanglement Distribution
- **Paper**: arXiv:2601.08581
- **Method**: Entanglement-swapping measurements for deterministic distribution
- **Improvement**: Moves beyond probabilistic entanglement generation

### Experimental COW-QKD
- **Paper**: arXiv:2601.06772
- **Published**: Science Advances 2026
- **Achievement**: Practical coherent one-way QKD with simplified setup

### Source-Independent Protocols
| Paper | Protocol | Achievement |
|-------|----------|-------------|
| arXiv:2512.20038 | SI-QCKA | Efficient source-independent conference key agreement |
| arXiv:2512.18325 | SI-QSS | Source-independent secret sharing against coherent attacks |

### Gate-Based Microwave Quantum Repeater
- **Paper**: arXiv:2512.19896
- **Approach**: Grid-state encoding with autonomous error correction
- **Type**: Second-generation repeater architecture

### Hybrid Quantum Repeater Chains
- **Paper**: arXiv:2512.21655
- **Components**: Atom-based quantum processing units + quantum memory multiplexers
- **Focus**: Practical implementation challenges

---

## References

1. "Beijing-Shanghai quantum backbone" Nature 2021
2. DARPA QuANET program documentation
3. Quantum Internet Alliance technical reports
4. "PhotonSync: Long-distance quantum communication" 2025
5. NetSquid, SimulaQron, SeQUeNCe documentation
6. Experimental COW-QKD: arXiv:2601.06772, Sci. Adv. 2026
7. Teleported Quantum Gates: arXiv:2601.04848
8. Deterministic Swapping: arXiv:2601.08581
9. Gate-Based MW Repeater: arXiv:2512.19896
10. Hybrid Repeater Chains: arXiv:2512.21655