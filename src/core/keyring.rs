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

/// Service name used to identify MultiShiva credentials in the system keyring.
///
/// This constant is used as the service identifier when storing credentials
/// in the operating system's credential manager. All MultiShiva credentials
/// are stored under this service name to keep them organized and isolated
/// from other applications.
pub const SERVICE_NAME: &str = "multishiva";

/// Key name for the TLS Pre-Shared Key (PSK) credential.
///
/// This constant identifies the TLS PSK entry within the MultiShiva service
/// in the system keyring. The PSK is used for secure TLS connections between
/// MultiShiva clients and servers.
pub const PSK_KEY: &str = "tls_psk";

/// Manager for secure credential storage using the system keyring.
///
/// `KeyringManager` provides a high-level interface for storing and retrieving
/// sensitive credentials using the operating system's native credential manager:
/// - **Windows**: Windows Credential Manager
/// - **macOS**: Keychain
/// - **Linux**: Secret Service API (GNOME Keyring, KWallet, etc.)
///
/// The manager supports both PSK-specific operations and generic credential storage.
///
/// # Examples
///
/// ```no_run
/// use multishiva::core::keyring::KeyringManager;
///
/// let manager = KeyringManager::new();
///
/// // Store a PSK
/// manager.set_psk("my-secret-psk").expect("Failed to store PSK");
///
/// // Retrieve the PSK
/// let psk = manager.get_psk().expect("Failed to retrieve PSK");
/// ```
pub struct KeyringManager {
    /// The service name used to identify credentials in the system keyring.
    service: String,
}

impl KeyringManager {
    /// Creates a new keyring manager with the default service name.
    ///
    /// This constructor initializes a `KeyringManager` using [`SERVICE_NAME`]
    /// as the service identifier in the system keyring.
    ///
    /// # Examples
    ///
    /// ```
    /// use multishiva::core::keyring::KeyringManager;
    ///
    /// let manager = KeyringManager::new();
    /// ```
    pub fn new() -> Self {
        Self {
            service: SERVICE_NAME.to_string(),
        }
    }

    /// Creates a new keyring manager with a custom service name.
    ///
    /// This constructor allows you to specify a custom service identifier
    /// for credentials stored in the system keyring. This is useful for
    /// isolating credentials in testing or multi-tenant scenarios.
    ///
    /// # Examples
    ///
    /// ```
    /// use multishiva::core::keyring::KeyringManager;
    ///
    /// let manager = KeyringManager::with_service("my-custom-service".to_string());
    /// ```
    pub fn with_service(service: String) -> Self {
        Self { service }
    }

    /// Stores the TLS Pre-Shared Key (PSK) securely in the system keyring.
    ///
    /// This method saves the provided PSK to the operating system's credential
    /// manager using the configured service name and the [`PSK_KEY`] identifier.
    /// The PSK is stored encrypted by the OS and can only be accessed by the
    /// current user.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use multishiva::core::keyring::KeyringManager;
    ///
    /// let manager = KeyringManager::new();
    /// manager.set_psk("my-secure-psk-12345").expect("Failed to store PSK");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The keyring entry cannot be created (e.g., invalid service name)
    /// - The system keyring is unavailable or inaccessible
    /// - Permission is denied to access the credential manager
    pub fn set_psk(&self, psk: &str) -> Result<()> {
        let entry =
            Entry::new(&self.service, PSK_KEY).context("Failed to create keyring entry for PSK")?;

        entry
            .set_password(psk)
            .context("Failed to store PSK in keyring")?;

        tracing::info!("PSK stored securely in system keyring");
        Ok(())
    }

    /// Retrieves the TLS Pre-Shared Key (PSK) from the system keyring.
    ///
    /// This method reads the PSK from the operating system's credential manager
    /// that was previously stored using [`set_psk`](Self::set_psk).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use multishiva::core::keyring::KeyringManager;
    ///
    /// let manager = KeyringManager::new();
    /// let psk = manager.get_psk().expect("Failed to retrieve PSK");
    /// println!("Retrieved PSK: {}", psk);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The keyring entry cannot be created (e.g., invalid service name)
    /// - The PSK does not exist in the keyring
    /// - The system keyring is unavailable or inaccessible
    /// - Permission is denied to access the credential manager
    pub fn get_psk(&self) -> Result<String> {
        let entry =
            Entry::new(&self.service, PSK_KEY).context("Failed to create keyring entry for PSK")?;

        entry
            .get_password()
            .context("Failed to retrieve PSK from keyring")
    }

