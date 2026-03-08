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
tiny-update-check = "1"
```

For pure-Rust TLS (larger binary, but better cross-compilation):

```toml
[dependencies]
tiny-update-check = { version = "1", default-features = false, features = ["rustls"] }
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

### Custom Cache Directory

```rust
use tiny_update_check::UpdateChecker;
use std::path::PathBuf;

// Use a custom cache directory
let checker = UpdateChecker::new("my-crate", "1.0.0")
    .cache_dir(Some(PathBuf::from("/tmp/my-cache")));

// Or disable caching entirely
let checker = UpdateChecker::new("my-crate", "1.0.0")
    .cache_dir(None);
```

### Pre-release Versions

By default, pre-release versions (e.g., `2.0.0-alpha.1`) are excluded from update
checks. Opt in to receive pre-release notifications:

```rust
use tiny_update_check::UpdateChecker;

let checker = UpdateChecker::new("my-crate", "1.0.0")
    .include_prerelease(true);
```

### Async Usage

Enable the `async` feature for async applications:

```toml
[dependencies]
tiny-update-check = { version = "1", features = ["async"] }
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

```rust,no_run
use tiny_update_check::r#async::UpdateChecker;

#[tokio::main]
async fn main() {
    let checker = UpdateChecker::new("my-crate", "1.0.0");
    if let Ok(Some(update)) = checker.check().await {
        eprintln!("Update available: {} -> {}", update.current, update.latest);
    }
}
```

## Features

| Feature | Default | Description |
|---------|---------|-------------|
| `native-tls` | ✅ | Uses system TLS libraries, smaller binary |
| `do-not-track` | ✅ | Respects the `DO_NOT_TRACK` environment variable |
| `rustls` | | Pure Rust TLS, no system dependencies |
| `async` | | Async support using `reqwest` |
| `response-body` | | Includes the raw crates.io response body in `UpdateInfo` |

### Update Messages

You can attach a message to update notifications by hosting a plain text file
at a URL:

```rust
use tiny_update_check::UpdateChecker;

let checker = UpdateChecker::new("my-crate", "1.0.0")
    .message_url("https://example.com/my-crate-update-message.txt");

if let Ok(Some(update)) = checker.check() {
    eprintln!("Update available: {} -> {}", update.current, update.latest);
    if let Some(msg) = &update.message {
        eprintln!("{msg}");
    }
}
```

The message is fetched only when an update is available. If the fetch fails, the
update check still succeeds with `message` set to `None`.

### Raw Response Body

Enable the `response-body` feature to access the full crates.io API response:

```toml
[dependencies]
tiny-update-check = { version = "1", features = ["response-body"] }
```

This adds a `response_body: Option<String>` field to `UpdateInfo`, letting you
extract any field from the crates.io response using your own parsing logic.

## Error Handling

The `Error` enum covers all failure modes:

| Variant | Cause |
|---------|-------|
| `HttpError` | Network or HTTP request failure |
| `ParseError` | Failed to parse crates.io response JSON |
| `VersionError` | Invalid semver version string |
| `CacheError` | Cache file I/O failure |
| `InvalidCrateName` | Crate name fails validation (empty, too long, invalid characters) |

Update checks are designed to fail gracefully — errors should typically be
logged and ignored so they don't disrupt the user's workflow.

## How It Works

1. Checks cache file (in platform cache directory) for recent version info
2. If cache is stale (default: 24 hours), queries crates.io API
3. Compares versions using semver
4. Returns `Some(UpdateInfo)` if newer version exists

Cache locations by platform:
- **Linux**: `$XDG_CACHE_HOME/<crate>-update-check` or `$HOME/.cache/<crate>-update-check`
- **macOS**: `$HOME/Library/Caches/<crate>-update-check`
- **Windows**: `%LOCALAPPDATA%\<crate>-update-check`

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
