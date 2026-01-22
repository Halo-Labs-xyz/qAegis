#!/bin/bash
# ============================================================================
# QRMS OP Stack L2 Rollup Setup Script
# Based on: opstack/docs/create-l2-rollup-example/scripts/setup-rollup.sh
# ============================================================================

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Configuration
L1_CHAIN_ID=11155111
L2_CHAIN_ID_DECIMAL=${L2_CHAIN_ID:-42069}
L2_CHAIN_ID=$(printf "0x%064x" "$L2_CHAIN_ID_DECIMAL")
P2P_ADVERTISE_IP=${P2P_ADVERTISE_IP:-127.0.0.1}

WORKSPACE_DIR="$(pwd)"
DEPLOYER_DIR="$WORKSPACE_DIR/deployer"
SEQUENCER_DIR="$WORKSPACE_DIR/sequencer"
BATCHER_DIR="$WORKSPACE_DIR/batcher"
PROPOSER_DIR="$WORKSPACE_DIR/proposer"
CHALLENGER_DIR="$WORKSPACE_DIR/challenger"
DISPUTE_MON_DIR="$WORKSPACE_DIR/dispute-mon"

check_prerequisites() {
    log_info "Checking prerequisites..."
    if ! command -v op-deployer &>/dev/null && [ ! -f "./op-deployer" ]; then
        log_error "op-deployer not found. Run: make download"
        exit 1
    fi
    log_success "Prerequisites OK"
}

validate_env() {
    log_info "Validating .env..."
    [ ! -f ".env" ] && { log_error ".env not found. Copy env.example to .env"; exit 1; }
    set -a; source .env; set +a
    [ -z "$L1_RPC_URL" ] && { log_error "L1_RPC_URL not set"; exit 1; }
    [ -z "$L1_BEACON_URL" ] && { log_error "L1_BEACON_URL not set"; exit 1; }
    [ -z "$PRIVATE_KEY" ] && { log_error "PRIVATE_KEY not set"; exit 1; }
    log_success ".env validated"
}

generate_addresses() {
    log_info "Generating addresses..."
    mkdir -p "$DEPLOYER_DIR/addresses"
    cd "$DEPLOYER_DIR/addresses"

    for role in admin base_fee_vault_recipient l1_fee_vault_recipient sequencer_fee_vault_recipient system_config unsafe_block_signer batcher proposer challenger; do
        private_key=$(openssl rand -hex 32)
        address="0x$(echo "$private_key" | head -c 40)"
        echo "$address" > "${role}_address.txt"
        log_info "  $role: $address"
    done

    cd "$WORKSPACE_DIR"
    log_success "Addresses generated"
}

init_deployer() {
    log_info "Initializing op-deployer..."
    cd "$DEPLOYER_DIR"

    cat > .env << EOF
L1_RPC_URL=$L1_RPC_URL
PRIVATE_KEY=$PRIVATE_KEY
EOF

    rm -rf .deployer

    if [ -f "$WORKSPACE_DIR/op-deployer" ]; then
        "$WORKSPACE_DIR/op-deployer" init \
            --l1-chain-id $L1_CHAIN_ID \
            --l2-chain-ids "$L2_CHAIN_ID_DECIMAL" \
            --workdir .deployer \
            --intent-type standard
    else
        op-deployer init \
            --l1-chain-id $L1_CHAIN_ID \
            --l2-chain-ids "$L2_CHAIN_ID_DECIMAL" \
            --workdir .deployer \
            --intent-type standard
    fi

    cd "$WORKSPACE_DIR"
    log_success "op-deployer initialized"
}

