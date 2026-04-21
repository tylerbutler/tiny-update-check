# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v1.1.3 - 2026-04-21


### Dependencies

#### Use minreq 3.0.0-rc.0 for native-tls and ureq 3.3.0 for rustls

The `native-tls` feature now uses `minreq 3.0.0-rc.0` (system TLS), which produces the smallest binary (~540 KB). The `rustls` feature keeps `ureq 3.3.0` because minreq's rustls backend uses `aws-lc-rs` rather than `ring`, adding ~1.7 MB. Both dependencies are now optional and only included when their respective feature is enabled.


## v1.1.2 - 2026-04-21


### Security

#### Replace minreq with ureq 3.3.0 to fix RUSTSEC-2026-0098 and RUSTSEC-2026-0099

`minreq 2.x` depends on `rustls 0.21` (EOL), which uses `rustls-webpki 0.101.x` — vulnerable to two name constraint validation bugs. No published version of `minreq` contains a fix. Switching to `ureq 3.3.0` resolves both advisories by pulling in `rustls-webpki 0.103.13`. The public API and feature flags (`native-tls`, `rustls`) are unchanged.


## v1.1.1 - 2026-03-14


### Dependencies

#### Replace manual JSON parsing with serde_json

Replaced ~25 lines of manual string parsing with `serde_json` for extracting `newest_version` from the crates.io API response. This fixes a bug where the parser could extract a version from a nested object instead of the top-level `crate` object, and reduces maintenance burden.


## v1.1.0 - 2026-03-14


### Added

#### Add update message support via message_url() builder method

Configure a URL to fetch a plain text message from when an update is detected. Messages are best-effort (failures are silent), only fetched when an update exists, and trimmed/truncated to 4KB. Accessed via the new DetailedUpdateInfo::message field using the check_detailed() method.

#### Add response-body feature flag and DetailedUpdateInfo struct

New response-body feature flag includes the raw crates.io API response in DetailedUpdateInfo::response_body, enabling custom field parsing. New check_detailed() method returns DetailedUpdateInfo with optional message and response_body fields, maintaining backward compatibility with UpdateInfo.


### Fixed

#### Fix reqwest rustls feature flag for async support

Updated the `async` feature's reqwest dependency to use the `rustls` feature instead of the deprecated `rustls-tls` feature.


### Changed

#### Bump minimum supported Rust version to 1.87

The minimum supported Rust version (MSRV) has been increased from 1.85 to 1.87.


### Dependencies

#### Replace dirs with manual cache dir lookup

Replaced the `dirs` crate with ~10 lines of platform-specific code for cache directory detection. This removes `dirs` and its transitive dependencies (`dirs-sys`, `redox_users`, `libredox`, `getrandom` 0.2), eliminating the duplicate `getrandom` version warning.

#### Replace ureq with minreq

Replaced `ureq` HTTP client with `minreq` for a significantly smaller dependency tree. `minreq` is designed for simple blocking HTTP requests, aligning with the crate's "tiny" philosophy. Both `native-tls` and `rustls` feature flags are preserved.

#### Upgrade reqwest from 0.12 to 0.13.2

#### Update semver and minreq dependency versions


## v1.0.0 - 2026-02-06

### Added

#### Add DO_NOT_TRACK environment variable support (#17)

### Changed

#### Simplify code and prepare for 1.0 release (#13)

#### Simplify code with idiomatic Rust patterns (#10)

### Fixed

#### Update Cargo.lock for 1.0.0 version

## v0.1.0 - 2026-02-03

### Added

#### Initial implementation of tiny-update-check

### Fixed

#### Correct template leftovers and enhance release pipeline (#2)

