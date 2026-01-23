# End-to-End Testing Guide

This guide walks through testing the complete QuantumAegis system.

## Prerequisites

```bash
# Required tools
docker --version
docker-compose --version
cargo --version
forge --version
jq --version
curl --version
```

## Quick Test (Simple)

Test what's currently running:

```bash
bash scripts/test-e2e-simple.sh
```

## Full End-to-End Test

### Step 1: Start L2 Rollup

```bash
cd rollup

# If first time, initialize
make init

# Edit .env with your L1_RPC_URL and PRIVATE_KEY
# Then setup (deploys L1 contracts)
make setup

# Start L2 services
make up

# Verify L2 is running
make test-l2
```

Expected output:
```
Testing L2...
0x1234
L2 OK
```

### Step 2: Deploy QRMS Contracts

```bash
cd contracts

# Set environment variables
export ADMIN_PRIVATE_KEY="0x..."  # Your private key
export QRM_UPDATER_ADDRESS="0x..." # Address that can update QRM

# Deploy
forge script script/Deploy.s.sol \
    --rpc-url http://localhost:8545 \
    --broadcast \
    --private-key "$ADMIN_PRIVATE_KEY" \
    --sig "run(address)" "$QRM_UPDATER_ADDRESS"
```

### Step 3: Start QRMS Service

```bash
cd services/qrms

# Build (first time)
cargo build --release

# Run
cargo run --release
```

In another terminal, verify:
```bash
curl http://localhost:5050/api/status | jq .
```

### Step 4: Run Tests

```bash
# Test L2
cd rollup
make test-l2

# Test QRMS
make test-qrms

# Test QVM integration
cd ../services/qrms
cargo test qvm

# Run simple E2E test
cd ../..
bash scripts/test-e2e-simple.sh
```

## Manual Testing

### 1. Test L2 RPC

```bash
curl -X POST -H "Content-Type: application/json" \
    --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
    http://localhost:8545 | jq .
```

### 2. Test QRMS API

```bash
# Status
curl http://localhost:5050/api/status | jq .

# QRM History
curl http://localhost:5050/api/qrm/history | jq .

# Blocks
curl http://localhost:5050/api/blocks | jq .
```

### 3. Test Threat Injection

```bash
curl -X POST http://localhost:5050/api/inject_threat \
    -H "Content-Type: application/json" \
    -d '{
        "category": "digital_signatures",
        "severity": 0.8,
        "description": "Test threat from E2E test"
    }' | jq .

# Check updated status
sleep 2
curl http://localhost:5050/api/status | jq .
```

### 4. Test QVM Integration

```bash
cd services/qrms

# Run QVM tests
cargo test qvm

# Run qubit picker tests
cargo test qubit_picker
```

### 5. Test Web Dashboard

Open browser:
```
http://localhost:5050
```

You should see the QRMS dashboard with:
- Current risk score
- Threat indicators
- Block history
- Real-time updates

## Troubleshooting

### L2 Not Starting

```bash
cd rollup

# Check logs
docker-compose logs op-geth
docker-compose logs op-node

# Restart
docker-compose restart

# Full reset (WARNING: deletes data)
make clean
make setup
make up
```

### QRMS Not Starting

```bash
cd services/qrms

# Check for compilation errors
cargo check

# Check if port 5050 is in use
lsof -i :5050

# Run with verbose logging
RUST_LOG=debug cargo run --release
```

### Contracts Not Deploying

```bash
cd contracts

# Check L2 is running
curl -X POST -H "Content-Type: application/json" \
    --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
    http://localhost:8545

# Check deployer has funds
cast balance <DEPLOYER_ADDRESS> --rpc-url http://localhost:8545

# If no funds, fund the account (L2 should have pre-funded accounts)
```

## Expected Results

### Successful E2E Test Output

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  1. TESTING L2 ROLLUP
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[TEST] L2 RPC endpoint
[PASS] L2 RPC responding (block: 42)
[TEST] L2 Chain ID
[PASS] L2 Chain ID: 0xa455

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  2. TESTING QRMS SERVICE
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[TEST] QRMS Status endpoint
[PASS] QRMS status endpoint responding
[TEST] QRMS QRM History
[PASS] QRM history endpoint responding (entries: 5)
[TEST] QRMS Blocks endpoint
[PASS] Blocks endpoint responding

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  3. TESTING QVM INTEGRATION
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[TEST] Running QVM unit tests
[PASS] QVM tests passed

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  TEST SUMMARY
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Tests Passed: 6
Tests Failed: 0

✓ All tests passed!
```

## Next Steps

After successful E2E test:

1. **Monitor Services**
   ```bash
   # L2 logs
   cd rollup && docker-compose logs -f
   
   # QRMS logs
   # (if running in foreground, logs appear in terminal)
   ```

2. **View Dashboard**
   - Open http://localhost:5050
   - Monitor real-time threat indicators
   - Watch risk score updates

3. **Inject Threats**
   ```bash
   curl -X POST http://localhost:5050/api/inject_high_threat
   ```

4. **Check QVM Oracle**
   - QVM runs automatically in QRMS
   - Check logs for oracle assessments
   - Monitor era transitions
