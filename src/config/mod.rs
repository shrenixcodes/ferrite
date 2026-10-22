//! Application configuration for Ferrite.
//!
//! This module defines [`AppConfig`], the validated configuration type used
//! throughout the application, and [`AppConfigBuilder`] for constructing
//! one from several possibly-untrusted values at once — in particular, when
//! loading settings from a TOML configuration file; see [`file`]. The
//! types in this top-level module are free of any terminal rendering or
//! filesystem access, so they can be tested in isolation.

pub mod file;

use std::fmt;

/// The smallest refresh interval accepted by [`AppConfig`], in milliseconds.
pub const MIN_REFRESH_INTERVAL_MS: u32 = 100;
/// The largest refresh interval accepted by [`AppConfig`], in milliseconds.
pub const MAX_REFRESH_INTERVAL_MS: u32 = 60_000;
/// The refresh interval used when no explicit value is supplied, in
/// milliseconds.
pub const DEFAULT_REFRESH_INTERVAL_MS: u32 = 1_000;

/// The smallest metric history capacity accepted by [`AppConfig`].
pub const MIN_HISTORY_CAPACITY: usize = 1;
/// The largest metric history capacity accepted by [`AppConfig`].
///
/// At the default 1-second refresh interval, 3,600 samples is one hour of
/// history; there is no need to retain more in memory for a monitor that
/// never persists history to disk.
pub const MAX_HISTORY_CAPACITY: usize = 3_600;
/// The history capacity used when no explicit value is supplied.
pub const DEFAULT_HISTORY_CAPACITY: usize = crate::history::DEFAULT_HISTORY_CAPACITY;

/// The smallest alert threshold accepted by [`AppConfig`], as a percentage.
pub const MIN_ALERT_THRESHOLD_PERCENT: f32 = 0.0;
/// The largest alert threshold accepted by [`AppConfig`], as a percentage.
pub const MAX_ALERT_THRESHOLD_PERCENT: f32 = 100.0;
/// The CPU alert threshold used when no explicit value is supplied.
pub const DEFAULT_CPU_ALERT_THRESHOLD_PERCENT: f32 = 90.0;
/// The memory alert threshold used when no explicit value is supplied.
pub const DEFAULT_MEMORY_ALERT_THRESHOLD_PERCENT: f32 = 90.0;
/// The disk alert threshold used when no explicit value is supplied.
pub const DEFAULT_DISK_ALERT_THRESHOLD_PERCENT: f32 = 90.0;

/// Which metric the process table is sorted by at startup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DefaultSortMode {
    /// Highest CPU usage first.
    #[default]
    Cpu,
    /// Highest memory usage first.
    Memory,
}

/// Errors that can occur while constructing or validating an [`AppConfig`].
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigError {
    /// The requested refresh interval fell outside the supported range.
    RefreshIntervalOutOfRange {
        /// The value that was rejected, in milliseconds.
        value: u32,
    },
    /// The requested history capacity fell outside the supported range.
    HistoryCapacityOutOfRange {
        /// The value that was rejected.
        value: usize,
    },
    /// The requested CPU alert threshold fell outside the supported range.
    CpuAlertThresholdOutOfRange {
        /// The value that was rejected, as a percentage.
        value: f32,
    },
    /// The requested memory alert threshold fell outside the supported
    /// range.
    MemoryAlertThresholdOutOfRange {
        /// The value that was rejected, as a percentage.
        value: f32,
    },
    /// The requested disk alert threshold fell outside the supported range.
    DiskAlertThresholdOutOfRange {
        /// The value that was rejected, as a percentage.
        value: f32,
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
            ConfigError::HistoryCapacityOutOfRange { value } => write!(
                f,
                "history capacity of {value} is out of range: must be between \
                 {MIN_HISTORY_CAPACITY} and {MAX_HISTORY_CAPACITY} inclusive"
            ),
            ConfigError::CpuAlertThresholdOutOfRange { value } => write!(
                f,
                "CPU alert threshold of {value} is out of range: must be a finite percentage \
                 between {MIN_ALERT_THRESHOLD_PERCENT} and {MAX_ALERT_THRESHOLD_PERCENT} \
                 inclusive"
            ),
            ConfigError::MemoryAlertThresholdOutOfRange { value } => write!(
                f,
                "memory alert threshold of {value} is out of range: must be a finite \
                 percentage between {MIN_ALERT_THRESHOLD_PERCENT} and \
                 {MAX_ALERT_THRESHOLD_PERCENT} inclusive"
            ),
            ConfigError::DiskAlertThresholdOutOfRange { value } => write!(
                f,
                "disk alert threshold of {value} is out of range: must be a finite percentage \
                 between {MIN_ALERT_THRESHOLD_PERCENT} and {MAX_ALERT_THRESHOLD_PERCENT} \
                 inclusive"
            ),
        }
    }
}

