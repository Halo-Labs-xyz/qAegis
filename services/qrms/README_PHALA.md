# Phala Network TEE Integration

## Overview

QuantumAegis sequencer with Phala Network TEE Cloud integration provides hardware-backed security for quantum-resistant transaction sequencing.

## Features

- **Hardware TEE**: Intel TDX or AMD SEV enclaves
- **Quantum Resistance**: ML-DSA-87 + SLH-DSA-256s dual signatures
- **Asset Protection**: On-chain and off-chain data protection
- **Intelligence Ordering**: Risk-aware transaction sequencing
- **State Migration**: Seamless upgrades with checkpointing

## Quick Start

### 1. Configuration

Copy and edit `phala.toml`:

```bash
cp phala.toml phala.local.toml
# Edit phala.local.toml with your settings
```

### 2. Build

```bash
cargo build --release --features phala
```

### 3. Run Example

```bash
cargo run --example phala_sequencer_example --features phala
```

### 4. Deploy

```bash
# Generate deployment script
cargo run --bin phala-deploy -- --config phala.toml

# Execute deployment
./deploy.sh
```

## Architecture

See [Phala Integration Documentation](../../docs/architecture/phala_integration.md)

## Deployment

See [Phala Deployment Guide](../../docs/deployment/PHALA_TEE.md)

## API Usage

### Initialize Sequencer

```rust
use qrms::phala_tee::PhalaTeeSequencer;

let mut sequencer = PhalaTeeSequencer::new(
    "worker_id".to_string(),
    "enclave_id".to_string(),
    "TDX".to_string(),  // or "SEV"
);
```

### Register Asset

```rust
use qrms::phala_tee::{AssetProtection, AssetType, AccessPolicy, MigrationState};

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

### Submit Transaction

```rust
use qrms::phala_tee::EncryptedTransaction;

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

### Create Batch

```rust
let mut apqc = AdaptivePqcLayer::new();
let tee_key = get_tee_key(); // From Phala TEE

if let Some(batch) = sequencer.create_quantum_batch(&mut apqc, &tee_key).await {
    // Submit batch to L2
    submit_to_l2(batch).await;
}
```

## Intelligence Ordering

```rust
sequencer.intelligence_mode = IntelligenceOrdering::Hybrid;
```

Modes:
- `RiskAware`: Order by risk level
- `AssetProtection`: Prioritize protected assets
- `MigrationAware`: Group migration transactions
- `Hybrid`: Combine all strategies

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

## Documentation

- [Phala Integration](../../docs/architecture/phala_integration.md)
- [Phala Deployment](../../docs/deployment/PHALA_TEE.md)
- [TEE Architecture](../../docs/architecture/phase3_tee.md)
