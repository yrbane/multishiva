use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Topology {
    machines: HashMap<String, Position>,
    edges: HashMap<String, HashMap<Edge, String>>,
}

#[derive(Debug, Clone)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Edge {
    Right,
    Left,
    Top,
    Bottom,
}

impl Default for Topology {
    fn default() -> Self {
        Self::new()
    }
}

impl Topology {
    pub fn new() -> Self {
        Self {
            machines: HashMap::new(),
            edges: HashMap::new(),
        }
    }

    pub fn machine_count(&self) -> usize {
        self.machines.len()
    }

    pub fn add_machine(&mut self, name: String, pos: Position) {
        self.machines.insert(name, pos);
    }

    pub fn add_edge(&mut self, from: String, edge: Edge, to: String) {
        self.edges.entry(from).or_default().insert(edge, to);
    }

    pub fn get_neighbor(&self, machine: &str, edge: &Edge) -> Option<&String> {
        self.edges.get(machine)?.get(edge)
    }

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
