# Phala TEE Sequencer Quick Start

## 5-Minute Setup

### 1. Install Dependencies

```bash
# Install Phala CLI (if not already installed)
cargo install phala-cli

# Install Rust dependencies
cd services/qrms
cargo build --features phala
```

### 2. Configure

```bash
# Copy configuration template
cp phala.toml phala.local.toml

# Edit with your settings
# - Set network (testnet for testing)
# - Configure worker requirements
# - Set risk thresholds
```

### 3. Run Example

```bash
# Run example integration
cargo run --example phala_sequencer_example --features phala
```

### 4. Deploy (Testnet)

```bash
# Generate deployment script
cargo run --bin phala-deploy --features phala -- --config phala.local.toml > deploy.sh
chmod +x deploy.sh

# Deploy to Phala testnet
./deploy.sh
```

## Basic Usage

```rust
use qrms::phala_tee::PhalaTeeSequencer;
use qrms::apqc::AdaptivePqcLayer;

// Initialize
let mut sequencer = PhalaTeeSequencer::new(
    "worker_001".to_string(),
    "enclave_001".to_string(),
    "TDX".to_string(),
);

let mut apqc = AdaptivePqcLayer::new();

// Register asset
sequencer.register_asset(asset_protection);

// Submit transaction
sequencer.submit_encrypted(encrypted_tx);

// Create batch
let batch = sequencer.create_quantum_batch(&mut apqc, tee_key).await;
```

## Key Features

✅ **Quantum-Resistant**: ML-DSA-87 + SLH-DSA-256s dual signatures  
✅ **Asset Protection**: On-chain and off-chain data protection  
✅ **Intelligence Ordering**: Risk-aware transaction sequencing  
✅ **State Migration**: Seamless upgrades with checkpointing  
✅ **TEE Security**: Hardware-backed Phala Network enclaves  

## Documentation

- Full Guide: [Phala Deployment](../../docs/deployment/PHALA_TEE.md)
- Architecture: [Phala Integration](../../docs/architecture/phala_integration.md)
- Example: [phala_sequencer_example.rs](./examples/phala_sequencer_example.rs)
