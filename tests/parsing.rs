//! Tests for JSON parsing logic
//!
//! These tests verify that the parser correctly extracts `newest_version`
//! from various JSON formats returned by the crates.io API.

use tiny_update_check::extract_newest_version;

const REAL_RESPONSE: &str = include_str!("fixtures/serde_response.json");
const COMPACT_JSON: &str = include_str!("fixtures/compact.json");
const PRETTY_JSON: &str = include_str!("fixtures/pretty.json");
const SPACED_COLON: &str = include_str!("fixtures/spaced_colon.json");
const MISSING_CRATE: &str = include_str!("fixtures/missing_crate.json");
const MISSING_VERSION: &str = include_str!("fixtures/missing_version.json");

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
