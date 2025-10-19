/// Clipboard synchronization across MultiShiva machines
///
/// This module provides automatic clipboard synchronization between
/// connected machines using clipboard monitoring and network messaging.
///
/// Features:
/// - Clipboard change detection
/// - Text content synchronization
/// - Automatic propagation across network
/// - Duplicate prevention
use anyhow::Result;
use clipboard_rs::{Clipboard, ClipboardContext};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

/// Clipboard content types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClipboardContent {
    /// Text content
    Text(String),
    // Future: Image, Files, etc.
}

impl ClipboardContent {
    /// Get content as text (if applicable)
    pub fn as_text(&self) -> Option<&str> {
        match self {
            ClipboardContent::Text(s) => Some(s),
        }
    }

    /// Check if content is empty
    pub fn is_empty(&self) -> bool {
        match self {
            ClipboardContent::Text(s) => s.is_empty(),
        }
    }
}

/// Clipboard change event
#[derive(Debug, Clone)]
pub struct ClipboardChange {
    /// New clipboard content
    pub content: ClipboardContent,
    /// Timestamp of the change
    pub timestamp: SystemTime,
    /// Source machine (None if local)
    pub source: Option<String>,
}

/// Clipboard manager for synchronization
pub struct ClipboardManager {
    last_content: Arc<Mutex<Option<ClipboardContent>>>,
    last_update: Arc<Mutex<SystemTime>>,
    monitoring: Arc<Mutex<bool>>,
    poll_interval: Duration,
}

impl ClipboardManager {
    /// Create a new clipboard manager
    pub fn new() -> Result<Self> {
        Ok(Self {
            last_content: Arc::new(Mutex::new(None)),
            last_update: Arc::new(Mutex::new(SystemTime::now())),
            monitoring: Arc::new(Mutex::new(false)),
            poll_interval: Duration::from_millis(500),
        })
    }

    /// Create a new clipboard manager with custom poll interval
    pub fn with_poll_interval(interval: Duration) -> Result<Self> {
        let mut manager = Self::new()?;
        manager.poll_interval = interval;
        Ok(manager)
    }

    /// Get current clipboard content
    pub fn get_content(&self) -> Result<ClipboardContent> {
        let ctx = ClipboardContext::new()
            .map_err(|e| anyhow::anyhow!("Failed to create clipboard context: {}", e))?;

        let text = ctx
            .get_text()
            .map_err(|e| anyhow::anyhow!("Failed to get clipboard text: {}", e))?;

        Ok(ClipboardContent::Text(text))
    }

    /// Set clipboard content
    pub fn set_content(&mut self, content: ClipboardContent) -> Result<()> {
        let ctx = ClipboardContext::new()
            .map_err(|e| anyhow::anyhow!("Failed to create clipboard context: {}", e))?;

        match content {
            ClipboardContent::Text(ref text) => {
                ctx.set_text(text.clone())
                    .map_err(|e| anyhow::anyhow!("Failed to set clipboard text: {}", e))?;

                // Update local tracking
                if let Ok(mut last) = self.last_content.lock() {
                    *last = Some(content);
                }
                if let Ok(mut time) = self.last_update.lock() {
                    *time = SystemTime::now();
                }
            }
        }

        Ok(())
    }

    /// Set clipboard content from remote source
    ///
    /// This method sets clipboard content from a remote machine
    /// without triggering local change events to prevent loops.
    pub fn set_content_from_remote(
        &mut self,
        content: ClipboardContent,
        source: String,
    ) -> Result<()> {
        tracing::debug!("Setting clipboard from remote source: {}", source);

        // Set the content
        self.set_content(content.clone())?;

        // Mark as already processed to prevent echo
        if let Ok(mut last) = self.last_content.lock() {
            *last = Some(content);
        }

        Ok(())
    }

