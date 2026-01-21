# Voxel Forge - Makefile
# All build, test, and development commands

.PHONY: all build release run test lint fmt fmt-check check clean install-hooks docs bench audit update help
.PHONY: coverage coverage-html coverage-open coverage-all setup-tools setup-system setup ci ci-build

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
	cargo test

## Run tests with output
test-verbose:
	cargo test -- --nocapture

## Run clippy linter
lint:
	cargo clippy --all-targets

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
# CODE COVERAGE
# =============================================================================

## Run tests with coverage report (requires cargo-llvm-cov)
coverage:
	cargo llvm-cov --lib

## Run coverage and generate HTML report
coverage-html:
	cargo llvm-cov --lib --html
	@echo "Coverage report generated at target/llvm-cov/html/index.html"

## Run coverage and open HTML report
coverage-open:
	cargo llvm-cov --lib --html --open

## Run coverage with all features (requires: make setup-system first for audio)
coverage-all:
	@echo "Note: --all-features includes audio which requires libasound2-dev"
	@echo "Run 'make setup-system' first if this fails"
	cargo llvm-cov --all-features --lib

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

## Install cargo extension tools (global)
setup-tools:
	@echo "Installing rustup components..."
	rustup component add rustfmt clippy llvm-tools-preview
	@echo ""
	@echo "Installing cargo tools (this may take a few minutes)..."
	cargo install cargo-audit cargo-outdated cargo-llvm-cov
	@echo ""
	@echo "Cargo tools installed successfully!"
	@echo "  - cargo-audit     : Security vulnerability scanner"
	@echo "  - cargo-outdated  : Check for outdated dependencies"
	@echo "  - cargo-llvm-cov  : Code coverage reports"

## Install system dependencies (Linux only)
setup-system:
	@echo "Installing system dependencies for Linux..."
	@echo "You may need to run this with sudo or enter your password:"
	sudo apt install -y pkg-config libasound2-dev libudev-dev
	@echo "System dependencies installed!"

## Setup development environment (full)
setup: install-hooks setup-tools
	@echo ""
	@echo "========================================="
	@echo "Development environment setup complete!"
	@echo "========================================="
	@echo ""
	@echo "Optional: Run 'make setup-system' to install Linux system deps"
	@echo "          (needed for audio support)"

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
	@echo "Code Coverage:"
	@echo "  make coverage     - Run tests with coverage report"
	@echo "  make coverage-html - Generate HTML coverage report"
	@echo "  make coverage-open - Generate and open HTML report"
	@echo ""
	@echo "Documentation:"
	@echo "  make docs         - Generate and open docs"
	@echo ""
	@echo "Dependencies:"
	@echo "  make audit        - Security audit"
	@echo "  make update       - Update dependencies"
	@echo "  make outdated     - Check for outdated deps"
	@echo ""
	@echo "Development Setup:"
	@echo "  make setup        - Full dev environment setup"
	@echo "  make setup-tools  - Install cargo tools only"
	@echo "  make setup-system - Install Linux system deps"
	@echo "  make install-hooks - Install git hooks"
	@echo "  make clean        - Clean build artifacts"
	@echo ""
	@echo "CI/CD:"
	@echo "  make ci           - Run CI checks"
	@echo "  make ci-build     - Full CI build"
