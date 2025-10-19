use anyhow::Result;

pub struct FocusManager {
    current_focus: String,
}

impl FocusManager {
    pub fn new(initial_focus: String) -> Self {
        Self {
            current_focus: initial_focus,
        }
    }

    pub async fn transfer_focus(&mut self, target: String) -> Result<()> {
        self.current_focus = target;
        Ok(())
    }

    pub fn current(&self) -> &str {
        &self.current_focus
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
