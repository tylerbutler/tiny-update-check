//! # tiny-update-check
//!
//! A minimal, lightweight crate update checker for Rust CLI applications.
//!
//! This crate provides a simple way to check if a newer version of your crate
//! is available on crates.io, with built-in caching to avoid excessive API requests.
//!
//! ## Features
//!
//! - **Minimal dependencies**: Only `minreq` and `semver`
//! - **Small binary impact**: ~0.5MB with `native-tls` (vs ~1.4MB for alternatives)
//! - **Simple file-based caching**: Configurable cache duration (default: 24 hours)
//! - **TLS flexibility**: Choose `native-tls` (default) or `rustls`
//!
//! ## Quick Start
//!
//! ```no_run
//! use tiny_update_check::UpdateChecker;
//!
//! let checker = UpdateChecker::new("my-crate", "1.0.0");
//! if let Ok(Some(update)) = checker.check() {
//!     eprintln!("Update available: {} -> {}", update.current, update.latest);
//! }
//! ```
//!
//! ## With Custom Configuration
//!
//! ```no_run
//! use tiny_update_check::UpdateChecker;
//! use std::time::Duration;
//!
//! let checker = UpdateChecker::new("my-crate", "1.0.0")
//!     .cache_duration(Duration::from_secs(60 * 60)) // 1 hour
//!     .timeout(Duration::from_secs(10));
//!
//! if let Ok(Some(update)) = checker.check() {
//!     eprintln!("New version {} released!", update.latest);
//! }
//! ```
//!
//! ## Feature Flags
//!
//! - `native-tls` (default): Uses system TLS, smaller binary size
//! - `rustls`: Pure Rust TLS, better for cross-compilation
//! - `async`: Enables async support using `reqwest`
//! - `do-not-track` (default): Respects [`DO_NOT_TRACK`] environment variable
//! - `response-body`: Includes the raw crates.io response body in [`UpdateInfo`]
//!
//! ## Update Messages
//!
//! You can attach a message to update notifications by hosting a plain text file
//! at a URL and configuring the checker with [`UpdateChecker::message_url`]:
//!
//! ```no_run
//! use tiny_update_check::UpdateChecker;
//!
//! let checker = UpdateChecker::new("my-crate", "1.0.0")
//!     .message_url("https://example.com/my-crate-update-message.txt");
//!
//! if let Ok(Some(update)) = checker.check() {
//!     eprintln!("Update available: {} -> {}", update.current, update.latest);
//!     if let Some(msg) = &update.message {
//!         eprintln!("{msg}");
//!     }
//! }
//! ```
//!
//! ## `DO_NOT_TRACK` Support
//!
//! When the `do-not-track` feature is enabled (default), the checker respects
//! the [`DO_NOT_TRACK`] environment variable standard. If `DO_NOT_TRACK=1` is set,
//! update checks will return `Ok(None)` without making network requests.
//!
//! To disable `DO_NOT_TRACK` support, disable the feature at compile time:
//!
//! ```toml
//! [dependencies]
//! tiny-update-check = { version = "0.1", default-features = false, features = ["native-tls"] }
//! ```
//!
//! [`DO_NOT_TRACK`]: https://consoledonottrack.com/

/// Async update checking module (requires `async` feature).
///
/// This module provides async versions of the update checker using `reqwest`.
#[cfg(feature = "async")]
pub mod r#async;

use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

pub(crate) const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

const MAX_MESSAGE_SIZE: usize = 4096;

/// Trim and truncate a message body to at most [`MAX_MESSAGE_SIZE`] bytes,
/// splitting on a valid UTF-8 char boundary.
///
/// Returns `None` if the input is empty or whitespace-only.
pub(crate) fn truncate_message(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.len() > MAX_MESSAGE_SIZE {
        let mut end = MAX_MESSAGE_SIZE;
        while !trimmed.is_char_boundary(end) {
            end -= 1;
        }
        Some(trimmed[..end].to_string())
    } else {
        Some(trimmed.to_string())
    }
}

/// Information about an available update.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateInfo {
    /// The currently running version.
    pub current: String,
    /// The latest available version on crates.io.
    pub latest: String,
    /// An optional message from the crate author.
    ///
    /// Populated when [`UpdateChecker::message_url`] is configured and the
    /// message was successfully fetched. The message is plain text, trimmed,
    /// and truncated to 4KB.
    pub message: Option<String>,
    /// The raw response body from crates.io.
    ///
    /// Only available when the `response-body` feature is enabled. This lets
    /// you extract any field from the crates.io API response using your own
    /// parsing logic.
    ///
    /// This is `None` when the version was served from cache.
    #[cfg(feature = "response-body")]
    pub response_body: Option<String>,
}

