//! Async update checking using reqwest.
//!
//! This module provides async versions of the update checker, available
//! when the `async` feature is enabled.
//!
//! # Example
//!
//! ```no_run
//! use tiny_update_check::r#async::UpdateChecker;
//!
//! # async fn example() {
//! let checker = UpdateChecker::new("my-crate", "1.0.0");
//! if let Ok(Some(update)) = checker.check().await {
//!     eprintln!("Update available: {} -> {}", update.current, update.latest);
//! }
//! # }
//! ```

use std::path::PathBuf;
use std::time::Duration;

use crate::{extract_newest_version, validate_crate_name, Error, UpdateInfo};

/// An async update checker for crates.io.
///
/// This is the async equivalent of [`crate::UpdateChecker`], using `reqwest`
/// for HTTP requests instead of `ureq`.
#[derive(Debug, Clone)]
pub struct UpdateChecker {
    crate_name: String,
    current_version: String,
    cache_duration: Duration,
    timeout: Duration,
    cache_dir: Option<PathBuf>,
    include_prerelease: bool,
}

impl UpdateChecker {
    /// Create a new async update checker for the given crate.
    #[must_use]
    pub fn new(crate_name: impl Into<String>, current_version: impl Into<String>) -> Self {
        Self {
            crate_name: crate_name.into(),
            current_version: current_version.into(),
            cache_duration: Duration::from_secs(24 * 60 * 60),
            timeout: Duration::from_secs(5),
            cache_dir: dirs::cache_dir(),
            include_prerelease: false,
        }
    }

    /// Set the cache duration. Defaults to 24 hours.
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
    #[must_use]
    pub fn cache_dir(mut self, dir: Option<PathBuf>) -> Self {
        self.cache_dir = dir;
        self
    }

    /// Include pre-release versions in update checks. Defaults to `false`.
    #[must_use]
    pub const fn include_prerelease(mut self, include: bool) -> Self {
        self.include_prerelease = include;
        self
    }

    /// Check for updates asynchronously.
    ///
    /// Returns `Ok(Some(UpdateInfo))` if a newer version is available,
    /// `Ok(None)` if already on the latest version,
    /// or `Err` if the check failed.
    pub async fn check(&self) -> Result<Option<UpdateInfo>, Error> {
        validate_crate_name(&self.crate_name)?;
        let latest = self.get_latest_version().await?;

        let current = semver::Version::parse(&self.current_version)
            .map_err(|e| Error::VersionError(format!("Invalid current version: {e}")))?;
        let latest_ver = semver::Version::parse(&latest)
            .map_err(|e| Error::VersionError(format!("Invalid latest version: {e}")))?;

        if !self.include_prerelease && !latest_ver.pre.is_empty() {
            return Ok(None);
        }

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
    async fn get_latest_version(&self) -> Result<String, Error> {
        use std::fs;
        use std::time::SystemTime;

        let cache_path = self.cache_path();

        // Check cache first
        if self.cache_duration > Duration::ZERO {
            if let Some(ref path) = cache_path {
                if let Ok(metadata) = fs::metadata(path) {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(age) = SystemTime::now().duration_since(modified) {
                            if age < self.cache_duration {
                                if let Ok(cached) = fs::read_to_string(path) {
                                    return Ok(cached.trim().to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        // Fetch from crates.io
        let latest = self.fetch_latest_version().await?;

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

    /// Fetch the latest version from crates.io asynchronously.
    async fn fetch_latest_version(&self) -> Result<String, Error> {
        let url = format!("https://crates.io/api/v1/crates/{}", self.crate_name);

        let client = reqwest::Client::builder()
            .timeout(self.timeout)
            .user_agent(concat!(
                env!("CARGO_PKG_NAME"),
                "/",
                env!("CARGO_PKG_VERSION")
            ))
            .build()
            .map_err(|e| Error::HttpError(e.to_string()))?;

        let body = client
            .get(&url)
            .send()
            .await
            .map_err(|e| Error::HttpError(e.to_string()))?
            .text()
            .await
            .map_err(|e| Error::HttpError(e.to_string()))?;

        extract_newest_version(&body)
    }
}

/// Convenience function to check for updates asynchronously with default settings.
///
/// # Example
///
/// ```no_run
/// # async fn example() {
/// if let Ok(Some(update)) = tiny_update_check::r#async::check("my-crate", "1.0.0").await {
///     eprintln!("Update available: {} -> {}", update.current, update.latest);
/// }
/// # }
/// ```
pub async fn check(
    crate_name: impl Into<String>,
    current_version: impl Into<String>,
) -> Result<Option<UpdateInfo>, Error> {
    UpdateChecker::new(crate_name, current_version).check().await
}
