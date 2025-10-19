use multishiva::core::topology::{Edge, Position, Topology};

#[test]
fn test_topology_creation() {
    let topology = Topology::new();
    assert_eq!(topology.machine_count(), 0);
}

#[test]
fn test_topology_add_machine() {
    let mut topology = Topology::new();
    topology.add_machine("desktop".to_string(), Position { x: 0, y: 0 });
    topology.add_machine("laptop".to_string(), Position { x: 1, y: 0 });
    assert_eq!(topology.machine_count(), 2);
}

#[test]
fn test_topology_add_edge() {
    let mut topology = Topology::new();
    topology.add_machine("desktop".to_string(), Position { x: 0, y: 0 });
    topology.add_machine("laptop".to_string(), Position { x: 1, y: 0 });

    topology.add_edge("desktop".to_string(), Edge::Right, "laptop".to_string());

    let neighbor = topology.get_neighbor("desktop", &Edge::Right);
    assert_eq!(neighbor, Some(&"laptop".to_string()));
}

#[test]
fn test_topology_edge_detection_right() {
    let mut topology = Topology::new();
    topology.add_machine("desktop".to_string(), Position { x: 0, y: 0 });
    topology.add_machine("laptop".to_string(), Position { x: 1, y: 0 });
    topology.add_edge("desktop".to_string(), Edge::Right, "laptop".to_string());

    // Test edge detection at right edge (1919 pixels on a 1920px screen)
    let edge = topology.detect_edge("desktop", 1919, 500, 1920, 3);
    assert_eq!(edge, Some(Edge::Right));
}

#[test]
fn test_topology_edge_detection_left() {
    let mut topology = Topology::new();
    topology.add_machine("laptop".to_string(), Position { x: 1, y: 0 });
    topology.add_machine("desktop".to_string(), Position { x: 0, y: 0 });
    topology.add_edge("laptop".to_string(), Edge::Left, "desktop".to_string());

    // Test edge detection at left edge
    let edge = topology.detect_edge("laptop", 0, 500, 1920, 3);
    assert_eq!(edge, Some(Edge::Left));
}

#[test]
fn test_topology_edge_detection_top() {
    let mut topology = Topology::new();
    topology.add_machine("main".to_string(), Position { x: 0, y: 0 });
    topology.add_machine("top".to_string(), Position { x: 0, y: -1 });
    topology.add_edge("main".to_string(), Edge::Top, "top".to_string());

    let edge = topology.detect_edge("main", 500, 0, 1920, 3);
    assert_eq!(edge, Some(Edge::Top));
}

#[test]
fn test_topology_edge_detection_bottom() {
    let mut topology = Topology::new();
    topology.add_machine("main".to_string(), Position { x: 0, y: 0 });
    topology.add_machine("bottom".to_string(), Position { x: 0, y: 1 });
    topology.add_edge("main".to_string(), Edge::Bottom, "bottom".to_string());

    // Assuming screen height is 1080
    let edge = topology.detect_edge("main", 500, 1079, 1920, 3);
    assert_eq!(edge, Some(Edge::Bottom));
}

#[test]
fn test_topology_no_edge_detected_middle() {
    let mut topology = Topology::new();
    topology.add_machine("desktop".to_string(), Position { x: 0, y: 0 });

    // Cursor in the middle of the screen
    let edge = topology.detect_edge("desktop", 960, 540, 1920, 3);
    assert!(edge.is_none());
}

#[test]
fn test_topology_edge_threshold() {
    let mut topology = Topology::new();
    topology.add_machine("desktop".to_string(), Position { x: 0, y: 0 });
    topology.add_machine("laptop".to_string(), Position { x: 1, y: 0 });
    topology.add_edge("desktop".to_string(), Edge::Right, "laptop".to_string());

    // Just outside threshold (1916 with threshold 3)
    let edge = topology.detect_edge("desktop", 1916, 500, 1920, 3);
    assert!(edge.is_none());

    // Within threshold (1917 with threshold 3)
    let edge = topology.detect_edge("desktop", 1917, 500, 1920, 3);
    assert_eq!(edge, Some(Edge::Right));
}

#[test]
fn test_topology_no_neighbor_configured() {
    let mut topology = Topology::new();
    topology.add_machine("desktop".to_string(), Position { x: 0, y: 0 });

    // Even at edge, should return None if no neighbor configured
    let edge = topology.detect_edge("desktop", 1919, 500, 1920, 3);
    assert!(edge.is_none());
}

#[test]
fn test_topology_calculate_relative_position() {
    let topology = Topology::new();

    // Moving from right edge to left edge of next screen
    let (rel_x, rel_y) = topology.calculate_relative_position(1919, 500, 1920, 1080);
    assert_eq!(rel_x, 0); // Should wrap to left edge
    assert_eq!(rel_y, 500); // Y should stay the same
}
