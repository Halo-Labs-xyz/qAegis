# Changelog

All notable changes to QuantumAegis will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2026-01-24

### Added
- **QVM Oracle Layer**: Google Cirq-based quantum virtual machine
  - Willow (105Q), Weber (72Q), Rainbow (53Q) processor support
  - Grover's algorithm threat assessment (symmetric crypto)
  - Shor's algorithm threat assessment (public key crypto)
  - Automatic quantum era transitions
- **Qubit Picking**: Hardware qubit selection based on calibration data
  - Single-qubit Pauli error analysis
  - Two-qubit gate error characterization
  - FSim error modeling
  - Readout error compensation
  - Multiple picking strategies (Balanced, MinimizeError, MaximizeCoherence)
- **Aegis-TEE Sequencer**: Primary TEE implementation
  - TDX/SEV/SGX enclave support
  - Phala Network redundancy layer (optional fallback)
  - Intelligence-based transaction ordering
  - Asset protection system

### Changed
- **TEE Architecture**: Aegis-TEE is now primary, Phala Network demoted to redundancy
- **Docker Images**: Updated op-node to v1.16.5, op-geth to v1.101605.0
- **Rollup Infrastructure**: Full infra profile with batcher, proposer, dispute-mon
- **Documentation**: Comprehensive updates for consistency across all modules

### Fixed
- `rollup.json` compatibility with newer op-node versions (strip unsupported fields)
- Docker Compose service configurations for batcher, proposer, challenger
- Rust compilation warnings for unused imports

## [0.2.0] - 2026-01-23

### Added
- **Phala Network TEE Integration**
  - Encrypted mempool for MEV protection
  - Asset protection (on-chain and off-chain)
  - State migration system
  - Quantum-resistant batching
- **Real PQC Cryptography**
  - ML-DSA-87 via pqcrypto-dilithium
  - SLH-DSA-256s via pqcrypto-sphincsplus
  - ECDSA secp256k1 via k256
- **OP Stack Deployment Scripts**
  - Automated L1 contract deployment
  - Config generation (genesis.json, rollup.json)
  - Docker Compose orchestration

### Changed
- Chain ID updated to 16584

## [0.1.0] - 2026-01-22

### Added
- Initial monorepo structure
- QRMS Rust service with mock PQC
- Solidity contracts (PQCVerifier, QRMSOracle, SequencerAttestation)
- OP Stack L2 rollup configuration
- 12-category quantum threat monitoring
- Web GUI and CLI interfaces
- CI/CD pipeline
- Documentation framework
