use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Current config version for migration
pub const CONFIG_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_version")]
    pub version: u32,
    pub self_name: String,
    pub mode: ConfigMode,
    pub port: u16,
    pub host_address: Option<String>, // For agent mode: address of host to connect to
    pub tls: TlsConfig,
    pub edges: HashMap<String, String>,
    pub hotkeys: Option<Hotkeys>,
    pub behavior: Option<Behavior>,
}

fn default_version() -> u32 {
    CONFIG_VERSION
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ConfigMode {
    Host,
    Agent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    pub psk: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hotkeys {
    pub focus_return: Option<String>,
    pub kill_switch: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Behavior {
    pub edge_threshold_px: Option<u32>,
    pub friction_ms: Option<u64>,
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
    /// Load config from a file
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

    /// Save config to a file with backup
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

    /// Get default config path
    pub fn default_path() -> PathBuf {
        if let Some(config_dir) = dirs::config_dir() {
            config_dir.join("multishiva").join("config.yml")
        } else {
            PathBuf::from("multishiva.yml")
        }
    }

    /// Load config with automatic fallback
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

    /// Validate configuration
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
                if self.host_address.is_none() {
                    anyhow::bail!("host_address is required for agent mode");
                }
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

    /// Check if config file is corrupted
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

    /// Auto-save configuration periodically
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
        let config = Config {
            self_name: "test".to_string(),
            tls: TlsConfig {
                psk: "test-psk".to_string(),
            },
            mode: ConfigMode::Agent,
            host_address: None,
            ..Default::default()
        };
        assert!(config.validate().is_err());
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
