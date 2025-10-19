use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// TLS certificate fingerprint for MITM detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fingerprint {
    machine_name: String,
    hash: String,
    #[serde(default)]
    first_seen: Option<String>,
    #[serde(default)]
    last_verified: Option<String>,
}

impl PartialEq for Fingerprint {
    fn eq(&self, other: &Self) -> bool {
        self.machine_name == other.machine_name && self.hash == other.hash
    }
}

impl Eq for Fingerprint {}

impl Fingerprint {
    /// Create a new fingerprint with a known hash
    pub fn new(machine_name: impl Into<String>, hash: impl Into<String>) -> Self {
        Self {
            machine_name: machine_name.into(),
            hash: hash.into(),
            first_seen: Some(chrono::Utc::now().to_rfc3339()),
            last_verified: Some(chrono::Utc::now().to_rfc3339()),
        }
    }

    /// Create a fingerprint from certificate data
    pub fn from_cert_data(machine_name: impl Into<String>, cert_data: &[u8]) -> Self {
        let hash = Self::calculate_hash(cert_data);
        Self::new(machine_name, hash)
    }

    /// Calculate SHA-256 hash of certificate data
    fn calculate_hash(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        hex::encode(result)
    }

    /// Get machine name
    pub fn machine_name(&self) -> &str {
        &self.machine_name
    }

    /// Get hash
    pub fn hash(&self) -> &str {
        &self.hash
    }

    /// Verify if a hash matches this fingerprint
    pub fn verify(&self, cert_hash: &str) -> bool {
        self.hash == cert_hash
    }

    /// Update last verified timestamp
    pub fn touch(&mut self) {
        self.last_verified = Some(chrono::Utc::now().to_rfc3339());
    }
}

/// Storage for TLS fingerprints
#[derive(Debug)]
pub struct FingerprintStore {
    path: PathBuf,
    fingerprints: HashMap<String, Fingerprint>,
}

impl FingerprintStore {
    /// Create a new fingerprint store at the given path
    pub fn new(path: PathBuf) -> Result<Self> {
        let fingerprints = if path.exists() {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("Failed to read fingerprints from {:?}", path))?;
            serde_json::from_str(&content)
                .with_context(|| format!("Failed to parse fingerprints from {:?}", path))?
        } else {
            // Create parent directory if it doesn't exist
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create directory {:?}", parent))?;
            }
            HashMap::new()
        };

        Ok(Self { path, fingerprints })
    }

    /// Get the default store path (~/.config/multishiva/fingerprints.json)
    pub fn default_path() -> PathBuf {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("multishiva");
        config_dir.join("fingerprints.json")
    }

    /// Load store from default path
    pub fn load_default() -> Result<Self> {
        Self::new(Self::default_path())
    }

    /// Save a fingerprint for a machine
    pub fn save(
        &mut self,
        machine_name: impl Into<String>,
        fingerprint: Fingerprint,
    ) -> Result<()> {
        let machine_name = machine_name.into();
        self.fingerprints.insert(machine_name, fingerprint);
        self.persist()
    }

    /// Get a fingerprint for a machine
    pub fn get(&self, machine_name: &str) -> Option<&Fingerprint> {
        self.fingerprints.get(machine_name)
    }

    /// Remove a fingerprint
    pub fn remove(&mut self, machine_name: &str) -> Result<()> {
        self.fingerprints.remove(machine_name);
        self.persist()
    }

    /// List all fingerprints
    pub fn list_all(&self) -> Vec<&Fingerprint> {
        self.fingerprints.values().collect()
    }

    /// Persist fingerprints to disk
    fn persist(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.fingerprints)
            .context("Failed to serialize fingerprints")?;
        fs::write(&self.path, json)
            .with_context(|| format!("Failed to write fingerprints to {:?}", self.path))?;
        Ok(())
    }

    /// Verify a certificate hash against stored fingerprint
    /// Returns Ok(true) if verified, Ok(false) if first connection, Err if mismatch
    pub fn verify_or_save(
        &mut self,
        machine_name: &str,
        cert_hash: &str,
    ) -> Result<FingerprintVerification> {
        match self.get(machine_name) {
            Some(stored_fp) => {
                if stored_fp.verify(cert_hash) {
                    Ok(FingerprintVerification::Verified)
                } else {
                    Ok(FingerprintVerification::Mismatch {
                        stored: stored_fp.hash().to_string(),
                        received: cert_hash.to_string(),
                    })
                }
            }
            None => {
                // First connection - save fingerprint
                let fp = Fingerprint::new(machine_name, cert_hash);
                self.save(machine_name, fp)?;
                Ok(FingerprintVerification::FirstConnection)
            }
        }
    }
}

/// Result of fingerprint verification
#[derive(Debug, PartialEq)]
pub enum FingerprintVerification {
    /// Fingerprint matches - connection is safe
    Verified,
    /// First connection - fingerprint saved
    FirstConnection,
    /// Fingerprint mismatch - possible MITM attack
    Mismatch { stored: String, received: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fingerprint_hash_calculation() {
        let data = b"test data";
        let hash = Fingerprint::calculate_hash(data);
        // SHA-256 of "test data"
        assert_eq!(hash.len(), 64); // SHA-256 produces 64 hex characters
    }

    #[test]
    fn test_fingerprint_touch() {
        let mut fp = Fingerprint::new("machine", "hash");
        let first_verified = fp.last_verified.clone();

        std::thread::sleep(std::time::Duration::from_millis(10));
        fp.touch();

        assert_ne!(fp.last_verified, first_verified);
    }
}
