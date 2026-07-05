# Common developer tasks. Run `make help` for the list.

.DEFAULT_GOAL := help

.PHONY: help run test fmt fmt-check lint check db-up db-down

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-12s\033[0m %s\n", $$1, $$2}'

run: ## Run the web server
	cargo run

test: ## Run all tests
	cargo test

fmt: ## Format the code
	cargo fmt

fmt-check: ## Check formatting (no changes)
	cargo fmt --all -- --check

lint: ## Run clippy with warnings denied
	cargo clippy --all-targets -- -D warnings

check: fmt-check lint test ## Full CI check: format, lint, test

db-up: ## Start the local Postgres container
	docker compose up -d

db-down: ## Stop the local Postgres container
	docker compose down
