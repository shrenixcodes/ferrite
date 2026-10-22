//! Loading [`AppConfig`](super::AppConfig) from an optional TOML
//! configuration file.
//!
//! Ferrite never writes a configuration file itself — it only reads one if
//! present — so an existing user file is never overwritten or clobbered.
//! See [`default_config_path`] for the search location and precedence.

use std::path::{Path, PathBuf};
use std::{fmt, fs, io};

use serde::Deserialize;

use super::{AppConfig, AppConfigBuilder, ConfigError, DefaultSortMode};

/// A process sort mode as written in a configuration file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
enum RawSortMode {
    Cpu,
    Memory,
}

impl From<RawSortMode> for DefaultSortMode {
    fn from(value: RawSortMode) -> Self {
        match value {
            RawSortMode::Cpu => DefaultSortMode::Cpu,
            RawSortMode::Memory => DefaultSortMode::Memory,
        }
    }
}

/// The on-disk shape of Ferrite's configuration file.
///
/// Every field is optional; an absent field keeps [`AppConfig`]'s default
/// for that setting. Unknown fields are rejected (`deny_unknown_fields`)
/// so a typo in the file produces a specific, loud error instead of being
/// silently ignored.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawConfig {
    refresh_interval_ms: Option<u32>,
    history_capacity: Option<usize>,
    cpu_alert_threshold_percent: Option<f32>,
    memory_alert_threshold_percent: Option<f32>,
    disk_alert_threshold_percent: Option<f32>,
    default_sort_mode: Option<RawSortMode>,
}

impl RawConfig {
    fn into_builder(self) -> AppConfigBuilder {
        let mut builder = AppConfigBuilder::default();
        if let Some(value) = self.refresh_interval_ms {
            builder = builder.refresh_interval_ms(value);
        }
        if let Some(value) = self.history_capacity {
            builder = builder.history_capacity(value);
        }
        if let Some(value) = self.cpu_alert_threshold_percent {
            builder = builder.cpu_alert_threshold_percent(value);
        }
        if let Some(value) = self.memory_alert_threshold_percent {
            builder = builder.memory_alert_threshold_percent(value);
        }
        if let Some(value) = self.disk_alert_threshold_percent {
            builder = builder.disk_alert_threshold_percent(value);
        }
        if let Some(value) = self.default_sort_mode {
            builder = builder.default_sort_mode(value.into());
        }
        builder
    }
}

/// An error that can occur while loading a configuration file.
#[derive(Debug)]
pub enum ConfigFileError {
    /// The file could not be read (for example, permission denied). A
    /// simply-missing file is not represented here — see
    /// [`load_or_default`], which treats that as "use defaults."
    Io(io::Error),
    /// The file's contents were not valid TOML.
    Parse(toml::de::Error),
    /// The file parsed, but one of its values failed [`AppConfig`]
    /// validation.
    Invalid(ConfigError),
}

impl fmt::Display for ConfigFileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigFileError::Io(err) => write!(f, "could not read configuration file: {err}"),
            ConfigFileError::Parse(err) => {
                write!(f, "configuration file is not valid TOML: {err}")
            }
            ConfigFileError::Invalid(err) => {
                write!(f, "configuration file has an invalid value: {err}")
            }
        }
    }
}

impl std::error::Error for ConfigFileError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConfigFileError::Io(err) => Some(err),
            ConfigFileError::Parse(err) => Some(err),
            ConfigFileError::Invalid(err) => Some(err),
        }
    }
}

impl From<ConfigError> for ConfigFileError {
    fn from(err: ConfigError) -> Self {
        ConfigFileError::Invalid(err)
    }
}

/// Parses `contents` as a configuration file and validates it into an
/// [`AppConfig`].
///
/// This is the pure parsing/validation step, decoupled from any
/// filesystem or path-resolution concern, so it is directly testable with
/// in-memory strings rather than real files.
pub fn parse(contents: &str) -> Result<AppConfig, ConfigFileError> {
    let raw: RawConfig = toml::from_str(contents).map_err(ConfigFileError::Parse)?;
    Ok(raw.into_builder().build()?)
}

/// Reads and validates the configuration file at `path`.
pub fn load_from_path(path: &Path) -> Result<AppConfig, ConfigFileError> {
    let contents = fs::read_to_string(path).map_err(ConfigFileError::Io)?;
    parse(&contents)
}

/// Loads the configuration file at [`default_config_path`], if one exists;
/// otherwise returns [`AppConfig::default`].
///
/// A missing file is not an error: Ferrite works with sensible defaults
/// out of the box, and never creates or writes this file itself, so an
/// existing file is never at risk of being overwritten. A file that
/// exists but is malformed or has an invalid value *is* still an error —
/// silently falling back to defaults there would hide a real mistake from
/// the user.
pub fn load_or_default() -> Result<AppConfig, ConfigFileError> {
    let Some(path) = default_config_path() else {
        return Ok(AppConfig::default());
    };

    match load_from_path(&path) {
        Ok(config) => Ok(config),
        Err(ConfigFileError::Io(err)) if err.kind() == io::ErrorKind::NotFound => {
            Ok(AppConfig::default())
        }
        Err(err) => Err(err),
    }
}

/// The platform-specific path Ferrite looks for its configuration file
/// at:
///
/// - **Windows:** `%APPDATA%\ferrite\config.toml`
/// - **macOS:** `$HOME/Library/Application Support/ferrite/config.toml`
/// - **Linux/other Unix:** `$XDG_CONFIG_HOME/ferrite/config.toml`, falling
///   back to `$HOME/.config/ferrite/config.toml` if `XDG_CONFIG_HOME` is
///   unset.
///
/// There is exactly one search location (no precedence chain across
/// multiple candidate paths); returns `None` if the relevant environment
/// variable is not set, in which case [`load_or_default`] falls back to
/// [`AppConfig::default`].
pub fn default_config_path() -> Option<PathBuf> {
    config_dir(|key| std::env::var(key).ok()).map(|dir| dir.join("ferrite").join("config.toml"))
}

