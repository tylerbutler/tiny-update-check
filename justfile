#!/usr/bin/env just --justfile

# Common aliases for faster development
alias b := build
alias br := build-release
alias t := test
alias tf := test-fast
alias l := lint
alias f := format
alias fc := fmt-check
alias a := audit
alias c := check
alias tc := test-coverage
alias d := docs
alias gc := generate-configs
alias cc := check-configs
alias cl := changelog

export RUST_BACKTRACE := "1"

set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

# Default recipe - list available commands
default:
    @just --list

# ============================================
# Build Commands
# ============================================

# Build the project in debug mode
build *ARGS='':
    cargo build {{ARGS}}

# Build the project in release mode
build-release *ARGS='':
    cargo build --release {{ARGS}}

# ============================================
# Test Commands
# ============================================

# Run all tests
test *ARGS='':
    cargo test {{ARGS}}

# Run tests with nextest (faster parallel execution)
test-fast *ARGS='':
    cargo nextest run {{ARGS}}

# Run tests with coverage (generates lcov.info)
test-coverage:
    #!/usr/bin/env bash
    set -euo pipefail
    source <(cargo llvm-cov show-env --export-prefix)
    cargo build --all-features
    cargo llvm-cov --no-clean --all-features --lcov --output-path lcov.info

# Generate HTML coverage report
coverage-html:
    cargo llvm-cov nextest --all-features --html --output-dir coverage

# Open coverage report in browser
coverage-report: coverage-html
    open coverage/html/index.html || xdg-open coverage/html/index.html 2>/dev/null || echo "Open coverage/html/index.html manually"

# ============================================
# Lint & Format Commands
# ============================================

# Run clippy lints
lint *ARGS='':
    cargo clippy {{ARGS}} -- -D warnings

# Format code
format *ARGS='':
    cargo fmt --all -- {{ARGS}}

# Check formatting without changes
fmt-check:
    cargo fmt --all -- --check

# Run all checks (format, lint, test)
check: fmt-check lint test

# ============================================
# Security Commands
# ============================================

# Security audit
audit:
    cargo audit
    cargo deny check

# ============================================
# Documentation Commands
# ============================================

# Build documentation
docs:
    RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features

# Open documentation in browser
docs-open: docs
    cargo doc --no-deps --all-features --open

# ============================================
# Changelog & Config Commands
# ============================================

# Regenerate all configs from commit-types.json
generate-configs:
    python3 scripts/generate-cliff-configs.py
    python3 scripts/generate-commitlint-config.py

# Check that generated configs are in sync
check-configs:
    python3 scripts/check-configs-sync.py

# Generate changelog
changelog: generate-configs
    git-cliff -o CHANGELOG.md

# ============================================
# Binary Size Commands (Linux only)
# ============================================

# Record release binary size to metrics/binary-size.txt
[linux]
record-size:
    #!/usr/bin/env bash
    set -euo pipefail
    BINARY="target/release/rust-template"
    if [ ! -f "$BINARY" ]; then
        echo "Release binary not found. Run 'just build-release' first."
        exit 1
    fi
    VERSION=$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version')
    SIZE=$(stat --format="%s" "$BINARY")
    HUMAN=$(numfmt --to=iec --suffix=B "$SIZE")
    DATE=$(date +%Y-%m-%d)
    mkdir -p metrics
    echo "$DATE v$VERSION $SIZE $HUMAN" >> metrics/binary-size.txt
    echo "Recorded: $DATE v$VERSION $SIZE ($HUMAN)"

# ============================================
# Watch Commands
# ============================================

# Watch for changes and run tests
watch-test:
    cargo watch -x test

# Watch for changes and run clippy
watch-lint:
    cargo watch -x clippy

# ============================================
# CI & Release Commands
# ============================================

# Run all CI checks locally
ci: fmt-check lint check-configs test audit build

# PR checks (mimics CI workflow)
pr: fmt-check lint check-configs test-coverage audit build

# ============================================
# Utility Commands
# ============================================

# Clean build artifacts
clean:
    cargo clean

# Install the binary locally
install:
    cargo install --path .

# Update dependencies
update:
    cargo update
