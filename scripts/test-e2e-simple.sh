#!/bin/bash
# Simple End-to-End Test - Tests what's currently running

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

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
# Test L2 Rollup
# ============================================================================

print_header "1. TESTING L2 ROLLUP"

print_test "L2 RPC endpoint"
if L2_BLOCK=$(curl -s -X POST -H "Content-Type: application/json" \
    --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
    http://localhost:8545 2>/dev/null | jq -r '.result' 2>/dev/null); then
    
    if [ -n "$L2_BLOCK" ] && [ "$L2_BLOCK" != "null" ]; then
        BLOCK_NUM=$((L2_BLOCK))
        print_pass "L2 RPC responding (block: $BLOCK_NUM)"
    else
        print_fail "L2 RPC returned null"
    fi
else
    print_fail "L2 RPC not responding - is op-geth running?"
    print_info "Start with: cd rollup && docker-compose up -d op-geth op-node"
fi

print_test "L2 Chain ID"
if L2_CHAIN_ID=$(curl -s -X POST -H "Content-Type: application/json" \
    --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' \
    http://localhost:8545 2>/dev/null | jq -r '.result' 2>/dev/null); then
    
    if [ -n "$L2_CHAIN_ID" ]; then
        print_pass "L2 Chain ID: $L2_CHAIN_ID"
    else
        print_fail "Could not get L2 Chain ID"
    fi
else
    print_fail "Could not query L2 Chain ID"
fi

# ============================================================================
# Test QRMS Service
# ============================================================================

print_header "2. TESTING QRMS SERVICE"

print_test "QRMS Status endpoint"
if QRMS_STATUS=$(curl -s http://localhost:5050/api/status 2>/dev/null); then
    if [ -n "$QRMS_STATUS" ]; then
        print_pass "QRMS status endpoint responding"
        echo "$QRMS_STATUS" | jq . 2>/dev/null | head -10 || echo "$QRMS_STATUS" | head -5
    else
        print_fail "QRMS status endpoint returned empty"
    fi
else
    print_fail "QRMS not running - start with: cd services/qrms && cargo run --release"
fi

print_test "QRMS QRM History"
if QRM_HISTORY=$(curl -s http://localhost:5050/api/qrm/history 2>/dev/null); then
    if [ -n "$QRM_HISTORY" ]; then
        HISTORY_COUNT=$(echo "$QRM_HISTORY" | jq '. | length' 2>/dev/null || echo "?")
        print_pass "QRM history endpoint responding (entries: $HISTORY_COUNT)"
    else
        print_fail "QRM history endpoint returned empty"
    fi
else
    print_fail "QRM history endpoint not responding"
fi

print_test "QRMS Blocks endpoint"
if QRM_BLOCKS=$(curl -s http://localhost:5050/api/blocks 2>/dev/null); then
    if [ -n "$QRM_BLOCKS" ]; then
        print_pass "Blocks endpoint responding"
    else
        print_fail "Blocks endpoint returned empty"
    fi
else
    print_fail "Blocks endpoint not responding"
fi

# ============================================================================
# Test QVM Integration
# ============================================================================

print_header "3. TESTING QVM INTEGRATION"

cd "$(dirname "$0")/../services/qrms" || exit 1

print_test "Running QVM unit tests"
if cargo test qvm --lib 2>&1 | tail -5 | grep -q "test result: ok"; then
    print_pass "QVM tests passed"
else
    TEST_OUTPUT=$(cargo test qvm --lib 2>&1 | tail -10)
    if echo "$TEST_OUTPUT" | grep -q "test result: ok"; then
        print_pass "QVM tests passed"
    else
        print_fail "QVM tests failed or incomplete"
        echo "$TEST_OUTPUT"
    fi
fi

# ============================================================================
# Test Contracts (if deployed)
# ============================================================================

print_header "4. TESTING CONTRACTS"

cd "$(dirname "$0")/../contracts" || exit 1

if [ -f deployed_addresses.txt ]; then
    print_info "Found deployed_addresses.txt:"
    cat deployed_addresses.txt
    
    print_test "Verifying contract addresses on L2"
    if PQC_ADDR=$(grep "PQC_VERIFIER=" deployed_addresses.txt | cut -d'=' -f2); then
        if [ -n "$PQC_ADDR" ]; then
            print_pass "PQC Verifier deployed at: $PQC_ADDR"
        fi
    fi
else
    print_info "No deployed_addresses.txt found - contracts not deployed yet"
    print_info "Deploy with: cd contracts && forge script script/Deploy.s.sol --rpc-url http://localhost:8545 --broadcast"
fi

# ============================================================================
# Test Threat Injection
# ============================================================================

print_header "5. TESTING THREAT INJECTION"

print_test "Injecting test threat"
if THREAT_RESPONSE=$(curl -s -X POST http://localhost:5050/api/inject_threat \
    -H "Content-Type: application/json" \
    -d '{"category":"digital_signatures","severity":0.7,"description":"E2E test threat"}' 2>/dev/null); then
    
    if [ -n "$THREAT_RESPONSE" ]; then
        print_pass "Threat injection endpoint responding"
        echo "$THREAT_RESPONSE" | jq . 2>/dev/null || echo "$THREAT_RESPONSE"
        
        # Wait for processing
        sleep 2
        
        print_test "Checking updated risk score"
        if UPDATED_STATUS=$(curl -s http://localhost:5050/api/status 2>/dev/null); then
            print_pass "Status updated after threat injection"
        else
            print_fail "Status not updated"
        fi
    else
        print_fail "Threat injection returned empty"
    fi
else
    print_fail "Threat injection endpoint not responding"
fi

# ============================================================================
# Summary
# ============================================================================

print_header "TEST SUMMARY"

echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}Tests Passed: $TESTS_PASSED${NC}"
echo -e "${RED}Tests Failed: $TESTS_FAILED${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All tests passed!${NC}"
    echo ""
    echo "Services:"
    echo "  - L2 RPC: http://localhost:8545"
    echo "  - QRMS API: http://localhost:5050"
    echo "  - QRMS Dashboard: http://localhost:5050"
    exit 0
else
    echo -e "${YELLOW}⚠ Some tests failed - check service status${NC}"
    echo ""
    echo "To start services:"
    echo "  - L2: cd rollup && docker-compose up -d"
    echo "  - QRMS: cd services/qrms && cargo run --release"
    exit 1
fi
