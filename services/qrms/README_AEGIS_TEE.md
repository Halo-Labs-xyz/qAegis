# Aegis-TEE: Primary TEE Sequencer

## Overview

QuantumAegis uses **Aegis-TEE** as the primary Trusted Execution Environment sequencer, with Phala Network TEE Cloud available as an optional redundancy/fallback layer for enhanced reliability and distributed security.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│              AEGIS-TEE (Primary)                         │
│  ┌───────────┐ ┌───────────┐ ┌───────────┐            │
│  │ Encrypted │ │  Asset    │ │ Migration │            │
│  │  Mempool  │ │Protection │ │  System   │            │
│  └───────────┘ └───────────┘ └───────────┘            │
│                              │                          │
│  ┌───────────────────────────▼──────────────────────┐  │
│  │      Phala Network Redundancy (Optional)        │  │
│  │  Provides fallback and distributed verification │  │
│  └──────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

## Features

- **Primary TEE**: Aegis-TEE infrastructure (self-hosted)
- **Redundancy**: Phala Network TEE Cloud (optional fallback)
- **Hardware TEE**: Intel TDX, AMD SEV, or Intel SGX enclaves
- **Quantum Resistance**: ML-DSA-87 + SLH-DSA-256s dual signatures
- **Asset Protection**: On-chain and off-chain data protection
- **Intelligence Ordering**: Risk-aware transaction sequencing
- **State Migration**: Seamless upgrades with checkpointing

## Quick Start

### 1. Initialize Aegis-TEE Sequencer

```rust
use qrms::aegis_tee::AegisTeeSequencer;

// Without Phala redundancy
let mut sequencer = AegisTeeSequencer::new(
    "aegis_worker_0".to_string(),
    "aegis_enclave_0".to_string(),
    "TDX".to_string(),  // or "SEV", "SGX"
    None,  // No Phala redundancy
);

// With Phala redundancy (optional)
let mut sequencer = AegisTeeSequencer::new(
    "aegis_worker_0".to_string(),
    "aegis_enclave_0".to_string(),
    "TDX".to_string(),
    Some((
        "phala_worker_0".to_string(),
        "phala_enclave_0".to_string(),
    )),  // Phala redundancy enabled
);
```

### 2. Register Asset

```rust
use qrms::aegis_tee::{AssetProtection, AssetType, AccessPolicy, MigrationState};

let asset = AssetProtection {
    asset_id: "token_001".to_string(),
    asset_type: AssetType::OnChainToken,
    chain_id: Some(16584),
    contract_address: Some("0x...".to_string()),
    encryption_key: vec![],
    access_policy: AccessPolicy {
        allowed_operations: vec!["transfer".to_string()],
        requires_pqc: true,
        requires_tee: true,
        risk_threshold: 5000,
    },
    migration_state: MigrationState::Active,
};

sequencer.register_asset(asset);
```

### 3. Submit Transaction

```rust
use qrms::aegis_tee::EncryptedTransaction;
use chrono::Utc;

let encrypted_tx = EncryptedTransaction {
    tx_id: "tx_001".to_string(),
    encrypted_data: encrypted_data,
    asset_refs: vec!["token_001".to_string()],
    priority_fee: 100,
    timestamp: Utc::now(),
    risk_level: 5000,
    requires_migration: false,
};

sequencer.submit_encrypted(encrypted_tx);
```

### 4. Create Batch

```rust
use qrms::apqc::AdaptivePqcLayer;

let mut apqc = AdaptivePqcLayer::new();
let tee_key = get_tee_key(); // From Aegis-TEE

if let Some(batch) = sequencer.create_quantum_batch(&mut apqc, &tee_key).await {
    // Batch includes Aegis-TEE attestation
    // If Phala redundancy is enabled, batch.attestation.phala_redundancy will be Some(...)
    submit_to_l2(batch).await;
}
```

## Phala Redundancy Configuration

### Enable/Disable Phala Redundancy

```rust
// Enable Phala redundancy
sequencer.set_phala_redundancy(
    true,
    Some("phala_worker_0".to_string()),
    Some("phala_enclave_0".to_string()),
);

// Disable Phala redundancy
sequencer.set_phala_redundancy(false, None, None);
```

### Redundancy Attestation

When Phala redundancy is enabled, each batch attestation includes both:
- **Primary**: Aegis-TEE attestation (`aegis_verification: true`)
- **Redundancy**: Phala Network attestation (if enabled, in `phala_redundancy` field)

```rust
if let Some(batch) = sequencer.create_quantum_batch(&mut apqc, &tee_key).await {
    // Primary attestation
    assert!(batch.attestation.aegis_verification);
    
    // Optional Phala redundancy
    if let Some(phala_attestation) = &batch.attestation.phala_redundancy {
        assert!(phala_attestation.phala_verification);
    }
}
```

## Intelligence Ordering

```rust
use qrms::aegis_tee::IntelligenceOrdering;

sequencer.intelligence_mode = IntelligenceOrdering::Hybrid;
```

Modes:
- `RiskAware`: Order by risk level (high risk first)
- `AssetProtection`: Prioritize protected assets
- `MigrationAware`: Group migration transactions
- `Hybrid`: Combine all strategies (default)

## State Migration

```rust
// Start migration
sequencer.start_migration();

// Create batch (includes checkpoint)
let batch = sequencer.create_quantum_batch(&mut apqc, &tee_key).await;

// Complete migration
if let Some(checkpoint) = batch.migration_checkpoint {
    sequencer.complete_migration(checkpoint);
}
```

## Migration from PhalaTeeSequencer

If you're using the deprecated `PhalaTeeSequencer`, migrate to `AegisTeeSequencer`:

```rust
// Old (deprecated)
use qrms::phala_tee::PhalaTeeSequencer;
let mut sequencer = PhalaTeeSequencer::new(...);

// New (recommended)
use qrms::aegis_tee::AegisTeeSequencer;
let mut sequencer = AegisTeeSequencer::new(
    "aegis_worker_0".to_string(),
    "aegis_enclave_0".to_string(),
    "TDX".to_string(),
    Some(("phala_worker_0".to_string(), "phala_enclave_0".to_string())), // Optional Phala redundancy
);
```

## Documentation

- [Aegis-TEE Architecture](../../docs/architecture/aegis_tee.md)
- [Phala Redundancy Guide](../../docs/architecture/phala_integration.md)
- [TEE Architecture](../../docs/architecture/phase3_tee.md)
