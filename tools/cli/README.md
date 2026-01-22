# CLI Tools

This directory contains command-line utilities for QuantumAegis.

## Available Tools

- `qrms-cli` - Main QRMS monitoring CLI (located in `services/qrms/src/bin/qrms-cli.rs`)

## Usage

```bash
# Run QRMS CLI
cd services/qrms
cargo run --release --bin qrms-cli
```

## Future Tools

- `keygen` - Generate PQC key pairs
- `verify` - Verify PQC signatures
- `monitor` - Monitor threat indicators
