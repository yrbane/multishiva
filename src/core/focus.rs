use anyhow::Result;
use tokio::time::{sleep, Duration};

/// Manages focus state across multiple machines in a multi-monitor setup.
///
/// The `FocusManager` tracks which machine currently has focus, maintains a history
/// of focus transitions, and supports configurable friction delays to prevent rapid
/// focus switching. It also tracks cursor position at the time of focus transfer.
///
/// # Examples
///
/// ```
/// use multishiva::core::focus::FocusManager;
///
/// let mut manager = FocusManager::new("primary".to_string());
/// assert_eq!(manager.current(), "primary");
/// assert!(manager.has_focus("primary"));
/// ```
pub struct FocusManager {
    current_focus: String,
    host_machine: String,
    current_position: (i32, i32),
    focus_history: Vec<String>,
    friction_ms: u64,
}

impl FocusManager {
    /// Creates a new `FocusManager` with the specified initial focus target.
    ///
    /// The initial focus target is set as both the current focus and the host machine.
    /// The cursor position is initialized to (0, 0), and friction delay is disabled (0ms).
    ///
    /// # Arguments
    ///
    /// * `initial_focus` - The name of the machine that initially has focus
    ///
    /// # Examples
    ///
    /// ```
    /// use multishiva::core::focus::FocusManager;
    ///
    /// let manager = FocusManager::new("laptop".to_string());
    /// assert_eq!(manager.current(), "laptop");
    /// assert_eq!(manager.current_position(), (0, 0));
    /// ```
    pub fn new(initial_focus: String) -> Self {
        let host = initial_focus.clone();
        Self {
            current_focus: initial_focus.clone(),
            host_machine: host,
            current_position: (0, 0),
            focus_history: vec![initial_focus],
            friction_ms: 0,
        }
    }

    /// Transfers focus to the specified target machine at the given cursor position.
    ///
    /// If the target machine already has focus, this is a no-op. Otherwise, focus is
    /// transferred to the target, the cursor position is updated, and the target is
    /// added to the focus history. If friction delay is configured, it will sleep
    /// for the specified duration before completing the transfer.
    ///
    /// # Arguments
    ///
    /// * `target` - The name of the machine to transfer focus to
    /// * `x` - The x-coordinate of the cursor position
    /// * `y` - The y-coordinate of the cursor position
    ///
    /// # Examples
    ///
    /// ```
    /// use multishiva::core::focus::FocusManager;
    ///
    /// # tokio_test::block_on(async {
    /// let mut manager = FocusManager::new("primary".to_string());
    /// manager.transfer_focus("secondary".to_string(), 100, 200).await.unwrap();
    /// assert_eq!(manager.current(), "secondary");
    /// assert_eq!(manager.current_position(), (100, 200));
    /// # });
    /// ```
    ///
    /// # Errors
    ///
    /// Currently always returns `Ok(())`, but the `Result` type is used for future
    /// extensibility where focus transfer operations might fail.
    pub async fn transfer_focus(&mut self, target: String, x: i32, y: i32) -> Result<()> {
        // Don't transfer if already at target
        if self.current_focus == target {
            return Ok(());
        }

        // Apply friction delay if configured
        if self.friction_ms > 0 {
            sleep(Duration::from_millis(self.friction_ms)).await;
        }

        self.current_focus = target.clone();
        self.current_position = (x, y);
        self.focus_history.push(target);

        Ok(())
    }

    /// Returns focus to the host machine.
    ///
    /// This is a convenience method that transfers focus back to the machine that was
    /// initially designated as the host (the machine specified in `new()`). The cursor
    /// position is reset to (0, 0).
    ///
    /// # Examples
    ///
    /// ```
    /// use multishiva::core::focus::FocusManager;
    ///
    /// # tokio_test::block_on(async {
    /// let mut manager = FocusManager::new("primary".to_string());
    /// manager.transfer_focus("secondary".to_string(), 100, 200).await.unwrap();
    /// manager.return_to_host().await.unwrap();
    /// assert_eq!(manager.current(), "primary");
    /// assert_eq!(manager.current_position(), (0, 0));
    /// # });
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying `transfer_focus` operation fails.
    pub async fn return_to_host(&mut self) -> Result<()> {
        self.transfer_focus(self.host_machine.clone(), 0, 0).await
    }

