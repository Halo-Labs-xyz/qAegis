# QuantumAegis Deployment Status

## L2 Testnet (Sepolia)

**Chain ID**: 42069  
**RPC**: `http://localhost:8545`  
**Status**: Operational

### Deployed Contracts

| Contract | Address | Purpose |
|----------|---------|---------|
| **PQCVerifier** | `0x5cB41403BEbF7E69EA07Db1A2879227d0162f07E` | On-chain PQC signature verification |
| **QRMSOracle** | `0x73dC7846190C3CfcE831bA67Aa8fF14b812078B7` | Risk scores & algorithm state |
| **SequencerAttestation** | `0xeF6eD4863b0FCF6e77d6E6B81E192cF374D2Df21` | TEE attestation verification |

### L1 Contracts (Sepolia)

| Contract | Address | Purpose |
|----------|---------|---------|
| **SystemConfig** | `0xbfd1e2e4ead4c5bf032312535a3fc1a846f4d0c5` | L2 system configuration |
| **OptimismPortal** | `0x3172a5e22803612f14ac58f622e650cc4c647c47` | L1→L2 deposits |
| **DisputeGameFactory** | `0x1f3be7189674f48871e138802982297dfda9d0d8` | Fault proofs |
| **Batch Inbox** | `0x00e9bfcadbfb1f294e9a66bc0573878525f5015c` | L2 batch submissions |

## Services

| Service | Port | Status |
|---------|------|--------|
| **op-geth** | 8545 | Running |
| **op-node** | 8547 | Running |
| **op-batcher** | - | Submitting batches |
| **QRMS API** | 5050 | Healthy |
| **QRMS gRPC** | 9090 | Running |

## Quick Commands

```bash
# Start all services
cd rollup/opstack && make up

# View logs
docker-compose logs -f

# Check status
curl http://localhost:5050/api/status | jq

# Deploy contracts
cd contracts && forge script script/Deploy.s.sol --rpc-url http://localhost:8545 --broadcast
```
