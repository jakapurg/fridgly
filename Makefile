# Common developer tasks. Run `make help` for the list.

.DEFAULT_GOAL := help

.PHONY: help run dev test fmt fmt-check lint check db-up db-down

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-12s\033[0m %s\n", $$1, $$2}'

run: ## Run the web server
	cargo run

dev: ## Hot-reload dev server (rebuild + restart + browser refresh on change)
	@command -v cargo-watch >/dev/null 2>&1 || { echo "cargo-watch not found. Install with: cargo install cargo-watch"; exit 1; }
	cargo watch -w crates -w migrations -x 'run -p fridgly-web --features dev'

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
