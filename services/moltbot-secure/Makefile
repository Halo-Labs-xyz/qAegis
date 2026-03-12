.PHONY: help build test run deploy clean docker-build docker-up docker-down docker-logs ssl-gen

help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Available targets:'
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  %-15s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

build: ## Build the Rust project
	cargo build --release

test: ## Run tests
	cargo test

run: ## Run the service locally
	cargo run

deploy: ## Deploy using Docker Compose
	./scripts/deploy.sh

clean: ## Clean build artifacts
	cargo clean
	rm -rf target/

docker-build: ## Build Docker image
	docker-compose build

docker-up: ## Start Docker containers
	docker-compose up -d

docker-down: ## Stop Docker containers
	docker-compose down

docker-logs: ## View Docker logs
	docker-compose logs -f

ssl-gen: ## Generate SSL certificates
	./nginx/generate-ssl.sh

check: ## Run cargo check
	cargo check

fmt: ## Format code
	cargo fmt

clippy: ## Run clippy linter
	cargo clippy -- -D warnings
