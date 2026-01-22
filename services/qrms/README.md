# QRMS Service

Quantum Resistance Model System - Core monitoring and PQC service.

## Features

- Real PQC signatures (ML-DSA-87, SLH-DSA-256s)
- Hybrid ECDSA + PQC dual signatures
- 12-category threat monitoring
- Adaptive algorithm rotation
- Web GUI and CLI interfaces

## Usage

```bash
# Run server
cargo run --release

# Run CLI monitor
cargo run --release --bin qrms-cli
```

## API

- REST: `http://localhost:5050/api/status`
- WebSocket: `ws://localhost:5050/ws`
- GUI: `http://localhost:5050`
