# ============================================================================
# QRMS OP Stack L2 Rollup - Makefile
# Based on: opstack/docs/create-l2-rollup-example/Makefile
# ============================================================================

.PHONY: help init download setup up down logs status clean reset test-l1 test-l2 test-qrms cli deploy-contracts

-include .env

# Default target
help: ## Show this help
	@echo "QRMS OP Stack L2 Rollup Commands:"
	@grep -E '^[a-zA-Z0-9_-]+:.*?## .*' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

# ============================================================================
# Initialization
# ============================================================================

init: download ## Initialize project (download op-deployer, copy env)
	@if [ ! -f .env ]; then \
		cp env.example .env; \
		echo "Created .env - edit with your values"; \
	fi
	@echo "Run: make setup"

download: ## Download op-deployer binary
	./scripts/download-op-deployer.sh

# ============================================================================
# Setup
# ============================================================================

setup: ## Deploy L1 contracts and configure components
	@if [ ! -f op-deployer ]; then \
		echo "op-deployer not found. Run: make download"; \
		exit 1; \
	fi
	@if [ ! -f .env ]; then \
		echo ".env not found. Run: make init"; \
		exit 1; \
	fi
	./scripts/setup-rollup.sh

# ============================================================================
# Run Services
# ============================================================================

up: ## Start core services (QRMS + op-geth + op-node)
	docker-compose up -d qrms op-geth op-node
	@sleep 3
	@make test-qrms || true

up-all: ## Start ALL services (requires make setup first)
	docker-compose --profile infra up -d
	@make test-l2
	@make test-qrms

down: ## Stop all services
	docker-compose --profile infra down

restart: ## Restart all services
	docker-compose restart

restart-%: ## Restart specific service (e.g., make restart-qrms)
	docker-compose restart $*

# ============================================================================
# Logs & Status
# ============================================================================

logs: ## View all service logs
	docker-compose logs -f

logs-%: ## View specific service logs (e.g., make logs-qrms)
	docker-compose logs -f $*

status: ## Show service status
	docker-compose ps

# ============================================================================
# Testing
# ============================================================================

test-l1: ## Test L1 connectivity
	@echo "Testing L1..."
	@if [ -f .env ]; then set -a; source .env; set +a; fi; \
	curl -s -X POST -H "Content-Type: application/json" \
		--data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
		"$$L1_RPC_URL" | jq -r '.result' && echo "L1 OK" || echo "L1 FAIL"

test-l2: ## Test L2 connectivity
	@echo "Testing L2..."
	@curl -s -X POST -H "Content-Type: application/json" \
		--data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
		http://localhost:8545 | jq -r '.result' 2>/dev/null && echo "L2 OK" || echo "L2 not running"

test-qrms: ## Test QRMS API
	@echo "Testing QRMS..."
	@curl -s http://localhost:5050/api/status | jq . 2>/dev/null || echo "QRMS not running"

# ============================================================================
# QRMS CLI
# ============================================================================

cli: ## Launch QRMS CLI
	@cd ../qrms-rust && ./target/release/qrms-cli localhost:5050

# ============================================================================
# Contracts
# ============================================================================

deploy-contracts: ## Deploy QRMS contracts to L2
	@echo "Deploying contracts..."
	@cd contracts && forge script script/Deploy.s.sol --rpc-url http://localhost:8545 --broadcast

# ============================================================================
# Clean
# ============================================================================

clean: ## Remove containers and volumes
	@mkdir -p batcher proposer challenger sequencer dispute-mon
	@echo "# cleanup" > batcher/.env 2>/dev/null || true
	@echo "# cleanup" > proposer/.env 2>/dev/null || true
	@echo "# cleanup" > challenger/.env 2>/dev/null || true
	@echo "# cleanup" > dispute-mon/.env 2>/dev/null || true
	docker-compose down -v --remove-orphans 2>/dev/null || true

reset: ## Complete reset - removes all data
	@echo "This will reset everything!"
	@read -p "Continue? (y/N) " c && [ "$$c" = "y" ] || exit 1
	make clean
	rm -rf deployer sequencer batcher proposer challenger dispute-mon
	@echo "Reset complete. Run: make setup"
