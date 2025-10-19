use std::collections::HashMap;

/// Represents the network topology of connected machines in a multi-screen setup.
///
/// A `Topology` manages the spatial arrangement of machines and their edge connections,
/// allowing for cursor movement across screen boundaries between different machines.
/// Each machine has a position and can have edges connected to neighboring machines.
///
/// # Examples
///
/// ```
/// use multishiva::core::topology::{Topology, Position, Edge};
///
/// let mut topology = Topology::new();
/// topology.add_machine("laptop".to_string(), Position { x: 0, y: 0 });
/// topology.add_machine("desktop".to_string(), Position { x: 1, y: 0 });
/// topology.add_edge("laptop".to_string(), Edge::Right, "desktop".to_string());
///
/// assert_eq!(topology.machine_count(), 2);
/// ```
#[derive(Debug, Clone)]
pub struct Topology {
    machines: HashMap<String, Position>,
    edges: HashMap<String, HashMap<Edge, String>>,
}

/// Represents a 2D position in the topology coordinate system.
///
/// Positions are used to spatially arrange machines in the topology.
/// The coordinate system uses integer values to represent logical positions,
/// not pixel coordinates.
///
/// # Examples
///
/// ```
/// use multishiva::core::topology::Position;
///
/// let pos = Position { x: 0, y: 0 };
/// assert_eq!(pos.x, 0);
/// assert_eq!(pos.y, 0);
/// ```
#[derive(Debug, Clone)]
pub struct Position {
    /// The x-coordinate in the topology grid.
    pub x: i32,
    /// The y-coordinate in the topology grid.
    pub y: i32,
}

/// Represents a screen edge direction for machine connections.
///
/// Each edge variant corresponds to a cardinal direction on a screen,
/// used to define which side of a machine connects to another machine.
///
/// # Examples
///
/// ```
/// use multishiva::core::topology::Edge;
///
/// let edge = Edge::Right;
/// assert_eq!(edge, Edge::Right);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Edge {
    /// The right edge of the screen.
    Right,
    /// The left edge of the screen.
    Left,
    /// The top edge of the screen.
    Top,
    /// The bottom edge of the screen.
    Bottom,
}

impl Default for Topology {
    fn default() -> Self {
        Self::new()
    }
}

impl Topology {
    /// Creates a new empty topology.
    ///
    /// The topology is initialized with no machines or edge connections.
    ///
    /// # Examples
    ///
    /// ```
    /// use multishiva::core::topology::Topology;
    ///
    /// let topology = Topology::new();
    /// assert_eq!(topology.machine_count(), 0);
    /// ```
    pub fn new() -> Self {
        Self {
            machines: HashMap::new(),
            edges: HashMap::new(),
        }
    }

    /// Returns the number of machines in the topology.
    ///
    /// # Examples
    ///
    /// ```
    /// use multishiva::core::topology::{Topology, Position};
    ///
    /// let mut topology = Topology::new();
    /// assert_eq!(topology.machine_count(), 0);
    ///
    /// topology.add_machine("machine1".to_string(), Position { x: 0, y: 0 });
    /// assert_eq!(topology.machine_count(), 1);
    /// ```
    pub fn machine_count(&self) -> usize {
        self.machines.len()
    }

    /// Adds a machine to the topology at the specified position.
    ///
    /// If a machine with the same name already exists, it will be replaced with
    /// the new position.
    ///
    /// # Arguments
    ///
    /// * `name` - The unique identifier for the machine
    /// * `pos` - The position of the machine in the topology
    ///
    /// # Examples
    ///
    /// ```
    /// use multishiva::core::topology::{Topology, Position};
    ///
    /// let mut topology = Topology::new();
    /// topology.add_machine("server".to_string(), Position { x: 1, y: 2 });
    /// assert_eq!(topology.machine_count(), 1);
    /// ```
    pub fn add_machine(&mut self, name: String, pos: Position) {
        self.machines.insert(name, pos);
    }

    /// Adds a directional edge connection between two machines.
    ///
    /// Creates a connection from the source machine's specified edge to the target machine.
    /// This is a one-way connection; if bidirectional connectivity is needed, call this
    /// method twice with reversed parameters and opposite edges.
    ///
    /// # Arguments
    ///
    /// * `from` - The name of the source machine
    /// * `edge` - The edge of the source machine that connects to the target
    /// * `to` - The name of the target machine
    ///
    /// # Examples
    ///
    /// ```
    /// use multishiva::core::topology::{Topology, Position, Edge};
    ///
    /// let mut topology = Topology::new();
    /// topology.add_machine("left".to_string(), Position { x: 0, y: 0 });
    /// topology.add_machine("right".to_string(), Position { x: 1, y: 0 });
    ///
    /// // Connect left's right edge to right machine
    /// topology.add_edge("left".to_string(), Edge::Right, "right".to_string());
    /// // Connect right's left edge to left machine (bidirectional)
    /// topology.add_edge("right".to_string(), Edge::Left, "left".to_string());
    /// ```
    pub fn add_edge(&mut self, from: String, edge: Edge, to: String) {
        self.edges.entry(from).or_default().insert(edge, to);
    }

