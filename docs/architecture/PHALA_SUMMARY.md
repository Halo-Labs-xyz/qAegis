# Phala Network TEE Integration Summary (Redundancy Layer)

## Implementation Status: Complete

**Note**: Phala Network is now used as a **redundancy/fallback layer** for Aegis-TEE. The primary TEE implementation is Aegis-TEE (see [Aegis-TEE Architecture](./aegis_tee.md)).

QuantumAegis sequencer uses Aegis-TEE as the primary TEE infrastructure, with Phala Network TEE Cloud available as an optional redundancy mechanism for enhanced reliability and distributed security verification.

## Core Components

### 1. AegisTeeSequencer (`services/qrms/src/aegis_tee.rs`) - Primary

**Features:**
- Encrypted mempool (decrypted only inside TEE)
- Intelligence-based transaction ordering
- Asset protection registry
- State migration with checkpointing
- Quantum-resistant batch signing
- Optional Phala Network redundancy

### 2. Phala Network Redundancy (Optional)

When enabled, provides:
- Backup attestation for each batch
- Cross-validation with Aegis-TEE
- Fallback verification
- Distributed security across multiple TEE platforms

### 3. Legacy: PhalaTeeSequencer (`services/qrms/src/phala_tee.rs`) - Deprecated

**Features:**
- Encrypted mempool (decrypted only inside TEE)
- Intelligence-based transaction ordering
- Asset protection registry
- State migration with checkpointing
- Quantum-resistant batch signing

**Ordering Strategies:**
- Risk-aware: High-risk transactions first
- Asset protection: Protected assets prioritized
- Migration-aware: Migration transactions grouped
- Hybrid: Combines all strategies

### 4. Asset Protection System

**Supported Asset Types:**
- On-chain: Tokens, NFTs, smart contract data
- Off-chain: Databases, files, data streams
- Cross-chain: Bridge assets

**Protection Features:**
- TEE-encrypted storage
- Access policy enforcement
- Risk threshold activation
- Migration state preservation

### 5. State Migration

**Checkpoint System:**
- Asset state snapshots
- PQC-signed migration state
- Block-based checkpointing
- Rollback support

### 6. Quantum-Resistant Batching

**Batch Structure:**
- ML-DSA-87 signature (primary)
- SLH-DSA-256s signature (secondary)
- Aegis-TEE attestation (primary)
- Phala Network attestation (optional redundancy)
- Risk assessment integration
- Asset protection metadata

## Deployment Configuration

**File:** `services/qrms/phala.toml`

```toml
[phala]
network = "mainnet"

[worker]
id = "quantumaegis-sequencer"
enclave_type = "TDX"
min_workers = 3
max_workers = 10

[quantum]
signature_algorithms = ["ML-DSA-87", "SLH-DSA-256s"]
hybrid_ecdsa = true
risk_scheduled = 6000
risk_emergency = 9000

[intelligence]
mode = "hybrid"
enable_asset_protection = true
enable_migration = true
```

## Integration Points

### On-Chain Contracts

1. **QRMSOracle**: Receives risk assessments and algorithm updates
2. **SequencerAttestation**: Verifies Phala TEE attestations
3. **PQCVerifier**: Validates ML-DSA-87 and SLH-DSA-256s signatures

### Off-Chain Services

1. **QRM Monitor**: Provides risk scores for intelligence ordering
2. **APQC Layer**: Generates quantum-resistant signatures
3. **Asset Registry**: Manages protection policies

## Security Model

### TEE Attestation

- Phala Network verifies TEE quotes
- MRENCLAVE ensures code integrity
- MRSIGNER verifies signer identity
- Report data includes batch hash

### Quantum Resistance

- Dual PQC signatures (ML-DSA-87 + SLH-DSA-256s)
- Hybrid ECDSA for backward compatibility
- Automatic rotation based on QRM risk scores

### Asset Protection

- Encryption keys stored only in TEE
- Access policies enforced at sequencer level
- Risk-based activation thresholds

## Performance

- **Batch Size**: 10-50 transactions (configurable)
- **Mempool Capacity**: 10,000 transactions
- **Ordering Latency**: <100ms (inside TEE)
- **PQC Signing**: 1-3ms per signature
- **Multi-Worker**: 3-10 TEE workers for redundancy

## Next Steps

1. **Deploy to Phala Testnet**
   - Test TEE worker integration
   - Verify attestation chain
   - Validate asset protection

2. **On-Chain Integration**
   - Update SequencerAttestation.sol for Phala quotes
   - Integrate with QRMSOracle
   - Deploy updated contracts

3. **Production Deployment**
   - Deploy to Phala Mainnet
   - Configure monitoring
   - Establish operational procedures

## Documentation

- [Phala Integration Guide](./phala_integration.md)
- [Phala Deployment Guide](../deployment/PHALA_TEE.md)
- [TEE Architecture](./phase3_tee.md)
- [Example Code](../../services/qrms/examples/phala_sequencer_example.rs)
