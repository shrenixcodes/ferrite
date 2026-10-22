//! Application configuration for Ferrite.
//!
//! This module defines [`AppConfig`], the configuration type used
//! throughout the application. It is intentionally free of any terminal
//! rendering or operating-system dependent code so that it can be tested in
//! isolation and reused by future milestones.

/// The refresh interval used when no explicit value is supplied, in
/// milliseconds.
pub const DEFAULT_REFRESH_INTERVAL_MS: u32 = 1_000;

/// Application configuration for Ferrite.
///
/// `AppConfig` currently stores only the refresh interval, which controls
/// how often (in milliseconds) the application is expected to refresh its
/// view of system state. Future milestones may extend this type, but it is
/// deliberately minimal for now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppConfig {
    refresh_interval_ms: u32,
}

impl AppConfig {
    /// Returns the configured refresh interval, in milliseconds.
    pub fn refresh_interval_ms(&self) -> u32 {
        self.refresh_interval_ms
    }
}

impl Default for AppConfig {
    /// Returns the default configuration, using
    /// [`DEFAULT_REFRESH_INTERVAL_MS`] as the refresh interval.
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
    }
}
