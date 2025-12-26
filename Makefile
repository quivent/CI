# Makefile for CI (Collaborative Intelligence CLI)
# A Rust-based CLI tool

# Project settings
BINARY_NAME := CI
CARGO := cargo
INSTALL_DIR := $(HOME)/.local/bin

# Check if Rust is installed
RUST_INSTALLED := $(shell command -v cargo 2>/dev/null)

# Build profiles
.PHONY: all build release debug clean test check fmt lint install uninstall help
.PHONY: check-rust install-rust stage commit push ship

# Default target
all: build

# Check if Rust is installed
check-rust:
ifndef RUST_INSTALLED
	$(error Rust is not installed. Run 'make install-rust' to install it)
endif

# Install Rust via rustup
install-rust:
	@if command -v cargo >/dev/null 2>&1; then \
		echo "Rust is already installed:"; \
		rustc --version; \
		cargo --version; \
	else \
		echo "Installing Rust via rustup..."; \
		curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y; \
		echo ""; \
		echo "Rust installed successfully!"; \
		echo "Please run: source $$HOME/.cargo/env"; \
		echo "Then run 'make build' again."; \
	fi

# Build in debug mode
build: check-rust
	$(CARGO) build

# Build in release mode (optimized)
release: check-rust
	$(CARGO) build --release

# Alias for debug build
debug: build

# Run tests
test: check-rust
	$(CARGO) test

# Run tests with output
test-verbose: check-rust
	$(CARGO) test -- --nocapture

# Check code without building
check: check-rust
	$(CARGO) check

# Format code
fmt: check-rust
	$(CARGO) fmt

# Check formatting without modifying
fmt-check: check-rust
	$(CARGO) fmt -- --check

# Run clippy linter
lint: check-rust
	$(CARGO) clippy -- -D warnings

# Run clippy with all targets
lint-all: check-rust
	$(CARGO) clippy --all-targets --all-features -- -D warnings

# Clean build artifacts
clean:
	@if command -v cargo >/dev/null 2>&1; then \
		$(CARGO) clean; \
	else \
		rm -rf target; \
	fi

# Install to local bin directory
install: release
	@mkdir -p $(INSTALL_DIR)
	@cp target/release/$(BINARY_NAME) $(INSTALL_DIR)/
	@cp target/release/cargo-install-ci $(INSTALL_DIR)/ 2>/dev/null || true
	@echo "Installed $(BINARY_NAME) to $(INSTALL_DIR)"

# Uninstall from local bin directory
uninstall:
	@rm -f $(INSTALL_DIR)/$(BINARY_NAME)
	@rm -f $(INSTALL_DIR)/cargo-install-ci
	@echo "Uninstalled $(BINARY_NAME) from $(INSTALL_DIR)"

# Run the CLI (debug build)
run: check-rust
	$(CARGO) run

# Run the CLI with arguments (use: make run-args ARGS="your args here")
run-args: check-rust
	$(CARGO) run -- $(ARGS)

# Run the CLI (release build)
run-release: check-rust
	$(CARGO) run --release

# Build documentation
doc: check-rust
	$(CARGO) doc --no-deps

# Open documentation in browser
doc-open: check-rust
	$(CARGO) doc --no-deps --open

# Update dependencies
update: check-rust
	$(CARGO) update

# Show dependency tree
deps: check-rust
	$(CARGO) tree

# Full CI check (format, lint, test)
ci: fmt-check lint test

# Development workflow: format, check, build
dev: fmt check build

# Git: stage all changes
stage:
	git add -A
	@echo "Staged all changes"
	@git status --short

# Git: commit with message (use: make commit MSG="your message")
commit: stage
	@if [ -z "$(MSG)" ]; then \
		echo "Error: Please provide a commit message with MSG=\"your message\""; \
		exit 1; \
	fi
	git commit -m "$(MSG)"

# Git: push to origin
push:
	git push origin $$(git branch --show-current)

# Git: stage, commit, and push (use: make ship MSG="your message")
ship: commit push
	@echo "Changes shipped to GitHub"

# Setup: install Rust and build
setup: install-rust
	@if command -v cargo >/dev/null 2>&1; then \
		$(CARGO) build --release; \
	else \
		echo "Please run 'source $$HOME/.cargo/env' and then 'make build'"; \
	fi

# Help
help:
	@echo "CI (Collaborative Intelligence CLI) - Build Commands"
	@echo ""
	@echo "Usage: make [target]"
	@echo ""
	@echo "Setup (run first if Rust is not installed):"
	@echo "  install-rust Install Rust via rustup"
	@echo "  setup        Install Rust and build the project"
	@echo ""
	@echo "Build targets:"
	@echo "  build        Build in debug mode (default)"
	@echo "  release      Build in release mode (optimized)"
	@echo "  debug        Alias for build"
	@echo "  clean        Remove build artifacts"
	@echo ""
	@echo "Testing:"
	@echo "  test         Run tests"
	@echo "  test-verbose Run tests with output"
	@echo "  check        Check code without building"
	@echo ""
	@echo "Code quality:"
	@echo "  fmt          Format code"
	@echo "  fmt-check    Check formatting"
	@echo "  lint         Run clippy linter"
	@echo "  lint-all     Run clippy on all targets"
	@echo ""
	@echo "Installation:"
	@echo "  install      Build release and install to ~/.local/bin"
	@echo "  uninstall    Remove from ~/.local/bin"
	@echo ""
	@echo "Running:"
	@echo "  run          Run in debug mode"
	@echo "  run-args     Run with args (ARGS=\"...\")"
	@echo "  run-release  Run in release mode"
	@echo ""
	@echo "Documentation:"
	@echo "  doc          Build documentation"
	@echo "  doc-open     Build and open documentation"
	@echo ""
	@echo "Git:"
	@echo "  stage        Stage all changes"
	@echo "  commit       Stage and commit (MSG=\"...\")"
	@echo "  push         Push to origin"
	@echo "  ship         Stage, commit, push (MSG=\"...\")"
	@echo ""
	@echo "Other:"
	@echo "  update       Update dependencies"
	@echo "  deps         Show dependency tree"
	@echo "  ci           Full CI check (fmt, lint, test)"
	@echo "  dev          Dev workflow (fmt, check, build)"
