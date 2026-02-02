# tiny-update-check

A minimal, lightweight crate update checker for Rust CLI applications.

[![Crates.io](https://img.shields.io/crates/v/tiny-update-check.svg)](https://crates.io/crates/tiny-update-check)
[![Documentation](https://docs.rs/tiny-update-check/badge.svg)](https://docs.rs/tiny-update-check)
[![License](https://img.shields.io/crates/l/tiny-update-check.svg)](LICENSE-MIT)

## Why?

Existing update checker crates add significant binary bloat:

| Crate | Binary Impact |
|-------|---------------|
| **tiny-update-check** | ~0.5MB |
| update-informer | ~1.4MB |
| updates | ~1.8MB |

This crate achieves minimal size by:
- Using `native-tls` (system TLS) instead of bundling rustls
- Minimal JSON parsing (string search instead of serde)
- Simple file-based caching

## Installation

```toml
[dependencies]
tiny-update-check = "0.1"
```

For pure-Rust TLS (larger binary, but better cross-compilation):

```toml
[dependencies]
tiny-update-check = { version = "0.1", default-features = false, features = ["rustls"] }
```

## Usage

### Simple

```rust
use tiny_update_check::check;

fn main() {
    // Check at end of your CLI's main function
    if let Ok(Some(update)) = check(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")) {
        eprintln!("Update available: {} -> {}", update.current, update.latest);
    }
}
```

### With Configuration

```rust
use tiny_update_check::UpdateChecker;
use std::time::Duration;

fn main() {
    let checker = UpdateChecker::new(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
        .cache_duration(Duration::from_secs(60 * 60)) // 1 hour cache
        .timeout(Duration::from_secs(10));            // 10 second timeout

    match checker.check() {
        Ok(Some(update)) => {
            eprintln!("New version {} available!", update.latest);
            eprintln!("You have version {}", update.current);
        }
        Ok(None) => {} // Already on latest
        Err(e) => {
            // Silently ignore errors - don't disrupt the user's workflow
            log::debug!("Update check failed: {}", e);
        }
    }
}
```

### Disable Caching

```rust
use tiny_update_check::UpdateChecker;
use std::time::Duration;

let checker = UpdateChecker::new("my-crate", "1.0.0")
    .cache_duration(Duration::ZERO);  // Always fetch fresh
```

## Features

- `native-tls` (default) - Uses system TLS libraries, smaller binary
- `rustls` - Pure Rust TLS, no system dependencies

## How It Works

1. Checks cache file (`~/.cache/<crate>-update-check`) for recent version info
2. If cache is stale (default: 24 hours), queries crates.io API
3. Compares versions using semver
4. Returns `Some(UpdateInfo)` if newer version exists

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