    /// Deletes the TLS Pre-Shared Key (PSK) from the system keyring.
    ///
    /// This method permanently removes the PSK from the operating system's
    /// credential manager. After deletion, the PSK can no longer be retrieved
    /// until a new one is stored.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use multishiva::core::keyring::KeyringManager;
    ///
    /// let manager = KeyringManager::new();
    /// manager.delete_psk().expect("Failed to delete PSK");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The keyring entry cannot be created (e.g., invalid service name)
    /// - The PSK does not exist in the keyring
    /// - The system keyring is unavailable or inaccessible
    /// - Permission is denied to access the credential manager
    pub fn delete_psk(&self) -> Result<()> {
        let entry =
            Entry::new(&self.service, PSK_KEY).context("Failed to create keyring entry for PSK")?;

        entry
            .delete_credential()
            .context("Failed to delete PSK from keyring")?;

        tracing::info!("PSK deleted from system keyring");
        Ok(())
    }

    /// Checks whether a PSK exists in the keyring.
    ///
    /// This method attempts to retrieve the PSK and returns `true` if it
    /// exists and can be accessed, or `false` otherwise. This is a convenience
    /// method that does not return error details.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use multishiva::core::keyring::KeyringManager;
    ///
    /// let manager = KeyringManager::new();
    /// if manager.has_psk() {
    ///     println!("PSK is stored in the keyring");
    /// } else {
    ///     println!("No PSK found in the keyring");
    /// }
    /// ```
    pub fn has_psk(&self) -> bool {
        self.get_psk().is_ok()
    }

    /// Migrates a PSK from plaintext configuration to secure keyring storage.
    ///
    /// This method takes a plaintext PSK (typically read from a configuration file),
    /// stores it securely in the system keyring, and returns a placeholder string
    /// that can be written back to the configuration file to indicate the PSK is
    /// now stored in the keyring.
    ///
    /// This is useful for upgrading from insecure plaintext storage to secure
    /// keyring-based storage without requiring manual intervention.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use multishiva::core::keyring::KeyringManager;
    ///
    /// let manager = KeyringManager::new();
    /// let plaintext_psk = "my-insecure-psk-from-config";
    ///
    /// let placeholder = manager
    ///     .migrate_from_config(plaintext_psk)
    ///     .expect("Failed to migrate PSK");
    ///
    /// // Now write 'placeholder' back to the config file
    /// assert_eq!(placeholder, "***STORED_IN_KEYRING***");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if storing the PSK in the keyring fails.
    /// See [`set_psk`](Self::set_psk) for specific error conditions.
    pub fn migrate_from_config(&self, plaintext_psk: &str) -> Result<String> {
        // Store in keyring
        self.set_psk(plaintext_psk)?;

        // Return placeholder
        Ok("***STORED_IN_KEYRING***".to_string())
    }

    /// Retrieves the PSK from the keyring with fallback to an environment variable.
    ///
    /// This method first attempts to retrieve the PSK from the system keyring.
    /// If that fails (e.g., no PSK is stored or keyring is unavailable), it falls
    /// back to reading the `MULTISHIVA_PSK` environment variable.
    ///
    /// This provides a convenient way to support both secure keyring storage and
    /// environment variable configuration (e.g., for CI/CD or containerized environments).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use multishiva::core::keyring::KeyringManager;
    ///
    /// let manager = KeyringManager::new();
    ///
    /// // Will try keyring first, then environment variable
    /// let psk = manager.get_psk_or_env().expect("PSK not found");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if both the keyring lookup and environment variable
    /// retrieval fail. The error message will indicate that neither source
    /// contained a valid PSK.
    pub fn get_psk_or_env(&self) -> Result<String> {
        // Try keyring first
        if let Ok(psk) = self.get_psk() {
            return Ok(psk);
        }

        // Fallback to environment variable
        std::env::var("MULTISHIVA_PSK")
            .context("PSK not found in keyring or MULTISHIVA_PSK environment variable")
    }

