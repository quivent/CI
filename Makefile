# Makefile for CI (Collaborative Intelligence CLI)
# Cross-platform: works on Windows (with Git Bash/MinGW), macOS, Linux

# Detect OS
ifeq ($(OS),Windows_NT)
    DETECTED_OS := Windows
    EXE_EXT := .exe
    INSTALL_DIR := $(USERPROFILE)\.local\bin
    RUSTUP_INIT := powershell -Command "& {Invoke-WebRequest -Uri 'https://win.rustup.rs/x86_64' -OutFile '$$env:TEMP\rustup-init.exe'; Start-Process -FilePath '$$env:TEMP\rustup-init.exe' -ArgumentList '-y' -Wait}"
    SHELL := cmd.exe
    MKDIR := if not exist "$(subst /,\,$(INSTALL_DIR))" mkdir "$(subst /,\,$(INSTALL_DIR))"
    CP := copy
    RM := del /Q
    RMDIR := rmdir /S /Q
else
    DETECTED_OS := $(shell uname -s)
    EXE_EXT :=
    INSTALL_DIR := $(HOME)/.local/bin
    RUSTUP_INIT := curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    MKDIR := mkdir -p $(INSTALL_DIR)
    CP := cp
    RM := rm -f
    RMDIR := rm -rf
endif

BINARY_NAME := CI
CARGO := cargo

.PHONY: all build release debug clean test check fmt lint install uninstall help
.PHONY: setup install-rust run run-release doc stage commit push ship

# Default: setup everything and build
all: setup build

# Check and install Rust if needed, then build
setup:
ifeq ($(OS),Windows_NT)
	@where cargo >nul 2>&1 || (echo Installing Rust... && $(RUSTUP_INIT) && echo Please restart your terminal and run 'make' again)
	@where cargo >nul 2>&1 && cargo build --release || echo Restart terminal to complete Rust installation
else
	@command -v cargo >/dev/null 2>&1 || (echo "Installing Rust..." && $(RUSTUP_INIT) && . "$$HOME/.cargo/env")
	@command -v cargo >/dev/null 2>&1 && cargo build --release || (echo "Run: source $$HOME/.cargo/env && make")
endif

# Install Rust only
install-rust:
ifeq ($(OS),Windows_NT)
	@where cargo >nul 2>&1 && (echo Rust already installed && cargo --version) || (echo Installing Rust... && $(RUSTUP_INIT))
else
	@command -v cargo >/dev/null 2>&1 && echo "Rust already installed" && cargo --version || (echo "Installing Rust..." && $(RUSTUP_INIT))
endif

# Build debug
build:
	$(CARGO) build

# Build release
release:
	$(CARGO) build --release

debug: build

# Tests
test:
	$(CARGO) test

test-verbose:
	$(CARGO) test -- --nocapture

check:
	$(CARGO) check

# Code quality
fmt:
	$(CARGO) fmt

fmt-check:
	$(CARGO) fmt -- --check

lint:
	$(CARGO) clippy -- -D warnings

lint-all:
	$(CARGO) clippy --all-targets --all-features -- -D warnings

# Clean
clean:
ifeq ($(OS),Windows_NT)
	@if exist target $(RMDIR) target
else
	@$(RMDIR) target 2>/dev/null || true
endif

# Install binary
install: release
ifeq ($(OS),Windows_NT)
	@$(MKDIR)
	@$(CP) target\release\$(BINARY_NAME)$(EXE_EXT) "$(subst /,\,$(INSTALL_DIR))\\"
	@echo Installed to $(INSTALL_DIR)
else
	@$(MKDIR)
	@$(CP) target/release/$(BINARY_NAME)$(EXE_EXT) $(INSTALL_DIR)/
	@echo "Installed to $(INSTALL_DIR)"
endif

uninstall:
ifeq ($(OS),Windows_NT)
	@$(RM) "$(subst /,\,$(INSTALL_DIR))\$(BINARY_NAME)$(EXE_EXT)" 2>nul || echo Already removed
else
	@$(RM) $(INSTALL_DIR)/$(BINARY_NAME)$(EXE_EXT)
	@echo "Uninstalled from $(INSTALL_DIR)"
endif

# Run
run:
	$(CARGO) run

run-args:
	$(CARGO) run -- $(ARGS)

run-release:
	$(CARGO) run --release

# Docs
doc:
	$(CARGO) doc --no-deps

doc-open:
	$(CARGO) doc --no-deps --open

# Dependencies
update:
	$(CARGO) update

deps:
	$(CARGO) tree

# Workflows
ci: fmt-check lint test

dev: fmt check build

# Git operations
stage:
	git add -A
	git status --short

commit:
ifeq ($(MSG),)
	@echo "Usage: make commit MSG=\"your message\""
else
	git add -A
	git commit -m "$(MSG)"
endif

push:
	git push

ship:
ifeq ($(MSG),)
	@echo "Usage: make ship MSG=\"your message\""
else
	git add -A
	git commit -m "$(MSG)"
	git push
endif

help:
	@echo CI Build System - Works on Windows, macOS, Linux
	@echo.
	@echo Usage: make [target]
	@echo.
	@echo FIRST TIME SETUP:
	@echo   make          Install Rust if needed + build release
	@echo   make setup    Same as above
	@echo.
	@echo BUILD:
	@echo   make build    Debug build
	@echo   make release  Release build
	@echo   make clean    Remove build artifacts
	@echo.
	@echo TEST:
	@echo   make test     Run tests
	@echo   make check    Check without building
	@echo   make lint     Run clippy
	@echo   make ci       Full CI (fmt + lint + test)
	@echo.
	@echo INSTALL:
	@echo   make install    Install to ~/.local/bin
	@echo   make uninstall  Remove installation
	@echo.
	@echo GIT:
	@echo   make commit MSG="msg"  Stage + commit
	@echo   make ship MSG="msg"    Stage + commit + push
