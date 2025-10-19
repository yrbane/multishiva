use multishiva::core::config::{Config, ConfigMode};
use multishiva::core::events::{Event, Key, MouseButton};
use multishiva::core::focus::FocusManager;
use multishiva::core::network::Network;
use multishiva::core::simulation::SimulationMode;
use multishiva::core::topology::{Edge, Position, Topology};
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_integration_host_agent_communication() {
    // Setup: Create host and agent networks
    let mut host_network = Network::new("shared-psk".to_string());
    let mut agent_network = Network::new("shared-psk".to_string());

    // Start host
    let port = host_network.start_host(0).await.unwrap();
    sleep(Duration::from_millis(100)).await;

    // Connect agent
    agent_network
        .connect_to_host(&format!("127.0.0.1:{}", port))
        .await
        .unwrap();

    // Send event from agent to host
    let event = Event::MouseMove { x: 500, y: 300 };
    agent_network.send_event(event).await.unwrap();

    // Cleanup
    host_network.stop().await;
    agent_network.stop().await;
}

#[tokio::test]
async fn test_integration_focus_transfer_with_topology() {
    // Create topology with two machines
    let mut topology = Topology::new();
    topology.add_machine("host".to_string(), Position { x: 0, y: 0 });
    topology.add_machine("agent1".to_string(), Position { x: 1920, y: 0 });
    topology.add_edge("host".to_string(), Edge::Right, "agent1".to_string());

    // Create focus manager
    let mut focus = FocusManager::new("host".to_string());

    // Simulate mouse at right edge of host screen
    let edge = topology.detect_edge("host", 1910, 500, 1920, 10);
    assert_eq!(edge, Some(Edge::Right));

    // Get neighbor
    let neighbor = topology.get_neighbor("host", &Edge::Right);
    assert_eq!(neighbor, Some(&"agent1".to_string()));

    // Transfer focus
    focus
        .transfer_focus("agent1".to_string(), 0, 500)
        .await
        .unwrap();
    assert_eq!(focus.current(), "agent1");
}

#[tokio::test]
async fn test_integration_simulation_mode_full_scenario() {
    // Create simulation with multiple VMs
    let mut sim = SimulationMode::new();
    sim.add_virtual_machine("host".to_string(), 1920, 1080);
    sim.add_virtual_machine("agent1".to_string(), 1920, 1080);
    sim.add_virtual_machine("agent2".to_string(), 1920, 1080);

    // Create topology
    let mut topology = Topology::new();
    topology.add_machine("host".to_string(), Position { x: 0, y: 0 });
    topology.add_machine("agent1".to_string(), Position { x: 1920, y: 0 });
    topology.add_machine("agent2".to_string(), Position { x: 3840, y: 0 });
    topology.add_edge("host".to_string(), Edge::Right, "agent1".to_string());
    topology.add_edge("agent1".to_string(), Edge::Right, "agent2".to_string());

    // Simulate workflow: move mouse across machines on host (at right edge)
    sim.send_event_to("host", Event::MouseMove { x: 1910, y: 500 })
        .await
        .unwrap();

    // Verify host cursor position
    let host = sim.get_virtual_machine("host").unwrap();
    assert_eq!(host.cursor_position(), (1910, 500));

    // Detect edge crossing (x=1910 is at edge with threshold=10)
    let edge = topology.detect_edge("host", 1910, 500, 1920, 10);
    assert_eq!(edge, Some(Edge::Right));

    // Get neighbor and transfer to agent1
    let neighbor = topology.get_neighbor("host", &Edge::Right).unwrap();
    assert_eq!(neighbor, &"agent1".to_string());

    sim.send_event_to(neighbor, Event::MouseMove { x: 0, y: 500 })
        .await
        .unwrap();

    // Verify agent1 received event
    let agent1 = sim.get_virtual_machine("agent1").unwrap();
    assert_eq!(agent1.cursor_position(), (0, 500));
}

