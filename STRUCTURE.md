# QuantumAegis Repository Structure

## Directory Tree

```
quantum-aegis/
│
├── README.md                 # Project overview
├── LICENSE                   # MIT License
├── DIRECTORY_TREE.md        # Detailed structure
├── DEPLOYMENT.md             # Deployment status
├── CONTRIBUTING.md           # Contribution guidelines
│
├── .github/
│   └── workflows/
│       └── ci.yml               # CI/CD pipeline
│
├── services/                 # Rust Services
│   └── qrms/
│       ├── Cargo.toml
│       ├── qvm.toml             # QVM configuration
│       ├── phala.toml           # Phala redundancy config
│       ├── src/
│       │   ├── main.rs          # Server entry
│       │   ├── bin/
│       │   │   └── qrms-cli.rs # Terminal UI
│       │   ├── qrm.rs          # Threat monitor
│       │   ├── qvm.rs          # Quantum Virtual Machine
│       │   ├── apqc.rs         # PQC layer
│       │   ├── crypto.rs       # Real PQC impl
│       │   ├── aegis_tee.rs    # Aegis-TEE sequencer (primary)
│       │   ├── phala_tee.rs    # Phala TEE (deprecated, redundancy)
│       │   ├── phala_deploy.rs # Phala deployment config
│       │   ├── sequencer.rs    # Base sequencer
│       │   ├── chain.rs        # Chain state
│       │   ├── state.rs        # App state
│       │   └── handlers.rs     # HTTP/WS handlers
│       └── static/             # Web GUI
│
├── contracts/                # Solidity Contracts
│   ├── foundry.toml
│   ├── src/
│   │   ├── PQCVerifier.sol
│   │   ├── QRMSOracle.sol
│   │   └── SequencerAttestation.sol
│   └── script/
│       └── Deploy.s.sol
│
├── rollup/                   # OP Stack L2
│   ├── Makefile
│   ├── docker-compose.yml
│   ├── opstack/                 # OP Stack config
│   │   ├── docker-compose.yml
│   │   ├── scripts/
│   │   │   ├── setup-rollup.sh
│   │   │   └── download-op-deployer.sh
│   │   └── deployer/           # Deployment state
│   ├── config/                  # Generated configs
│   │   ├── genesis.json
│   │   ├── rollup.json
│   │   └── jwt.txt
│   └── docker/
│       └── Dockerfile.qrms
│
├── docs/                     # Documentation
│   ├── architecture/
│   │   ├── README.md
│   │   ├── stack_architecture.md
│   │   ├── qrms_implementation.md
│   │   ├── quantum_resistance_model.md
│   │   └── threat_taxonomy.md
│   ├── deployment/
│   │   └── README.md
│   └── api/
│
├── scripts/                   # Utility Scripts
│   ├── deploy/
│   └── setup/
│
└── tools/                    # Dev Tools
    └── cli/
```

## Component Overview

### Services (`services/qrms/`)
**Language**: Rust  
**Purpose**: Core QRMS monitoring and PQC operations

- **QVM**: Quantum Virtual Machine oracle (Grover/Shor threat assessment)
- **QRM**: 12-category threat monitoring, risk scoring
- **APQC**: Real PQC signatures (ML-DSA, SLH-DSA), hybrid ECDSA
- **Aegis-TEE**: Primary TEE sequencer (TDX/SEV/SGX)
- **Phala Redundancy**: Optional Phala Network fallback
- **Chain**: L2 state integration

### Contracts (`contracts/`)
**Language**: Solidity  
**Framework**: Foundry

- **PQCVerifier**: On-chain PQC signature verification
- **QRMSOracle**: Risk scores and algorithm state oracle
- **SequencerAttestation**: TEE attestation verification

### Rollup (`rollup/`)
**Stack**: OP Stack  
**Orchestration**: Docker Compose

- OP Stack L2 configuration
- Service deployment automation
- L1 contract deployment scripts

### Documentation (`docs/`)
- Architecture specifications
- Deployment procedures
- API references

## File Counts

- **Rust**: ~10 source files
- **Solidity**: 3 contracts + deployment script
- **Documentation**: 12+ markdown files
- **Configuration**: Docker, Foundry, Cargo configs

## Key Features

- **QVM Oracle**: Google Cirq-based quantum threat assessment
- **PQC Cryptography**: ML-DSA-87, SLH-DSA-256s (real implementations)
- **Hybrid Signatures**: ECDSA + PQC dual signatures
- **12-Category Threat Monitoring**: Comprehensive quantum risk assessment
- **Adaptive Algorithm Rotation**: Automatic PQC upgrades based on risk
- **Aegis-TEE Sequencer**: Primary TEE with Phala redundancy
- **OP Stack L2**: Full rollup deployment with Docker orchestration
- **Web GUI + CLI**: Real-time monitoring interfaces
