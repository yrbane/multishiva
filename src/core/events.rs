use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    MouseMove { x: i32, y: i32 },
    MouseClick { button: MouseButton },
    MouseButtonPress { button: MouseButton },
    MouseButtonRelease { button: MouseButton },
    MouseScroll { delta_x: i64, delta_y: i64 },
    KeyPress { key: Key },
    KeyRelease { key: Key },
    FocusGrant { target: String, x: i32, y: i32 },
    FocusRelease,
    Heartbeat,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Key {
    // Letters
    KeyA,
    KeyB,
    KeyC,
    KeyD,
    KeyE,
    KeyF,
    KeyG,
    KeyH,
    KeyI,
    KeyJ,
    KeyK,
    KeyL,
    KeyM,
    KeyN,
    KeyO,
    KeyP,
    KeyQ,
    KeyR,
    KeyS,
    KeyT,
    KeyU,
    KeyV,
    KeyW,
    KeyX,
    KeyY,
    KeyZ,

    // Modifiers
    ControlLeft,
    ControlRight,
    ShiftLeft,
    ShiftRight,
    AltLeft,
    AltRight,
    MetaLeft,
    MetaRight,

    // Special keys
    Escape,
    Return,
    Space,
    Backspace,
    Tab,
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_event_serialization() {
        // Basic smoke test - empty test for now
    }
}
