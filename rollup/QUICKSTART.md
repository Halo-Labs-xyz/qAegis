# QuantumAegis Quick Start

Single command end-to-end setup from `rollup/` directory.

## Complete Stack Startup

```bash
cd rollup
make all
```

This runs:
1. `make init` - Downloads op-deployer, creates .env
2. `make setup` - Deploys L1 contracts, generates L2 config
3. `make up` - Starts L2 (op-geth + op-node) + QRMS service
4. Ready for contract deployment and testing

## Manual Steps

### 1. Initialize (first time only)

```bash
make init
# Edit .env with L1_RPC_URL, L1_BEACON_URL, PRIVATE_KEY
```

### 2. Setup L2

```bash
make setup
```

### 3. Start Stack

```bash
make up
```

Starts:
- L2 RPC: http://localhost:8545
- QRMS API: http://localhost:5050

### 4. Deploy Contracts

```bash
export ADMIN_PRIVATE_KEY="0x..."
export QRM_UPDATER_ADDRESS="0x..."
make deploy-contracts
```

### 5. Test

```bash
make test-l2      # Test L2
make test-qrms    # Test QRMS
make test-e2e     # Full end-to-end test
```

## Service Management

```bash
make status       # Show running services
make logs         # View all logs
make logs-qrms    # View QRMS logs
make down         # Stop everything
```

## Troubleshooting

**L2 not starting:**
```bash
make logs-op-geth
make logs-op-node
```

**QRMS not starting:**
```bash
tail -f /tmp/qrms.log
```

**Reset everything:**
```bash
make reset
make all
```
