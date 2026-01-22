# QRMS OP Stack L2 Rollup

OP Stack L2 testnet with QRMS (Quantum Resistance Model System) integration.

Based on: `opstack/docs/create-l2-rollup-example/`

## Quick Start

```bash
cd rollup

# 1. Initialize
make init

# 2. Configure
# Edit .env with L1_RPC_URL, L1_BEACON_URL, PRIVATE_KEY

# 3. Setup (deploys L1 contracts, generates configs)
make setup

# 4. Start
make up

# 5. Verify
make test-l2      # L2 RPC
make test-qrms    # QRMS API
```

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  QRMS (Rust)                              Port 5050: REST + WS              │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                                   │
│  │   QRM    │──│   APQC   │──│   SEQ    │                                   │
│  │ 12 cats  │  │ Dual PQC │  │ TEE Sim  │                                   │
│  └──────────┘  └──────────┘  └──────────┘                                   │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
┌───────────────────────────────────┼─────────────────────────────────────────┐
│  OP Stack L2                      │                                          │
│  ┌──────────┐  ┌──────────┐  ┌────┴─────┐  ┌──────────┐  ┌──────────┐      │
│  │ op-geth  │──│ op-node  │──│ batcher  │──│ proposer │──│challenger│      │
│  │   8545   │  │   8547   │  │          │  │          │  │          │      │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘  └──────────┘      │
│                     │                                                        │
│  ┌──────────────────┴───────────────────────────────────────────────────┐   │
│  │  Contracts: PQCVerifier, QRMSOracle, SequencerAttestation            │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │ L1 DA
                                    ▼
                           ┌─────────────────┐
                           │    Sepolia      │
                           └─────────────────┘
```

## Directory Structure

```
rollup/
├── Makefile                 # Commands
├── docker-compose.yml       # All services
├── env.example              # Environment template
├── op-deployer              # Binary (after make download)
├── deployer/                # Contract deployments
├── sequencer/               # op-geth + op-node config
├── batcher/                 # op-batcher config
├── proposer/                # op-proposer config
├── challenger/              # op-challenger config
├── dispute-mon/             # Dispute monitor config
├── contracts/               # QRMS Solidity
│   └── src/
│       ├── PQCVerifier.sol
│       ├── QRMSOracle.sol
│       └── SequencerAttestation.sol
├── scripts/
│   ├── download-op-deployer.sh
│   └── setup-rollup.sh
└── docker/
    └── Dockerfile.qrms
```

## Services

| Service | Port | Description |
|---------|------|-------------|
| qrms | 5050 | QRMS REST + WebSocket |
| op-geth | 8545 | L2 JSON-RPC |
| op-node | 8547 | Rollup RPC |
| dispute-mon | 7300 | Metrics |

## Commands

```bash
make help            # Show all commands
make init            # Download op-deployer, create .env
make setup           # Full deployment
make up              # Start services
make down            # Stop services
make logs            # View logs
make logs-qrms       # QRMS logs
make status          # Service status
make test-l1         # Test L1
make test-l2         # Test L2
make test-qrms       # Test QRMS
make cli             # Launch QRMS CLI
make deploy-contracts # Deploy QRMS contracts
make clean           # Remove containers
make reset           # Full reset
```

## Smart Contracts

### PQCVerifier

On-chain PQC signature verification. Simulates precompiles until custom op-geth fork is deployed.

| Function | Gas | Description |
|----------|-----|-------------|
| `verifyDual()` | 65,000 | Verify ML-DSA + SLH-DSA (AND) |
| `verifyDualOr()` | 15,000-65,000 | Verify either (OR) |
| `verifyMldsa()` | 15,000 | ML-DSA-87 only |
| `verifySlhdsa()` | 50,000 | SLH-DSA-256s only |

### QRMSOracle

On-chain risk score and algorithm rotation.

| Function | Description |
|----------|-------------|
| `updateRiskScore()` | Update overall risk (QRM agent) |
| `updateCategoryRisk()` | Update per-category risk |
| `scheduleRotation()` | Schedule algorithm rotation |
| `executeRotation()` | Execute after grace period |
| `emergencyRotation()` | Immediate rotation (risk >= 9000) |

### SequencerAttestation

TEE batch verification.

| Function | Description |
|----------|-------------|
| `registerSequencer()` | Register sequencer with MRENCLAVE |
| `verifyAttestation()` | Verify batch attestation |
| `getBatchHash()` | Get verified batch hash |

## Configuration

### Environment Variables

```bash
# L1 (Required)
L1_RPC_URL=https://sepolia.infura.io/v3/YOUR_KEY
L1_CHAIN_ID=11155111

