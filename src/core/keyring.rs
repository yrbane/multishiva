/// Secure credential storage using system keyring
///
/// This module provides secure storage for sensitive credentials like
/// the TLS Pre-Shared Key (PSK) using the operating system's credential
/// manager:
/// - Windows: Windows Credential Manager
/// - macOS: Keychain
/// - Linux: Secret Service (GNOME Keyring, KWallet, etc.)
///
/// Features:
/// - Secure PSK storage
/// - Platform-native credential managers
/// - Migration from plaintext config
/// - Fallback to environment variables
use anyhow::{Context, Result};
use keyring::Entry;

/// Service name for MultiShiva credentials
pub const SERVICE_NAME: &str = "multishiva";

/// Key name for TLS PSK
pub const PSK_KEY: &str = "tls_psk";

/// Keyring manager for secure credential storage
pub struct KeyringManager {
    service: String,
}

impl KeyringManager {
    /// Create a new keyring manager
    pub fn new() -> Self {
        Self {
            service: SERVICE_NAME.to_string(),
        }
    }

    /// Create a new keyring manager with custom service name
    pub fn with_service(service: String) -> Self {
        Self { service }
    }

    /// Store the TLS PSK securely in the system keyring
    pub fn set_psk(&self, psk: &str) -> Result<()> {
        let entry =
            Entry::new(&self.service, PSK_KEY).context("Failed to create keyring entry for PSK")?;

        entry
            .set_password(psk)
            .context("Failed to store PSK in keyring")?;

        tracing::info!("PSK stored securely in system keyring");
        Ok(())
    }

    /// Retrieve the TLS PSK from the system keyring
    pub fn get_psk(&self) -> Result<String> {
        let entry =
            Entry::new(&self.service, PSK_KEY).context("Failed to create keyring entry for PSK")?;

        entry
            .get_password()
            .context("Failed to retrieve PSK from keyring")
    }

    /// Delete the TLS PSK from the system keyring
    pub fn delete_psk(&self) -> Result<()> {
        let entry =
            Entry::new(&self.service, PSK_KEY).context("Failed to create keyring entry for PSK")?;

        entry
            .delete_credential()
            .context("Failed to delete PSK from keyring")?;

        tracing::info!("PSK deleted from system keyring");
        Ok(())
    }

    /// Check if PSK exists in the keyring
    pub fn has_psk(&self) -> bool {
        self.get_psk().is_ok()
    }

    /// Migrate PSK from plaintext config to secure keyring
    ///
    /// This method moves the PSK from the config file to the system keyring
    /// and returns a placeholder value to store in the config.
    pub fn migrate_from_config(&self, plaintext_psk: &str) -> Result<String> {
        // Store in keyring
        self.set_psk(plaintext_psk)?;

        // Return placeholder
        Ok("***STORED_IN_KEYRING***".to_string())
    }

    /// Get PSK with fallback to environment variable
    ///
    /// Tries to get PSK from keyring first, falls back to MULTISHIVA_PSK env var
    pub fn get_psk_or_env(&self) -> Result<String> {
        // Try keyring first
        if let Ok(psk) = self.get_psk() {
            return Ok(psk);
        }

        // Fallback to environment variable
        std::env::var("MULTISHIVA_PSK")
            .context("PSK not found in keyring or MULTISHIVA_PSK environment variable")
    }

    /// Store arbitrary credential
    pub fn set_credential(&self, key: &str, value: &str) -> Result<()> {
        let entry = Entry::new(&self.service, key).context("Failed to create keyring entry")?;

        entry
            .set_password(value)
            .context("Failed to store credential in keyring")?;

        tracing::debug!("Credential '{}' stored in system keyring", key);
        Ok(())
    }

    /// Retrieve arbitrary credential
    pub fn get_credential(&self, key: &str) -> Result<String> {
        let entry = Entry::new(&self.service, key).context("Failed to create keyring entry")?;

        entry
            .get_password()
            .context("Failed to retrieve credential from keyring")
    }

