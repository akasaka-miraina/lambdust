# Lambdust Development Makefile

.PHONY: help test coverage lint fmt doc clean install-tools

# Default target
help:
	@echo "Lambdust Development Commands:"
	@echo "  test           - Run all tests"
	@echo "  coverage       - Generate test coverage report"
	@echo "  coverage-open  - Generate and open coverage report in browser"
	@echo "  lint           - Run clippy linter"
	@echo "  fmt            - Format code with rustfmt"
	@echo "  doc            - Generate documentation"
	@echo "  doc-open       - Generate and open documentation"
	@echo "  clean          - Clean build artifacts"
	@echo "  install-tools  - Install development tools"

# Run tests
test:
	cargo test --all-features
	cargo test --doc

# Generate coverage report
coverage:
	cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
	cargo llvm-cov --all-features --workspace --html --output-dir target/coverage

# Generate and open coverage report
coverage-open:
	cargo llvm-cov --all-features --workspace --open

# Show coverage summary
coverage-summary:
	cargo llvm-cov --all-features --workspace --summary-only

# Run clippy
lint:
	cargo clippy --all-features --lib --tests --benches -- -D warnings

# Format code
fmt:
	cargo fmt --all

# Check formatting
fmt-check:
	cargo fmt --all -- --check

# Generate documentation
doc:
	cargo doc --no-deps --all-features

# Generate and open documentation
doc-open:
	cargo doc --no-deps --all-features --open

# Clean build artifacts
clean:
	cargo clean
	rm -rf target/coverage
	rm -f lcov.info

# Install development tools
install-tools:
	cargo install cargo-llvm-cov
	rustup component add llvm-tools-preview
	rustup component add clippy
	rustup component add rustfmt

# Build release
release:
	cargo build --release

# Full CI check (what CI runs)
ci-check: fmt-check lint test coverage doc

# Quick development check
dev-check: fmt lint test