/// Errors that can occur during update checking.
#[derive(Debug)]
pub enum Error {
    /// Failed to make HTTP request to crates.io.
    HttpError(String),
    /// Failed to parse response from crates.io.
    ParseError(String),
    /// Failed to parse version string.
    VersionError(String),
    /// Cache I/O error.
    CacheError(String),
    /// Invalid crate name provided.
    InvalidCrateName(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HttpError(msg) => write!(f, "HTTP error: {msg}"),
            Self::ParseError(msg) => write!(f, "Parse error: {msg}"),
            Self::VersionError(msg) => write!(f, "Version error: {msg}"),
            Self::CacheError(msg) => write!(f, "Cache error: {msg}"),
            Self::InvalidCrateName(msg) => write!(f, "Invalid crate name: {msg}"),
        }
    }
}

impl std::error::Error for Error {}

/// A lightweight update checker for crates.io.
///
/// # Example
///
/// ```no_run
/// use tiny_update_check::UpdateChecker;
///
/// let checker = UpdateChecker::new("my-crate", "1.0.0");
/// match checker.check() {
///     Ok(Some(update)) => println!("Update available: {}", update.latest),
///     Ok(None) => println!("Already on latest version"),
///     Err(e) => eprintln!("Failed to check for updates: {}", e),
/// }
/// ```
#[derive(Debug, Clone)]
pub struct UpdateChecker {
    crate_name: String,
    current_version: String,
    cache_duration: Duration,
    timeout: Duration,
    cache_dir: Option<PathBuf>,
    include_prerelease: bool,
    message_url: Option<String>,
}

impl UpdateChecker {
    /// Create a new update checker for the given crate.
    ///
    /// # Arguments
    ///
    /// * `crate_name` - The name of your crate on crates.io
    /// * `current_version` - The currently running version (typically from `env!("CARGO_PKG_VERSION")`)
    #[must_use]
    pub fn new(crate_name: impl Into<String>, current_version: impl Into<String>) -> Self {
        Self {
            crate_name: crate_name.into(),
            current_version: current_version.into(),
            cache_duration: Duration::from_secs(24 * 60 * 60), // 24 hours
            timeout: Duration::from_secs(5),
            cache_dir: cache_dir(),
            include_prerelease: false,
            message_url: None,
        }
    }

    /// Set the cache duration. Defaults to 24 hours.
    ///
    /// Set to `Duration::ZERO` to disable caching.
    #[must_use]
    pub const fn cache_duration(mut self, duration: Duration) -> Self {
        self.cache_duration = duration;
        self
    }

    /// Set the HTTP request timeout. Defaults to 5 seconds.
    #[must_use]
    pub const fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set a custom cache directory. Defaults to system cache directory.
    ///
    /// Set to `None` to disable caching.
    #[must_use]
    pub fn cache_dir(mut self, dir: Option<PathBuf>) -> Self {
        self.cache_dir = dir;
        self
    }

    /// Include pre-release versions in update checks. Defaults to `false`.
    ///
    /// When `false` (the default), versions like `2.0.0-alpha.1` or `2.0.0-beta`
    /// will not be reported as available updates. Set to `true` to receive
    /// notifications about pre-release versions.
    #[must_use]
    pub const fn include_prerelease(mut self, include: bool) -> Self {
        self.include_prerelease = include;
        self
    }

    /// Set a URL to fetch an update message from.
    ///
    /// When an update is available, the checker will make a separate HTTP request
    /// to this URL and include the response as [`UpdateInfo::message`]. The URL
    /// should serve plain text.
    ///
    /// The fetch is best-effort: if it fails, the update check still succeeds
    /// with `message` set to `None`. The message is trimmed and truncated to 4KB.
    #[must_use]
    pub fn message_url(mut self, url: impl Into<String>) -> Self {
        self.message_url = Some(url.into());
        self
    }

