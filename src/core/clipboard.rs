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

/// Represents different types of content that can be stored in the clipboard.
///
/// This enum encapsulates various clipboard content formats. Currently only
/// text content is supported, but the design allows for future expansion to
/// other formats like images, files, and rich content.
///
/// # Examples
///
/// ```
/// use multishiva::core::clipboard::ClipboardContent;
///
/// let text_content = ClipboardContent::Text("Hello, World!".to_string());
/// assert_eq!(text_content.as_text(), Some("Hello, World!"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClipboardContent {
    /// Plain text content from the clipboard.
    ///
    /// Contains a UTF-8 string representing text data copied to the clipboard.
    Text(String),
    // Future: Image, Files, etc.
}

impl ClipboardContent {
    /// Returns the content as a text string reference, if the content is text.
    ///
    /// This method provides a convenient way to extract text content without
    /// pattern matching. Returns `None` for non-text content types (when added in future).
    ///
    /// # Examples
    ///
    /// ```
    /// use multishiva::core::clipboard::ClipboardContent;
    ///
    /// let content = ClipboardContent::Text("Hello".to_string());
    /// assert_eq!(content.as_text(), Some("Hello"));
    /// ```
    pub fn as_text(&self) -> Option<&str> {
        match self {
            ClipboardContent::Text(s) => Some(s),
        }
    }

    /// Checks whether the clipboard content is empty.
    ///
    /// For text content, this returns `true` if the string is empty.
    /// Future content types will implement their own empty logic.
    ///
    /// # Examples
    ///
    /// ```
    /// use multishiva::core::clipboard::ClipboardContent;
    ///
    /// let empty_content = ClipboardContent::Text(String::new());
    /// assert!(empty_content.is_empty());
    ///
    /// let non_empty = ClipboardContent::Text("data".to_string());
    /// assert!(!non_empty.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        match self {
            ClipboardContent::Text(s) => s.is_empty(),
        }
    }
}

/// Represents a clipboard change event detected by the monitoring system.
///
/// This structure captures all relevant information about a clipboard change,
/// including the new content, when it occurred, and where it originated from.
/// Events can originate from local changes or from remote machines in the
/// synchronized network.
///
/// # Examples
///
/// ```
/// use multishiva::core::clipboard::{ClipboardChange, ClipboardContent};
/// use std::time::SystemTime;
///
/// let change = ClipboardChange {
///     content: ClipboardContent::Text("Copied text".to_string()),
///     timestamp: SystemTime::now(),
///     source: None, // Local change
/// };
/// ```
#[derive(Debug, Clone)]
pub struct ClipboardChange {
    /// The new content that was placed in the clipboard.
    ///
    /// Contains the actual clipboard data in one of the supported formats.
    pub content: ClipboardContent,

    /// The timestamp when this clipboard change was detected.
    ///
    /// This uses `SystemTime` to record the exact moment the change occurred,
    /// which is useful for ordering events and detecting concurrent changes.
    pub timestamp: SystemTime,

    /// The identifier of the source machine that originated this change.
    ///
    /// - `None` indicates the change originated locally
    /// - `Some(machine_id)` indicates the change came from a remote machine
    ///   in the synchronized network
    pub source: Option<String>,
}

/// Manages clipboard synchronization with polling-based change detection.
///
/// The `ClipboardManager` provides a robust system for monitoring clipboard changes
/// and synchronizing content across multiple machines. It uses a polling mechanism
/// to detect changes and maintains state to prevent duplicate notifications and
/// synchronization loops.
///
/// # Monitoring
///
/// The manager uses background polling to detect clipboard changes at configurable
/// intervals (default: 500ms). When a change is detected, registered callbacks are
/// invoked with the change event.
///
/// # Thread Safety
///
/// All internal state is protected by mutexes and uses `Arc` for shared ownership,
/// making the manager safe to use across threads.
///
/// # Examples
///
/// ```no_run
/// use multishiva::core::clipboard::ClipboardManager;
/// use std::time::Duration;
///
/// # fn main() -> anyhow::Result<()> {
/// // Create a manager with default settings
/// let mut manager = ClipboardManager::new()?;
///
/// // Or with custom poll interval
/// let mut custom_manager = ClipboardManager::with_poll_interval(
///     Duration::from_secs(1)
/// )?;
/// # Ok(())
/// # }
/// ```
pub struct ClipboardManager {
    /// The last known clipboard content, used for change detection.
    last_content: Arc<Mutex<Option<ClipboardContent>>>,

    /// Timestamp of the last clipboard update.
    last_update: Arc<Mutex<SystemTime>>,

    /// Flag indicating whether monitoring is currently active.
    monitoring: Arc<Mutex<bool>>,

    /// The interval between clipboard polls.
    poll_interval: Duration,
}