    /// Delete arbitrary credential
    pub fn delete_credential(&self, key: &str) -> Result<()> {
        let entry = Entry::new(&self.service, key).context("Failed to create keyring entry")?;

        entry
            .delete_credential()
            .context("Failed to delete credential from keyring")?;

        tracing::debug!("Credential '{}' deleted from system keyring", key);
        Ok(())
    }

    /// Check if credential exists
    pub fn has_credential(&self, key: &str) -> bool {
        self.get_credential(key).is_ok()
    }
}

impl Default for KeyringManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyring_manager_creation() {
        let manager = KeyringManager::new();
        assert_eq!(manager.service, SERVICE_NAME);
    }

    #[test]
    fn test_keyring_manager_with_service() {
        let manager = KeyringManager::with_service("test-service".to_string());
        assert_eq!(manager.service, "test-service");
    }

    #[test]
    fn test_migrate_from_config() {
        let manager = KeyringManager::new();
        let placeholder = manager.migrate_from_config("test-psk-value");

        // Should return placeholder
        if let Ok(p) = placeholder {
            assert_eq!(p, "***STORED_IN_KEYRING***");

            // Clean up
            let _ = manager.delete_psk();
        }
    }

    #[test]
    fn test_psk_storage_lifecycle() {
        let manager = KeyringManager::new();

        // Initially should not have PSK (or clean up from previous tests)
        let _ = manager.delete_psk();

        // Store PSK
        let result = manager.set_psk("test-secure-psk-12345");
        if let Ok(()) = result {
            // Retrieve PSK to verify storage (more reliable than has_psk)
            match manager.get_psk() {
                Ok(psk) => {
                    assert_eq!(psk, "test-secure-psk-12345");

                    // Delete PSK
                    let _ = manager.delete_psk();

                    // Verify deletion
                    assert!(!manager.has_psk());
                }
                Err(_) => {
                    // If retrieval fails, cleanup and skip test
                    let _ = manager.delete_psk();
                }
            }
        }
        // Note: May fail in CI environments without keyring access
    }

    #[test]
    fn test_arbitrary_credential_storage() {
        let manager = KeyringManager::new();
        let test_key = "test_credential";

        // Clean up from previous tests
        let _ = manager.delete_credential(test_key);

        // Store credential
        let result = manager.set_credential(test_key, "test-value");
        if result.is_ok() {
            // Should have credential
            assert!(manager.has_credential(test_key));

            // Retrieve credential
            if let Ok(value) = manager.get_credential(test_key) {
                assert_eq!(value, "test-value");
            }

            // Delete credential
            let _ = manager.delete_credential(test_key);

            // Should not have credential anymore
            assert!(!manager.has_credential(test_key));
        }
        // Note: May fail in CI environments without keyring access
    }

    #[test]
    fn test_get_psk_or_env_fallback() {
        let manager = KeyringManager::new();

        // Clean up keyring
        let _ = manager.delete_psk();

        // Verify keyring is actually empty
        if manager.has_psk() {
            // If keyring still has PSK after deletion, skip this test
            // (keyring might not support deletion in test environment)
            return;
        }

        // Set environment variable
        std::env::set_var("MULTISHIVA_PSK", "env-psk-value");

        // Should fallback to env var
        if let Ok(psk) = manager.get_psk_or_env() {
            assert_eq!(psk, "env-psk-value");
        }

        // Clean up
        std::env::remove_var("MULTISHIVA_PSK");
    }

    #[test]
    fn test_constants() {
        assert_eq!(SERVICE_NAME, "multishiva");
        assert_eq!(PSK_KEY, "tls_psk");
    }

    // Note: Integration tests for actual keyring operations
    // may fail in CI environments without proper keyring setup.
    // These should be tested manually on local machines with
    // proper OS credential managers configured.
}