impl std::error::Error for ConfigError {}

/// Validated application configuration for Ferrite.
///
/// The default configuration ([`AppConfig::default`]) is always valid. A
/// custom configuration is built with [`AppConfig::builder`], which
/// validates every field together in [`AppConfigBuilder::build`] and
/// returns a descriptive [`ConfigError`] on the first invalid one, rather
/// than panicking.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AppConfig {
    refresh_interval_ms: u32,
    history_capacity: usize,
    cpu_alert_threshold_percent: f32,
    memory_alert_threshold_percent: f32,
    disk_alert_threshold_percent: f32,
    default_sort_mode: DefaultSortMode,
}

impl AppConfig {
    /// Starts building a custom configuration from Ferrite's defaults.
    pub fn builder() -> AppConfigBuilder {
        AppConfigBuilder::default()
    }

    /// Creates an `AppConfig` with every field defaulted except the
    /// refresh interval.
    ///
    /// Equivalent to
    /// `AppConfig::builder().refresh_interval_ms(value).build()`. The
    /// value must fall within
    /// [`MIN_REFRESH_INTERVAL_MS`]..=[`MAX_REFRESH_INTERVAL_MS`], inclusive;
    /// values outside that range are rejected with
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
        Self::builder()
            .refresh_interval_ms(refresh_interval_ms)
            .build()
    }

    /// The configured refresh interval, in milliseconds.
    pub fn refresh_interval_ms(&self) -> u32 {
        self.refresh_interval_ms
    }

    /// The configured maximum number of samples retained per metric
    /// history (see [`crate::history::History`]).
    pub fn history_capacity(&self) -> usize {
        self.history_capacity
    }

    /// The configured CPU usage percentage above which a CPU alert
    /// activates.
    pub fn cpu_alert_threshold_percent(&self) -> f32 {
        self.cpu_alert_threshold_percent
    }

    /// The configured memory usage percentage above which a memory alert
    /// activates.
    pub fn memory_alert_threshold_percent(&self) -> f32 {
        self.memory_alert_threshold_percent
    }

    /// The configured disk usage percentage above which a disk alert
    /// activates.
    pub fn disk_alert_threshold_percent(&self) -> f32 {
        self.disk_alert_threshold_percent
    }

    /// The process table's sort mode at startup.
    pub fn default_sort_mode(&self) -> DefaultSortMode {
        self.default_sort_mode
    }
}

impl Default for AppConfig {
    /// Returns the default configuration. The default is always valid by
    /// construction.
    fn default() -> Self {
        AppConfigBuilder::default()
            .build()
            .expect("AppConfigBuilder's own defaults are always within range")
    }
}

/// Builds an [`AppConfig`] from several possibly-untrusted values at once,
/// validating all of them together in [`AppConfigBuilder::build`].
///
/// Every setter is infallible and simply stages a value; nothing is
/// validated until `build()` is called, so several fields loaded from an
/// external source (such as a configuration file) can be applied in one
/// pass before validation runs.
#[derive(Debug, Clone)]
pub struct AppConfigBuilder {
    refresh_interval_ms: u32,
    history_capacity: usize,
    cpu_alert_threshold_percent: f32,
    memory_alert_threshold_percent: f32,
    disk_alert_threshold_percent: f32,
    default_sort_mode: DefaultSortMode,
}

