use anyhow::Result;
use std::collections::HashMap;
use tokio::time::{sleep, Duration};

use crate::core::events::Event;

#[derive(Debug, Clone)]
pub struct VirtualMachine {
    name: String,
    screen_width: u32,
    screen_height: u32,
    cursor_x: i32,
    cursor_y: i32,
    recorded_events: Vec<Event>,
}

impl VirtualMachine {
    pub fn new(name: String, screen_width: u32, screen_height: u32) -> Self {
        Self {
            name: name.clone(),
            screen_width,
            screen_height,
            cursor_x: (screen_width / 2) as i32,
            cursor_y: (screen_height / 2) as i32,
            recorded_events: Vec::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn screen_size(&self) -> (u32, u32) {
        (self.screen_width, self.screen_height)
    }

    pub fn cursor_position(&self) -> (i32, i32) {
        (self.cursor_x, self.cursor_y)
    }

    pub fn set_cursor_position(&mut self, x: i32, y: i32) {
        // Clamp to screen bounds
        self.cursor_x = x.clamp(0, self.screen_width as i32);
        self.cursor_y = y.clamp(0, self.screen_height as i32);
    }

    pub async fn inject_event(&mut self, event: Event) -> Result<()> {
        // Record the event
        self.recorded_events.push(event.clone());

        // Simulate the event
        match event {
            Event::MouseMove { x, y } => {
                self.set_cursor_position(x, y);
            }
            Event::MouseButtonPress { .. }
            | Event::MouseButtonRelease { .. }
            | Event::MouseClick { .. }
            | Event::MouseScroll { .. }
            | Event::KeyPress { .. }
            | Event::KeyRelease { .. }
            | Event::FocusGrant { .. }
            | Event::FocusRelease
            | Event::Heartbeat => {
                // Just record these events, no state change needed for simulation
            }
        }

        Ok(())
    }

    pub fn recorded_events(&self) -> &[Event] {
        &self.recorded_events
    }

    pub fn clear_events(&mut self) {
        self.recorded_events.clear();
    }
}

pub struct SimulationMode {
    virtual_machines: HashMap<String, VirtualMachine>,
    network_latency_ms: u64,
    total_events_sent: usize,
}

impl Default for SimulationMode {
    fn default() -> Self {
        Self::new()
    }
}

impl SimulationMode {
    pub fn new() -> Self {
        Self {
            virtual_machines: HashMap::new(),
            network_latency_ms: 0,
            total_events_sent: 0,
        }
    }

    pub fn add_virtual_machine(&mut self, name: String, width: u32, height: u32) {
        let vm = VirtualMachine::new(name.clone(), width, height);
        self.virtual_machines.insert(name, vm);
    }

    pub fn remove_virtual_machine(&mut self, name: &str) {
        self.virtual_machines.remove(name);
    }

    pub fn get_virtual_machine(&self, name: &str) -> Option<&VirtualMachine> {
        self.virtual_machines.get(name)
    }

    pub fn get_virtual_machine_mut(&mut self, name: &str) -> Option<&mut VirtualMachine> {
        self.virtual_machines.get_mut(name)
    }

    pub fn virtual_machine_count(&self) -> usize {
        self.virtual_machines.len()
    }

    pub fn set_network_latency(&mut self, latency_ms: u64) {
        self.network_latency_ms = latency_ms;
    }

    pub async fn send_event_to(&mut self, target: &str, event: Event) -> Result<()> {
        // Simulate network latency
        if self.network_latency_ms > 0 {
            sleep(Duration::from_millis(self.network_latency_ms)).await;
        }

        // Send event to target VM
        if let Some(vm) = self.virtual_machines.get_mut(target) {
            vm.inject_event(event).await?;
            self.total_events_sent += 1;
        } else {
            anyhow::bail!("Virtual machine '{}' not found", target);
        }

        Ok(())
    }

    pub fn get_statistics(&self) -> SimulationStatistics {
        SimulationStatistics {
            total_events_sent: self.total_events_sent,
            virtual_machine_count: self.virtual_machines.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SimulationStatistics {
    pub total_events_sent: usize,
    pub virtual_machine_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtual_machine_creation() {
        let vm = VirtualMachine::new("test".to_string(), 1920, 1080);
        assert_eq!(vm.name(), "test");
        assert_eq!(vm.screen_size(), (1920, 1080));
    }

    #[test]
    fn test_simulation_mode_creation() {
        let sim = SimulationMode::new();
        assert_eq!(sim.virtual_machine_count(), 0);
    }
}
