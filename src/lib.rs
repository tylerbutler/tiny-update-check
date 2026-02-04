//! # tiny-update-check
//!
//! A minimal, lightweight crate update checker for Rust CLI applications.
//!
//! This crate provides a simple way to check if a newer version of your crate
//! is available on crates.io, with built-in caching to avoid excessive API requests.
//!
//! ## Features
//!
//! - **Minimal dependencies**: Only `ureq` and `semver`
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

use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

/// Information about an available update.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateInfo {
    /// The currently running version.
    pub current: String,
    /// The latest available version on crates.io.
    pub latest: String,
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
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HttpError(msg) => write!(f, "HTTP error: {msg}"),
            Self::ParseError(msg) => write!(f, "Parse error: {msg}"),
            Self::VersionError(msg) => write!(f, "Version error: {msg}"),
            Self::CacheError(msg) => write!(f, "Cache error: {msg}"),
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
            cache_dir: dirs::cache_dir(),
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

    /// Check for updates.
    ///
    /// Returns `Ok(Some(UpdateInfo))` if a newer version is available,
    /// `Ok(None)` if already on the latest version,
    /// or `Err` if the check failed.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails, the response cannot be parsed,
    /// or version comparison fails.
    pub fn check(&self) -> Result<Option<UpdateInfo>, Error> {
        let latest = self.get_latest_version()?;

        let current = semver::Version::parse(&self.current_version)
            .map_err(|e| Error::VersionError(format!("Invalid current version: {e}")))?;
        let latest_ver = semver::Version::parse(&latest)
            .map_err(|e| Error::VersionError(format!("Invalid latest version: {e}")))?;

        if latest_ver > current {
            Ok(Some(UpdateInfo {
                current: self.current_version.clone(),
                latest,
            }))
        } else {
            Ok(None)
        }
    }

    /// Get the latest version, using cache if available and fresh.
    fn get_latest_version(&self) -> Result<String, Error> {
        let cache_path = self.cache_path();

        // Check cache first
        if self.cache_duration > Duration::ZERO {
            if let Some(ref path) = cache_path {
                if let Some(cached) = self.read_cache(path) {
                    return Ok(cached);
                }
            }
        }

        // Fetch from crates.io
        let latest = self.fetch_latest_version()?;

        // Update cache
        if let Some(ref path) = cache_path {
            let _ = fs::write(path, &latest);
        }

        Ok(latest)
    }

    /// Get the cache file path.
    fn cache_path(&self) -> Option<PathBuf> {
        self.cache_dir
            .as_ref()
            .map(|d| d.join(format!("{}-update-check", self.crate_name)))
    }

    /// Read from cache if it exists and is fresh.
    fn read_cache(&self, path: &std::path::Path) -> Option<String> {
        let metadata = fs::metadata(path).ok()?;
        let modified = metadata.modified().ok()?;
        let age = SystemTime::now().duration_since(modified).ok()?;

        if age < self.cache_duration {
            fs::read_to_string(path).ok().map(|s| s.trim().to_string())
        } else {
            None
        }
    }

    /// Fetch the latest version from crates.io.
    fn fetch_latest_version(&self) -> Result<String, Error> {
        let url = format!("https://crates.io/api/v1/crates/{}", self.crate_name);

        let agent: ureq::Agent = ureq::Agent::config_builder()
            .timeout_global(Some(self.timeout))
            .user_agent(concat!(
                env!("CARGO_PKG_NAME"),
                "/",
                env!("CARGO_PKG_VERSION")
            ))
            .tls_config(build_tls_config())
            .build()
            .into();

        let body = agent
            .get(&url)
            .call()
            .map_err(|e| Error::HttpError(e.to_string()))?
            .into_body()
            .read_to_string()
            .map_err(|e| Error::HttpError(e.to_string()))?;

        // Parse JSON minimally - look for "newest_version":"X.Y.Z"
        let marker = r#""newest_version":""#;
        let start = body
            .find(marker)
            .ok_or_else(|| Error::ParseError("newest_version not found".to_string()))?
            + marker.len();
        let end = body[start..]
            .find('"')
            .ok_or_else(|| Error::ParseError("version end quote not found".to_string()))?
            + start;

        Ok(body[start..end].to_string())
    }
}

/// Build TLS configuration based on enabled features.
fn build_tls_config() -> ureq::tls::TlsConfig {
    #[cfg(not(any(feature = "native-tls", feature = "rustls")))]
    compile_error!("Either 'native-tls' or 'rustls' feature must be enabled");

    #[cfg(feature = "native-tls")]
    let provider = ureq::tls::TlsProvider::NativeTls;

    #[cfg(all(feature = "rustls", not(feature = "native-tls")))]
    let provider = ureq::tls::TlsProvider::Rustls;

    ureq::tls::TlsConfig::builder().provider(provider).build()
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
    }
}
