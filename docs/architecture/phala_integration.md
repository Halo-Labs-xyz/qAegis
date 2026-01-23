# Phala Network TEE Integration

## Overview

QuantumAegis integrates with Phala Network's TEE Cloud to provide hardware-backed security for quantum-resistant transaction sequencing.

## Architecture

```mermaid
graph TB
    subgraph PhalaCloud["Phala Network TEE Cloud"]
        subgraph Enclave["TEE Enclave TDX/SEV"]
            SEQ[PhalaTeeSequencer]
            APQC[Adaptive PQC Layer]
            QRM[Quantum Resistance Monitor]
            ASSETS[Asset Registry]
            MIGRATION[Migration Manager]
        end
        
        ATTEST[Phala Attestation]
        QUOTE[TEE Quote Verification]
    end
    
    subgraph External["External Systems"]
        USERS[Users]
        APPS[Applications]
        CHAINS[Blockchains]
    end
    
    subgraph L2["OP Stack L2"]
        OP_NODE[op-node]
        CONTRACTS[QRMSOracle<br/>SequencerAttestation]
    end
    
    USERS -->|Encrypted TXs| SEQ
    APPS -->|Asset Registration| ASSETS
    SEQ -->|Decrypt & Order| SEQ
    SEQ -->|PQC Sign| APQC
    SEQ -->|Risk Check| QRM
    SEQ -->|Protect| ASSETS
    SEQ -->|Checkpoint| MIGRATION
    
    SEQ -->|Attest| ATTEST
    ATTEST -->|Verify| QUOTE
    
    SEQ -->|Batches| OP_NODE
    SEQ -->|Updates| CONTRACTS
    
    OP_NODE -->|State| CHAINS
    CONTRACTS -->|Oracle| CHAINS
```

## Components

### PhalaTeeSequencer

Core sequencer running inside Phala TEE enclave.

**Features:**
- Encrypted mempool (decrypted only in TEE)
- Intelligence-based ordering
- Asset protection registry
- Migration checkpointing
- Quantum-resistant batch signing

**Location:** `services/qrms/src/phala_tee.rs`

### Asset Protection

Protects both on-chain and off-chain assets:

```rust
pub enum AssetType {
    OnChainToken,      // ERC-20, ERC-721
    OnChainNFT,
    OnChainData,       // Smart contract state
    OffChainDatabase,  // External databases
    OffChainFile,      // File storage
    OffChainStream,    // Data streams
    CrossChainBridge,  // Bridge assets
}
```

### Intelligence Ordering

Four ordering strategies:

1. **Risk-Aware**: High-risk transactions first
2. **Asset Protection**: Protected assets prioritized
3. **Migration-Aware**: Migration transactions grouped
4. **Hybrid**: Combines all strategies

### Migration System

State migration with checkpointing:

```rust
pub struct MigrationCheckpoint {
    pub checkpoint_id: String,
    pub block_number: u64,
    pub state_hash: String,
    pub asset_snapshots: Vec<AssetSnapshot>,
    pub pqc_signature: String,  // ML-DSA-87
}
```

## Data Flow

### Transaction Submission

```mermaid
sequenceDiagram
    participant User
    participant API
    participant TEE
    participant QRM
    participant APQC
    participant L2

    User->>API: Submit Transaction
    API->>API: Encrypt with TEE Key
    API->>TEE: Encrypted Transaction
    TEE->>TEE: Decrypt (inside enclave)
    TEE->>QRM: Check Risk
    QRM->>TEE: Risk Score
    TEE->>TEE: Order by Intelligence
    TEE->>APQC: Request PQC Signatures
    APQC->>TEE: ML-DSA + SLH-DSA
    TEE->>TEE: Generate Attestation
    TEE->>L2: Submit Batch
    L2->>User: Confirmation
```

### Asset Protection

```mermaid
sequenceDiagram
    participant App
    participant TEE
    participant Registry
    participant QRM

    App->>TEE: Register Asset
    TEE->>Registry: Store Protection Policy
    Registry->>QRM: Monitor Risk
    QRM->>Registry: Risk Update
    Registry->>TEE: Activate Protection
    TEE->>App: Protection Active
```

### State Migration

```mermaid
sequenceDiagram
    participant Admin
    participant TEE
    participant Checkpoint
    participant L2

    Admin->>TEE: Start Migration
    TEE->>TEE: Create Checkpoint
    TEE->>Checkpoint: Snapshot Assets
    TEE->>TEE: Sign with PQC
    TEE->>L2: Store Checkpoint
    TEE->>TEE: Process Migration
    TEE->>L2: Update State
    TEE->>Admin: Migration Complete
```

## Security Model

### TEE Attestation

```rust
pub struct PhalaAttestation {
    pub worker_id: String,
    pub enclave_id: String,
    pub quote: Vec<u8>,              // TEE quote
    pub quote_type: String,           // "TDX" or "SEV"
    pub mr_enclave: String,          // Code measurement
    pub mr_signer: String,           // Signer measurement
    pub report_data: Vec<u8>,        // Batch hash
    pub phala_verification: bool,    // Network verified
}
```

### Quantum Resistance

- **ML-DSA-87**: Primary signature (NIST Level 5)
- **SLH-DSA-256s**: Secondary signature (NIST Level 5)
- **Hybrid ECDSA**: Backward compatibility
- **Automatic Rotation**: Based on QRM risk scores

### Asset Encryption

- All asset data encrypted with TEE-protected keys
- Keys never leave enclave
- Access policies enforced at sequencer level

## Performance

### Batch Processing

- **Batch Size**: 10-50 transactions (configurable)
- **Mempool Capacity**: 10,000 transactions
- **Ordering Latency**: <100ms (inside TEE)
- **PQC Signing**: 1-3ms per signature

### Scalability

- **Multi-Worker**: 3-10 TEE workers (redundancy)
- **Load Balancing**: Automatic across workers
- **State Sync**: Checkpoint-based synchronization

## Deployment

See [Phala TEE Deployment Guide](../deployment/PHALA_TEE.md) for detailed deployment instructions.

## Configuration

Example `phala.toml`:

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

## References

- [Phala Network](https://phala.network)
- [TEE Architecture](./phase3_tee.md)
- [Quantum Cryptography](./phase2_cryptography.md)
- [Threat Intelligence](./phase4_threat_intelligence.md)