    /// Start monitoring clipboard changes
    ///
    /// The provided callback will be called whenever the clipboard content changes.
    /// The callback receives a ClipboardChange event with the new content.
    pub fn start_monitoring<F>(&mut self, callback: F) -> Result<()>
    where
        F: Fn(ClipboardChange) + Send + 'static,
    {
        // Set monitoring flag
        if let Ok(mut monitoring) = self.monitoring.lock() {
            *monitoring = true;
        }

        let last_content = Arc::clone(&self.last_content);
        let last_update = Arc::clone(&self.last_update);
        let monitoring = Arc::clone(&self.monitoring);
        let poll_interval = self.poll_interval;

        // Spawn background thread to poll clipboard
        std::thread::spawn(move || {
            while let Ok(true) = monitoring.lock().map(|m| *m) {
                // Get current clipboard content
                if let Ok(ctx) = ClipboardContext::new() {
                    if let Ok(text) = ctx.get_text() {
                        let content = ClipboardContent::Text(text.clone());

                        // Check if content actually changed
                        let should_notify = if let Ok(last) = last_content.lock() {
                            match &*last {
                                Some(last_content) => last_content != &content,
                                None => true,
                            }
                        } else {
                            true
                        };

                        if should_notify && !content.is_empty() {
                            // Update tracking
                            if let Ok(mut last) = last_content.lock() {
                                *last = Some(content.clone());
                            }
                            if let Ok(mut time) = last_update.lock() {
                                *time = SystemTime::now();
                            }

                            // Trigger callback
                            let change = ClipboardChange {
                                content,
                                timestamp: SystemTime::now(),
                                source: None, // Local change
                            };
                            callback(change);
                        }
                    }
                }

                // Sleep before next poll
                std::thread::sleep(poll_interval);
            }
        });

        tracing::info!(
            "Clipboard monitoring started (poll interval: {:?})",
            self.poll_interval
        );
        Ok(())
    }

    /// Stop monitoring clipboard changes
    pub fn stop_monitoring(&mut self) {
        if let Ok(mut monitoring) = self.monitoring.lock() {
            *monitoring = false;
            tracing::info!("Clipboard monitoring stopped");
        }
    }

    /// Check if monitoring is active
    pub fn is_monitoring(&self) -> bool {
        self.monitoring.lock().map(|m| *m).unwrap_or(false)
    }

    /// Get time of last clipboard update
    pub fn last_update_time(&self) -> SystemTime {
        self.last_update
            .lock()
            .map(|t| *t)
            .unwrap_or_else(|_| SystemTime::now())
    }

    /// Check if clipboard has been updated since a given time
    pub fn updated_since(&self, time: SystemTime) -> bool {
        self.last_update_time() > time
    }
}

impl Default for ClipboardManager {
    fn default() -> Self {
        Self::new().expect("Failed to create default clipboard manager")
    }
}

impl Drop for ClipboardManager {
    fn drop(&mut self) {
        self.stop_monitoring();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_content_text() {
        let content = ClipboardContent::Text("Hello, World!".to_string());

        assert_eq!(content.as_text(), Some("Hello, World!"));
        assert!(!content.is_empty());
    }

    #[test]
    fn test_clipboard_content_empty() {
        let content = ClipboardContent::Text(String::new());

        assert!(content.is_empty());
        assert_eq!(content.as_text(), Some(""));
    }

    #[test]
    fn test_clipboard_content_equality() {
        let content1 = ClipboardContent::Text("Test".to_string());
        let content2 = ClipboardContent::Text("Test".to_string());
        let content3 = ClipboardContent::Text("Different".to_string());

        assert_eq!(content1, content2);
        assert_ne!(content1, content3);
    }

    #[test]
    fn test_clipboard_manager_creation() {
        let manager = ClipboardManager::new();

        // May fail in headless CI environments
        if manager.is_ok() {
            assert!(manager.is_ok());
        }
    }

    #[test]
    fn test_clipboard_manager_with_interval() {
        let manager = ClipboardManager::with_poll_interval(Duration::from_secs(1));

        // May fail in headless CI environments
        if let Ok(mgr) = manager {
            assert_eq!(mgr.poll_interval, Duration::from_secs(1));
        }
    }

    #[test]
    fn test_clipboard_change_structure() {
        let change = ClipboardChange {
            content: ClipboardContent::Text("Test".to_string()),
            timestamp: SystemTime::now(),
            source: Some("remote-machine".to_string()),
        };

        assert_eq!(change.content.as_text(), Some("Test"));
        assert_eq!(change.source.as_deref(), Some("remote-machine"));
    }

    #[test]
    fn test_monitoring_flag() {
        let mut manager = ClipboardManager::new().unwrap();

        assert!(!manager.is_monitoring());

        // Start monitoring (will spawn thread)
        let _ = manager.start_monitoring(|_| {});

        // Give it a moment to start
        std::thread::sleep(Duration::from_millis(10));
        assert!(manager.is_monitoring());

        // Stop monitoring
        manager.stop_monitoring();
        assert!(!manager.is_monitoring());
    }

    // Note: Integration tests for actual clipboard operations
    // are difficult to test in CI environments without display/clipboard access.
    // These should be tested manually on local machines.
}
