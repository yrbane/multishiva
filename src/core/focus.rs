use anyhow::Result;
use tokio::time::{sleep, Duration};

pub struct FocusManager {
    current_focus: String,
    host_machine: String,
    current_position: (i32, i32),
    focus_history: Vec<String>,
    friction_ms: u64,
}

impl FocusManager {
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

    pub async fn return_to_host(&mut self) -> Result<()> {
        self.transfer_focus(self.host_machine.clone(), 0, 0).await
    }

    pub fn current(&self) -> &str {
        &self.current_focus
    }

    pub fn has_focus(&self, machine: &str) -> bool {
        self.current_focus == machine
    }

    pub fn current_position(&self) -> (i32, i32) {
        self.current_position
    }

    pub fn focus_history(&self) -> &[String] {
        &self.focus_history
    }

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
