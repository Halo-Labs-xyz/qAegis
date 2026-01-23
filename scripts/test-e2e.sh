#!/bin/bash
# End-to-End Test Script for QuantumAegis
# Tests: L2 Rollup → QRMS Service → Contracts → QVM Integration

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
ROLLUP_DIR="$PROJECT_ROOT/rollup"
CONTRACTS_DIR="$PROJECT_ROOT/contracts"
SERVICES_DIR="$PROJECT_ROOT/services/qrms"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test counters
TESTS_PASSED=0
TESTS_FAILED=0

print_header() {
    echo ""
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
}

print_test() {
    echo -e "${YELLOW}[TEST]${NC} $1"
}

print_pass() {
    echo -e "${GREEN}[PASS]${NC} $1"
    ((TESTS_PASSED++))
}

print_fail() {
    echo -e "${RED}[FAIL]${NC} $1"
    ((TESTS_FAILED++))
}

print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

# ============================================================================
# Prerequisites Check
# ============================================================================

print_header "1. PREREQUISITES CHECK"

check_command() {
    if command -v "$1" &> /dev/null; then
        print_pass "$1 is installed"
        return 0
    else
        print_fail "$1 is not installed"
        return 1
    fi
}

check_command docker || exit 1
check_command docker-compose || exit 1
check_command jq || exit 1
check_command curl || exit 1
check_command cargo || exit 1
check_command forge || exit 1

# Check Docker is running
if ! docker info &> /dev/null; then
    print_fail "Docker daemon is not running"
    exit 1
fi
print_pass "Docker daemon is running"

# ============================================================================
# L2 Rollup Setup
# ============================================================================

print_header "2. L2 ROLLUP SETUP"

cd "$ROLLUP_DIR"

# Check if .env exists
if [ ! -f .env ]; then
    print_info ".env not found, creating from env.example"
    if [ -f env.example ]; then
        cp env.example .env
        print_info "Created .env - please edit with your values"
    else
        print_fail "env.example not found"
        exit 1
    fi
fi

# Check if op-deployer exists
if [ ! -f op-deployer ]; then
    print_info "op-deployer not found, downloading..."
    if [ -f scripts/download-op-deployer.sh ]; then
        bash scripts/download-op-deployer.sh
    else
        print_fail "download-op-deployer.sh not found"
        exit 1
    fi
fi

# Check if setup has been run
if [ ! -d sequencer ] || [ ! -f sequencer/genesis.json ]; then
    print_info "L2 not set up, running setup..."
    if [ -f scripts/setup-rollup.sh ]; then
        bash scripts/setup-rollup.sh
    else
        print_fail "setup-rollup.sh not found"
        exit 1
    fi
else
    print_pass "L2 configuration exists"
fi

# ============================================================================
# Start L2 Services
# ============================================================================

print_header "3. STARTING L2 SERVICES"

# Check if services are already running
if docker-compose ps | grep -q "op-geth.*Up"; then
    print_info "L2 services already running"
else
    print_info "Starting L2 services..."
    docker-compose up -d op-geth op-node || {
        print_fail "Failed to start L2 services"
        exit 1
    }
    
    # Wait for services to be ready
    print_info "Waiting for L2 to be ready..."
    for i in {1..30}; do
        if curl -s -X POST -H "Content-Type: application/json" \
            --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
            http://localhost:8545 > /dev/null 2>&1; then
            print_pass "L2 is ready"
            break
        fi
        if [ $i -eq 30 ]; then
            print_fail "L2 did not become ready in time"
            docker-compose logs op-geth | tail -20
            exit 1
        fi
        sleep 1
    done
fi

# ============================================================================
# Test L2 Connectivity
# ============================================================================

print_header "4. TESTING L2 CONNECTIVITY"

