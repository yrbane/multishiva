use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// TLS certificate fingerprint for MITM detection.
///
/// A fingerprint stores the SHA-256 hash of a TLS certificate associated with
/// a specific machine. This enables detection of potential man-in-the-middle
/// attacks by comparing subsequent connections against the initially trusted
/// certificate.
///
/// The fingerprint tracks:
/// - The machine name (hostname or identifier)
/// - The SHA-256 hash of the certificate
/// - When the fingerprint was first seen
/// - When it was last successfully verified
///
/// # Examples
///
/// ```
/// use multishiva::core::fingerprint::Fingerprint;
///
/// // Create a fingerprint from raw certificate data
/// let cert_data = b"certificate data";
/// let fp = Fingerprint::from_cert_data("example.com", cert_data);
///
/// // Verify a certificate hash
/// assert!(fp.verify(fp.hash()));
/// ```
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
    /// Creates a new fingerprint with a known hash.
    ///
    /// This constructor is used when you already have a computed hash value.
    /// The fingerprint's `first_seen` and `last_verified` timestamps are set
    /// to the current UTC time.
    ///
    /// # Arguments
    ///
    /// * `machine_name` - The hostname or identifier of the machine
    /// * `hash` - The SHA-256 hash of the certificate (as hex string)
    ///
    /// # Examples
    ///
    /// ```
    /// use multishiva::core::fingerprint::Fingerprint;
    ///
    /// let fp = Fingerprint::new("example.com", "abc123");
    /// assert_eq!(fp.machine_name(), "example.com");
    /// assert_eq!(fp.hash(), "abc123");
    /// ```
    pub fn new(machine_name: impl Into<String>, hash: impl Into<String>) -> Self {
        Self {
            machine_name: machine_name.into(),
            hash: hash.into(),
            first_seen: Some(chrono::Utc::now().to_rfc3339()),
            last_verified: Some(chrono::Utc::now().to_rfc3339()),
        }
    }

    /// Creates a fingerprint from raw certificate data.
    ///
    /// This constructor computes the SHA-256 hash of the provided certificate
    /// data and creates a new fingerprint with it. Use this when you have the
    /// raw certificate bytes and need to compute the hash.
    ///
    /// # Arguments
    ///
    /// * `machine_name` - The hostname or identifier of the machine
    /// * `cert_data` - The raw certificate data to hash
    ///
    /// # Examples
    ///
    /// ```
    /// use multishiva::core::fingerprint::Fingerprint;
    ///
    /// let cert_data = b"certificate data";
    /// let fp = Fingerprint::from_cert_data("example.com", cert_data);
    /// assert_eq!(fp.machine_name(), "example.com");
    /// ```
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

    /// Returns the machine name associated with this fingerprint.
    ///
    /// # Examples
    ///
    /// ```
    /// use multishiva::core::fingerprint::Fingerprint;
    ///
    /// let fp = Fingerprint::new("example.com", "hash123");
    /// assert_eq!(fp.machine_name(), "example.com");
    /// ```
    pub fn machine_name(&self) -> &str {
        &self.machine_name
    }

    /// Returns the SHA-256 hash of the certificate.
    ///
    /// The hash is returned as a hexadecimal string.
    ///
    /// # Examples
    ///
    /// ```
    /// use multishiva::core::fingerprint::Fingerprint;
    ///
    /// let fp = Fingerprint::new("example.com", "abc123");
    /// assert_eq!(fp.hash(), "abc123");
    /// ```
    pub fn hash(&self) -> &str {
        &self.hash
    }

    /// Verifies if a certificate hash matches this fingerprint.
    ///
    /// Returns `true` if the provided hash matches the stored hash,
    /// `false` otherwise. This is used to detect if a certificate
    /// has changed since it was first seen.
    ///
    /// # Arguments
    ///
    /// * `cert_hash` - The certificate hash to verify (as hex string)
    ///
    /// # Examples
    ///
    /// ```
    /// use multishiva::core::fingerprint::Fingerprint;
    ///
    /// let fp = Fingerprint::new("example.com", "abc123");
    /// assert!(fp.verify("abc123"));
    /// assert!(!fp.verify("different"));
    /// ```
    pub fn verify(&self, cert_hash: &str) -> bool {
        self.hash == cert_hash
    }

    /// Updates the last verified timestamp to the current time.
    ///
    /// This should be called after successfully verifying a certificate
    /// to track when the fingerprint was last confirmed as valid.
    ///
    /// # Examples
    ///
    /// ```
    /// use multishiva::core::fingerprint::Fingerprint;
    ///
    /// let mut fp = Fingerprint::new("example.com", "abc123");
    /// fp.touch();
    /// // last_verified is now updated to current time
    /// ```
    pub fn touch(&mut self) {
        self.last_verified = Some(chrono::Utc::now().to_rfc3339());
    }
}