    /// Retrieves the neighboring machine connected to a specific edge.
    ///
    /// Returns the name of the machine connected to the given edge of the specified machine,
    /// or `None` if the machine doesn't exist or has no connection on that edge.
    ///
    /// # Arguments
    ///
    /// * `machine` - The name of the machine to query
    /// * `edge` - The edge to check for a connection
    ///
    /// # Returns
    ///
    /// Returns `Some(&String)` containing the neighbor's name if a connection exists,
    /// or `None` if the machine doesn't exist or has no neighbor on the specified edge.
    ///
    /// # Examples
    ///
    /// ```
    /// use multishiva::core::topology::{Topology, Position, Edge};
    ///
    /// let mut topology = Topology::new();
    /// topology.add_machine("main".to_string(), Position { x: 0, y: 0 });
    /// topology.add_machine("aux".to_string(), Position { x: 1, y: 0 });
    /// topology.add_edge("main".to_string(), Edge::Right, "aux".to_string());
    ///
    /// assert_eq!(topology.get_neighbor("main", &Edge::Right), Some(&"aux".to_string()));
    /// assert_eq!(topology.get_neighbor("main", &Edge::Left), None);
    /// ```
    pub fn get_neighbor(&self, machine: &str, edge: &Edge) -> Option<&String> {
        self.edges.get(machine)?.get(edge)
    }

    /// Detects which edge of the screen a cursor position is near.
    ///
    /// Determines if a cursor at position (x, y) is within the threshold distance
    /// of any configured edge on the specified machine. Only returns edges that
    /// have connections configured for the machine.
    ///
    /// # Arguments
    ///
    /// * `machine` - The name of the machine to check
    /// * `x` - The x-coordinate of the cursor position (in pixels)
    /// * `y` - The y-coordinate of the cursor position (in pixels)
    /// * `screen_width` - The width of the screen (in pixels)
    /// * `threshold` - The distance from the edge (in pixels) to trigger detection
    ///
    /// # Returns
    ///
    /// Returns `Some(Edge)` if the cursor is near a configured edge, or `None` if
    /// the cursor is not near any configured edges or the machine doesn't exist.
    ///
    /// # Examples
    ///
    /// ```
    /// use multishiva::core::topology::{Topology, Position, Edge};
    ///
    /// let mut topology = Topology::new();
    /// topology.add_machine("screen1".to_string(), Position { x: 0, y: 0 });
    /// topology.add_edge("screen1".to_string(), Edge::Right, "screen2".to_string());
    ///
    /// // Cursor near right edge (assuming 1920px width, 10px threshold)
    /// let edge = topology.detect_edge("screen1", 1915, 500, 1920, 10);
    /// assert_eq!(edge, Some(Edge::Right));
    ///
    /// // Cursor in middle of screen
    /// let edge = topology.detect_edge("screen1", 960, 500, 1920, 10);
    /// assert_eq!(edge, None);
    /// ```
    ///
    /// # Note
    ///
    /// For bottom edge detection, the method currently assumes a screen height of 1080 pixels.
    /// This is a temporary implementation detail that will be improved to use actual screen
    /// dimensions in future versions.
    pub fn detect_edge(
        &self,
        machine: &str,
        x: i32,
        y: i32,
        screen_width: u32,
        threshold: u32,
    ) -> Option<Edge> {
        // Check if machine has any configured edges
        let machine_edges = self.edges.get(machine)?;

        let threshold = threshold as i32;
        let screen_width = screen_width as i32;

        // Check right edge
        if machine_edges.contains_key(&Edge::Right) && x >= screen_width - threshold {
            return Some(Edge::Right);
        }

        // Check left edge
        if machine_edges.contains_key(&Edge::Left) && x < threshold {
            return Some(Edge::Left);
        }

        // Check top edge
        if machine_edges.contains_key(&Edge::Top) && y < threshold {
            return Some(Edge::Top);
        }

        // Check bottom edge
        // Note: We assume screen_height for bottom edge detection
        // In practice, this would come from the actual screen dimensions
        if machine_edges.contains_key(&Edge::Bottom) {
            // Using a reasonable assumption for now
            // This will be improved when we have actual screen info
            let screen_height = 1080; // Default assumption
            if y >= screen_height - threshold {
                return Some(Edge::Bottom);
            }
        }

        None
    }

    /// Calculates the relative cursor position when transitioning between screens.
    ///
    /// This method computes where the cursor should appear on the target screen when
    /// crossing from one machine to another. The current implementation is simplified
    /// and assumes horizontal transitions (right edge to left edge).
    ///
    /// # Arguments
    ///
    /// * `_x` - The x-coordinate of the cursor on the source screen (currently unused)
    /// * `y` - The y-coordinate of the cursor on the source screen
    /// * `_screen_width` - The width of the source screen (currently unused)
    /// * `_screen_height` - The height of the source screen (currently unused)
    ///
    /// # Returns
    ///
    /// A tuple `(x, y)` representing the cursor position on the target screen.
    ///
    /// # Examples
    ///
    /// ```
    /// use multishiva::core::topology::Topology;
    ///
    /// let topology = Topology::new();
    /// let (x, y) = topology.calculate_relative_position(1920, 500, 1920, 1080);
    /// assert_eq!(x, 0);
    /// assert_eq!(y, 500);
    /// ```
    ///
    /// # Note
    ///
    /// This is a simplified implementation that wraps the x-coordinate to 0 and
    /// preserves the y-coordinate. Future versions will support:
    /// - Proper handling of all edge types (top, bottom, left, right)
    /// - Screen resolution differences between machines
    /// - Coordinate scaling and offset calculations
    pub fn calculate_relative_position(
        &self,
        _x: i32,
        y: i32,
        _screen_width: u32,
        _screen_height: u32,
    ) -> (i32, i32) {
        // When crossing from right edge to left edge
        // X wraps to 0, Y stays the same
        // This is a simplified implementation
        (0, y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topology_creation() {
        let topology = Topology::new();
        assert_eq!(topology.machines.len(), 0);
    }
}
