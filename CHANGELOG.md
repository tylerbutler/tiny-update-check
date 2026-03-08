# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v1.0.0 - 2026-02-03

### Added

- Pre-release version filtering with `include_prerelease()` builder method (excluded by default)
- Crate name validation with `InvalidCrateName` error variant
- Async support via `async` feature flag using `reqwest`
- `do-not-track` feature flag (default) to respect `DO_NOT_TRACK` environment variable
- `response-body` feature flag to include raw crates.io response in `UpdateInfo`
- `message_url()` builder method for attaching update messages
- `cache_dir()` builder method for custom cache directory

### Changed

- Switched HTTP client from `ureq` to `minreq` for smaller binary size
- Removed `dirs` dependency; cache directory detection is now built-in
- Improved JSON parsing to handle whitespace variations in crates.io responses

## v0.1.0 - 2026-02-03

### Added

#### Initial implementation of tiny-update-check

### Fixed

#### Correct template leftovers and enhance release pipeline (#2)

