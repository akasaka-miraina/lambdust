# Lambdust Development Makefile

.PHONY: help test coverage lint fmt doc clean install-tools index index-check

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
	@echo ""
	@echo "Code Index Management:"
	@echo "  index          - Generate/update code index"
	@echo "  index-check    - Validate code index is up to date"

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
	cargo clippy --all-features --lib --tests --benches -- -A clippy::all -D warnings

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

## Code Index Management

# Generate or update code index
index:
	@echo "🔍 Generating code index..."
	@python3 tools/index_generator.py --verbose
	@echo "✅ Code index updated"

# Check if code index is up to date
index-check:
	@echo "🔍 Validating code index..."
	@python3 tools/index_generator.py --output docs/CODE_INDEX_TEMP.md
	@if diff -q docs/CODE_INDEX_GENERATED.md docs/CODE_INDEX_TEMP.md > /dev/null 2>&1; then \
		echo "✅ Index is up to date"; \
		rm -f docs/CODE_INDEX_TEMP.md; \
	else \
		echo "❌ Index is outdated. Run 'make index' to update"; \
		rm -f docs/CODE_INDEX_TEMP.md; \
		exit 1; \
	fi

# R7RS-pico specific targets
check-pico:
	cargo check --features pico

test-pico:
	cargo test --features pico

demo-pico:
	cargo run --example r7rs_pico_demo --features pico

# Enhanced CI check with index validation
ci-check-full: fmt-check lint test coverage doc index-check

# Development workflow with index
dev-full: fmt lint test index