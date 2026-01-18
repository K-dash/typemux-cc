.PHONY: all check fmt lint test build clean

# Default target: run all
all: fmt lint test

# Format
fmt:
	cargo fmt

# Format check (for CI)
fmt-check:
	cargo fmt --check

# Lint (clippy)
lint:
	cargo clippy -- -D warnings

# Test
test:
	cargo test

# Build
build:
	cargo build

# Release build
release:
	cargo build --release

# Check (compile only, no binary generation)
check:
	cargo check

# Clean
clean:
	cargo clean

# For CI: fmt-check + lint + test
ci: fmt-check lint test
