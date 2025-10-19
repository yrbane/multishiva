use anyhow::Result;
use std::collections::HashMap;
use tokio::time::{sleep, Duration};

use crate::core::events::Event;

/// A virtual machine instance for simulation mode.
///
/// Represents a simulated remote machine that can receive events and track state
/// without requiring actual hardware. Each virtual machine has a screen size,
/// cursor position, and event history.
///
/// # Examples
///
/// ```
/// # use multishiva::core::simulation::VirtualMachine;
/// let vm = VirtualMachine::new("desktop-1".to_string(), 1920, 1080);
/// assert_eq!(vm.name(), "desktop-1");
/// assert_eq!(vm.screen_size(), (1920, 1080));
/// ```
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
    /// Creates a new virtual machine with the specified name and screen dimensions.
    ///
    /// The cursor is initialized to the center of the screen.
    ///
    /// # Examples
    ///
    /// ```
    /// # use multishiva::core::simulation::VirtualMachine;
    /// let vm = VirtualMachine::new("test-vm".to_string(), 1920, 1080);
    /// assert_eq!(vm.cursor_position(), (960, 540));
    /// ```
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

    /// Returns the name of this virtual machine.
    ///
    /// # Examples
    ///
    /// ```
    /// # use multishiva::core::simulation::VirtualMachine;
    /// let vm = VirtualMachine::new("my-vm".to_string(), 1920, 1080);
    /// assert_eq!(vm.name(), "my-vm");
    /// ```
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the screen dimensions as (width, height).
    ///
    /// # Examples
    ///
    /// ```
    /// # use multishiva::core::simulation::VirtualMachine;
    /// let vm = VirtualMachine::new("test".to_string(), 1920, 1080);
    /// assert_eq!(vm.screen_size(), (1920, 1080));
    /// ```
    pub fn screen_size(&self) -> (u32, u32) {
        (self.screen_width, self.screen_height)
    }

    /// Returns the current cursor position as (x, y).
    ///
    /// # Examples
    ///
    /// ```
    /// # use multishiva::core::simulation::VirtualMachine;
    /// let mut vm = VirtualMachine::new("test".to_string(), 1920, 1080);
    /// vm.set_cursor_position(100, 200);
    /// assert_eq!(vm.cursor_position(), (100, 200));
    /// ```
    pub fn cursor_position(&self) -> (i32, i32) {
        (self.cursor_x, self.cursor_y)
    }

    /// Sets the cursor position, clamping to screen bounds.
    ///
    /// Coordinates are clamped to the range [0, screen_width] and [0, screen_height].
    ///
    /// # Examples
    ///
    /// ```
    /// # use multishiva::core::simulation::VirtualMachine;
    /// let mut vm = VirtualMachine::new("test".to_string(), 1920, 1080);
    /// vm.set_cursor_position(100, 200);
    /// assert_eq!(vm.cursor_position(), (100, 200));
    ///
    /// // Out of bounds values are clamped
    /// vm.set_cursor_position(5000, -100);
    /// assert_eq!(vm.cursor_position(), (1920, 0));
    /// ```
    pub fn set_cursor_position(&mut self, x: i32, y: i32) {
        // Clamp to screen bounds
        self.cursor_x = x.clamp(0, self.screen_width as i32);
        self.cursor_y = y.clamp(0, self.screen_height as i32);
    }

    /// Injects an event into this virtual machine.
    ///
    /// The event is recorded in the event history and simulated state changes
    /// are applied (e.g., mouse move updates cursor position).
    ///
    /// # Errors
    ///
    /// Currently always returns `Ok(())`, but the signature allows for future
    /// error conditions during event simulation.
    ///
    /// # Examples
    ///
    /// ```
    /// # use multishiva::core::simulation::VirtualMachine;
    /// # use multishiva::core::events::Event;
    /// # tokio_test::block_on(async {
    /// let mut vm = VirtualMachine::new("test".to_string(), 1920, 1080);
    /// let event = Event::MouseMove { x: 500, y: 300 };
    /// vm.inject_event(event).await.unwrap();
    /// assert_eq!(vm.cursor_position(), (500, 300));
    /// assert_eq!(vm.recorded_events().len(), 1);
    /// # });
    /// ```
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

    /// Returns a slice of all recorded events.
    ///
    /// Events are stored in the order they were injected.
    ///
    /// # Examples
    ///
    /// ```
    /// # use multishiva::core::simulation::VirtualMachine;
    /// # use multishiva::core::events::Event;
    /// # tokio_test::block_on(async {
    /// let mut vm = VirtualMachine::new("test".to_string(), 1920, 1080);
    /// vm.inject_event(Event::MouseMove { x: 10, y: 20 }).await.unwrap();
    /// vm.inject_event(Event::Heartbeat).await.unwrap();
    /// assert_eq!(vm.recorded_events().len(), 2);
    /// # });
    /// ```
    pub fn recorded_events(&self) -> &[Event] {
        &self.recorded_events
    }

    /// Clears all recorded events from the history.
    ///
    /// # Examples
    ///
    /// ```
    /// # use multishiva::core::simulation::VirtualMachine;
    /// # use multishiva::core::events::Event;
    /// # tokio_test::block_on(async {
    /// let mut vm = VirtualMachine::new("test".to_string(), 1920, 1080);
    /// vm.inject_event(Event::Heartbeat).await.unwrap();
    /// assert_eq!(vm.recorded_events().len(), 1);
    /// vm.clear_events();
    /// assert_eq!(vm.recorded_events().len(), 0);
    /// # });
    /// ```
    pub fn clear_events(&mut self) {
        self.recorded_events.clear();
    }
}

