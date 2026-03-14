# Copilot Instructions

## Project Overview

A minimal, lightweight crate update checker for Rust CLI applications. Checks crates.io for newer versions with file-based caching. **Binary size is a primary design constraint** (~0.5MB vs ~1.4MB for alternatives) — avoid adding dependencies or features that increase binary size without justification.

## Commands

```bash
just build              # Debug build
just test               # Run all tests
just test-fast          # Run tests with cargo-nextest (parallel)
just lint               # Clippy lints (warnings are errors)
just format             # Format code
just check              # Format check + lint + test
just ci                 # Full CI: format check + lint + test + audit + build
just docs               # Build docs (warnings are errors)
```

Run a single test:

```bash
cargo test test_name                          # by name
cargo test --test integration                 # by test file
cargo test --test validation rejects_empty    # file + filter
```

Test with all features (as CI does):

```bash
cargo test --all-features
```

## Architecture

The crate exposes two parallel APIs sharing internal helpers:

- **Sync** (`src/lib.rs`): Uses `minreq` for HTTP. The `UpdateChecker` builder and `check()` convenience function.
- **Async** (`src/async.rs`): Uses `reqwest` for HTTP, behind the `async` feature flag. Mirrors the sync API.

Both APIs share core logic defined in `lib.rs`: `extract_newest_version()` (manual JSON parsing — no serde), `compare_versions()`, `read_cache()`, `validate_crate_name()`, and `truncate_message()`. These are `pub(crate)` functions.

Return types: `check()` returns `Option<UpdateInfo>`, `check_detailed()` returns `Option<DetailedUpdateInfo>` (includes optional message and response body).

### Feature Flags

- `native-tls` (default) — system TLS for `minreq`
- `rustls` — pure Rust TLS for `minreq`
- `async` — enables async module with `reqwest`
- `do-not-track` (default) — respects `DO_NOT_TRACK=1` env var
- `response-body` — includes raw crates.io JSON in `DetailedUpdateInfo`

`native-tls` and `rustls` are mutually exclusive TLS backends for the sync API.

## Conventions

- **Rust edition 2024**, MSRV 1.87, toolchain 1.89
- **Clippy**: `all`, `pedantic`, and `nursery` lint groups are set to `warn` in `Cargo.toml`. CI runs with `-D warnings`.
- **`unsafe` is forbidden** via `#[forbid(unsafe_code)]` in `Cargo.toml`
- **Conventional commits** required (e.g., `feat:`, `fix:`, `docs:`) — enforced by CI
- **Formatting**: `rustfmt.toml` sets `max_width = 100`
- **Changelogs**: Uses [changie](https://changie.dev/) — run `just change` to add an entry
- All builder methods use `#[must_use]` and return `Self` for chaining
