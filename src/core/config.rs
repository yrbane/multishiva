use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Current configuration version for migration compatibility.
///
/// This constant is used to track the configuration schema version and enable
/// automatic migration when loading older configuration files. When the configuration
/// format changes in a backwards-incompatible way, this version should be incremented.
pub const CONFIG_VERSION: u32 = 1;

/// Main configuration structure for the multishiva application.
///
/// This structure holds all configuration settings for both host and agent modes,
/// including network settings, TLS configuration, edge mappings, hotkeys, and
/// behavioral settings. Configurations can be loaded from YAML files and are
/// versioned for migration support.
///
/// # Examples
///
/// ```
/// use multishiva::core::config::{Config, ConfigMode};
///
/// // Create a default configuration
/// let config = Config::default();
/// assert_eq!(config.mode, ConfigMode::Host);
///
/// // Validate the configuration
/// # config.validate().is_err(); // Default config has empty PSK
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Configuration schema version for migration support.
    #[serde(default = "default_version")]
    pub version: u32,

    /// Unique identifier name for this instance.
    pub self_name: String,

    /// Operating mode: either Host or Agent.
    pub mode: ConfigMode,

    /// Network port for listening (host mode) or connecting (agent mode).
    pub port: u16,

    /// Host address to connect to (required in agent mode, unused in host mode).
    pub host_address: Option<String>,

    /// TLS/encryption configuration including pre-shared key.
    pub tls: TlsConfig,

    /// Map of edge names to connected agent names for defining screen edges.
    pub edges: HashMap<String, String>,

    /// Optional hotkey configuration for focus return and kill switch.
    pub hotkeys: Option<Hotkeys>,

    /// Optional behavioral settings like edge thresholds and timing parameters.
    pub behavior: Option<Behavior>,
}

fn default_version() -> u32 {
    CONFIG_VERSION
}

/// Operating mode for a multishiva instance.
///
/// Determines whether this instance acts as a host (server) or agent (client).
/// The mode affects validation requirements and network behavior.
///
/// # Examples
///
/// ```
/// use multishiva::core::config::ConfigMode;
///
/// let mode = ConfigMode::Host;
/// assert_eq!(mode, ConfigMode::Host);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ConfigMode {
    /// Host mode: acts as a server, listening for agent connections.
    Host,

    /// Agent mode: acts as a client, connecting to a host.
    /// Requires `host_address` to be configured.
    Agent,
}

/// TLS/encryption configuration.
///
/// Contains security-related settings for encrypted communication between
/// host and agents. The pre-shared key must be identical across all instances
/// that need to communicate.
///
/// # Examples
///
/// ```
/// use multishiva::core::config::TlsConfig;
///
/// let tls = TlsConfig {
///     psk: "my-secret-key".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Pre-shared key (PSK) for encrypted communication.
    /// Must be non-empty and identical across all communicating instances.
    pub psk: String,
}

/// Hotkey configuration for keyboard shortcuts.
///
/// Defines optional keyboard shortcuts for quick actions like returning focus
/// or activating a kill switch. Hotkey strings should follow the platform's
/// hotkey format.
///
/// # Examples
///
/// ```
/// use multishiva::core::config::Hotkeys;
///
/// let hotkeys = Hotkeys {
///     focus_return: Some("Ctrl+Alt+Home".to_string()),
///     kill_switch: Some("Ctrl+Shift+Esc".to_string()),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hotkeys {
    /// Hotkey to return focus to the primary screen.
    pub focus_return: Option<String>,

    /// Emergency hotkey to disable cursor sharing.
    pub kill_switch: Option<String>,
}

/// Behavioral settings for cursor movement and connection handling.
///
/// Configures fine-grained timing and threshold parameters that control
/// how the cursor behaves at screen edges and how the system handles
/// reconnection attempts.
///
/// # Examples
///
/// ```
/// use multishiva::core::config::Behavior;
///
/// let behavior = Behavior {
///     edge_threshold_px: Some(5),
///     friction_ms: Some(100),
///     reconnect_delay_ms: Some(5000),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Behavior {
    /// Distance in pixels from the screen edge to trigger transition.
    pub edge_threshold_px: Option<u32>,

    /// Delay in milliseconds before cursor crosses to another screen.
    /// Helps prevent accidental transitions.
    pub friction_ms: Option<u64>,

    /// Delay in milliseconds between reconnection attempts.
    pub reconnect_delay_ms: Option<u64>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: CONFIG_VERSION,
            self_name: "multishiva".to_string(),
            mode: ConfigMode::Host,
            port: 53421,
            host_address: None,
            tls: TlsConfig { psk: String::new() },
            edges: HashMap::new(),
            hotkeys: None,
            behavior: None,
        }
    }
}