update_intent() {
    log_info "Updating intent..."
    cd "$DEPLOYER_DIR"

    ADMIN_ADDR=$(cat addresses/admin_address.txt)
    BASE_FEE_VAULT_ADDR=$(cat addresses/base_fee_vault_recipient_address.txt)
    L1_FEE_VAULT_ADDR=$(cat addresses/l1_fee_vault_recipient_address.txt)
    SEQUENCER_FEE_VAULT_ADDR=$(cat addresses/sequencer_fee_vault_recipient_address.txt)
    SYSTEM_CONFIG_ADDR=$(cat addresses/system_config_address.txt)
    UNSAFE_BLOCK_SIGNER_ADDR=$(cat addresses/unsafe_block_signer_address.txt)
    BATCHER_ADDR=$(cat addresses/batcher_address.txt)
    PROPOSER_ADDR=$(cat addresses/proposer_address.txt)
    CHALLENGER_ADDR=$(cat addresses/challenger_address.txt)

    L2_CHAIN_ID_HEX=$(printf "0x%064x" "$L2_CHAIN_ID_DECIMAL")
    sed -i.bak "s|id = .*|id = \"$L2_CHAIN_ID_HEX\"|" .deployer/intent.toml
    sed -i.bak "s|baseFeeVaultRecipient = .*|baseFeeVaultRecipient = \"$BASE_FEE_VAULT_ADDR\"|" .deployer/intent.toml
    sed -i.bak "s|l1FeeVaultRecipient = .*|l1FeeVaultRecipient = \"$L1_FEE_VAULT_ADDR\"|" .deployer/intent.toml
    sed -i.bak "s|sequencerFeeVaultRecipient = .*|sequencerFeeVaultRecipient = \"$SEQUENCER_FEE_VAULT_ADDR\"|" .deployer/intent.toml
    sed -i.bak "s|operatorFeeVaultRecipient = .*|operatorFeeVaultRecipient = \"$ADMIN_ADDR\"|" .deployer/intent.toml
    sed -i.bak "s|chainFeesRecipient = .*|chainFeesRecipient = \"$ADMIN_ADDR\"|" .deployer/intent.toml
    sed -i.bak "s|liquidityControllerOwner = .*|liquidityControllerOwner = \"$ADMIN_ADDR\"|" .deployer/intent.toml
    sed -i.bak "s|systemConfigOwner = .*|systemConfigOwner = \"$SYSTEM_CONFIG_ADDR\"|" .deployer/intent.toml
    sed -i.bak "s|unsafeBlockSigner = .*|unsafeBlockSigner = \"$UNSAFE_BLOCK_SIGNER_ADDR\"|" .deployer/intent.toml
    sed -i.bak "s|batcher = .*|batcher = \"$BATCHER_ADDR\"|" .deployer/intent.toml
    sed -i.bak "s|proposer = .*|proposer = \"$PROPOSER_ADDR\"|" .deployer/intent.toml
    sed -i.bak "s|challenger = .*|challenger = \"$CHALLENGER_ADDR\"|" .deployer/intent.toml
    sed -i.bak "s|fundDevAccounts = .*|fundDevAccounts = true|" .deployer/intent.toml
    # Disable revenue share to avoid zero address issues
    sed -i.bak "s|useRevenueShare = .*|useRevenueShare = false|" .deployer/intent.toml
    
    # Remove customGasToken section (use ETH as native token)
    sed -i.bak '/\[chains.customGasToken\]/,/^$/d' .deployer/intent.toml

    cd "$WORKSPACE_DIR"
    log_success "Intent updated"
}

deploy_contracts() {
    log_info "Deploying L1 contracts..."
    cd "$DEPLOYER_DIR"

    if [ -f "$WORKSPACE_DIR/op-deployer" ]; then
        "$WORKSPACE_DIR/op-deployer" apply \
            --workdir .deployer \
            --l1-rpc-url "$L1_RPC_URL" \
            --private-key "$PRIVATE_KEY"
    else
        op-deployer apply \
            --workdir .deployer \
            --l1-rpc-url "$L1_RPC_URL" \
            --private-key "$PRIVATE_KEY"
    fi

    cd "$WORKSPACE_DIR"
    log_success "L1 contracts deployed"
}

generate_config() {
    log_info "Generating chain config..."
    cd "$DEPLOYER_DIR"

    if [ -f "$WORKSPACE_DIR/op-deployer" ]; then
        "$WORKSPACE_DIR/op-deployer" inspect genesis --workdir .deployer "$L2_CHAIN_ID_DECIMAL" > .deployer/genesis.json
        "$WORKSPACE_DIR/op-deployer" inspect rollup --workdir .deployer "$L2_CHAIN_ID_DECIMAL" > .deployer/rollup.json
    else
        op-deployer inspect genesis --workdir .deployer "$L2_CHAIN_ID_DECIMAL" > .deployer/genesis.json
        op-deployer inspect rollup --workdir .deployer "$L2_CHAIN_ID_DECIMAL" > .deployer/rollup.json
    fi

    cd "$WORKSPACE_DIR"
    log_success "Config generated"
}

setup_sequencer() {
    log_info "Setting up sequencer..."
    mkdir -p "$SEQUENCER_DIR"
    cd "$SEQUENCER_DIR"

    cp "$DEPLOYER_DIR/.deployer/genesis.json" .
    cp "$DEPLOYER_DIR/.deployer/rollup.json" .

    openssl rand -hex 32 > jwt.txt
    chmod 600 jwt.txt

    cat > .env << EOF
L1_RPC_URL=$L1_RPC_URL
L1_BEACON_URL=$L1_BEACON_URL
PRIVATE_KEY=$PRIVATE_KEY
P2P_ADVERTISE_IP=$P2P_ADVERTISE_IP
L2_CHAIN_ID=$L2_CHAIN_ID_DECIMAL
EOF

    cd "$WORKSPACE_DIR"
    log_success "Sequencer ready"
}