/// The main simulation mode controller.
///
/// Manages multiple virtual machines and simulates network behavior including
/// latency. Provides statistics tracking for events sent during simulation.
///
/// # Examples
///
/// ```
/// # use multishiva::core::simulation::SimulationMode;
/// # use multishiva::core::events::Event;
/// # tokio_test::block_on(async {
/// let mut sim = SimulationMode::new();
/// sim.add_virtual_machine("vm1".to_string(), 1920, 1080);
/// sim.set_network_latency(10);
///
/// sim.send_event_to("vm1", Event::Heartbeat).await.unwrap();
/// assert_eq!(sim.get_statistics().total_events_sent, 1);
/// # });
/// ```
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
    /// Creates a new simulation mode instance with no virtual machines.
    ///
    /// # Examples
    ///
    /// ```
    /// # use multishiva::core::simulation::SimulationMode;
    /// let sim = SimulationMode::new();
    /// assert_eq!(sim.virtual_machine_count(), 0);
    /// ```
    pub fn new() -> Self {
        Self {
            virtual_machines: HashMap::new(),
            network_latency_ms: 0,
            total_events_sent: 0,
        }
    }

    /// Adds a new virtual machine to the simulation.
    ///
    /// If a virtual machine with the same name already exists, it will be replaced.
    ///
    /// # Examples
    ///
    /// ```
    /// # use multishiva::core::simulation::SimulationMode;
    /// let mut sim = SimulationMode::new();
    /// sim.add_virtual_machine("desktop-1".to_string(), 1920, 1080);
    /// assert_eq!(sim.virtual_machine_count(), 1);
    /// ```
    pub fn add_virtual_machine(&mut self, name: String, width: u32, height: u32) {
        let vm = VirtualMachine::new(name.clone(), width, height);
        self.virtual_machines.insert(name, vm);
    }

    /// Removes a virtual machine from the simulation.
    ///
    /// Does nothing if the virtual machine does not exist.
    ///
    /// # Examples
    ///
    /// ```
    /// # use multishiva::core::simulation::SimulationMode;
    /// let mut sim = SimulationMode::new();
    /// sim.add_virtual_machine("vm1".to_string(), 1920, 1080);
    /// sim.remove_virtual_machine("vm1");
    /// assert_eq!(sim.virtual_machine_count(), 0);
    /// ```
    pub fn remove_virtual_machine(&mut self, name: &str) {
        self.virtual_machines.remove(name);
    }

    /// Gets an immutable reference to a virtual machine by name.
    ///
    /// Returns `None` if the virtual machine does not exist.
    ///
    /// # Examples
    ///
    /// ```
    /// # use multishiva::core::simulation::SimulationMode;
    /// let mut sim = SimulationMode::new();
    /// sim.add_virtual_machine("vm1".to_string(), 1920, 1080);
    /// let vm = sim.get_virtual_machine("vm1").unwrap();
    /// assert_eq!(vm.name(), "vm1");
    /// ```
    pub fn get_virtual_machine(&self, name: &str) -> Option<&VirtualMachine> {
        self.virtual_machines.get(name)
    }

    /// Gets a mutable reference to a virtual machine by name.
    ///
    /// Returns `None` if the virtual machine does not exist.
    ///
    /// # Examples
    ///
    /// ```
    /// # use multishiva::core::simulation::SimulationMode;
    /// let mut sim = SimulationMode::new();
    /// sim.add_virtual_machine("vm1".to_string(), 1920, 1080);
    /// if let Some(vm) = sim.get_virtual_machine_mut("vm1") {
    ///     vm.set_cursor_position(100, 200);
    /// }
    /// ```
    pub fn get_virtual_machine_mut(&mut self, name: &str) -> Option<&mut VirtualMachine> {
        self.virtual_machines.get_mut(name)
    }

    /// Returns the number of virtual machines in the simulation.
    ///
    /// # Examples
    ///
    /// ```
    /// # use multishiva::core::simulation::SimulationMode;
    /// let mut sim = SimulationMode::new();
    /// assert_eq!(sim.virtual_machine_count(), 0);
    /// sim.add_virtual_machine("vm1".to_string(), 1920, 1080);
    /// assert_eq!(sim.virtual_machine_count(), 1);
    /// ```
    pub fn virtual_machine_count(&self) -> usize {
        self.virtual_machines.len()
    }

    /// Sets the simulated network latency in milliseconds.
    ///
    /// This latency is applied as a delay when sending events to virtual machines.
    ///
    /// # Examples
    ///
    /// ```
    /// # use multishiva::core::simulation::SimulationMode;
    /// let mut sim = SimulationMode::new();
    /// sim.set_network_latency(50); // 50ms latency
    /// ```
    pub fn set_network_latency(&mut self, latency_ms: u64) {
        self.network_latency_ms = latency_ms;
    }

    /// Sends an event to a target virtual machine with simulated network latency.
    ///
    /// The event is delivered after waiting for the configured network latency.
    /// Increments the total events sent counter on success.
    ///
    /// # Errors
    ///
    /// Returns an error if the target virtual machine does not exist.
    ///
    /// # Examples
    ///
    /// ```
    /// # use multishiva::core::simulation::SimulationMode;
    /// # use multishiva::core::events::Event;
    /// # tokio_test::block_on(async {
    /// let mut sim = SimulationMode::new();
    /// sim.add_virtual_machine("vm1".to_string(), 1920, 1080);
    /// sim.set_network_latency(10);
    ///
    /// sim.send_event_to("vm1", Event::Heartbeat).await.unwrap();
    /// assert_eq!(sim.get_statistics().total_events_sent, 1);
    ///
    /// // Sending to non-existent VM returns error
    /// assert!(sim.send_event_to("vm2", Event::Heartbeat).await.is_err());
    /// # });
    /// ```
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

    /// Returns simulation statistics.
    ///
    /// # Examples
    ///
    /// ```
    /// # use multishiva::core::simulation::SimulationMode;
    /// # use multishiva::core::events::Event;
    /// # tokio_test::block_on(async {
    /// let mut sim = SimulationMode::new();
    /// sim.add_virtual_machine("vm1".to_string(), 1920, 1080);
    /// sim.send_event_to("vm1", Event::Heartbeat).await.unwrap();
    ///
    /// let stats = sim.get_statistics();
    /// assert_eq!(stats.total_events_sent, 1);
    /// assert_eq!(stats.virtual_machine_count, 1);
    /// # });
    /// ```
    pub fn get_statistics(&self) -> SimulationStatistics {
        SimulationStatistics {
            total_events_sent: self.total_events_sent,
            virtual_machine_count: self.virtual_machines.len(),
        }
    }
}

/// Statistics about simulation activity.
///
/// Contains counters and metrics about events sent and virtual machines
/// in the simulation.
///
/// # Examples
///
/// ```
/// # use multishiva::core::simulation::{SimulationMode, SimulationStatistics};
/// # use multishiva::core::events::Event;
/// # tokio_test::block_on(async {
/// let mut sim = SimulationMode::new();
/// sim.add_virtual_machine("vm1".to_string(), 1920, 1080);
/// sim.send_event_to("vm1", Event::Heartbeat).await.unwrap();
///
/// let stats = sim.get_statistics();
/// assert_eq!(stats.total_events_sent, 1);
/// assert_eq!(stats.virtual_machine_count, 1);
/// # });
/// ```
#[derive(Debug, Clone)]
pub struct SimulationStatistics {
    /// Total number of events successfully sent to virtual machines.
    pub total_events_sent: usize,
    /// Current number of virtual machines in the simulation.
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
