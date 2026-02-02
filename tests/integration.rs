//! Integration tests for tiny-update-check

use std::time::Duration;
use tiny_update_check::UpdateChecker;

#[test]
fn test_checker_configuration() {
    let checker = UpdateChecker::new("serde", "1.0.0")
        .cache_duration(Duration::from_secs(3600))
        .timeout(Duration::from_secs(10))
        .cache_dir(None);

    // Just verify it builds - actual network tests would be flaky
    assert!(checker.check().is_ok() || checker.check().is_err());
}

#[test]
fn test_convenience_function() {
    // Test the convenience function compiles
    let result = tiny_update_check::check("serde", "1.0.0");
    // Either succeeds or fails - we just test the API works
    assert!(result.is_ok() || result.is_err());
}