    /// Check for updates.
    ///
    /// Returns `Ok(Some(UpdateInfo))` if a newer version is available,
    /// `Ok(None)` if already on the latest version (or if `DO_NOT_TRACK=1` is set
    /// and the `do-not-track` feature is enabled),
    /// or `Err` if the check failed.
    ///
    /// # Errors
    ///
    /// Returns an error if the crate name is invalid, the HTTP request fails,
    /// the response cannot be parsed, or version comparison fails.
    pub fn check(&self) -> Result<Option<UpdateInfo>, Error> {
        #[cfg(feature = "do-not-track")]
        if do_not_track_enabled() {
            return Ok(None);
        }

        validate_crate_name(&self.crate_name)?;
        #[cfg(feature = "response-body")]
        let (latest, response_body) = self.get_latest_version()?;
        #[cfg(not(feature = "response-body"))]
        let (latest, _) = self.get_latest_version()?;

        let mut update = compare_versions(&self.current_version, latest, self.include_prerelease)?;

        if let Some(ref mut info) = update {
            if let Some(ref url) = self.message_url {
                info.message = self.fetch_message(url);
            }
            #[cfg(feature = "response-body")]
            {
                info.response_body = response_body;
            }
        }

        Ok(update)
    }

    /// Get the latest version, using cache if available and fresh.
    fn get_latest_version(&self) -> Result<(String, Option<String>), Error> {
        let path = self
            .cache_dir
            .as_ref()
            .map(|d| d.join(format!("{}-update-check", self.crate_name)));

        // Check cache first
        if self.cache_duration > Duration::ZERO {
            if let Some(ref path) = path {
                if let Some(cached) = read_cache(path, self.cache_duration) {
                    return Ok((cached, None));
                }
            }
        }

        // Fetch from crates.io
        let (latest, response_body) = self.fetch_latest_version()?;

        // Update cache
        if let Some(ref path) = path {
            let _ = fs::write(path, &latest);
        }

        Ok((latest, response_body))
    }

    /// Fetch the latest version from crates.io.
    fn fetch_latest_version(&self) -> Result<(String, Option<String>), Error> {
        let url = format!("https://crates.io/api/v1/crates/{}", self.crate_name);

        let response = minreq::get(&url)
            .with_timeout(self.timeout.as_secs())
            .with_header("User-Agent", USER_AGENT)
            .send()
            .map_err(|e| Error::HttpError(e.to_string()))?;

        let body = response
            .as_str()
            .map_err(|e| Error::HttpError(e.to_string()))?;

        let version = extract_newest_version(body)?;

        #[cfg(feature = "response-body")]
        return Ok((version, Some(body.to_string())));

        #[cfg(not(feature = "response-body"))]
        Ok((version, None))
    }

    /// Fetch a plain text message from the configured URL.
    ///
    /// Best-effort: returns `None` on any failure.
    fn fetch_message(&self, url: &str) -> Option<String> {
        let response = minreq::get(url)
            .with_timeout(self.timeout.as_secs())
            .with_header("User-Agent", USER_AGENT)
            .send()
            .ok()?;

        let body = response.as_str().ok()?;
        truncate_message(body)
    }
}

/// Compare current and latest versions, returning `UpdateInfo` if an update is available.
pub(crate) fn compare_versions(
    current_version: &str,
    latest: String,
    include_prerelease: bool,
) -> Result<Option<UpdateInfo>, Error> {
    let current = semver::Version::parse(current_version)
        .map_err(|e| Error::VersionError(format!("Invalid current version: {e}")))?;
    let latest_ver = semver::Version::parse(&latest)
        .map_err(|e| Error::VersionError(format!("Invalid latest version: {e}")))?;

    if !include_prerelease && !latest_ver.pre.is_empty() {
        return Ok(None);
    }

    if latest_ver > current {
        Ok(Some(UpdateInfo {
            current: current_version.to_string(),
            latest,
            message: None,
            #[cfg(feature = "response-body")]
            response_body: None,
        }))
    } else {
        Ok(None)
    }
}

/// Read from cache if it exists and is fresh.
pub(crate) fn read_cache(path: &std::path::Path, cache_duration: Duration) -> Option<String> {
    let metadata = fs::metadata(path).ok()?;
    let modified = metadata.modified().ok()?;
    let age = SystemTime::now().duration_since(modified).ok()?;

    if age < cache_duration {
        fs::read_to_string(path).ok().map(|s| s.trim().to_string())
    } else {
        None
    }
}