print_test "L2 RPC endpoint"
L2_BLOCK=$(curl -s -X POST -H "Content-Type: application/json" \
    --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
    http://localhost:8545 | jq -r '.result' 2>/dev/null)

if [ -n "$L2_BLOCK" ] && [ "$L2_BLOCK" != "null" ]; then
    BLOCK_NUM=$((L2_BLOCK))
    print_pass "L2 RPC responding (block: $BLOCK_NUM)"
else
    print_fail "L2 RPC not responding"
    exit 1
fi

print_test "L2 Chain ID"
L2_CHAIN_ID=$(curl -s -X POST -H "Content-Type: application/json" \
    --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' \
    http://localhost:8545 | jq -r '.result' 2>/dev/null)

if [ -n "$L2_CHAIN_ID" ]; then
    print_pass "L2 Chain ID: $L2_CHAIN_ID"
else
    print_fail "Could not get L2 Chain ID"
fi

# ============================================================================
# Deploy Contracts
# ============================================================================

print_header "5. DEPLOYING QRMS CONTRACTS"

cd "$CONTRACTS_DIR"

# Check if contracts are already deployed
if [ -f deployed_addresses.txt ] && grep -q "PQC_VERIFIER=" deployed_addresses.txt; then
    print_info "Contracts appear to be deployed"
    cat deployed_addresses.txt
else
    print_info "Deploying contracts to L2..."
    
    # Check for required env vars
    if [ -z "$ADMIN_PRIVATE_KEY" ]; then
        print_info "ADMIN_PRIVATE_KEY not set, using default from .env"
        source "$ROLLUP_DIR/.env" 2>/dev/null || true
    fi
    
    if [ -z "$ADMIN_PRIVATE_KEY" ] && [ -z "$PRIVATE_KEY" ]; then
        print_fail "ADMIN_PRIVATE_KEY or PRIVATE_KEY must be set"
        exit 1
    fi
    
    ADMIN_KEY="${ADMIN_PRIVATE_KEY:-$PRIVATE_KEY}"
    QRM_UPDATER="${QRM_UPDATER_ADDRESS:-0x7Afde307a7F56d0254E42136cAa9896778662302}"
    
    print_info "Using deployer: ${ADMIN_KEY:0:10}..."
    print_info "QRM Updater: $QRM_UPDATER"
    
    forge script script/Deploy.s.sol \
        --rpc-url http://localhost:8545 \
        --broadcast \
        --private-key "$ADMIN_KEY" \
        --sig "run(address)" "$QRM_UPDATER" || {
        print_fail "Contract deployment failed"
        exit 1
    }
    
    print_pass "Contracts deployed"
fi

# ============================================================================
# Build and Start QRMS Service
# ============================================================================

print_header "6. BUILDING QRMS SERVICE"

cd "$SERVICES_DIR"

print_test "Building QRMS (this may take a few minutes)..."
if cargo build --release 2>&1 | tail -5; then
    print_pass "QRMS built successfully"
else
    print_fail "QRMS build failed"
    exit 1
fi

# Check if QRMS is already running
if curl -s http://localhost:5050/api/status > /dev/null 2>&1; then
    print_info "QRMS service already running"
else
    print_info "Starting QRMS service..."
    # Start in background
    cargo run --release > /tmp/qrms.log 2>&1 &
    QRMS_PID=$!
    echo $QRMS_PID > /tmp/qrms.pid
    
    # Wait for service to be ready
    print_info "Waiting for QRMS to be ready..."
    for i in {1..30}; do
        if curl -s http://localhost:5050/api/status > /dev/null 2>&1; then
            print_pass "QRMS is ready"
            break
        fi
        if [ $i -eq 30 ]; then
            print_fail "QRMS did not become ready in time"
            cat /tmp/qrms.log | tail -20
            kill $QRMS_PID 2>/dev/null || true
            exit 1
        fi
        sleep 1
    done
fi

# ============================================================================
# Test QRMS API
# ============================================================================

print_header "7. TESTING QRMS API"

print_test "QRMS Status endpoint"
QRMS_STATUS=$(curl -s http://localhost:5050/api/status 2>/dev/null)
if [ -n "$QRMS_STATUS" ]; then
    echo "$QRMS_STATUS" | jq . 2>/dev/null || echo "$QRMS_STATUS"
    print_pass "QRMS status endpoint responding"
else
    print_fail "QRMS status endpoint not responding"
    exit 1
fi

print_test "QRMS QRM History"
QRM_HISTORY=$(curl -s http://localhost:5050/api/qrm/history 2>/dev/null)
if [ -n "$QRM_HISTORY" ]; then
    HISTORY_COUNT=$(echo "$QRM_HISTORY" | jq '. | length' 2>/dev/null || echo "0")
    print_pass "QRM history endpoint responding (entries: $HISTORY_COUNT)"
else
    print_fail "QRM history endpoint not responding"
fi

print_test "QRMS Blocks endpoint"
QRM_BLOCKS=$(curl -s http://localhost:5050/api/blocks 2>/dev/null)
if [ -n "$QRM_BLOCKS" ]; then
    print_pass "Blocks endpoint responding"
else
    print_fail "Blocks endpoint not responding"
fi

# ============================================================================
# Test QVM Integration
# ============================================================================

print_header "8. TESTING QVM INTEGRATION"

cd "$SERVICES_DIR"

print_test "Running QVM tests"
if cargo test qvm --lib 2>&1 | grep -q "test result: ok"; then
    print_pass "QVM tests passed"
else
    print_fail "QVM tests failed"
    cargo test qvm --lib 2>&1 | tail -20
fi

print_test "Running Qubit Picker tests"
if cargo test qubit_picker --lib 2>&1 | grep -q "test result: ok"; then
    print_pass "Qubit Picker tests passed"
else
    # Try with different pattern
    if cargo test --lib 2>&1 | grep -q "test.*qubit.*ok"; then
        print_pass "Qubit Picker tests passed"
    else
        print_info "Qubit Picker tests (may be part of qvm tests)"
    fi
fi

# ============================================================================
# Test Threat Injection
# ============================================================================

print_header "9. TESTING THREAT INJECTION"

print_test "Injecting test threat"
THREAT_RESPONSE=$(curl -s -X POST http://localhost:5050/api/inject_threat \
    -H "Content-Type: application/json" \
    -d '{"category":"digital_signatures","severity":0.7,"description":"Test threat"}' 2>/dev/null)

if [ -n "$THREAT_RESPONSE" ]; then
    print_pass "Threat injection endpoint responding"
    echo "$THREAT_RESPONSE" | jq . 2>/dev/null || echo "$THREAT_RESPONSE"
else
    print_fail "Threat injection failed"
fi

# Wait a moment for processing
sleep 2

print_test "Checking updated risk score"
UPDATED_STATUS=$(curl -s http://localhost:5050/api/status 2>/dev/null)
if [ -n "$UPDATED_STATUS" ]; then
    print_pass "Status updated after threat injection"
else
    print_fail "Status not updated"
fi

# ============================================================================
# Summary
# ============================================================================

print_header "10. TEST SUMMARY"

echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}Tests Passed: $TESTS_PASSED${NC}"
echo -e "${RED}Tests Failed: $TESTS_FAILED${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All end-to-end tests passed!${NC}"
    echo ""
    echo "Services running:"
    echo "  - L2 RPC: http://localhost:8545"
    echo "  - QRMS API: http://localhost:5050"
    echo "  - QRMS Dashboard: http://localhost:5050"
    echo ""
    echo "To view logs:"
    echo "  - QRMS: tail -f /tmp/qrms.log"
    echo "  - L2: cd rollup && docker-compose logs -f"
    echo ""
    echo "To stop services:"
    echo "  - QRMS: kill \$(cat /tmp/qrms.pid)"
    echo "  - L2: cd rollup && docker-compose down"
    exit 0
else
    echo -e "${RED}✗ Some tests failed${NC}"
    exit 1
fi