/// Persistent storage for TLS certificate fingerprints.
///
/// The fingerprint store manages a collection of certificate fingerprints,
/// storing them persistently in a JSON file. It provides functionality to
/// save, retrieve, and verify fingerprints for multiple machines.
///
/// The store automatically handles:
/// - Loading existing fingerprints from disk
/// - Creating the storage directory if it doesn't exist
/// - Persisting changes to disk
/// - First-time certificate acceptance (TOFU - Trust On First Use)
///
/// # Examples
///
/// ```no_run
/// use multishiva::core::fingerprint::FingerprintStore;
///
/// // Load the default store
/// let mut store = FingerprintStore::load_default()?;
///
/// // Verify or save a certificate
/// match store.verify_or_save("example.com", "abc123")? {
///     FingerprintVerification::Verified => println!("Certificate verified"),
///     FingerprintVerification::FirstConnection => println!("First connection, fingerprint saved"),
///     FingerprintVerification::Mismatch { stored, received } => {
///         println!("WARNING: Certificate mismatch!");
///     }
/// }
/// # Ok::<(), anyhow::Error>(())
/// ```
#[derive(Debug)]
pub struct FingerprintStore {
    path: PathBuf,
    fingerprints: HashMap<String, Fingerprint>,
}

impl FingerprintStore {
    /// Creates a new fingerprint store at the specified path.
    ///
    /// If the file exists, fingerprints are loaded from it. If the file doesn't
    /// exist, an empty store is created and the parent directories are created
    /// if necessary.
    ///
    /// # Arguments
    ///
    /// * `path` - The file path where fingerprints will be stored
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file exists but cannot be read
    /// - The file exists but contains invalid JSON
    /// - The parent directory cannot be created
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use multishiva::core::fingerprint::FingerprintStore;
    /// use std::path::PathBuf;
    ///
    /// let store = FingerprintStore::new(PathBuf::from("/tmp/fingerprints.json"))?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
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

    /// Returns the default store path for fingerprints.
    ///
    /// The default path is `~/.config/multishiva/fingerprints.json` on Unix
    /// systems, or the equivalent on other platforms. If the config directory
    /// cannot be determined, falls back to the current directory.
    ///
    /// # Examples
    ///
    /// ```
    /// use multishiva::core::fingerprint::FingerprintStore;
    ///
    /// let path = FingerprintStore::default_path();
    /// println!("Default fingerprint store: {:?}", path);
    /// ```
    pub fn default_path() -> PathBuf {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("multishiva");
        config_dir.join("fingerprints.json")
    }

