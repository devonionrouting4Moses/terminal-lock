.PHONY: help build release install test clean check fmt clippy docs run setup fix-integration

# Default target
help:
	@echo "Terminal Lock - Makefile targets:"
	@echo "  make build           - Build debug version"
	@echo "  make release         - Build optimized release version"
	@echo "  make install         - Install to ~/.local/bin + configure shells"
	@echo "  make fix-integration - Fix shell integration (if not working)"
	@echo "  make test            - Run all tests"
	@echo "  make check           - Run cargo check"
	@echo "  make fmt             - Format code"
	@echo "  make clippy          - Run clippy lints"
	@echo "  make docs            - Generate documentation"
	@echo "  make clean           - Clean build artifacts"
	@echo "  make run             - Run in development mode"
	@echo "  make setup           - Run setup wizard"

# Build debug version
build:
	cargo build

# Build release version (optimized)
release:
	cargo build --release

# Install to ~/.local/bin
install: release
	@echo "Installing terminal-lock to ~/.local/bin..."
	@mkdir -p ~/.local/bin
	@cp target/release/terminal-lock ~/.local/bin/
	@chmod +x ~/.local/bin/terminal-lock
	@echo "✓ Binary installed"
	@echo ""
	@echo "Configuring shell integration..."
	@chmod +x fix-integration.sh
	@./fix-integration.sh
	@echo ""
	@echo "✓ Installation complete!"
	@echo ""
	@echo "IMPORTANT: Close this terminal and open a new one!"

# Fix shell integration (if not working)
fix-integration:
	@echo "Fixing shell integration..."
	@chmod +x fix-integration.sh
	@./fix-integration.sh

# Run all tests
test:
	cargo test --workspace

# Run tests with output
test-verbose:
	cargo test --workspace -- --nocapture

# Run specific crate tests
test-core:
	cargo test -p terminal-lock-core

test-auth:
	cargo test -p terminal-lock-auth

test-config:
	cargo test -p terminal-lock-config

# Check compilation without building
check:
	cargo check --workspace

# Format code
fmt:
	cargo fmt --all

# Check formatting
fmt-check:
	cargo fmt --all -- --check

# Run clippy lints
clippy:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

# Generate documentation
docs:
	cargo doc --workspace --no-deps --open

# Clean build artifacts
clean:
	cargo clean
	@rm -rf ~/.terminal_lock/session.* ~/.terminal_lock/lockout.* ~/.terminal_lock/animation.*

# Run in development mode
run:
	cargo run --bin terminal-lock -- auth

# Run setup wizard
setup:
	cargo run --bin terminal-lock -- setup

# Run preferences
preferences:
	cargo run --bin terminal-lock -- preferences

# Check status
status:
	cargo run --bin terminal-lock -- status

# Reset attempts
reset:
	cargo run --bin terminal-lock -- reset

# Run benchmarks
bench:
	cargo bench

# Build all examples
examples:
	cargo build --examples

# Watch for changes and rebuild
watch:
	cargo watch -x build

# Update dependencies
update:
	cargo update

# Audit dependencies for security
audit:
	cargo audit

# Create release tarball
package: release
	@mkdir -p dist
	@tar -czf dist/terminal-lock-$(shell cargo pkgid | cut -d'#' -f2).tar.gz \
		-C target/release terminal-lock \
		README.md LICENSE
	@echo "✓ Package created in dist/"

# Full CI check (format, clippy, test)
ci: fmt-check clippy test
	@echo "✓ All CI checks passed!"