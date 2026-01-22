# Changelog

All notable changes to QuantumAegis will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial monorepo structure
- QRMS Rust service with PQC cryptography
- Solidity contracts (PQCVerifier, QRMSOracle, SequencerAttestation)
- OP Stack L2 rollup deployment
- Documentation
- CI/CD pipeline
- Test suite structure

### Features
- ML-DSA-87 (Dilithium-5) signature support
- SLH-DSA-256s (SPHINCS+) signature support
- Hybrid ECDSA + PQC dual signatures
- 12-category quantum threat monitoring
- Adaptive algorithm rotation
- TEE sequencer integration
- Web GUI and CLI interfaces

## [0.1.0] - 2026-01-22

### Added
- Initial release
- L2 testnet deployment (Chain ID: 42069)
- Contract deployment on L2
- PQC cryptography integration