    /// Loads the fingerprint store from the default path.
    ///
    /// This is a convenience method that combines `default_path()` and `new()`.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file exists but cannot be read
    /// - The file exists but contains invalid JSON
    /// - The parent directory cannot be created
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use multishiva::core::fingerprint::FingerprintStore;
    ///
    /// let store = FingerprintStore::load_default()?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn load_default() -> Result<Self> {
        Self::new(Self::default_path())
    }

    /// Saves a fingerprint for a machine and persists it to disk.
    ///
    /// If a fingerprint already exists for the machine, it will be replaced.
    /// The changes are immediately written to the store file.
    ///
    /// # Arguments
    ///
    /// * `machine_name` - The hostname or identifier of the machine
    /// * `fingerprint` - The fingerprint to save
    ///
    /// # Errors
    ///
    /// Returns an error if the store cannot be written to disk.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use multishiva::core::fingerprint::{FingerprintStore, Fingerprint};
    ///
    /// let mut store = FingerprintStore::load_default()?;
    /// let fp = Fingerprint::new("example.com", "abc123");
    /// store.save("example.com", fp)?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn save(
        &mut self,
        machine_name: impl Into<String>,
        fingerprint: Fingerprint,
    ) -> Result<()> {
        let machine_name = machine_name.into();
        self.fingerprints.insert(machine_name, fingerprint);
        self.persist()
    }

    /// Retrieves the stored fingerprint for a machine.
    ///
    /// Returns `None` if no fingerprint has been stored for the given machine.
    ///
    /// # Arguments
    ///
    /// * `machine_name` - The hostname or identifier of the machine
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use multishiva::core::fingerprint::FingerprintStore;
    ///
    /// let store = FingerprintStore::load_default()?;
    /// if let Some(fp) = store.get("example.com") {
    ///     println!("Hash: {}", fp.hash());
    /// }
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn get(&self, machine_name: &str) -> Option<&Fingerprint> {
        self.fingerprints.get(machine_name)
    }

    /// Removes a fingerprint for a machine and persists the change to disk.
    ///
    /// If no fingerprint exists for the machine, this is a no-op but the
    /// store is still persisted to disk.
    ///
    /// # Arguments
    ///
    /// * `machine_name` - The hostname or identifier of the machine
    ///
    /// # Errors
    ///
    /// Returns an error if the store cannot be written to disk.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use multishiva::core::fingerprint::FingerprintStore;
    ///
    /// let mut store = FingerprintStore::load_default()?;
    /// store.remove("example.com")?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn remove(&mut self, machine_name: &str) -> Result<()> {
        self.fingerprints.remove(machine_name);
        self.persist()
    }

    /// Returns a list of all stored fingerprints.
    ///
    /// The fingerprints are returned in an arbitrary order.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use multishiva::core::fingerprint::FingerprintStore;
    ///
    /// let store = FingerprintStore::load_default()?;
    /// for fp in store.list_all() {
    ///     println!("{}: {}", fp.machine_name(), fp.hash());
    /// }
    /// # Ok::<(), anyhow::Error>(())
    /// ```
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

    /// Verifies a certificate hash against the stored fingerprint, or saves it if first connection.
    ///
    /// This implements the Trust On First Use (TOFU) security model:
    /// - If this is the first connection to the machine, the fingerprint is saved
    /// - If the hash matches the stored fingerprint, verification succeeds
    /// - If the hash doesn't match, a mismatch is reported (potential MITM attack)
    ///
    /// # Arguments
    ///
    /// * `machine_name` - The hostname or identifier of the machine
    /// * `cert_hash` - The SHA-256 hash of the certificate to verify
    ///
    /// # Returns
    ///
    /// Returns a `FingerprintVerification` indicating the result:
    /// - `Verified` - The hash matches the stored fingerprint
    /// - `FirstConnection` - No stored fingerprint, the provided hash was saved
    /// - `Mismatch` - The hash doesn't match the stored fingerprint
    ///
    /// # Errors
    ///
    /// Returns an error if the fingerprint cannot be saved to disk (only on first connection).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use multishiva::core::fingerprint::{FingerprintStore, FingerprintVerification};
    ///
    /// let mut store = FingerprintStore::load_default()?;
    /// match store.verify_or_save("example.com", "abc123")? {
    ///     FingerprintVerification::Verified => {
    ///         println!("Certificate verified successfully");
    ///     }
    ///     FingerprintVerification::FirstConnection => {
    ///         println!("First connection, fingerprint saved");
    ///     }
    ///     FingerprintVerification::Mismatch { stored, received } => {
    ///         eprintln!("WARNING: Certificate mismatch detected!");
    ///         eprintln!("Stored: {}", stored);
    ///         eprintln!("Received: {}", received);
    ///     }
    /// }
    /// # Ok::<(), anyhow::Error>(())
    /// ```
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

/// Result of a fingerprint verification operation.
///
/// This enum represents the three possible outcomes when verifying a
/// certificate fingerprint against the store.
///
/// # Examples
///
/// ```no_run
/// use multishiva::core::fingerprint::{FingerprintStore, FingerprintVerification};
///
/// let mut store = FingerprintStore::load_default()?;
/// match store.verify_or_save("example.com", "abc123")? {
///     FingerprintVerification::Verified => {
///         println!("Safe to connect");
///     }
///     FingerprintVerification::FirstConnection => {
///         println!("First time connecting, fingerprint saved");
///     }
///     FingerprintVerification::Mismatch { stored, received } => {
///         eprintln!("WARNING: Possible MITM attack!");
///     }
/// }
/// # Ok::<(), anyhow::Error>(())
/// ```
#[derive(Debug, PartialEq)]
pub enum FingerprintVerification {
    /// The fingerprint matches the stored value - connection is safe.
    Verified,
    /// First connection to this machine - fingerprint has been saved.
    ///
    /// This implements the Trust On First Use (TOFU) security model.
    FirstConnection,
    /// The fingerprint does not match the stored value.
    ///
    /// This indicates a possible man-in-the-middle (MITM) attack or
    /// legitimate certificate rotation. Manual verification is recommended.
    Mismatch {
        /// The fingerprint hash stored in the database
        stored: String,
        /// The fingerprint hash received from the current connection
        received: String,
    },
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
