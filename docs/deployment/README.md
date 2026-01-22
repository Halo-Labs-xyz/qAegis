# QuantumAegis Deployment Guide

## Prerequisites

- Docker & Docker Compose
- Foundry (for contracts)
- Rust toolchain (for services)
- Sepolia ETH for L1 deployment

## Quick Start

### 1. Deploy L2 Rollup

```bash
cd rollup/opstack
cp .example.env .env
# Edit .env with your L1_RPC_URL, L1_BEACON_URL, PRIVATE_KEY

make setup    # Deploy L1 contracts and generate L2 config
make up        # Start L2 chain
```

### 2. Deploy QRMS Contracts

```bash
cd ../../contracts
cp ../rollup/opstack/.env .env
# Update ADMIN_PRIVATE_KEY and QRM_UPDATER_ADDRESS

forge script script/Deploy.s.sol --rpc-url http://localhost:8545 --broadcast
```

### 3. Start QRMS Services

```bash
cd ../services/qrms
cargo build --release
cargo run --release
```

## Service Endpoints

| Service | Endpoint | Description |
|---------|----------|-------------|
| L2 RPC | `http://localhost:8545` | L2 JSON-RPC |
| QRMS API | `http://localhost:5050` | QRMS REST API |
| QRMS WS | `ws://localhost:5050/ws` | Real-time events |
| QRMS GUI | `http://localhost:5050` | Web dashboard |

## Contract Addresses

After deployment, addresses are saved to:
- `rollup/opstack/deployer/.deployer/state.json` (L1 contracts)
- `contracts/broadcast/Deploy.s.sol/42069/run-latest.json` (L2 contracts)
