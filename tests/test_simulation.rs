use multishiva::core::events::Event;
use multishiva::core::simulation::{SimulationMode, VirtualMachine};
use tokio::time::Duration;

#[tokio::test]
async fn test_virtual_machine_creation() {
    let vm = VirtualMachine::new("test-vm".to_string(), 1920, 1080);
    assert_eq!(vm.name(), "test-vm");
    assert_eq!(vm.screen_size(), (1920, 1080));
}

#[tokio::test]
async fn test_virtual_machine_cursor_position() {
    let mut vm = VirtualMachine::new("test-vm".to_string(), 1920, 1080);

    // Initial position should be center
    let (x, y) = vm.cursor_position();
    assert_eq!(x, 960);
    assert_eq!(y, 540);

    // Move cursor
    vm.set_cursor_position(100, 200);
    let (x, y) = vm.cursor_position();
    assert_eq!(x, 100);
    assert_eq!(y, 200);
}

#[tokio::test]
async fn test_virtual_machine_event_injection() {
    let mut vm = VirtualMachine::new("test-vm".to_string(), 1920, 1080);

    // Inject mouse move event
    let event = Event::MouseMove { x: 500, y: 300 };
    let result = vm.inject_event(event).await;
    assert!(result.is_ok());

    // Cursor should have moved
    let (x, y) = vm.cursor_position();
    assert_eq!(x, 500);
    assert_eq!(y, 300);
}

#[tokio::test]
async fn test_virtual_machine_event_recording() {
    let mut vm = VirtualMachine::new("test-vm".to_string(), 1920, 1080);

    // Inject several events
    vm.inject_event(Event::MouseMove { x: 100, y: 100 })
        .await
        .unwrap();
    vm.inject_event(Event::MouseMove { x: 200, y: 200 })
        .await
        .unwrap();
    vm.inject_event(Event::MouseMove { x: 300, y: 300 })
        .await
        .unwrap();

    // Should have recorded all events
    let events = vm.recorded_events();
    assert_eq!(events.len(), 3);
}

#[tokio::test]
async fn test_simulation_mode_creation() {
    let sim = SimulationMode::new();
    assert_eq!(sim.virtual_machine_count(), 0);
}

#[tokio::test]
async fn test_simulation_mode_add_virtual_machine() {
    let mut sim = SimulationMode::new();

    sim.add_virtual_machine("vm1".to_string(), 1920, 1080);
    sim.add_virtual_machine("vm2".to_string(), 1920, 1080);

    assert_eq!(sim.virtual_machine_count(), 2);
}

#[tokio::test]
async fn test_simulation_mode_get_virtual_machine() {
    let mut sim = SimulationMode::new();

    sim.add_virtual_machine("vm1".to_string(), 1920, 1080);

    let vm = sim.get_virtual_machine("vm1");
    assert!(vm.is_some());
    assert_eq!(vm.unwrap().name(), "vm1");

    let vm = sim.get_virtual_machine("nonexistent");
    assert!(vm.is_none());
}

#[tokio::test]
async fn test_simulation_mode_send_event_to_vm() {
    let mut sim = SimulationMode::new();

    sim.add_virtual_machine("vm1".to_string(), 1920, 1080);

    let event = Event::MouseMove { x: 400, y: 500 };
    let result = sim.send_event_to("vm1", event).await;
    assert!(result.is_ok());

    let vm = sim.get_virtual_machine("vm1").unwrap();
    let (x, y) = vm.cursor_position();
    assert_eq!(x, 400);
    assert_eq!(y, 500);
}

#[tokio::test]
async fn test_simulation_mode_event_routing() {
    let mut sim = SimulationMode::new();

    sim.add_virtual_machine("vm1".to_string(), 1920, 1080);
    sim.add_virtual_machine("vm2".to_string(), 1920, 1080);

    // Send events to different VMs
    sim.send_event_to("vm1", Event::MouseMove { x: 100, y: 100 })
        .await
        .unwrap();
    sim.send_event_to("vm2", Event::MouseMove { x: 200, y: 200 })
        .await
        .unwrap();

    // Each VM should have its own cursor position
    let vm1 = sim.get_virtual_machine("vm1").unwrap();
    let (x1, y1) = vm1.cursor_position();
    assert_eq!(x1, 100);
    assert_eq!(y1, 100);

    let vm2 = sim.get_virtual_machine("vm2").unwrap();
    let (x2, y2) = vm2.cursor_position();
    assert_eq!(x2, 200);
    assert_eq!(y2, 200);
}

#[tokio::test]
async fn test_simulation_mode_replay_events() {
    let mut sim = SimulationMode::new();

    sim.add_virtual_machine("vm1".to_string(), 1920, 1080);

    // Record events
    sim.send_event_to("vm1", Event::MouseMove { x: 100, y: 100 })
        .await
        .unwrap();
    sim.send_event_to("vm1", Event::MouseMove { x: 200, y: 200 })
        .await
        .unwrap();
    sim.send_event_to("vm1", Event::MouseMove { x: 300, y: 300 })
        .await
        .unwrap();

    let vm = sim.get_virtual_machine("vm1").unwrap();
    let events = vm.recorded_events();
    assert_eq!(events.len(), 3);
}

#[tokio::test]
async fn test_simulation_mode_clear_events() {
    let mut sim = SimulationMode::new();

    sim.add_virtual_machine("vm1".to_string(), 1920, 1080);

    // Record events
    sim.send_event_to("vm1", Event::MouseMove { x: 100, y: 100 })
        .await
        .unwrap();
    sim.send_event_to("vm1", Event::MouseMove { x: 200, y: 200 })
        .await
        .unwrap();

    // Clear events
    let vm = sim.get_virtual_machine_mut("vm1").unwrap();
    vm.clear_events();

    let events = vm.recorded_events();
    assert_eq!(events.len(), 0);
}

#[tokio::test]
async fn test_simulation_mode_latency_simulation() {
    let mut sim = SimulationMode::new();
    sim.set_network_latency(50); // 50ms latency

    sim.add_virtual_machine("vm1".to_string(), 1920, 1080);

    let start = std::time::Instant::now();
    sim.send_event_to("vm1", Event::MouseMove { x: 100, y: 100 })
        .await
        .unwrap();
    let elapsed = start.elapsed();

    // Should have at least the latency delay
    assert!(elapsed >= Duration::from_millis(50));
}

#[tokio::test]
async fn test_virtual_machine_bounds_checking() {
    let mut vm = VirtualMachine::new("test-vm".to_string(), 800, 600);

    // Try to move cursor outside bounds
    vm.set_cursor_position(1000, 700);

    // Should clamp to screen bounds
    let (x, y) = vm.cursor_position();
    assert!(x <= 800);
    assert!(y <= 600);
}

#[tokio::test]
async fn test_simulation_mode_remove_vm() {
    let mut sim = SimulationMode::new();

    sim.add_virtual_machine("vm1".to_string(), 1920, 1080);
    assert_eq!(sim.virtual_machine_count(), 1);

    sim.remove_virtual_machine("vm1");
    assert_eq!(sim.virtual_machine_count(), 0);
}

#[tokio::test]
async fn test_simulation_mode_statistics() {
    let mut sim = SimulationMode::new();

    sim.add_virtual_machine("vm1".to_string(), 1920, 1080);

    // Send multiple events
    for i in 0..10 {
        sim.send_event_to(
            "vm1",
            Event::MouseMove {
                x: i * 10,
                y: i * 10,
            },
        )
        .await
        .unwrap();
    }

    let stats = sim.get_statistics();
    assert_eq!(stats.total_events_sent, 10);
}