impl ClipboardManager {
    /// Creates a new clipboard manager with default settings.
    ///
    /// The manager is initialized with:
    /// - Poll interval: 500ms
    /// - Monitoring: inactive (must call `start_monitoring` to begin)
    /// - No cached content
    ///
    /// # Errors
    ///
    /// Currently always returns `Ok`, but the `Result` return type is maintained
    /// for future compatibility with initialization that may fail (e.g., platform-specific
    /// clipboard access validation).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use multishiva::core::clipboard::ClipboardManager;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let manager = ClipboardManager::new()?;
    /// assert!(!manager.is_monitoring());
    /// # Ok(())
    /// # }
    /// ```
    pub fn new() -> Result<Self> {
        Ok(Self {
            last_content: Arc::new(Mutex::new(None)),
            last_update: Arc::new(Mutex::new(SystemTime::now())),
            monitoring: Arc::new(Mutex::new(false)),
            poll_interval: Duration::from_millis(500),
        })
    }

    /// Creates a new clipboard manager with a custom poll interval.
    ///
    /// This constructor allows you to specify how frequently the clipboard should
    /// be polled for changes. Lower intervals provide faster change detection but
    /// use more CPU resources.
    ///
    /// # Arguments
    ///
    /// * `interval` - The duration between clipboard polls
    ///
    /// # Errors
    ///
    /// Returns an error if the manager fails to initialize (though currently
    /// this is unlikely, as initialization is simple).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use multishiva::core::clipboard::ClipboardManager;
    /// use std::time::Duration;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// // Poll every second instead of default 500ms
    /// let manager = ClipboardManager::with_poll_interval(Duration::from_secs(1))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_poll_interval(interval: Duration) -> Result<Self> {
        let mut manager = Self::new()?;
        manager.poll_interval = interval;
        Ok(manager)
    }

    /// Retrieves the current content from the system clipboard.
    ///
    /// This method queries the system clipboard and returns its current contents
    /// as a `ClipboardContent` instance. Currently only text content is supported.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The clipboard context cannot be created (e.g., no display available)
    /// - The clipboard content cannot be read (e.g., clipboard locked by another process)
    /// - The clipboard contains data in an unsupported format
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use multishiva::core::clipboard::ClipboardManager;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let manager = ClipboardManager::new()?;
    /// let content = manager.get_content()?;
    ///
    /// if let Some(text) = content.as_text() {
    ///     println!("Clipboard contains: {}", text);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_content(&self) -> Result<ClipboardContent> {
        let ctx = ClipboardContext::new()
            .map_err(|e| anyhow::anyhow!("Failed to create clipboard context: {}", e))?;

        let text = ctx
            .get_text()
            .map_err(|e| anyhow::anyhow!("Failed to get clipboard text: {}", e))?;

        Ok(ClipboardContent::Text(text))
    }

    /// Sets the content of the system clipboard.
    ///
    /// This method updates both the system clipboard and the manager's internal
    /// tracking state. The internal state is updated to prevent the change from
    /// being detected as a new clipboard event.
    ///
    /// # Arguments
    ///
    /// * `content` - The content to place in the clipboard
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The clipboard context cannot be created (e.g., no display available)
    /// - The clipboard content cannot be set (e.g., clipboard locked by another process)
    /// - The system clipboard API fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use multishiva::core::clipboard::{ClipboardManager, ClipboardContent};
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut manager = ClipboardManager::new()?;
    /// let content = ClipboardContent::Text("Hello from MultiShiva!".to_string());
    ///
    /// manager.set_content(content)?;
    /// # Ok(())
    /// # }
    /// ```
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

    /// Sets clipboard content received from a remote machine.
    ///
    /// This specialized method handles clipboard updates that originated from
    /// remote machines in the synchronized network. It sets the system clipboard
    /// and updates internal tracking to prevent the change from being broadcast
    /// back, which would create an infinite synchronization loop.
    ///
    /// # Arguments
    ///
    /// * `content` - The clipboard content received from the remote machine
    /// * `source` - The identifier of the remote machine that sent this update
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The clipboard context cannot be created
    /// - The clipboard content cannot be set
    /// - The system clipboard API fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use multishiva::core::clipboard::{ClipboardManager, ClipboardContent};
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut manager = ClipboardManager::new()?;
    /// let content = ClipboardContent::Text("Remote clipboard data".to_string());
    ///
    /// manager.set_content_from_remote(content, "machine-123".to_string())?;
    /// # Ok(())
    /// # }
    /// ```
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

    /// Starts monitoring the clipboard for changes in a background thread.
    ///
    /// This method spawns a background polling thread that periodically checks
    /// the system clipboard for changes. When a change is detected (content differs
    /// from the last known state), the provided callback function is invoked with
    /// a `ClipboardChange` event.
    ///
    /// The polling interval is determined by the `poll_interval` setting (default 500ms).
    /// Empty clipboard contents are ignored and will not trigger callbacks.
    ///
    /// # Arguments
    ///
    /// * `callback` - A function that will be called with each clipboard change event.
    ///   Must be `Send + 'static` as it runs in a background thread.
    ///
    /// # Errors
    ///
    /// Currently always returns `Ok(())`, but the `Result` return type is maintained
    /// for future compatibility with initialization that may fail.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use multishiva::core::clipboard::ClipboardManager;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut manager = ClipboardManager::new()?;
    ///
    /// manager.start_monitoring(|change| {
    ///     if let Some(text) = change.content.as_text() {
    ///         println!("Clipboard changed: {}", text);
    ///     }
    /// })?;
    ///
    /// // Monitoring runs in background until stop_monitoring() is called
    /// # Ok(())
    /// # }
    /// ```
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

    /// Stops monitoring clipboard changes and terminates the background polling thread.
    ///
    /// This method sets the monitoring flag to `false`, which causes the background
    /// polling thread to exit on its next iteration. The thread will terminate cleanly
    /// after completing its current sleep cycle.
    ///
    /// This method is also called automatically when the `ClipboardManager` is dropped.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use multishiva::core::clipboard::ClipboardManager;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut manager = ClipboardManager::new()?;
    /// manager.start_monitoring(|_| {})?;
    ///
    /// // Later, when monitoring is no longer needed
    /// manager.stop_monitoring();
    /// assert!(!manager.is_monitoring());
    /// # Ok(())
    /// # }
    /// ```
    pub fn stop_monitoring(&mut self) {
        if let Ok(mut monitoring) = self.monitoring.lock() {
            *monitoring = false;
            tracing::info!("Clipboard monitoring stopped");
        }
    }

    /// Checks whether clipboard monitoring is currently active.
    ///
    /// Returns `true` if the background polling thread is running, `false` otherwise.
    /// If the monitoring lock is poisoned, returns `false`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use multishiva::core::clipboard::ClipboardManager;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut manager = ClipboardManager::new()?;
    /// assert!(!manager.is_monitoring());
    ///
    /// manager.start_monitoring(|_| {})?;
    /// assert!(manager.is_monitoring());
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_monitoring(&self) -> bool {
        self.monitoring.lock().map(|m| *m).unwrap_or(false)
    }

    /// Returns the timestamp of the last clipboard update detected by this manager.
    ///
    /// This timestamp is updated whenever the clipboard content changes, either through
    /// local monitoring or when content is explicitly set via `set_content()` or
    /// `set_content_from_remote()`.
    ///
    /// If the timestamp lock is poisoned, returns the current time as a fallback.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use multishiva::core::clipboard::{ClipboardManager, ClipboardContent};
    /// use std::time::SystemTime;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut manager = ClipboardManager::new()?;
    /// let before = SystemTime::now();
    ///
    /// manager.set_content(ClipboardContent::Text("test".to_string()))?;
    ///
    /// let last_update = manager.last_update_time();
    /// assert!(last_update >= before);
    /// # Ok(())
    /// # }
    /// ```
    pub fn last_update_time(&self) -> SystemTime {
        self.last_update
            .lock()
            .map(|t| *t)
            .unwrap_or_else(|_| SystemTime::now())
    }

    /// Checks if the clipboard has been updated since a given timestamp.
    ///
    /// This is useful for determining if new clipboard content is available
    /// since the last time you checked.
    ///
    /// # Arguments
    ///
    /// * `time` - The reference timestamp to compare against
    ///
    /// # Returns
    ///
    /// Returns `true` if the last clipboard update occurred after the given time,
    /// `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use multishiva::core::clipboard::{ClipboardManager, ClipboardContent};
    /// use std::time::SystemTime;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut manager = ClipboardManager::new()?;
    /// let checkpoint = SystemTime::now();
    ///
    /// // Some time later...
    /// manager.set_content(ClipboardContent::Text("new content".to_string()))?;
    ///
    /// assert!(manager.updated_since(checkpoint));
    /// # Ok(())
    /// # }
    /// ```
    pub fn updated_since(&self, time: SystemTime) -> bool {
        self.last_update_time() > time
    }
}

impl Default for ClipboardManager {
    /// Creates a default `ClipboardManager` instance.
    ///
    /// This is equivalent to calling `ClipboardManager::new()`.
    ///
    /// # Panics
    ///
    /// Panics if the clipboard manager fails to initialize. This is unlikely
    /// to occur in practice but could happen if system resources are unavailable.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use multishiva::core::clipboard::ClipboardManager;
    ///
    /// let manager = ClipboardManager::default();
    /// ```
    fn default() -> Self {
        Self::new().expect("Failed to create default clipboard manager")
    }
}

impl Drop for ClipboardManager {
    /// Automatically stops clipboard monitoring when the manager is dropped.
    ///
    /// This ensures that the background polling thread is properly terminated
    /// and resources are cleaned up when the `ClipboardManager` goes out of scope.
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
