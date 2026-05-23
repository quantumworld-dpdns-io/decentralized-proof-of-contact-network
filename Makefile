.PHONY: all build test lint clean docker help

all: build

# === Build ===

build:
	cargo build --workspace

build-release:
	cargo build --workspace --release

build-core:
	cargo build -p poi-core

build-node:
	cargo build -p poi-node

build-cli:
	cargo build -p poi-cli

build-api:
	cargo build -p poi-api

# === Test ===

test:
	cargo test --workspace

test-core:
	cargo test -p poi-core

test-node:
	cargo test -p poi-node

test-integration:
	cargo test --workspace -- --ignored

test-robot:
	cd tests/robot && robot --outputdir results --variable NODE_URL:http://localhost:3000 test-suites/

test-robot-all:
	cd tests/robot && robot --outputdir results --variable NODE_URL:http://localhost:3000 test-suites/

test-robot-security:
	cd tests/robot && robot --outputdir results/security --variable NODE_URL:http://localhost:3000 test-suites/99-security/

test-robot-owasp:
	cd tests/robot && robot --outputdir results/owasp --variable NODE_URL:http://localhost:3000 test-suites/99-security/owasp-top10.robot

# === Lint ===

lint:
	cargo clippy --workspace -- -D warnings
	cargo fmt --all --check

lint-fix:
	cargo clippy --workspace --fix --allow-dirty
	cargo fmt --all

# === Coverage ===

coverage:
	cargo tarpaulin --workspace --out html --out xml

# === Audit ===

audit:
	cargo audit
	cargo deny check

# === Docs ===

docs:
	cargo doc --workspace --no-deps

docs-open:
	cargo doc --workspace --no-deps --open

# === Clean ===

clean:
	cargo clean
	rm -rf tests/robot/results/
	rm -rf target/

# === Docker ===

docker-node:
	docker build -t poi-node -f docker/node.Dockerfile .

docker-api:
	docker build -t poi-api -f docker/api.Dockerfile .

docker-all:
	docker compose -f docker/docker-compose.yml build

docker-up:
	docker compose -f docker/docker-compose.yml up -d

docker-down:
	docker compose -f docker/docker-compose.yml down

# === CI ===

ci: lint build test

# === Dev ===

dev-setup:
	@echo "Setting up development environment..."
	@which rustup || curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
	rustup component add clippy rustfmt
	cargo install cargo-audit cargo-deny cargo-tarpaulin
	pip install robotframework robotframework-requests robotframework-seleniumlibrary
	@echo "Dev setup complete"

dev-node:
	cargo run -p poi-node -- --config config.dev.toml

dev-cli:
	cargo run -p poi-cli -- $(CMD)

dev-api:
	cargo run -p poi-api

# === Help ===

help:
	@echo "Decentralized Proof-of-Contact Network - Build System"
	@echo ""
	@echo "Targets:"
	@echo "  build             Build all workspace crates"
	@echo "  build-release     Build release binaries"
	@echo "  build-core        Build core library"
	@echo "  build-node        Build node binary"
	@echo "  build-cli         Build CLI tool"
	@echo "  build-api         Build API server"
	@echo "  test              Run all tests"
	@echo "  test-core         Run core library tests"
	@echo "  test-node         Run node tests"
	@echo "  test-robot        Run Robot Framework tests"
	@echo "  test-robot-owasp  Run OWASP security tests"
	@echo "  lint              Run clippy and fmt check"
	@echo "  lint-fix          Auto-fix lint issues"
	@echo "  coverage          Generate code coverage report"
	@echo "  audit             Run security audit"
	@echo "  docs              Build rustdoc documentation"
	@echo "  clean             Clean build artifacts"
	@echo "  docker-node       Build node Docker image"
	@echo "  docker-up         Start all Docker services"
	@echo "  docker-down       Stop all Docker services"
	@echo "  ci                Full CI pipeline (lint + build + test)"
	@echo "  dev-setup         Install development dependencies"
	@echo "  dev-node          Run node in development mode"
	@echo "  dev-cli           Run CLI tool with CMD argument"
	@echo "  help              Show this help message"
