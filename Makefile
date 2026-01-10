# Oxide Browser - Build Makefile

.PHONY: all build release debug clean test check docs \
        macos macos-universal macos-dmg macos-notarize \
        linux windows help

# Default target
all: release

# Detect OS
UNAME_S := $(shell uname -s)
UNAME_M := $(shell uname -m)

# Version
VERSION := $(shell grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')

#==============================================================================
# Build Targets
#==============================================================================

## build: Build in debug mode
build:
	cargo build -p browser

## release: Build in release mode
release:
	cargo build --release -p browser

## debug: Build in debug mode with full debug info
debug:
	RUSTFLAGS="-C debuginfo=2" cargo build -p browser

## check: Run cargo check
check:
	cargo check --workspace

## test: Run all tests
test:
	cargo test --workspace

## bench: Run benchmarks
bench:
	cargo bench -p browser

## docs: Generate documentation
docs:
	cargo doc --workspace --no-deps --open

## clean: Clean build artifacts
clean:
	cargo clean
	rm -rf dist/

## fmt: Format code
fmt:
	cargo fmt --all

## clippy: Run clippy lints
clippy:
	cargo clippy --workspace -- -D warnings

#==============================================================================
# macOS Targets
#==============================================================================

## macos: Build macOS application bundle
macos:
	@echo "Building Oxide Browser for macOS..."
	./packaging/macos/build-macos.sh

## macos-universal: Build universal macOS binary (Intel + Apple Silicon)
macos-universal:
	@echo "Building universal macOS application..."
	./packaging/macos/build-macos.sh --universal

## macos-dmg: Build macOS DMG installer
macos-dmg:
	@echo "Building macOS DMG..."
	./packaging/macos/build-macos.sh --universal --dmg

## macos-signed: Build and sign macOS application
macos-signed:
ifndef SIGNING_IDENTITY
	$(error SIGNING_IDENTITY is not set)
endif
	@echo "Building and signing macOS application..."
	./packaging/macos/build-macos.sh --universal --sign "$(SIGNING_IDENTITY)" --dmg

## macos-notarize: Build, sign, and notarize macOS application
macos-notarize:
ifndef SIGNING_IDENTITY
	$(error SIGNING_IDENTITY is not set)
endif
ifndef APPLE_ID
	$(error APPLE_ID is not set)
endif
	@echo "Building, signing, and notarizing macOS application..."
	./packaging/macos/build-macos.sh --universal --sign "$(SIGNING_IDENTITY)" --notarize --dmg

## macos-icon: Generate macOS app icon
macos-icon:
	./packaging/macos/generate-icon.sh

## macos-clean: Clean macOS build artifacts
macos-clean:
	rm -rf dist/macos
	rm -rf target/aarch64-apple-darwin
	rm -rf target/x86_64-apple-darwin
	rm -rf target/release/universal

#==============================================================================
# Linux Targets
#==============================================================================

## linux: Build Linux binary
linux:
	cargo build --release -p browser

## linux-deb: Build Debian package (requires cargo-deb)
linux-deb:
	cargo deb -p browser

## linux-rpm: Build RPM package (requires cargo-rpm)
linux-rpm:
	cargo rpm build

## linux-appimage: Build AppImage
linux-appimage:
	@echo "AppImage packaging not yet implemented"
	@exit 1

#==============================================================================
# Windows Targets
#==============================================================================

## windows: Build Windows binary (cross-compile or native)
windows:
ifeq ($(UNAME_S),Windows_NT)
	cargo build --release -p browser
else
	cargo build --release -p browser --target x86_64-pc-windows-gnu
endif

## windows-msi: Build Windows MSI installer (requires cargo-wix)
windows-msi:
	cargo wix -p browser

#==============================================================================
# Development Targets
#==============================================================================

## run: Run the browser in debug mode
run:
	cargo run -p browser

## run-release: Run the browser in release mode
run-release:
	cargo run --release -p browser

## run-url: Run the browser with a specific URL
run-url:
ifndef URL
	$(error URL is not set. Usage: make run-url URL=https://example.com)
endif
	cargo run --release -p browser -- $(URL)

## watch: Watch for changes and rebuild
watch:
	cargo watch -x 'build -p browser'

## profile: Build with profiling enabled
profile:
	RUSTFLAGS="-C instrument-coverage" cargo build --release -p browser

#==============================================================================
# Installation
#==============================================================================

## install: Install the browser locally
install:
	cargo install --path crates/browser

## uninstall: Uninstall the browser
uninstall:
	cargo uninstall oxide-browser

#==============================================================================
# CI/CD Targets
#==============================================================================

## ci: Run all CI checks
ci: fmt check clippy test

## ci-build: Build for CI
ci-build:
	cargo build --release --workspace

## ci-test: Run tests for CI
ci-test:
	cargo test --workspace --no-fail-fast

#==============================================================================
# Help
#==============================================================================

## help: Show this help message
help:
	@echo "Oxide Browser Build System"
	@echo ""
	@echo "Usage: make [target]"
	@echo ""
	@echo "Build Targets:"
	@grep -E '^## ' $(MAKEFILE_LIST) | sed 's/## /  /' | grep -E '^  (build|release|debug|check|test|bench|docs|clean|fmt|clippy):' | column -t -s ':'
	@echo ""
	@echo "macOS Targets:"
	@grep -E '^## ' $(MAKEFILE_LIST) | sed 's/## /  /' | grep -E '^  macos' | column -t -s ':'
	@echo ""
	@echo "Linux Targets:"
	@grep -E '^## ' $(MAKEFILE_LIST) | sed 's/## /  /' | grep -E '^  linux' | column -t -s ':'
	@echo ""
	@echo "Windows Targets:"
	@grep -E '^## ' $(MAKEFILE_LIST) | sed 's/## /  /' | grep -E '^  windows' | column -t -s ':'
	@echo ""
	@echo "Development Targets:"
	@grep -E '^## ' $(MAKEFILE_LIST) | sed 's/## /  /' | grep -E '^  (run|watch|profile)' | column -t -s ':'
	@echo ""
	@echo "Installation:"
	@grep -E '^## ' $(MAKEFILE_LIST) | sed 's/## /  /' | grep -E '^  (install|uninstall):' | column -t -s ':'
	@echo ""
	@echo "Environment Variables:"
	@echo "  SIGNING_IDENTITY    macOS code signing identity"
	@echo "  APPLE_ID            Apple ID for notarization"
	@echo "  APPLE_TEAM_ID       Apple Developer Team ID"
	@echo "  APPLE_PASSWORD      App-specific password for notarization"
	@echo "  URL                 URL to open with run-url target"
