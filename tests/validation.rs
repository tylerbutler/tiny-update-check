//! Tests for crate name validation
//!
//! Crate names must follow Cargo's rules:
//! - Non-empty
//! - ASCII alphanumeric, `-`, or `_` only
//! - Must start with an alphabetic character
//! - Maximum 64 characters

use tiny_update_check::{Error, UpdateChecker};

/// Helper to check if an error is InvalidCrateName
fn is_invalid_crate_name(err: &Error) -> bool {
    matches!(err, Error::InvalidCrateName(_))
}

#[test]
fn rejects_empty_crate_name() {
    let checker = UpdateChecker::new("", "1.0.0");
    let result = checker.check();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(is_invalid_crate_name(&err), "Expected InvalidCrateName, got: {err}");
}

#[test]
fn rejects_crate_name_with_spaces() {
    let checker = UpdateChecker::new("my crate", "1.0.0");
    let result = checker.check();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(is_invalid_crate_name(&err), "Expected InvalidCrateName, got: {err}");
}

#[test]
fn rejects_crate_name_with_special_chars() {
    let checker = UpdateChecker::new("my-crate!", "1.0.0");
    let result = checker.check();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(is_invalid_crate_name(&err), "Expected InvalidCrateName, got: {err}");
}

#[test]
fn rejects_crate_name_starting_with_number() {
    let checker = UpdateChecker::new("123crate", "1.0.0");
    let result = checker.check();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(is_invalid_crate_name(&err), "Expected InvalidCrateName, got: {err}");
}

#[test]
fn rejects_crate_name_starting_with_hyphen() {
    let checker = UpdateChecker::new("-my-crate", "1.0.0");
    let result = checker.check();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(is_invalid_crate_name(&err), "Expected InvalidCrateName, got: {err}");
}

#[test]
fn rejects_crate_name_starting_with_underscore() {
    let checker = UpdateChecker::new("_my_crate", "1.0.0");
    let result = checker.check();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(is_invalid_crate_name(&err), "Expected InvalidCrateName, got: {err}");
}

#[test]
fn rejects_crate_name_too_long() {
    let long_name = "a".repeat(65);
    let checker = UpdateChecker::new(long_name, "1.0.0");
    let result = checker.check();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(is_invalid_crate_name(&err), "Expected InvalidCrateName, got: {err}");
}

#[test]
fn accepts_valid_simple_name() {
    // This test will make a real HTTP request, so we just verify
    // it doesn't fail with InvalidCrateName
    let checker = UpdateChecker::new("serde", "0.0.1");
    let result = checker.check();
    // Should either succeed or fail with HTTP/Parse error, not InvalidCrateName
    if let Err(err) = &result {
        assert!(!is_invalid_crate_name(err), "Should not be InvalidCrateName: {err}");
    }
}

#[test]
fn accepts_valid_name_with_hyphens() {
    let checker = UpdateChecker::new("my-cool-crate", "0.0.1");
    let result = checker.check();
    if let Err(err) = &result {
        assert!(!is_invalid_crate_name(err), "Should not be InvalidCrateName: {err}");
    }
}

#[test]
fn accepts_valid_name_with_underscores() {
    let checker = UpdateChecker::new("my_cool_crate", "0.0.1");
    let result = checker.check();
    if let Err(err) = &result {
        assert!(!is_invalid_crate_name(err), "Should not be InvalidCrateName: {err}");
    }
}

#[test]
fn accepts_valid_name_with_numbers() {
    let checker = UpdateChecker::new("crate2", "0.0.1");
    let result = checker.check();
    if let Err(err) = &result {
        assert!(!is_invalid_crate_name(err), "Should not be InvalidCrateName: {err}");
    }
}

#[test]
fn accepts_max_length_name() {
    let max_name = "a".repeat(64);
    let checker = UpdateChecker::new(max_name, "0.0.1");
    let result = checker.check();
    if let Err(err) = &result {
        assert!(!is_invalid_crate_name(err), "Should not be InvalidCrateName: {err}");
    }
}
