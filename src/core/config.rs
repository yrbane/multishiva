use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub self_name: String,
    pub mode: ConfigMode,
    pub port: u16,
    pub tls: TlsConfig,
    pub edges: HashMap<String, String>,
    pub hotkeys: Option<Hotkeys>,
    pub behavior: Option<Behavior>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
            self_name: "multishiva".to_string(),
            mode: ConfigMode::Host,
            port: 53421,
            tls: TlsConfig { psk: String::new() },
            edges: HashMap::new(),
            hotkeys: None,
            behavior: None,
        }
    }
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<()> {
        if self.self_name.is_empty() {
            anyhow::bail!("self_name cannot be empty");
        }
        if self.tls.psk.is_empty() {
            anyhow::bail!("TLS PSK cannot be empty");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_config_structure() {
        // Basic smoke test - empty test for now
    }
}