impl Default for AppConfigBuilder {
    fn default() -> Self {
        Self {
            refresh_interval_ms: DEFAULT_REFRESH_INTERVAL_MS,
            history_capacity: DEFAULT_HISTORY_CAPACITY,
            cpu_alert_threshold_percent: DEFAULT_CPU_ALERT_THRESHOLD_PERCENT,
            memory_alert_threshold_percent: DEFAULT_MEMORY_ALERT_THRESHOLD_PERCENT,
            disk_alert_threshold_percent: DEFAULT_DISK_ALERT_THRESHOLD_PERCENT,
            default_sort_mode: DefaultSortMode::default(),
        }
    }
}

impl AppConfigBuilder {
    /// Sets the refresh interval, in milliseconds. Not validated until
    /// [`Self::build`].
    pub fn refresh_interval_ms(mut self, value: u32) -> Self {
        self.refresh_interval_ms = value;
        self
    }

    /// Sets the metric history capacity. Not validated until
    /// [`Self::build`].
    pub fn history_capacity(mut self, value: usize) -> Self {
        self.history_capacity = value;
        self
    }

    /// Sets the CPU alert threshold, as a percentage. Not validated until
    /// [`Self::build`].
    pub fn cpu_alert_threshold_percent(mut self, value: f32) -> Self {
        self.cpu_alert_threshold_percent = value;
        self
    }

    /// Sets the memory alert threshold, as a percentage. Not validated
    /// until [`Self::build`].
    pub fn memory_alert_threshold_percent(mut self, value: f32) -> Self {
        self.memory_alert_threshold_percent = value;
        self
    }

    /// Sets the disk alert threshold, as a percentage. Not validated
    /// until [`Self::build`].
    pub fn disk_alert_threshold_percent(mut self, value: f32) -> Self {
        self.disk_alert_threshold_percent = value;
        self
    }

    /// Sets the process table's default sort mode.
    pub fn default_sort_mode(mut self, value: DefaultSortMode) -> Self {
        self.default_sort_mode = value;
        self
    }

    /// Validates every staged field, returning the first violated
    /// constraint as a [`ConfigError`], or a valid [`AppConfig`].
    pub fn build(self) -> Result<AppConfig, ConfigError> {
        if !(MIN_REFRESH_INTERVAL_MS..=MAX_REFRESH_INTERVAL_MS).contains(&self.refresh_interval_ms)
        {
            return Err(ConfigError::RefreshIntervalOutOfRange {
                value: self.refresh_interval_ms,
            });
        }
        if !(MIN_HISTORY_CAPACITY..=MAX_HISTORY_CAPACITY).contains(&self.history_capacity) {
            return Err(ConfigError::HistoryCapacityOutOfRange {
                value: self.history_capacity,
            });
        }
        if !is_valid_threshold(self.cpu_alert_threshold_percent) {
            return Err(ConfigError::CpuAlertThresholdOutOfRange {
                value: self.cpu_alert_threshold_percent,
            });
        }
        if !is_valid_threshold(self.memory_alert_threshold_percent) {
            return Err(ConfigError::MemoryAlertThresholdOutOfRange {
                value: self.memory_alert_threshold_percent,
            });
        }
        if !is_valid_threshold(self.disk_alert_threshold_percent) {
            return Err(ConfigError::DiskAlertThresholdOutOfRange {
                value: self.disk_alert_threshold_percent,
            });
        }

        Ok(AppConfig {
            refresh_interval_ms: self.refresh_interval_ms,
            history_capacity: self.history_capacity,
            cpu_alert_threshold_percent: self.cpu_alert_threshold_percent,
            memory_alert_threshold_percent: self.memory_alert_threshold_percent,
            disk_alert_threshold_percent: self.disk_alert_threshold_percent,
            default_sort_mode: self.default_sort_mode,
        })
    }
}