# L2
L2_CHAIN_ID=42069
L2_BLOCK_TIME=2

# QRMS
QRM_RISK_THRESHOLD_SCHEDULED=6000
QRM_RISK_THRESHOLD_EMERGENCY=9000

# Keys (generate with make generate-keys)
ADMIN_PRIVATE_KEY=0x...
BATCHER_PRIVATE_KEY=0x...
PROPOSER_PRIVATE_KEY=0x...
```

### Funding Requirements (Sepolia ETH)

| Role | Amount | Purpose |
|------|--------|---------|
| Admin | 2 ETH | Contract deployment |
| Batcher | 1 ETH | L1 data posting |
| Proposer | 0.5 ETH | State root proposals |
| Challenger | 0.5 ETH | Dispute resolution |
| Sequencer | 0.1 ETH | Sequencing ops |

## Development Workflow

### 1. Local Testing

```bash
# Start QRMS standalone (no L2)
cd ../qrms-rust
./target/release/qrms &
./target/release/qrms-cli
```

### 2. Testnet Deployment

```bash
cd rollup
make init
# Configure .env
make setup
make up
```

### 3. Contract Interaction

```bash
# Check risk score
cast call $QRMS_ORACLE "getRiskScore()" --rpc-url $L2_RPC_URL

# Check algorithms
cast call $QRMS_ORACLE "getCurrentAlgorithms()" --rpc-url $L2_RPC_URL

# Send test transaction
cast send --private-key $SEQUENCER_PRIVATE_KEY \
  --rpc-url $L2_RPC_URL \
  --value 0.01ether \
  $RECIPIENT
```

## Monitoring

### QRMS CLI

```bash
make cli
# Keys: Tab (switch), s (start), x (stop), h (inject threats), q (quit)
```

### Logs

```bash
# All services
docker compose -f docker-compose.qrms.yml -f docker-compose.l2.yml logs -f

# Specific service
docker logs -f qrms
docker logs -f op-geth
docker logs -f op-node
```

### Metrics

| Service | Endpoint |
|---------|----------|
| op-geth | http://localhost:6060/debug/metrics |
| op-node | http://localhost:7300/metrics |
| op-batcher | http://localhost:7301/metrics |

## Troubleshooting

### Common Issues

**L1 connection failed**
```bash
make test-l1
# Verify L1_RPC_URL in .env
```

**L2 not producing blocks**
```bash
docker logs op-node --tail=100
# Check L1 sync status, batcher funding
```

**QRMS not responding**
```bash
docker logs qrms --tail=100
curl http://localhost:5050/api/status
```

**Contract deployment failed**
```bash
# Check L2 is synced
cast block-number --rpc-url $L2_RPC_URL
# Check account balance
cast balance $ADMIN_ADDRESS --rpc-url $L2_RPC_URL
```

## References

- [OP Stack Documentation](https://docs.optimism.io/)
- [op-deployer Tutorial](https://docs.optimism.io/chain-operators/tutorials/create-l2-rollup/create-l2-rollup)
- [QRMS Architecture](/docs/quantumlit/stack_architecture.md)
- [Threat Taxonomy](/docs/quantumlit/threat_taxonomy.md)