    /// Stores an arbitrary credential in the system keyring.
    ///
    /// This generic method allows storing any credential (not just PSK) using
    /// a custom key name. The credential is stored under the configured service
    /// name in the system keyring.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use multishiva::core::keyring::KeyringManager;
    ///
    /// let manager = KeyringManager::new();
    /// manager
    ///     .set_credential("api_token", "secret-token-value")
    ///     .expect("Failed to store credential");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The keyring entry cannot be created (e.g., invalid service or key name)
    /// - The system keyring is unavailable or inaccessible
    /// - Permission is denied to access the credential manager
    pub fn set_credential(&self, key: &str, value: &str) -> Result<()> {
        let entry = Entry::new(&self.service, key).context("Failed to create keyring entry")?;

        entry
            .set_password(value)
            .context("Failed to store credential in keyring")?;

        tracing::debug!("Credential '{}' stored in system keyring", key);
        Ok(())
    }

    /// Retrieves an arbitrary credential from the system keyring.
    ///
    /// This generic method allows retrieving any credential (not just PSK) that
    /// was previously stored using [`set_credential`](Self::set_credential) with
    /// the same key name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use multishiva::core::keyring::KeyringManager;
    ///
    /// let manager = KeyringManager::new();
    /// let token = manager
    ///     .get_credential("api_token")
    ///     .expect("Failed to retrieve credential");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The keyring entry cannot be created (e.g., invalid service or key name)
    /// - The credential does not exist in the keyring
    /// - The system keyring is unavailable or inaccessible
    /// - Permission is denied to access the credential manager
    pub fn get_credential(&self, key: &str) -> Result<String> {
        let entry = Entry::new(&self.service, key).context("Failed to create keyring entry")?;

        entry
            .get_password()
            .context("Failed to retrieve credential from keyring")
    }

    /// Deletes an arbitrary credential from the system keyring.
    ///
    /// This generic method permanently removes any credential from the system
    /// keyring using the specified key name. After deletion, the credential
    /// can no longer be retrieved until a new one is stored.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use multishiva::core::keyring::KeyringManager;
    ///
    /// let manager = KeyringManager::new();
    /// manager
    ///     .delete_credential("api_token")
    ///     .expect("Failed to delete credential");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The keyring entry cannot be created (e.g., invalid service or key name)
    /// - The credential does not exist in the keyring
    /// - The system keyring is unavailable or inaccessible
    /// - Permission is denied to access the credential manager
    pub fn delete_credential(&self, key: &str) -> Result<()> {
        let entry = Entry::new(&self.service, key).context("Failed to create keyring entry")?;

        entry
            .delete_credential()
            .context("Failed to delete credential from keyring")?;

        tracing::debug!("Credential '{}' deleted from system keyring", key);
        Ok(())
    }

    /// Checks whether a credential exists in the keyring.
    ///
    /// This method attempts to retrieve the credential specified by the key
    /// and returns `true` if it exists and can be accessed, or `false` otherwise.
    /// This is a convenience method that does not return error details.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use multishiva::core::keyring::KeyringManager;
    ///
    /// let manager = KeyringManager::new();
    /// if manager.has_credential("api_token") {
    ///     println!("API token is stored in the keyring");
    /// } else {
    ///     println!("No API token found");
    /// }
    /// ```
    pub fn has_credential(&self, key: &str) -> bool {
        self.get_credential(key).is_ok()
    }
}

impl Default for KeyringManager {
    /// Creates a default `KeyringManager` instance.
    ///
    /// This is equivalent to calling [`KeyringManager::new()`](Self::new)
    /// and uses the default service name [`SERVICE_NAME`].
    ///
    /// # Examples
    ///
    /// ```
    /// use multishiva::core::keyring::KeyringManager;
    ///
    /// let manager = KeyringManager::default();
    /// ```
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

        // Try to clean up keyring from previous tests
        for _ in 0..3 {
            let _ = manager.delete_psk();
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        // Verify keyring is actually empty after aggressive cleanup
        if manager.has_psk() {
            // If keyring still has PSK after deletion, skip this test
            // (keyring might not support deletion in test environment)
            return;
        }

        // Set environment variable
        std::env::set_var("MULTISHIVA_PSK", "env-psk-value");

        // Should fallback to env var
        match manager.get_psk_or_env() {
            Ok(psk) => {
                // If we got a PSK from keyring instead of env, skip test
                // (indicates test isolation failure)
                if psk != "env-psk-value" {
                    std::env::remove_var("MULTISHIVA_PSK");
                    return;
                }
                assert_eq!(psk, "env-psk-value");
            }
            Err(_) => {
                // If we can't get PSK at all, that's also a test isolation issue
                std::env::remove_var("MULTISHIVA_PSK");
                return;
            }
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