fn is_valid_threshold(value: f32) -> bool {
    value.is_finite()
        && (MIN_ALERT_THRESHOLD_PERCENT..=MAX_ALERT_THRESHOLD_PERCENT).contains(&value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_uses_documented_defaults() {
        let config = AppConfig::default();
        assert_eq!(config.refresh_interval_ms(), DEFAULT_REFRESH_INTERVAL_MS);
        assert_eq!(config.refresh_interval_ms(), 1_000);
        assert_eq!(config.history_capacity(), DEFAULT_HISTORY_CAPACITY);
        assert_eq!(
            config.cpu_alert_threshold_percent(),
            DEFAULT_CPU_ALERT_THRESHOLD_PERCENT
        );
        assert_eq!(
            config.memory_alert_threshold_percent(),
            DEFAULT_MEMORY_ALERT_THRESHOLD_PERCENT
        );
        assert_eq!(
            config.disk_alert_threshold_percent(),
            DEFAULT_DISK_ALERT_THRESHOLD_PERCENT
        );
        assert_eq!(config.default_sort_mode(), DefaultSortMode::Cpu);
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

    #[test]
    fn builder_can_set_every_field_at_once() {
        let config = AppConfig::builder()
            .refresh_interval_ms(2_000)
            .history_capacity(60)
            .cpu_alert_threshold_percent(80.0)
            .memory_alert_threshold_percent(75.0)
            .disk_alert_threshold_percent(95.0)
            .default_sort_mode(DefaultSortMode::Memory)
            .build()
            .unwrap();

        assert_eq!(config.refresh_interval_ms(), 2_000);
        assert_eq!(config.history_capacity(), 60);
        assert_eq!(config.cpu_alert_threshold_percent(), 80.0);
        assert_eq!(config.memory_alert_threshold_percent(), 75.0);
        assert_eq!(config.disk_alert_threshold_percent(), 95.0);
        assert_eq!(config.default_sort_mode(), DefaultSortMode::Memory);
    }

    #[test]
    fn builder_rejects_out_of_range_history_capacity() {
        let result = AppConfig::builder().history_capacity(0).build();
        assert_eq!(
            result,
            Err(ConfigError::HistoryCapacityOutOfRange { value: 0 })
        );

        let result = AppConfig::builder()
            .history_capacity(MAX_HISTORY_CAPACITY + 1)
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn builder_rejects_out_of_range_alert_thresholds() {
        assert_eq!(
            AppConfig::builder()
                .cpu_alert_threshold_percent(-1.0)
                .build(),
            Err(ConfigError::CpuAlertThresholdOutOfRange { value: -1.0 })
        );
        assert_eq!(
            AppConfig::builder()
                .memory_alert_threshold_percent(100.1)
                .build(),
            Err(ConfigError::MemoryAlertThresholdOutOfRange { value: 100.1 })
        );
        // NaN never equals itself, so this checks the error variant by
        // pattern rather than by equality against the payload.
        let result = AppConfig::builder()
            .disk_alert_threshold_percent(f32::NAN)
            .build();
        assert!(matches!(
            result,
            Err(ConfigError::DiskAlertThresholdOutOfRange { value }) if value.is_nan()
        ));
    }

    #[test]
    fn builder_accepts_boundary_alert_thresholds() {
        let config = AppConfig::builder()
            .cpu_alert_threshold_percent(0.0)
            .memory_alert_threshold_percent(100.0)
            .build()
            .unwrap();
        assert_eq!(config.cpu_alert_threshold_percent(), 0.0);
        assert_eq!(config.memory_alert_threshold_percent(), 100.0);
    }

    #[test]
    fn builder_validates_the_first_invalid_field_it_checks() {
        // Refresh interval is checked before history capacity, so an
        // invalid refresh interval is reported even if other fields are
        // also invalid.
        let result = AppConfig::builder()
            .refresh_interval_ms(0)
            .history_capacity(0)
            .build();
        assert_eq!(
            result,
            Err(ConfigError::RefreshIntervalOutOfRange { value: 0 })
        );
    }
}
