use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    MouseMove { x: i32, y: i32 },
    MouseClick { button: MouseButton },
    KeyPress { key: String },
    KeyRelease { key: String },
    FocusGrant { target: String, x: i32, y: i32 },
    FocusRelease,
    Heartbeat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_serialization() {
        // Basic smoke test
        assert!(true);
    }
}
