# QuantumAegis Directory Structure

```
quantum-aegis/
│
├── README.md                    # Project overview and quick start
├── LICENSE                      # MIT License
├── .gitignore                   # Git ignore patterns
├── DIRECTORY_TREE.md            # This file
│
├── services/                    # Core QRMS Services (Rust)
│   └── qrms/
│       ├── Cargo.toml           # Rust dependencies
│       ├── src/
│       │   ├── main.rs          # QRMS server entry point
│       │   ├── bin/
│       │   │   └── qrms-cli.rs  # Terminal UI for monitoring
│       │   ├── qrm.rs           # Quantum Resistance Monitor
│       │   ├── apqc.rs          # Adaptive PQC Layer
│       │   ├── crypto.rs        # Real PQC implementations
│       │   ├── sequencer.rs     # TEE Sequencer logic
│       │   ├── chain.rs         # Chain state management
│       │   ├── state.rs         # Application state
│       │   └── handlers.rs      # HTTP/WebSocket handlers
│       ├── static/              # Web GUI assets
│       └── README.md            # Service documentation
│
├── contracts/                   # Solidity Smart Contracts
│   ├── foundry.toml             # Foundry configuration
│   ├── src/
│   │   ├── PQCVerifier.sol      # On-chain PQC signature verification
│   │   ├── QRMSOracle.sol       # Risk score & algorithm oracle
│   │   └── SequencerAttestation.sol  # TEE attestation verification
│   ├── script/
│   │   └── Deploy.s.sol         # Deployment script
│   └── test/                    # Contract tests
│
├── rollup/                      # OP Stack L2 Deployment
│   ├── Makefile                 # Deployment commands
│   ├── docker-compose.yml       # Service orchestration
│   ├── opstack/                 # OP Stack configuration
│   │   ├── docker-compose.yml   # OP Stack services
│   │   ├── scripts/
│   │   │   ├── setup-rollup.sh  # L2 initialization
│   │   │   └── download-op-deployer.sh
│   │   ├── deployer/            # Deployment state
│   │   └── sequencer/           # L2 sequencer config
│   ├── config/                  # Generated configs
│   │   ├── genesis.json         # L2 genesis
│   │   ├── rollup.json          # Rollup config
│   │   └── jwt.txt              # JWT secret
│   ├── scripts/                 # Deployment utilities
│   └── docker/
│       └── Dockerfile.qrms      # QRMS container image
│
├── docs/                        # Documentation
│   ├── architecture/           # System architecture docs
│   │   ├── README.md            # Architecture overview
│   │   ├── stack_architecture.md
│   │   ├── qrms_implementation.md
│   │   ├── quantum_resistance_model.md
│   │   ├── threat_taxonomy.md
│   │   └── ...
│   ├── deployment/              # Deployment guides
│   │   ├── README.md
│   │   ├── l2-setup.md
│   │   └── contracts-deployment.md
│   └── api/                     # API documentation
│       ├── README.md
│       ├── rest-api.md
│       └── websocket-api.md
│
├── scripts/                     # Utility Scripts
│   ├── deploy/                  # Deployment scripts
│   └── setup/                   # Setup scripts
│
└── tools/                       # Development Tools
    └── cli/                     # CLI utilities
```

## Key Components

### Services (`services/qrms/`)
- **QRM**: Monitors 12 quantum threat categories, calculates risk scores
- **APQC**: Manages ML-DSA-87, SLH-DSA-256s, hybrid ECDSA signatures
- **Sequencer**: TEE-secured transaction ordering and batching
- **Chain**: L2 state tracking and integration

### Contracts (`contracts/`)
- **PQCVerifier**: On-chain verification of PQC signatures
- **QRMSOracle**: Stores risk scores and algorithm rotation state
- **SequencerAttestation**: Verifies TEE attestations from sequencer

### Rollup (`rollup/`)
- OP Stack L2 configuration and deployment
- Docker Compose orchestration
- Deployment scripts and state management

### Documentation (`docs/`)
- Architecture specifications
- Deployment procedures
- API references
