.PHONY: ci-check build test fmt clippy deny conformance coverage help

# Run all CI checks (matches GitHub Actions)
ci-check: build test fmt clippy deny
	@echo "All CI checks passed!"

# Individual check targets
build:
	@echo "Building workspace..."
	cargo build --workspace

test:
	@echo "Running tests..."
	cargo test --workspace

fmt:
	@echo "Checking code formatting..."
	cargo fmt --all -- --check

clippy:
	@echo "Running clippy lints..."
	cargo clippy --all-features --workspace -- -D warnings

# cargo-deny checks (security, licenses, bans)
deny:
	@echo "Running cargo-deny checks..."
	cargo deny check advisories
	cargo deny check bans licenses sources

# Conformance tests (optional, can be slow)
conformance:
	@echo "unning conformance tests..."
	cargo test --package partiql-conformance-tests --features "conformance_test"

# Code coverage (requires cargo-llvm-cov)
coverage:
	@echo "Generating code coverage..."
	@if ! command -v cargo-llvm-cov >/dev/null 2>&1; then \
		echo "Installing cargo-llvm-cov..."; \
		cargo install cargo-llvm-cov; \
	fi
	cargo llvm-cov --workspace --all-features --ignore-run-fail --html

# Help target
help:
	@echo "Available targets:"
	@echo "  ci-check     - Run all CI checks (build, test, fmt, clippy, deny)"
	@echo "  build        - Build the workspace"
	@echo "  test         - Run tests"
	@echo "  fmt          - Check code formatting"
	@echo "  clippy       - Run clippy lints"
	@echo "  deny         - Run cargo-deny security/license checks"
	@echo "  conformance  - Run conformance tests (slow)"
	@echo "  coverage     - Generate code coverage report"
	@echo "  help         - Show this help message"