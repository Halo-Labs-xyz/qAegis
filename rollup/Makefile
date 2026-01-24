# ============================================================================
# QRMS OP Stack L2 Rollup - Makefile
# Based on: opstack/docs/create-l2-rollup-example/Makefile
# ============================================================================

.PHONY: help all init download setup up up-docker down logs status clean reset test-l1 test-l2 test-qrms test-e2e cli deploy-contracts

-include .env

# Default target
help: ## Show this help
	@echo "QRMS OP Stack L2 Rollup Commands:"
	@awk 'BEGIN {FS = ":.*?## "}; /^[a-zA-Z0-9_-]+:.*?## / {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST) | sort

all: ## Complete end-to-end: init -> setup -> up -> deploy-contracts -> test-e2e
	@echo "=== QuantumAegis End-to-End Setup ==="
	@if [ ! -f .env ]; then make init; fi
	@if [ ! -d sequencer ] || [ ! -f sequencer/genesis.json ]; then make setup; fi
	@make up
	@echo "=== Stack ready. Run 'make deploy-contracts' to deploy contracts, then 'make test-e2e' to test ==="

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

up: ## Start entire stack: L2 (op-geth + op-node) + QRMS service
	@echo "Starting L2 services..."
	@docker-compose up -d op-geth op-node
	@sleep 5
	@make test-l2 || (echo "L2 failed to start"; exit 1)
	@echo "Starting QRMS service..."
	@cd ../services/qrms && cargo build --release --bin qrms 2>&1 | tail -3
	@cd ../services/qrms && cargo run --release --bin qrms > /tmp/qrms.log 2>&1 &
	@echo $$! > /tmp/qrms.pid
	@sleep 3
	@make test-qrms || (echo "QRMS failed to start"; exit 1)
	@echo "Stack ready: L2=http://localhost:8545 QRMS=http://localhost:5050"

up-docker: ## Start entire stack in Docker (L2 + QRMS container)
	@docker-compose --profile infra up -d
	@sleep 5
	@make test-l2
	@make test-qrms

down: ## Stop all services (L2 + QRMS)
	@if [ -f /tmp/qrms.pid ]; then \
		kill $$(cat /tmp/qrms.pid) 2>/dev/null || true; \
		rm /tmp/qrms.pid; \
		echo "QRMS stopped"; \
	fi
	@docker-compose --profile infra down

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

test-e2e: ## Run end-to-end tests (L2 + QRMS + Contracts)
	@echo "Running end-to-end tests..."
	@bash ../scripts/test-e2e-simple.sh

# ============================================================================
# QRMS CLI
# ============================================================================

cli: ## Launch QRMS CLI
	@cd ../services/qrms && cargo run --release --bin qrms-cli localhost:5050

# ============================================================================
# Contracts
# ============================================================================

deploy-contracts: ## Deploy QRMS contracts to L2
	@echo "Deploying contracts..."
	@if [ -z "$$ADMIN_PRIVATE_KEY" ] && [ -z "$$PRIVATE_KEY" ]; then \
		echo "Error: ADMIN_PRIVATE_KEY or PRIVATE_KEY must be set"; \
		exit 1; \
	fi
	@cd ../contracts && forge script script/Deploy.s.sol \
		--rpc-url http://localhost:8545 \
		--broadcast \
		--private-key "$${ADMIN_PRIVATE_KEY:-$$PRIVATE_KEY}" \
		--sig "run(address)" "$${QRM_UPDATER_ADDRESS:-0x7Afde307a7F56d0254E42136cAa9896778662302}"

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
