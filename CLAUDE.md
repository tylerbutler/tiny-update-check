# CLAUDE.md

This file provides guidance to Claude Code when working with tiny-update-check.

## Project Overview

A minimal, lightweight crate update checker for Rust CLI applications. Checks crates.io for newer versions with simple file-based caching.

**Key design goals:**
- Minimal binary size impact (~0.5MB vs ~1.4MB for alternatives)
- Simple API: `check(name, version)` or builder pattern
- Uses system TLS (`native-tls`) by default

## Common Commands

```bash
just build          # Build debug
just test           # Run tests
just lint           # Run clippy
just check          # All local checks
cargo doc --open    # View documentation
```

## Architecture

### Core Components

- `UpdateChecker` - Builder struct for configuring update checks
- `UpdateInfo` - Return type containing current/latest versions
- `Error` - Error enum for various failure modes
- `check()` - Convenience function for simple use cases

### Dependencies

Minimal by design:
- `ureq` - HTTP client (with `native-tls` or `rustls` feature)
- `semver` - Version parsing and comparison
- `dirs` - System cache directory detection

### Feature Flags

- `native-tls` (default) - Uses system TLS, smaller binary
- `rustls` - Pure Rust TLS, better cross-compilation

## Key Files

| File | Purpose |
|------|---------|
| `src/lib.rs` | All library code (single file) |
| `tests/integration.rs` | Integration tests |
| `Cargo.toml` | Package manifest with features |

## API Design

```rust
// Simple usage
if let Ok(Some(update)) = tiny_update_check::check("my-crate", "1.0.0") {
    eprintln!("Update: {} -> {}", update.current, update.latest);
}

// Builder pattern
let checker = UpdateChecker::new("my-crate", "1.0.0")
    .cache_duration(Duration::from_secs(3600))
    .timeout(Duration::from_secs(10));
```

## Testing

- Unit tests in `src/lib.rs` test configuration and error handling
- Integration tests verify API works (network tests are inherently flaky)
- Doc tests ensure examples compile