impl Config {
    /// Loads configuration from a YAML file with automatic migration.
    ///
    /// Reads and parses a configuration file from the specified path. If the
    /// configuration version is older than the current version, it will be
    /// automatically migrated to the latest schema.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the configuration file
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be read
    /// - The file content is not valid YAML
    /// - The YAML structure doesn't match the Config schema
    /// - Migration fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use multishiva::core::config::Config;
    ///
    /// let config = Config::from_file("config.yml")?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path))?;
        let mut config: Config = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path))?;

        // Migrate if needed
        if config.version < CONFIG_VERSION {
            config = Self::migrate(config)?;
        }

        Ok(config)
    }

    /// Saves configuration to a YAML file with automatic backup.
    ///
    /// Serializes the configuration to YAML format and writes it to the specified
    /// path. If the file already exists, it will be backed up with a `.backup`
    /// extension before being overwritten. Parent directories are created automatically
    /// if they don't exist.
    ///
    /// # Arguments
    ///
    /// * `path` - Destination path for the configuration file
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Parent directory creation fails
    /// - Backup operation fails
    /// - Serialization to YAML fails
    /// - File write operation fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use multishiva::core::config::Config;
    /// use std::path::Path;
    ///
    /// let config = Config::default();
    /// config.save_to_file(Path::new("config.yml"))?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {:?}", parent))?;
        }

        // Backup existing file if it exists
        if path.exists() {
            Self::backup_config(path)?;
        }

        // Serialize config
        let content = serde_yaml::to_string(self).context("Failed to serialize config")?;

        // Write to file
        std::fs::write(path, content)
            .with_context(|| format!("Failed to write config file: {:?}", path))?;

        tracing::info!("Configuration saved to: {:?}", path);
        Ok(())
    }

    /// Returns the default configuration file path.
    ///
    /// Attempts to use the system's standard configuration directory
    /// (e.g., `~/.config/multishiva/config.yml` on Linux). Falls back
    /// to `multishiva.yml` in the current directory if the system
    /// config directory cannot be determined.
    ///
    /// # Examples
    ///
    /// ```
    /// use multishiva::core::config::Config;
    ///
    /// let default_path = Config::default_path();
    /// println!("Default config location: {:?}", default_path);
    /// ```
    pub fn default_path() -> PathBuf {
        if let Some(config_dir) = dirs::config_dir() {
            config_dir.join("multishiva").join("config.yml")
        } else {
            PathBuf::from("multishiva.yml")
        }
    }

    /// Loads configuration from a file or returns default if file doesn't exist.
    ///
    /// Attempts to load configuration from the specified path, or from the
    /// default path if no path is provided. If the configuration file doesn't
    /// exist, returns a default configuration instead of erroring.
    ///
    /// # Arguments
    ///
    /// * `path` - Optional path to configuration file. Uses default path if `None`.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file exists but cannot be read
    /// - The file exists but contains invalid YAML
    /// - The file exists but doesn't match the Config schema
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use multishiva::core::config::Config;
    /// use std::path::Path;
    ///
    /// // Load from default location or use defaults
    /// let config = Config::load_or_default(None)?;
    ///
    /// // Load from specific path or use defaults
    /// let config = Config::load_or_default(Some(Path::new("my-config.yml")))?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn load_or_default(path: Option<&Path>) -> Result<Self> {
        let config_path = path
            .map(|p| p.to_path_buf())
            .unwrap_or_else(Self::default_path);

        if config_path.exists() {
            tracing::info!("Loading config from: {:?}", config_path);
            Self::from_file(config_path.to_str().unwrap_or("config.yml"))
        } else {
            tracing::warn!("Config file not found, using defaults: {:?}", config_path);
            Ok(Self::default())
        }
    }

    /// Validates the configuration for correctness and completeness.
    ///
    /// Checks that all required fields are properly set and that mode-specific
    /// requirements are met. Should be called after loading or modifying a
    /// configuration before using it.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - `self_name` is empty
    /// - `tls.psk` is empty
    /// - `port` is 0
    /// - In agent mode: `host_address` is None
    ///
    /// # Examples
    ///
    /// ```
    /// use multishiva::core::config::{Config, TlsConfig, ConfigMode};
    ///
    /// let mut config = Config::default();
    /// config.self_name = "my-machine".to_string();
    /// config.tls.psk = "secure-key".to_string();
    ///
    /// assert!(config.validate().is_ok());
    /// ```
    pub fn validate(&self) -> Result<()> {
        if self.self_name.is_empty() {
            anyhow::bail!("self_name cannot be empty");
        }
        if self.tls.psk.is_empty() {
            anyhow::bail!("TLS PSK cannot be empty");
        }
        if self.port == 0 {
            anyhow::bail!("port cannot be 0");
        }

        // Validate mode-specific requirements
        match self.mode {
            ConfigMode::Agent => {
                // host_address is now optional - if not provided, mDNS auto-discovery will be used
            }
            ConfigMode::Host => {
                // Host mode doesn't require additional validation
            }
        }

        Ok(())
    }

    /// Backup config file before overwriting
    fn backup_config(path: &Path) -> Result<()> {
        let backup_path = path.with_extension("yml.backup");
        std::fs::copy(path, &backup_path).with_context(|| {
            format!(
                "Failed to backup config from {:?} to {:?}",
                path, backup_path
            )
        })?;
        tracing::info!("Backed up config to: {:?}", backup_path);
        Ok(())
    }

    /// Migrate config from older version
    fn migrate(mut config: Config) -> Result<Self> {
        tracing::info!(
            "Migrating config from version {} to {}",
            config.version,
            CONFIG_VERSION
        );

        // For now, just update the version
        // In the future, add migration logic here
        config.version = CONFIG_VERSION;

        Ok(config)
    }

    /// Checks if a configuration file is valid and can be parsed.
    ///
    /// Attempts to read and parse the configuration file without loading it
    /// into memory. Useful for validating configuration files before use.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the configuration file to validate
    ///
    /// # Returns
    ///
    /// - `Ok(true)` if the file exists and is valid
    /// - `Ok(false)` if the file doesn't exist or is invalid
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read (permissions, I/O errors, etc.)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use multishiva::core::config::Config;
    /// use std::path::Path;
    ///
    /// let path = Path::new("config.yml");
    /// if Config::validate_file(path)? {
    ///     println!("Configuration file is valid");
    /// } else {
    ///     println!("Configuration file is missing or invalid");
    /// }
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn validate_file(path: &Path) -> Result<bool> {
        if !path.exists() {
            return Ok(false);
        }

        let content = std::fs::read_to_string(path)?;
        match serde_yaml::from_str::<Config>(&content) {
            Ok(_) => Ok(true),
            Err(e) => {
                tracing::error!("Config file validation failed: {}", e);
                Ok(false)
            }
        }
    }

    /// Saves configuration to the default location.
    ///
    /// Convenience method that saves the configuration to the default path
    /// determined by [`Config::default_path()`]. Useful for auto-save functionality
    /// or when the configuration path doesn't need to be specified.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Parent directory creation fails
    /// - Backup operation fails (if file exists)
    /// - Serialization to YAML fails
    /// - File write operation fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use multishiva::core::config::Config;
    ///
    /// let mut config = Config::default();
    /// config.self_name = "updated-name".to_string();
    /// config.auto_save()?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn auto_save(&self) -> Result<()> {
        let path = Self::default_path();
        self.save_to_file(&path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_config_structure() {
        let config = Config::default();
        assert_eq!(config.version, CONFIG_VERSION);
        assert_eq!(config.self_name, "multishiva");
        assert_eq!(config.mode, ConfigMode::Host);
        assert_eq!(config.port, 53421);
        assert!(config.host_address.is_none());
    }

    #[test]
    fn test_config_default_version() {
        let config = Config::default();
        assert_eq!(config.version, 1);
    }

    #[test]
    fn test_config_validate_empty_name() {
        let config = Config {
            self_name: String::new(),
            tls: TlsConfig {
                psk: "test-psk".to_string(),
            },
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validate_empty_psk() {
        let config = Config {
            self_name: "test".to_string(),
            tls: TlsConfig { psk: String::new() },
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validate_zero_port() {
        let config = Config {
            self_name: "test".to_string(),
            tls: TlsConfig {
                psk: "test-psk".to_string(),
            },
            port: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validate_agent_without_host() {
        // Agent mode without host_address should now be valid (mDNS auto-discovery)
        let config = Config {
            self_name: "test".to_string(),
            tls: TlsConfig {
                psk: "test-psk".to_string(),
            },
            mode: ConfigMode::Agent,
            host_address: None,
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validate_valid() {
        let config = Config {
            self_name: "test".to_string(),
            tls: TlsConfig {
                psk: "test-psk".to_string(),
            },
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test-config.yml");

        let config = Config {
            self_name: "test-machine".to_string(),
            tls: TlsConfig {
                psk: "test-psk-12345".to_string(),
            },
            port: 12345,
            ..Default::default()
        };

        // Save
        config.save_to_file(&config_path).unwrap();
        assert!(config_path.exists());

        // Load
        let loaded = Config::from_file(config_path.to_str().unwrap()).unwrap();
        assert_eq!(loaded.self_name, "test-machine");
        assert_eq!(loaded.tls.psk, "test-psk-12345");
        assert_eq!(loaded.port, 12345);
    }

    #[test]
    fn test_config_backup_on_save() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test-config.yml");

        let mut config = Config {
            self_name: "first".to_string(),
            tls: TlsConfig {
                psk: "psk1".to_string(),
            },
            ..Default::default()
        };

        // First save
        config.save_to_file(&config_path).unwrap();

        // Second save should create backup
        config.self_name = "second".to_string();
        config.save_to_file(&config_path).unwrap();

        // Check backup exists
        let backup_path = config_path.with_extension("yml.backup");
        assert!(backup_path.exists());

        // Check backup contains old data
        let backup = Config::from_file(backup_path.to_str().unwrap()).unwrap();
        assert_eq!(backup.self_name, "first");
    }

    #[test]
    fn test_config_validate_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("valid.yml");

        // Valid file
        let config = Config::default();
        config.save_to_file(&config_path).unwrap();
        assert!(Config::validate_file(&config_path).unwrap());

        // Invalid file
        let invalid_path = temp_dir.path().join("invalid.yml");
        std::fs::write(&invalid_path, "invalid: yaml: content: [").unwrap();
        assert!(!Config::validate_file(&invalid_path).unwrap());

        // Non-existent file
        let missing_path = temp_dir.path().join("missing.yml");
        assert!(!Config::validate_file(&missing_path).unwrap());
    }

    #[test]
    fn test_config_migration() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("old-config.yml");

        // Create old config (version 0)
        let old_content = r#"
version: 0
self_name: "old-machine"
mode: host
port: 53421
tls:
  psk: "old-psk"
edges: {}
"#;
        std::fs::write(&config_path, old_content).unwrap();

        // Load should auto-migrate
        let config = Config::from_file(config_path.to_str().unwrap()).unwrap();
        assert_eq!(config.version, CONFIG_VERSION);
        assert_eq!(config.self_name, "old-machine");
    }

    #[test]
    fn test_config_load_or_default() {
        let temp_dir = TempDir::new().unwrap();

        // Non-existent file should return default
        let missing_path = temp_dir.path().join("missing.yml");
        let config = Config::load_or_default(Some(&missing_path)).unwrap();
        assert_eq!(config.self_name, "multishiva");

        // Existing file should load
        let config_path = temp_dir.path().join("exists.yml");
        let saved_config = Config {
            self_name: "loaded".to_string(),
            tls: TlsConfig {
                psk: "loaded-psk".to_string(),
            },
            ..Default::default()
        };
        saved_config.save_to_file(&config_path).unwrap();

        let loaded = Config::load_or_default(Some(&config_path)).unwrap();
        assert_eq!(loaded.self_name, "loaded");
    }
}