#[tokio::test]
async fn test_integration_config_to_topology() {
    // Create a config programmatically (simulating loaded YAML)
    let config = Config {
        self_name: "host".to_string(),
        mode: ConfigMode::Host,
        port: 53421,
        tls: multishiva::core::config::TlsConfig {
            psk: "test-psk".to_string(),
        },
        edges: {
            let mut edges = std::collections::HashMap::new();
            edges.insert("right".to_string(), "agent1".to_string());
            edges
        },
        hotkeys: None,
        behavior: None,
    };

    // Validate config
    config.validate().unwrap();

    // Convert config edges to topology
    let mut topology = Topology::new();
    topology.add_machine(config.self_name.clone(), Position { x: 0, y: 0 });

    for (direction, target) in &config.edges {
        let edge = match direction.as_str() {
            "right" => Edge::Right,
            "left" => Edge::Left,
            "top" => Edge::Top,
            "bottom" => Edge::Bottom,
            _ => continue,
        };
        topology.add_edge(config.self_name.clone(), edge, target.clone());
    }

    // Verify topology
    let neighbor = topology.get_neighbor(&config.self_name, &Edge::Right);
    assert_eq!(neighbor, Some(&"agent1".to_string()));
}

#[tokio::test]
async fn test_integration_multiple_agents_with_focus() {
    // Create networks
    let mut host_network = Network::new("shared-psk".to_string());
    let mut agent1_network = Network::new("shared-psk".to_string());
    let mut agent2_network = Network::new("shared-psk".to_string());

    // Start host
    let port = host_network.start_host(0).await.unwrap();
    sleep(Duration::from_millis(100)).await;

    // Connect agents
    agent1_network
        .connect_to_host(&format!("127.0.0.1:{}", port))
        .await
        .unwrap();
    agent2_network
        .connect_to_host(&format!("127.0.0.1:{}", port))
        .await
        .unwrap();

    // Create focus manager
    let mut focus = FocusManager::new("host".to_string());

    // Transfer focus to agent1
    focus
        .transfer_focus("agent1".to_string(), 0, 0)
        .await
        .unwrap();
    assert_eq!(focus.current(), "agent1");

    // Transfer focus to agent2
    focus
        .transfer_focus("agent2".to_string(), 0, 0)
        .await
        .unwrap();
    assert_eq!(focus.current(), "agent2");

    // Return to host
    focus.return_to_host().await.unwrap();
    assert_eq!(focus.current(), "host");

    // Cleanup
    host_network.stop().await;
    agent1_network.stop().await;
    agent2_network.stop().await;
}

#[tokio::test]
async fn test_integration_event_serialization_over_network() {
    // Create networks
    let mut host_network = Network::new("shared-psk".to_string());
    let mut agent_network = Network::new("shared-psk".to_string());

    // Start host
    let port = host_network.start_host(0).await.unwrap();
    sleep(Duration::from_millis(100)).await;

    // Connect agent
    agent_network
        .connect_to_host(&format!("127.0.0.1:{}", port))
        .await
        .unwrap();

    // Send various event types
    let events = vec![
        Event::MouseMove { x: 100, y: 200 },
        Event::MouseButtonPress {
            button: MouseButton::Left,
        },
        Event::KeyPress { key: Key::KeyA },
        Event::FocusGrant {
            target: "agent1".to_string(),
            x: 50,
            y: 100,
        },
        Event::Heartbeat,
    ];

    for event in events {
        agent_network.send_event(event).await.unwrap();
    }

    // Cleanup
    host_network.stop().await;
    agent_network.stop().await;
}

#[tokio::test]
async fn test_integration_topology_edge_detection_all_sides() {
    let mut topology = Topology::new();
    topology.add_machine("center".to_string(), Position { x: 1920, y: 1080 });
    topology.add_edge(
        "center".to_string(),
        Edge::Right,
        "right_machine".to_string(),
    );
    topology.add_edge("center".to_string(), Edge::Left, "left_machine".to_string());
    topology.add_edge("center".to_string(), Edge::Top, "top_machine".to_string());
    topology.add_edge(
        "center".to_string(),
        Edge::Bottom,
        "bottom_machine".to_string(),
    );

    // Test right edge
    let edge = topology.detect_edge("center", 1910, 500, 1920, 10);
    assert_eq!(edge, Some(Edge::Right));
    assert_eq!(
        topology.get_neighbor("center", &Edge::Right),
        Some(&"right_machine".to_string())
    );

    // Test left edge
    let edge = topology.detect_edge("center", 5, 500, 1920, 10);
    assert_eq!(edge, Some(Edge::Left));
    assert_eq!(
        topology.get_neighbor("center", &Edge::Left),
        Some(&"left_machine".to_string())
    );

    // Test top edge
    let edge = topology.detect_edge("center", 960, 5, 1920, 10);
    assert_eq!(edge, Some(Edge::Top));
    assert_eq!(
        topology.get_neighbor("center", &Edge::Top),
        Some(&"top_machine".to_string())
    );
}

