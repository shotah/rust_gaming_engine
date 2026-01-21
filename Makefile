# Voxel Forge - Makefile
# All build, test, and development commands

.PHONY: all build release run test lint fmt fmt-check check clean install-hooks docs bench audit update help

# Default target
all: check build

# =============================================================================
# BUILD COMMANDS
# =============================================================================

## Build the project in debug mode
build:
	cargo build

## Build the project in release mode
release:
	cargo build --release

## Run the game in debug mode
run:
	cargo run

## Run the game in release mode
run-release:
	cargo run --release

# =============================================================================
# TESTING & QUALITY
# =============================================================================

## Run all tests
test:
	cargo test --all-features

## Run tests with output
test-verbose:
	cargo test --all-features -- --nocapture

## Run clippy linter
lint:
	cargo clippy --all-targets --all-features -- -D warnings

## Format code with rustfmt
fmt:
	cargo fmt --all

## Check code formatting (fails if not formatted)
fmt-check:
	cargo fmt --all -- --check

## Run all checks (format, lint, test)
check: fmt-check lint test
	@echo "All checks passed!"

## Run benchmarks
bench:
	cargo bench

# =============================================================================
# DOCUMENTATION
# =============================================================================

## Generate documentation
docs:
	cargo doc --no-deps --open

## Generate documentation without opening
docs-build:
	cargo doc --no-deps

# =============================================================================
# DEPENDENCY MANAGEMENT
# =============================================================================

## Audit dependencies for security vulnerabilities
audit:
	cargo audit

## Update all dependencies to latest versions
update:
	cargo update

## Check for outdated dependencies
outdated:
	cargo outdated

# =============================================================================
# DEVELOPMENT SETUP
# =============================================================================

## Install git pre-commit hooks
install-hooks:
	@echo "Installing pre-commit hook..."
	@cp scripts/pre-commit .git/hooks/pre-commit 2>/dev/null || \
		(mkdir -p .git/hooks && cp scripts/pre-commit .git/hooks/pre-commit)
	@chmod +x .git/hooks/pre-commit
	@echo "Pre-commit hook installed successfully!"

## Setup development environment
setup: install-hooks
	rustup component add rustfmt clippy
	cargo install cargo-audit cargo-outdated
	@echo "Development environment setup complete!"

# =============================================================================
# CLEANUP
# =============================================================================

## Clean build artifacts
clean:
	cargo clean

## Clean everything including lock file
clean-all: clean
	rm -f Cargo.lock

# =============================================================================
# CI/CD HELPERS
# =============================================================================

## Run CI checks (used by GitHub Actions)
ci: fmt-check lint test
	@echo "CI checks passed!"

## Build for CI (release mode with all checks)
ci-build: ci release

# =============================================================================
# HELP
# =============================================================================

## Show this help message
help:
	@echo "Voxel Forge - Available Commands"
	@echo "================================="
	@echo ""
	@echo "Build Commands:"
	@echo "  make build        - Build in debug mode"
	@echo "  make release      - Build in release mode"
	@echo "  make run          - Run in debug mode"
	@echo "  make run-release  - Run in release mode"
	@echo ""
	@echo "Testing & Quality:"
	@echo "  make test         - Run all tests"
	@echo "  make lint         - Run clippy linter"
	@echo "  make fmt          - Format code"
	@echo "  make fmt-check    - Check formatting"
	@echo "  make check        - Run all checks"
	@echo "  make bench        - Run benchmarks"
	@echo ""
	@echo "Documentation:"
	@echo "  make docs         - Generate and open docs"
	@echo ""
	@echo "Dependencies:"
	@echo "  make audit        - Security audit"
	@echo "  make update       - Update dependencies"
	@echo "  make outdated     - Check for outdated deps"
	@echo ""
	@echo "Development:"
	@echo "  make setup        - Setup dev environment"
	@echo "  make install-hooks - Install git hooks"
	@echo "  make clean        - Clean build artifacts"
	@echo ""
	@echo "CI/CD:"
	@echo "  make ci           - Run CI checks"
	@echo "  make ci-build     - Full CI build"