/// Extract the `newest_version` field from a crates.io API response.
///
/// This function parses the JSON response without requiring a full JSON parser,
/// handling various whitespace formats that the API might return.
pub(crate) fn extract_newest_version(body: &str) -> Result<String, Error> {
    // Find the "crate" object first to ensure we're in the right context
    let crate_start = body
        .find(r#""crate""#)
        .ok_or_else(|| Error::ParseError("'crate' field not found in response".to_string()))?;

    // Search from the crate field onward
    let search_region = &body[crate_start..];

    // Find "newest_version" within the crate object
    let version_key = r#""newest_version""#;
    let key_pos = search_region.find(version_key).ok_or_else(|| {
        Error::ParseError("'newest_version' field not found in response".to_string())
    })?;

    // Move past the key
    let after_key = &search_region[key_pos + version_key.len()..];

    // Find the colon (handles optional whitespace)
    let colon_pos = after_key.find(':').ok_or_else(|| {
        Error::ParseError("malformed JSON: missing colon after newest_version".to_string())
    })?;

    // Move past the colon and any whitespace
    let after_colon = &after_key[colon_pos + 1..];
    let after_colon_trimmed = after_colon.trim_start();

    // Find the opening quote
    if !after_colon_trimmed.starts_with('"') {
        return Err(Error::ParseError(
            "malformed JSON: expected quote after newest_version colon".to_string(),
        ));
    }

    // Extract the version string (everything until the closing quote)
    let version_start = &after_colon_trimmed[1..];
    let quote_end = version_start
        .find('"')
        .ok_or_else(|| Error::ParseError("malformed JSON: unclosed version string".to_string()))?;

    Ok(version_start[..quote_end].to_string())
}

/// Check if the `DO_NOT_TRACK` environment variable is set to a truthy value.
///
/// Returns `true` if `DO_NOT_TRACK` is set to `1` or `true` (case-insensitive).
#[cfg(feature = "do-not-track")]
pub(crate) fn do_not_track_enabled() -> bool {
    std::env::var("DO_NOT_TRACK")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

/// Validate a crate name according to Cargo's rules.
///
/// Valid crate names must:
/// - Be non-empty
/// - Start with an ASCII alphabetic character
/// - Contain only ASCII alphanumeric characters, `-`, or `_`
/// - Be at most 64 characters long
fn validate_crate_name(name: &str) -> Result<(), Error> {
    if name.is_empty() {
        return Err(Error::InvalidCrateName(
            "crate name cannot be empty".to_string(),
        ));
    }

    if name.len() > 64 {
        return Err(Error::InvalidCrateName(format!(
            "crate name exceeds 64 characters: {}",
            name.len()
        )));
    }

    let first_char = name.chars().next().unwrap(); // safe: checked non-empty
    if !first_char.is_ascii_alphabetic() {
        return Err(Error::InvalidCrateName(format!(
            "crate name must start with a letter, found: '{first_char}'"
        )));
    }

    for ch in name.chars() {
        if !ch.is_ascii_alphanumeric() && ch != '-' && ch != '_' {
            return Err(Error::InvalidCrateName(format!(
                "invalid character in crate name: '{ch}'"
            )));
        }
    }

    Ok(())
}

/// Returns the platform-specific user cache directory.
///
/// - **Linux**: `$XDG_CACHE_HOME` or `$HOME/.cache`
/// - **macOS**: `$HOME/Library/Caches`
/// - **Windows**: `%LOCALAPPDATA%`
pub(crate) fn cache_dir() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Library/Caches"))
    }

    #[cfg(target_os = "linux")]
    {
        std::env::var_os("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))
    }

    #[cfg(target_os = "windows")]
    {
        std::env::var_os("LOCALAPPDATA").map(PathBuf::from)
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        None
    }
}
/// Convenience function to check for updates with default settings.
///
/// # Example
///
/// ```no_run
/// if let Ok(Some(update)) = tiny_update_check::check("my-crate", "1.0.0") {
///     eprintln!("Update available: {} -> {}", update.current, update.latest);
/// }
/// ```
///
/// # Errors
///
/// Returns an error if the update check fails.
pub fn check(
    crate_name: impl Into<String>,
    current_version: impl Into<String>,
) -> Result<Option<UpdateInfo>, Error> {
    UpdateChecker::new(crate_name, current_version).check()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_info_display() {
        let info = UpdateInfo {
            current: "1.0.0".to_string(),
            latest: "2.0.0".to_string(),
            message: None,
            #[cfg(feature = "response-body")]
            response_body: None,
        };
        assert_eq!(info.current, "1.0.0");
        assert_eq!(info.latest, "2.0.0");
    }

    #[test]
    fn test_checker_builder() {
        let checker = UpdateChecker::new("test-crate", "1.0.0")
            .cache_duration(Duration::from_secs(3600))
            .timeout(Duration::from_secs(10));

        assert_eq!(checker.crate_name, "test-crate");
        assert_eq!(checker.current_version, "1.0.0");
        assert_eq!(checker.cache_duration, Duration::from_secs(3600));
        assert_eq!(checker.timeout, Duration::from_secs(10));
        assert!(checker.message_url.is_none());
    }

    #[test]
    fn test_cache_disabled() {
        let checker = UpdateChecker::new("test-crate", "1.0.0")
            .cache_duration(Duration::ZERO)
            .cache_dir(None);

        assert_eq!(checker.cache_duration, Duration::ZERO);
        assert!(checker.cache_dir.is_none());
    }

    #[test]
    fn test_error_display() {
        let err = Error::HttpError("connection failed".to_string());
        assert_eq!(err.to_string(), "HTTP error: connection failed");

        let err = Error::ParseError("invalid json".to_string());
        assert_eq!(err.to_string(), "Parse error: invalid json");

        let err = Error::InvalidCrateName("empty".to_string());
        assert_eq!(err.to_string(), "Invalid crate name: empty");
    }

    #[test]
    fn test_include_prerelease_default() {
        let checker = UpdateChecker::new("test-crate", "1.0.0");
        assert!(!checker.include_prerelease);
    }

    #[test]
    fn test_include_prerelease_enabled() {
        let checker = UpdateChecker::new("test-crate", "1.0.0").include_prerelease(true);
        assert!(checker.include_prerelease);
    }

    #[test]
    fn test_include_prerelease_disabled() {
        let checker = UpdateChecker::new("test-crate", "1.0.0").include_prerelease(false);
        assert!(!checker.include_prerelease);
    }

    // Parsing tests (moved from tests/parsing.rs)
    const REAL_RESPONSE: &str = include_str!("../tests/fixtures/serde_response.json");
    const COMPACT_JSON: &str = include_str!("../tests/fixtures/compact.json");
    const PRETTY_JSON: &str = include_str!("../tests/fixtures/pretty.json");
    const SPACED_COLON: &str = include_str!("../tests/fixtures/spaced_colon.json");
    const MISSING_CRATE: &str = include_str!("../tests/fixtures/missing_crate.json");
    const MISSING_VERSION: &str = include_str!("../tests/fixtures/missing_version.json");

    #[test]
    fn parses_real_crates_io_response() {
        let version = extract_newest_version(REAL_RESPONSE).unwrap();
        assert_eq!(version, "1.0.228");
    }

    #[test]
    fn parses_compact_json() {
        let version = extract_newest_version(COMPACT_JSON).unwrap();
        assert_eq!(version, "2.0.0");
    }

    #[test]
    fn parses_pretty_json() {
        let version = extract_newest_version(PRETTY_JSON).unwrap();
        assert_eq!(version, "3.1.4");
    }

    #[test]
    fn parses_whitespace_around_colon() {
        let version = extract_newest_version(SPACED_COLON).unwrap();
        assert_eq!(version, "1.2.3");
    }

    #[test]
    fn fails_on_missing_crate_field() {
        let result = extract_newest_version(MISSING_CRATE);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("crate"),
            "Error should mention 'crate' field: {err}"
        );
    }

    #[test]
    fn fails_on_missing_newest_version() {
        let result = extract_newest_version(MISSING_VERSION);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("newest_version"),
            "Error should mention 'newest_version' field: {err}"
        );
    }

    #[test]
    fn fails_on_empty_input() {
        let result = extract_newest_version("");
        assert!(result.is_err());
    }

    #[test]
    fn fails_on_malformed_json() {
        let result = extract_newest_version("not json at all");
        assert!(result.is_err());
    }

    // DO_NOT_TRACK tests
    #[cfg(feature = "do-not-track")]
    mod do_not_track_tests {
        use super::*;

        #[test]
        fn do_not_track_detects_1() {
            temp_env::with_var("DO_NOT_TRACK", Some("1"), || {
                assert!(do_not_track_enabled());
            });
        }

        #[test]
        fn do_not_track_detects_true() {
            temp_env::with_var("DO_NOT_TRACK", Some("true"), || {
                assert!(do_not_track_enabled());
            });
        }

        #[test]
        fn do_not_track_detects_true_case_insensitive() {
            temp_env::with_var("DO_NOT_TRACK", Some("TRUE"), || {
                assert!(do_not_track_enabled());
            });
        }

        #[test]
        fn do_not_track_ignores_other_values() {
            temp_env::with_var("DO_NOT_TRACK", Some("0"), || {
                assert!(!do_not_track_enabled());
            });
            temp_env::with_var("DO_NOT_TRACK", Some("false"), || {
                assert!(!do_not_track_enabled());
            });
            temp_env::with_var("DO_NOT_TRACK", Some("yes"), || {
                assert!(!do_not_track_enabled());
            });
        }

        #[test]
        fn do_not_track_disabled_when_unset() {
            temp_env::with_var("DO_NOT_TRACK", None::<&str>, || {
                assert!(!do_not_track_enabled());
            });
        }
    }

    #[test]
    fn test_message_url_default() {
        let checker = UpdateChecker::new("test-crate", "1.0.0");
        assert!(checker.message_url.is_none());
    }

    #[test]
    fn test_message_url_builder() {
        let checker = UpdateChecker::new("test-crate", "1.0.0")
            .message_url("https://example.com/message.txt");
        assert_eq!(
            checker.message_url.as_deref(),
            Some("https://example.com/message.txt")
        );
    }

    #[test]
    fn test_message_url_chainable() {
        let checker = UpdateChecker::new("test-crate", "1.0.0")
            .cache_duration(Duration::from_secs(3600))
            .message_url("https://example.com/msg.txt")
            .timeout(Duration::from_secs(10));
        assert_eq!(
            checker.message_url.as_deref(),
            Some("https://example.com/msg.txt")
        );
        assert_eq!(checker.timeout, Duration::from_secs(10));
    }

    #[test]
    fn test_compare_versions_returns_none_message() {
        let result = compare_versions("1.0.0", "2.0.0".to_string(), false)
            .unwrap()
            .unwrap();
        assert!(result.message.is_none());
    }

    #[test]
    fn test_update_info_with_message() {
        let info = UpdateInfo {
            current: "1.0.0".to_string(),
            latest: "2.0.0".to_string(),
            message: Some("Please update!".to_string()),
            #[cfg(feature = "response-body")]
            response_body: None,
        };
        assert_eq!(info.message.as_deref(), Some("Please update!"));
    }

    #[cfg(feature = "response-body")]
    #[test]
    fn test_update_info_with_response_body() {
        let info = UpdateInfo {
            current: "1.0.0".to_string(),
            latest: "2.0.0".to_string(),
            message: None,
            response_body: Some("{\"crate\":{}}".to_string()),
        };
        assert_eq!(info.response_body.as_deref(), Some("{\"crate\":{}}"));
    }

    #[test]
    fn test_truncate_message_empty() {
        assert_eq!(truncate_message(""), None);
    }

    #[test]
    fn test_truncate_message_whitespace_only() {
        assert_eq!(truncate_message("   \n\t  "), None);
    }

    #[test]
    fn test_truncate_message_ascii_within_limit() {
        assert_eq!(
            truncate_message("hello world"),
            Some("hello world".to_string())
        );
    }

    #[test]
    fn test_truncate_message_trims_whitespace() {
        assert_eq!(
            truncate_message("  hello world  \n"),
            Some("hello world".to_string())
        );
    }

    #[test]
    fn test_truncate_message_exactly_at_limit() {
        let msg = "a".repeat(4096);
        let result = truncate_message(&msg).unwrap();
        assert_eq!(result.len(), 4096);
    }

    #[test]
    fn test_truncate_message_ascii_over_limit() {
        let msg = "a".repeat(5000);
        let result = truncate_message(&msg).unwrap();
        assert_eq!(result.len(), 4096);
    }

    #[test]
    fn test_truncate_message_multibyte_at_boundary() {
        // '€' is 3 bytes in UTF-8. Fill so the 4096 boundary falls mid-character.
        let unit = "€"; // 3 bytes
        let count = 4096 / 3 + 1; // enough to exceed 4096 bytes
        let msg: String = unit.repeat(count);
        let result = truncate_message(&msg).unwrap();
        assert!(result.len() <= 4096);
        // Must end on a valid char boundary (no panic on further use)
        assert!(result.is_char_boundary(result.len()));
        // Should be the largest multiple of 3 that fits
        assert_eq!(result.len(), (4096 / 3) * 3);
    }
}
