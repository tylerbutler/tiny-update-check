//! Tests for async update checking (requires `async` feature)
//!
//! Run with: cargo test --features async --test async_check

#![cfg(feature = "async")]

use std::time::Duration;
use tiny_update_check::r#async::UpdateChecker;

#[tokio::test]
async fn async_checker_builds() {
    let _checker = UpdateChecker::new("serde", "1.0.0");
}

#[tokio::test]
async fn async_checker_with_options() {
    let _checker = UpdateChecker::new("serde", "1.0.0")
        .cache_duration(Duration::from_secs(3600))
        .timeout(Duration::from_secs(10))
        .include_prerelease(true);
}

#[tokio::test]
async fn async_check_real_crate() {
    let checker = UpdateChecker::new("serde", "0.0.1")
        .cache_duration(Duration::ZERO); // Disable cache for test

    let result = checker.check().await;
    assert!(result.is_ok(), "Check should succeed: {:?}", result.err());

    let update = result.unwrap();
    assert!(update.is_some(), "Should find update from 0.0.1");

    let info = update.unwrap();
    assert_eq!(info.current, "0.0.1");
    assert!(!info.latest.is_empty());
}

#[tokio::test]
async fn async_check_validates_crate_name() {
    let checker = UpdateChecker::new("", "1.0.0");
    let result = checker.check().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn async_convenience_function() {
    let result = tiny_update_check::r#async::check("serde", "0.0.1").await;
    assert!(result.is_ok());
}