#[tokio::test]
async fn test_integration_focus_history_tracking() {
    let mut focus = FocusManager::new("host".to_string());

    // Simulate a workflow across multiple machines
    focus
        .transfer_focus("agent1".to_string(), 0, 0)
        .await
        .unwrap();
    focus
        .transfer_focus("agent2".to_string(), 0, 0)
        .await
        .unwrap();
    focus
        .transfer_focus("agent3".to_string(), 0, 0)
        .await
        .unwrap();
    focus.return_to_host().await.unwrap();

    // Check history
    let history = focus.focus_history();
    assert_eq!(history.len(), 5); // host, agent1, agent2, agent3, host
    assert_eq!(history[0], "host");
    assert_eq!(history[1], "agent1");
    assert_eq!(history[2], "agent2");
    assert_eq!(history[3], "agent3");
    assert_eq!(history[4], "host");
}

#[tokio::test]
async fn test_integration_simulation_with_network_latency() {
    let mut sim = SimulationMode::new();
    sim.set_network_latency(25); // 25ms latency

    sim.add_virtual_machine("host".to_string(), 1920, 1080);
    sim.add_virtual_machine("agent1".to_string(), 1920, 1080);

    // Measure latency
    let start = std::time::Instant::now();
    sim.send_event_to("host", Event::MouseMove { x: 100, y: 100 })
        .await
        .unwrap();
    let elapsed1 = start.elapsed();

    let start = std::time::Instant::now();
    sim.send_event_to("agent1", Event::MouseMove { x: 200, y: 200 })
        .await
        .unwrap();
    let elapsed2 = start.elapsed();

    // Both should have latency
    assert!(elapsed1 >= Duration::from_millis(25));
    assert!(elapsed2 >= Duration::from_millis(25));

    // Check statistics
    let stats = sim.get_statistics();
    assert_eq!(stats.total_events_sent, 2);
}

#[tokio::test]
async fn test_integration_complete_workflow() {
    // This test simulates a complete workflow:
    // 1. Load configuration
    // 2. Setup topology
    // 3. Connect to network
    // 4. Transfer focus
    // 5. Send events

    // 1. Configuration
    let config = Config {
        self_name: "host".to_string(),
        mode: ConfigMode::Host,
        port: 53421,
        tls: multishiva::core::config::TlsConfig {
            psk: "integration-test".to_string(),
        },
        edges: {
            let mut edges = std::collections::HashMap::new();
            edges.insert("right".to_string(), "agent1".to_string());
            edges
        },
        hotkeys: None,
        behavior: None,
    };
    config.validate().unwrap();

    // 2. Topology
    let mut topology = Topology::new();
    topology.add_machine("host".to_string(), Position { x: 0, y: 0 });
    topology.add_machine("agent1".to_string(), Position { x: 1920, y: 0 });
    topology.add_edge("host".to_string(), Edge::Right, "agent1".to_string());

    // 3. Network
    let mut host_network = Network::new(config.tls.psk.clone());
    let mut agent_network = Network::new(config.tls.psk.clone());

    let port = host_network.start_host(0).await.unwrap();
    sleep(Duration::from_millis(100)).await;

    agent_network
        .connect_to_host(&format!("127.0.0.1:{}", port))
        .await
        .unwrap();

    // 4. Focus
    let mut focus = FocusManager::new("host".to_string());

    // Simulate mouse at right edge
    let cursor_x = 1910;
    let cursor_y = 500;

    let edge = topology.detect_edge("host", cursor_x, cursor_y, 1920, 10);
    if edge == Some(Edge::Right) {
        if let Some(neighbor) = topology.get_neighbor("host", &Edge::Right) {
            focus
                .transfer_focus(neighbor.clone(), 0, cursor_y)
                .await
                .unwrap();
            assert_eq!(focus.current(), "agent1");
        }
    }

    // 5. Send events
    agent_network
        .send_event(Event::MouseMove { x: 0, y: cursor_y })
        .await
        .unwrap();

    // Cleanup
    host_network.stop().await;
    agent_network.stop().await;
}
