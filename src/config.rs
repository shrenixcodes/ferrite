//! Application configuration for Ferrite.
//!
//! This module defines [`AppConfig`], the validated configuration type used
//! throughout the application. It is intentionally free of any terminal
//! rendering or operating-system dependent code so that it can be tested in
//! isolation and reused by future milestones.

use std::fmt;

/// The smallest refresh interval accepted by [`AppConfig`], in milliseconds.
pub const MIN_REFRESH_INTERVAL_MS: u32 = 100;

/// The largest refresh interval accepted by [`AppConfig`], in milliseconds.
pub const MAX_REFRESH_INTERVAL_MS: u32 = 60_000;

/// The refresh interval used when no explicit value is supplied, in
/// milliseconds.
pub const DEFAULT_REFRESH_INTERVAL_MS: u32 = 1_000;

/// Errors that can occur while constructing or validating an [`AppConfig`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    /// The requested refresh interval fell outside the supported range.
    RefreshIntervalOutOfRange {
        /// The value that was rejected, in milliseconds.
        value: u32,
    },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::RefreshIntervalOutOfRange { value } => write!(
                f,
                "refresh interval of {value} ms is out of range: must be between \
                 {MIN_REFRESH_INTERVAL_MS} and {MAX_REFRESH_INTERVAL_MS} ms inclusive"
            ),
        }
    }
}

impl std::error::Error for ConfigError {}

/// Validated application configuration for Ferrite.
///
/// `AppConfig` currently stores only the refresh interval, which controls
/// how often (in milliseconds) the application is expected to refresh its
/// view of system state. Future milestones may extend this type, but it is
/// deliberately minimal for now.
///
/// The default configuration is always valid, and the only way to construct
/// an `AppConfig` with a custom refresh interval is through
/// [`AppConfig::with_refresh_interval_ms`], which validates its input and
/// returns a descriptive [`ConfigError`] on failure instead of panicking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppConfig {
    refresh_interval_ms: u32,
}

impl AppConfig {
    /// Creates an `AppConfig` with the given refresh interval, in
    /// milliseconds.
    ///
    /// The value must fall within
    /// [`MIN_REFRESH_INTERVAL_MS`]..=[`MAX_REFRESH_INTERVAL_MS`], inclusive.
    /// Values outside that range are rejected with
    /// [`ConfigError::RefreshIntervalOutOfRange`] rather than causing a
    /// panic.
    ///
    /// # Examples
    ///
    /// ```
    /// use ferrite::config::AppConfig;
    ///
    /// let config = AppConfig::with_refresh_interval_ms(500).unwrap();
    /// assert_eq!(config.refresh_interval_ms(), 500);
    ///
    /// assert!(AppConfig::with_refresh_interval_ms(50).is_err());
    /// ```
    pub fn with_refresh_interval_ms(refresh_interval_ms: u32) -> Result<Self, ConfigError> {
        if (MIN_REFRESH_INTERVAL_MS..=MAX_REFRESH_INTERVAL_MS).contains(&refresh_interval_ms) {
            Ok(Self {
                refresh_interval_ms,
            })
        } else {
            Err(ConfigError::RefreshIntervalOutOfRange {
                value: refresh_interval_ms,
            })
        }
    }

    /// Returns the configured refresh interval, in milliseconds.
    pub fn refresh_interval_ms(&self) -> u32 {
        self.refresh_interval_ms
    }
}

impl Default for AppConfig {
    /// Returns the default configuration, using
    /// [`DEFAULT_REFRESH_INTERVAL_MS`] as the refresh interval.
    ///
    /// The default is always valid by construction.
    fn default() -> Self {
        Self {
            refresh_interval_ms: DEFAULT_REFRESH_INTERVAL_MS,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_uses_documented_default_interval() {
        let config = AppConfig::default();
        assert_eq!(config.refresh_interval_ms(), DEFAULT_REFRESH_INTERVAL_MS);
        assert_eq!(config.refresh_interval_ms(), 1_000);
    }

    #[test]
    fn accepts_minimum_boundary_value() {
        let config = AppConfig::with_refresh_interval_ms(MIN_REFRESH_INTERVAL_MS).unwrap();
        assert_eq!(config.refresh_interval_ms(), 100);
    }

    #[test]
    fn accepts_maximum_boundary_value() {
        let config = AppConfig::with_refresh_interval_ms(MAX_REFRESH_INTERVAL_MS).unwrap();
        assert_eq!(config.refresh_interval_ms(), 60_000);
    }

    #[test]
    fn rejects_value_just_below_minimum() {
        let result = AppConfig::with_refresh_interval_ms(99);
        assert_eq!(
            result,
            Err(ConfigError::RefreshIntervalOutOfRange { value: 99 })
        );
    }

    #[test]
    fn rejects_value_just_above_maximum() {
        let result = AppConfig::with_refresh_interval_ms(60_001);
        assert_eq!(
            result,
            Err(ConfigError::RefreshIntervalOutOfRange { value: 60_001 })
        );
    }

    #[test]
    fn rejects_zero() {
        let result = AppConfig::with_refresh_interval_ms(0);
        assert!(result.is_err());
    }

    #[test]
    fn error_message_is_descriptive() {
        let err = AppConfig::with_refresh_interval_ms(1).unwrap_err();
        let message = err.to_string();
        assert!(message.contains("100"));
        assert!(message.contains("60000"));
        assert!(message.contains('1'));
    }
}