setup_batcher() {
    log_info "Setting up batcher..."
    mkdir -p "$BATCHER_DIR"
    cd "$BATCHER_DIR"

    cp "$DEPLOYER_DIR/.deployer/state.json" .
    INBOX_ADDRESS=$(jq -r '.opChainDeployments[0].SystemConfigProxy' state.json)

    cat > .env << EOF
OP_BATCHER_L2_ETH_RPC=http://op-geth:8545
OP_BATCHER_ROLLUP_RPC=http://op-node:8547
OP_BATCHER_PRIVATE_KEY=$PRIVATE_KEY
OP_BATCHER_POLL_INTERVAL=1s
OP_BATCHER_SUB_SAFETY_MARGIN=6
OP_BATCHER_NUM_CONFIRMATIONS=1
OP_BATCHER_SAFE_ABORT_NONCE_TOO_LOW_COUNT=3
OP_BATCHER_INBOX_ADDRESS=$INBOX_ADDRESS
EOF

    cd "$WORKSPACE_DIR"
    log_success "Batcher ready"
}

setup_proposer() {
    log_info "Setting up proposer..."
    mkdir -p "$PROPOSER_DIR"
    cd "$PROPOSER_DIR"

    cp "$DEPLOYER_DIR/.deployer/state.json" .
    GAME_FACTORY_ADDR=$(jq -r '.opChainDeployments[0].DisputeGameFactoryProxy' state.json)

    cat > .env << EOF
OP_PROPOSER_GAME_FACTORY_ADDRESS=$GAME_FACTORY_ADDR
OP_PROPOSER_PRIVATE_KEY=$PRIVATE_KEY
OP_PROPOSER_POLL_INTERVAL=20s
OP_PROPOSER_GAME_TYPE=0
OP_PROPOSER_PROPOSAL_INTERVAL=3600s
EOF

    cd "$WORKSPACE_DIR"
    log_success "Proposer ready"
}

setup_challenger() {
    log_info "Setting up challenger..."
    mkdir -p "$CHALLENGER_DIR"
    cd "$CHALLENGER_DIR"

    cp "$DEPLOYER_DIR/.deployer/genesis.json" .
    cp "$DEPLOYER_DIR/.deployer/rollup.json" .

    GAME_FACTORY_ADDR=$(jq -r '.opChainDeployments[0].DisputeGameFactoryProxy' "$DEPLOYER_DIR/.deployer/state.json")

    cat > .env << EOF
OP_CHALLENGER_GAME_FACTORY_ADDRESS=$GAME_FACTORY_ADDR
OP_CHALLENGER_PRIVATE_KEY=$PRIVATE_KEY
EOF

    cd "$WORKSPACE_DIR"
    log_success "Challenger ready"
}

setup_dispute_monitor() {
    log_info "Setting up dispute monitor..."
    mkdir -p "$DISPUTE_MON_DIR"
    cd "$DISPUTE_MON_DIR"

    GAME_FACTORY_ADDRESS=$(jq -r '.opChainDeployments[0].DisputeGameFactoryProxy' "$DEPLOYER_DIR/.deployer/state.json")
    PROPOSER_ADDRESS=$(jq -r '.appliedIntent.chains[0].roles.proposer' "$DEPLOYER_DIR/.deployer/state.json")
    CHALLENGER_ADDRESS=$(jq -r '.appliedIntent.chains[0].roles.challenger' "$DEPLOYER_DIR/.deployer/state.json")

    cat > .env << EOF
ROLLUP_RPC=http://op-node:8547
OP_DISPUTE_MON_GAME_FACTORY_ADDRESS=$GAME_FACTORY_ADDRESS
PROPOSER_ADDRESS=$PROPOSER_ADDRESS
CHALLENGER_ADDRESS=$CHALLENGER_ADDRESS
OP_DISPUTE_MON_NETWORK=op-sepolia
OP_DISPUTE_MON_MONITOR_INTERVAL=10s
EOF

    mkdir -p logs
    cd "$WORKSPACE_DIR"
    log_success "Dispute monitor ready"
}

main() {
    log_info "=== QRMS OP Stack L2 Rollup Setup ==="
    log_info "L2 Chain ID: $L2_CHAIN_ID_DECIMAL"

    rm -rf "$DEPLOYER_DIR"
    mkdir -p "$DEPLOYER_DIR"

    validate_env
    check_prerequisites
    generate_addresses
    init_deployer
    update_intent
    deploy_contracts
    generate_config
    setup_sequencer
    setup_batcher
    setup_proposer
    setup_challenger
    setup_dispute_monitor

    log_success "=== Setup Complete ==="
    log_info "Run: docker-compose up -d"
    log_info "QRMS Dashboard: http://localhost:5050"
    log_info "L2 RPC: http://localhost:8545"
    log_info "Metrics: http://localhost:7300/metrics"
}

main "$@"
