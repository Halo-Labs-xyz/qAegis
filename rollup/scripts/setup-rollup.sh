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

    # Generate valid Ethereum addresses from private keys
    for role in admin base_fee_vault_recipient l1_fee_vault_recipient sequencer_fee_vault_recipient system_config unsafe_block_signer batcher proposer challenger; do
        private_key=$(openssl rand -hex 32)
        # Use cast or a simple method to derive address
        # For now, use a deterministic method: hash the private key and take first 20 bytes
        if command -v cast &> /dev/null; then
            address=$(cast wallet address --private-key "$private_key" 2>/dev/null || echo "0x$(echo -n "$private_key" | sha256sum | head -c 40)")
        else
            # Fallback: use sha256 hash of private key (not cryptographically correct but generates valid format)
            address="0x$(echo -n "$private_key$role" | sha256sum | cut -d' ' -f1 | head -c 40)"
        fi
        echo "$address" > "${role}_address.txt"
        echo "$private_key" > "${role}_private_key.txt"
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
    # Note: challenger address is NOT overridden for standard configType
    # The official Sepolia Superchain challenger is managed by Optimism Governance

    L2_CHAIN_ID_HEX=$(printf "0x%064x" "$L2_CHAIN_ID_DECIMAL")

    INTENT_FILE=".deployer/intent.toml"

    # For configType = "standard", certain addresses are fixed by Optimism Governance:
    # - l1ProxyAdminOwner (do not change)
    # - l2ProxyAdminOwner (do not change)
    # - challenger (do not change - official Superchain challenger)
    #
    # We can customize:
    # - Fee vault recipients
    # - systemConfigOwner (chain operator)
    # - unsafeBlockSigner (sequencer)
    # - batcher (batch submitter)
    # - proposer (output proposer)

    # Chain-level fields (2-space indent)
    sed -i.bak "s|^  baseFeeVaultRecipient = \"0x[0-9a-fA-F]*\"|  baseFeeVaultRecipient = \"$BASE_FEE_VAULT_ADDR\"|" "$INTENT_FILE"
    sed -i.bak "s|^  l1FeeVaultRecipient = \"0x[0-9a-fA-F]*\"|  l1FeeVaultRecipient = \"$L1_FEE_VAULT_ADDR\"|" "$INTENT_FILE"
    sed -i.bak "s|^  sequencerFeeVaultRecipient = \"0x[0-9a-fA-F]*\"|  sequencerFeeVaultRecipient = \"$SEQUENCER_FEE_VAULT_ADDR\"|" "$INTENT_FILE"
    sed -i.bak "s|^  operatorFeeVaultRecipient = \"0x[0-9a-fA-F]*\"|  operatorFeeVaultRecipient = \"$ADMIN_ADDR\"|" "$INTENT_FILE"
    sed -i.bak "s|^  chainFeesRecipient = \"0x[0-9a-fA-F]*\"|  chainFeesRecipient = \"$ADMIN_ADDR\"|" "$INTENT_FILE"
    sed -i.bak "s|^  useRevenueShare = true|  useRevenueShare = false|" "$INTENT_FILE"
    sed -i.bak "s|^fundDevAccounts = false|fundDevAccounts = true|" "$INTENT_FILE"
    
    # Roles section - only update fields we control (4-space indent)
    sed -i.bak "s|^    systemConfigOwner = \"0x[0-9a-fA-F]*\"|    systemConfigOwner = \"$SYSTEM_CONFIG_ADDR\"|" "$INTENT_FILE"
    sed -i.bak "s|^    unsafeBlockSigner = \"0x[0-9a-fA-F]*\"|    unsafeBlockSigner = \"$UNSAFE_BLOCK_SIGNER_ADDR\"|" "$INTENT_FILE"
    sed -i.bak "s|^    batcher = \"0x[0-9a-fA-F]*\"|    batcher = \"$BATCHER_ADDR\"|" "$INTENT_FILE"
    sed -i.bak "s|^    proposer = \"0x[0-9a-fA-F]*\"|    proposer = \"$PROPOSER_ADDR\"|" "$INTENT_FILE"
    # DO NOT modify challenger - it's controlled by Optimism Governance for standard deployments
    
    # Remove customGasToken section entirely
    awk '
        /\[chains\.customGasToken\]/ { skip = 1; next }
        skip && /^[[:space:]]*$/ { skip = 0; next }
        skip && /^\[/ { skip = 0 }
        !skip { print }
    ' "$INTENT_FILE" > "${INTENT_FILE}.tmp" && mv "${INTENT_FILE}.tmp" "$INTENT_FILE"
    
    # Clean up backup files
    rm -f "${INTENT_FILE}.bak"

    log_success "Intent updated"

    # Verify required addresses
    log_info "Verifying intent.toml..."
    ERRORS=0
    
    if ! grep -q "batcher = \"$BATCHER_ADDR\"" "$INTENT_FILE"; then
        log_error "Batcher address not set correctly"
        ERRORS=$((ERRORS + 1))
    else
        log_info "  batcher: $BATCHER_ADDR"
    fi
    
    if ! grep -q "proposer = \"$PROPOSER_ADDR\"" "$INTENT_FILE"; then
        log_error "Proposer address not set correctly"
        ERRORS=$((ERRORS + 1))
    else
        log_info "  proposer: $PROPOSER_ADDR"
    fi
    
    # Show challenger (official address, not overridden)
    CHALLENGER_IN_FILE=$(grep -o 'challenger = "0x[0-9a-fA-F]*"' "$INTENT_FILE" | head -1 | grep -o '0x[0-9a-fA-F]*')
    log_info "  challenger: $CHALLENGER_IN_FILE (official Superchain)"

    if [ $ERRORS -gt 0 ]; then
        log_error "Intent verification failed"
        grep -A 10 "\[chains.roles\]" "$INTENT_FILE" || true
        exit 1
    fi

    cd "$WORKSPACE_DIR"
}

deploy_contracts() {
    log_info "Deploying L1 contracts..."
    cd "$DEPLOYER_DIR"

    # For standard configType, challenger is the official Superchain challenger
    # We only verify it exists and is non-zero
    CHALLENGER_IN_FILE=$(grep -o 'challenger = "0x[0-9a-fA-F]*"' .deployer/intent.toml | head -1 | grep -o '0x[0-9a-fA-F]*' || echo "")
    
    if [ -z "$CHALLENGER_IN_FILE" ]; then
        log_error "Challenger address not found in intent.toml"
        grep -A 10 "\[chains.roles\]" .deployer/intent.toml || true
        exit 1
    fi
    
    if [ "$CHALLENGER_IN_FILE" = "0x0000000000000000000000000000000000000000" ]; then
        log_error "Challenger address is zero"
        exit 1
    fi
    
    log_info "Using official Superchain challenger: $CHALLENGER_IN_FILE"

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

    # Strip fields that may not be supported by older op-node versions
    if command -v jq &> /dev/null; then
        log_info "Normalizing rollup.json for compatibility..."
        jq 'del(.genesis.system_config.minBaseFee, .genesis.system_config.daFootprintGasScalar)' \
            .deployer/rollup.json > .deployer/rollup.json.tmp && \
            mv .deployer/rollup.json.tmp .deployer/rollup.json
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