    /// Returns the name of the machine that currently has focus.
    ///
    /// # Examples
    ///
    /// ```
    /// use multishiva::core::focus::FocusManager;
    ///
    /// let manager = FocusManager::new("workstation".to_string());
    /// assert_eq!(manager.current(), "workstation");
    /// ```
    pub fn current(&self) -> &str {
        &self.current_focus
    }

    /// Checks whether the specified machine currently has focus.
    ///
    /// # Arguments
    ///
    /// * `machine` - The name of the machine to check
    ///
    /// # Returns
    ///
    /// Returns `true` if the specified machine has focus, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use multishiva::core::focus::FocusManager;
    ///
    /// let manager = FocusManager::new("desktop".to_string());
    /// assert!(manager.has_focus("desktop"));
    /// assert!(!manager.has_focus("laptop"));
    /// ```
    pub fn has_focus(&self, machine: &str) -> bool {
        self.current_focus == machine
    }

    /// Returns the current cursor position as a tuple of (x, y) coordinates.
    ///
    /// The position represents the cursor coordinates at the time of the last focus
    /// transfer operation.
    ///
    /// # Examples
    ///
    /// ```
    /// use multishiva::core::focus::FocusManager;
    ///
    /// # tokio_test::block_on(async {
    /// let mut manager = FocusManager::new("main".to_string());
    /// manager.transfer_focus("other".to_string(), 250, 500).await.unwrap();
    /// assert_eq!(manager.current_position(), (250, 500));
    /// # });
    /// ```
    pub fn current_position(&self) -> (i32, i32) {
        self.current_position
    }

    /// Returns a slice containing the complete focus history.
    ///
    /// The history is ordered chronologically, with the initial focus target first
    /// and the most recent focus target last. Each focus transfer appends the target
    /// machine name to the history.
    ///
    /// # Examples
    ///
    /// ```
    /// use multishiva::core::focus::FocusManager;
    ///
    /// # tokio_test::block_on(async {
    /// let mut manager = FocusManager::new("machine1".to_string());
    /// manager.transfer_focus("machine2".to_string(), 0, 0).await.unwrap();
    /// manager.transfer_focus("machine3".to_string(), 0, 0).await.unwrap();
    ///
    /// let history = manager.focus_history();
    /// assert_eq!(history.len(), 3);
    /// assert_eq!(history[0], "machine1");
    /// assert_eq!(history[1], "machine2");
    /// assert_eq!(history[2], "machine3");
    /// # });
    /// ```
    pub fn focus_history(&self) -> &[String] {
        &self.focus_history
    }

    /// Sets the friction delay in milliseconds.
    ///
    /// Friction delay introduces an intentional pause before completing focus transfers,
    /// which can help prevent accidental or rapid focus switching. When set to 0 (default),
    /// no delay is applied. When set to a positive value, `transfer_focus` will sleep for
    /// the specified duration before completing the operation.
    ///
    /// # Arguments
    ///
    /// * `ms` - The friction delay in milliseconds
    ///
    /// # Examples
    ///
    /// ```
    /// use multishiva::core::focus::FocusManager;
    ///
    /// let mut manager = FocusManager::new("primary".to_string());
    /// manager.set_friction_ms(100); // 100ms delay before focus transfers
    /// ```
    pub fn set_friction_ms(&mut self, ms: u64) {
        self.friction_ms = ms;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_focus_manager_creation() {
        let manager = FocusManager::new("host".to_string());
        assert_eq!(manager.current(), "host");
    }
}
