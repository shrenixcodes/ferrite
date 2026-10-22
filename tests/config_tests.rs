//! Integration tests for the public `AppConfig` API.

use ferrite::config::{AppConfig, ConfigError, MAX_REFRESH_INTERVAL_MS, MIN_REFRESH_INTERVAL_MS};

#[test]
fn default_configuration_is_valid_and_matches_documented_value() {
    let config = AppConfig::default();
    assert_eq!(config.refresh_interval_ms(), 1_000);
}

#[test]
fn minimum_supported_interval_is_accepted() {
    let config = AppConfig::with_refresh_interval_ms(100).expect("100 ms must be valid");
    assert_eq!(config.refresh_interval_ms(), MIN_REFRESH_INTERVAL_MS);
}

#[test]
fn maximum_supported_interval_is_accepted() {
    let config = AppConfig::with_refresh_interval_ms(60_000).expect("60000 ms must be valid");
    assert_eq!(config.refresh_interval_ms(), MAX_REFRESH_INTERVAL_MS);
}

#[test]
fn interval_below_minimum_is_rejected_with_descriptive_error() {
    let result = AppConfig::with_refresh_interval_ms(99);
    assert_eq!(
        result,
        Err(ConfigError::RefreshIntervalOutOfRange { value: 99 })
    );
}

#[test]
fn interval_above_maximum_is_rejected_with_descriptive_error() {
    let result = AppConfig::with_refresh_interval_ms(60_001);
    assert_eq!(
        result,
        Err(ConfigError::RefreshIntervalOutOfRange { value: 60_001 })
    );
}

#[test]
fn a_mid_range_interval_round_trips_exactly() {
    let config = AppConfig::with_refresh_interval_ms(2_500).expect("2500 ms must be valid");
    assert_eq!(config.refresh_interval_ms(), 2_500);
}

#[test]
fn error_display_mentions_the_rejected_value_and_bounds() {
    let err = AppConfig::with_refresh_interval_ms(70_000).unwrap_err();
    let message = err.to_string();
    assert!(message.contains("70000"));
    assert!(message.contains("100"));
    assert!(message.contains("60000"));
}