/// The platform config-directory lookup, with environment-variable access
/// injected so it can be tested deterministically without touching the
/// real environment.
fn config_dir(getenv: impl Fn(&str) -> Option<String>) -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        getenv("APPDATA").map(PathBuf::from)
    }
    #[cfg(target_os = "macos")]
    {
        getenv("HOME").map(|home| PathBuf::from(home).join("Library/Application Support"))
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        getenv("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| getenv("HOME").map(|home| PathBuf::from(home).join(".config")))
    }
    #[cfg(not(any(target_os = "windows", unix)))]
    {
        let _ = getenv;
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_an_empty_file_into_defaults() {
        let config = parse("").unwrap();
        assert_eq!(config, AppConfig::default());
    }

    #[test]
    fn parses_every_supported_field() {
        let toml = r#"
            refresh_interval_ms = 2000
            history_capacity = 60
            cpu_alert_threshold_percent = 85.0
            memory_alert_threshold_percent = 80.0
            disk_alert_threshold_percent = 95.0
            default_sort_mode = "memory"
        "#;

        let config = parse(toml).unwrap();

        assert_eq!(config.refresh_interval_ms(), 2_000);
        assert_eq!(config.history_capacity(), 60);
        assert_eq!(config.cpu_alert_threshold_percent(), 85.0);
        assert_eq!(config.memory_alert_threshold_percent(), 80.0);
        assert_eq!(config.disk_alert_threshold_percent(), 95.0);
        assert_eq!(config.default_sort_mode(), DefaultSortMode::Memory);
    }

    #[test]
    fn partial_file_keeps_defaults_for_missing_fields() {
        let config = parse("refresh_interval_ms = 500").unwrap();
        assert_eq!(config.refresh_interval_ms(), 500);
        assert_eq!(
            config.history_capacity(),
            super::super::DEFAULT_HISTORY_CAPACITY
        );
    }

    #[test]
    fn malformed_toml_produces_a_parse_error() {
        let result = parse("this is not valid toml = = =");
        assert!(matches!(result, Err(ConfigFileError::Parse(_))));
    }

    #[test]
    fn unknown_field_is_rejected() {
        let result = parse("not_a_real_setting = 1");
        assert!(matches!(result, Err(ConfigFileError::Parse(_))));
    }

    #[test]
    fn invalid_value_produces_a_validation_error() {
        let result = parse("refresh_interval_ms = 1");
        assert!(matches!(
            result,
            Err(ConfigFileError::Invalid(
                ConfigError::RefreshIntervalOutOfRange { value: 1 }
            ))
        ));
    }

    #[test]
    fn invalid_sort_mode_string_is_a_parse_error() {
        let result = parse(r#"default_sort_mode = "fastest""#);
        assert!(matches!(result, Err(ConfigFileError::Parse(_))));
    }

    #[test]
    fn load_from_path_reports_missing_file_as_io_error() {
        let result = load_from_path(Path::new("/this/path/definitely/does/not/exist.toml"));
        assert!(matches!(result, Err(ConfigFileError::Io(_))));
    }

    #[test]
    fn the_shipped_example_config_file_parses_and_validates() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("config.example.toml");
        let contents = fs::read_to_string(&path).expect("config.example.toml must exist");

        let config = parse(&contents).expect("the example config must be valid");

        assert_eq!(config.refresh_interval_ms(), 1_000);
        assert_eq!(config.history_capacity(), 120);
        assert_eq!(config.cpu_alert_threshold_percent(), 90.0);
        assert_eq!(config.memory_alert_threshold_percent(), 90.0);
        assert_eq!(config.disk_alert_threshold_percent(), 90.0);
        assert_eq!(config.default_sort_mode(), DefaultSortMode::Cpu);
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn windows_config_dir_uses_appdata() {
        let dir = config_dir(|key| {
            (key == "APPDATA").then(|| "C:\\Users\\test\\AppData\\Roaming".to_string())
        });
        assert_eq!(
            dir,
            Some(PathBuf::from("C:\\Users\\test\\AppData\\Roaming"))
        );
    }

    #[test]
    #[cfg(all(unix, not(target_os = "macos")))]
    fn linux_config_dir_prefers_xdg_config_home() {
        let dir = config_dir(|key| match key {
            "XDG_CONFIG_HOME" => Some("/custom/config".to_string()),
            "HOME" => Some("/home/test".to_string()),
            _ => None,
        });
        assert_eq!(dir, Some(PathBuf::from("/custom/config")));
    }

    #[test]
    #[cfg(all(unix, not(target_os = "macos")))]
    fn linux_config_dir_falls_back_to_home_dot_config_without_xdg() {
        let dir = config_dir(|key| (key == "HOME").then(|| "/home/test".to_string()));
        assert_eq!(dir, Some(PathBuf::from("/home/test/.config")));
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn macos_config_dir_uses_home_library() {
        let dir = config_dir(|key| (key == "HOME").then(|| "/Users/test".to_string()));
        assert_eq!(
            dir,
            Some(PathBuf::from("/Users/test/Library/Application Support"))
        );
    }

    #[test]
    fn config_dir_is_none_when_relevant_env_vars_are_unset() {
        let dir = config_dir(|_| None);
        assert_eq!(dir, None);
    }
}
