use serde::{Deserialize, Serialize};

/// Represents all possible events that can occur in the multishiva system.
///
/// Events are the core communication mechanism for input handling and system state changes.
/// They can be serialized and deserialized for transmission between components.
///
/// # Examples
///
/// ```
/// use multishiva::core::events::{Event, MouseButton, Key};
///
/// // Create a mouse move event
/// let move_event = Event::MouseMove { x: 100, y: 200 };
///
/// // Create a key press event
/// let key_event = Event::KeyPress { key: Key::KeyA };
///
/// // Create a mouse click event
/// let click_event = Event::MouseClick { button: MouseButton::Left };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    /// Mouse cursor moved to a new position.
    MouseMove {
        /// The horizontal position in screen coordinates
        x: i32,
        /// The vertical position in screen coordinates
        y: i32,
    },

    /// Mouse button was clicked (press and release).
    MouseClick {
        /// The mouse button that was clicked
        button: MouseButton,
    },

    /// Mouse button was pressed down.
    MouseButtonPress {
        /// The mouse button that was pressed
        button: MouseButton,
    },

    /// Mouse button was released.
    MouseButtonRelease {
        /// The mouse button that was released
        button: MouseButton,
    },

    /// Mouse wheel was scrolled.
    MouseScroll {
        /// Horizontal scroll amount (positive = right, negative = left)
        delta_x: i64,
        /// Vertical scroll amount (positive = down, negative = up)
        delta_y: i64,
    },

    /// Keyboard key was pressed down.
    KeyPress {
        /// The key that was pressed
        key: Key,
    },

    /// Keyboard key was released.
    KeyRelease {
        /// The key that was released
        key: Key,
    },

    /// Focus was granted to a specific target at a position.
    FocusGrant {
        /// Identifier of the component receiving focus
        target: String,
        /// The horizontal position where focus was granted
        x: i32,
        /// The vertical position where focus was granted
        y: i32,
    },

    /// Focus was released from the current target.
    FocusRelease,

    /// Periodic heartbeat event for keepalive or timing purposes.
    Heartbeat,
}

/// Represents the physical buttons on a mouse.
///
/// This enum is used to identify which mouse button was involved in a mouse event.
///
/// # Examples
///
/// ```
/// use multishiva::core::events::MouseButton;
///
/// let left = MouseButton::Left;
/// let right = MouseButton::Right;
///
/// assert_eq!(left, MouseButton::Left);
/// assert_ne!(left, right);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MouseButton {
    /// The primary (left) mouse button.
    Left,

    /// The secondary (right) mouse button.
    Right,

    /// The tertiary (middle) mouse button, typically a scroll wheel click.
    Middle,
}

/// Represents keyboard keys that can be pressed or released.
///
/// This enum covers alphabetic keys, modifier keys, and common special keys.
/// The naming convention uses `Key` prefix for letter keys to avoid conflicts
/// with Rust keywords and for consistency.
///
/// # Examples
///
/// ```
/// use multishiva::core::events::Key;
///
/// let a_key = Key::KeyA;
/// let ctrl = Key::ControlLeft;
/// let enter = Key::Return;
///
/// assert_eq!(a_key, Key::KeyA);
/// assert_ne!(ctrl, Key::ControlRight);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Key {
    // Letters
    /// The A key.
    KeyA,
    /// The B key.
    KeyB,
    /// The C key.
    KeyC,
    /// The D key.
    KeyD,
    /// The E key.
    KeyE,
    /// The F key.
    KeyF,
    /// The G key.
    KeyG,
    /// The H key.
    KeyH,
    /// The I key.
    KeyI,
    /// The J key.
    KeyJ,
    /// The K key.
    KeyK,
    /// The L key.
    KeyL,
    /// The M key.
    KeyM,
    /// The N key.
    KeyN,
    /// The O key.
    KeyO,
    /// The P key.
    KeyP,
    /// The Q key.
    KeyQ,
    /// The R key.
    KeyR,
    /// The S key.
    KeyS,
    /// The T key.
    KeyT,
    /// The U key.
    KeyU,
    /// The V key.
    KeyV,
    /// The W key.
    KeyW,
    /// The X key.
    KeyX,
    /// The Y key.
    KeyY,
    /// The Z key.
    KeyZ,

    // Modifiers
    /// The left Control modifier key.
    ControlLeft,
    /// The right Control modifier key.
    ControlRight,
    /// The left Shift modifier key.
    ShiftLeft,
    /// The right Shift modifier key.
    ShiftRight,
    /// The left Alt (Option on macOS) modifier key.
    AltLeft,
    /// The right Alt (Option on macOS) modifier key.
    AltRight,
    /// The left Meta (Windows/Command) modifier key.
    MetaLeft,
    /// The right Meta (Windows/Command) modifier key.
    MetaRight,

    // Special keys
    /// The Escape key.
    Escape,
    /// The Return/Enter key.
    Return,
    /// The Space bar.
    Space,
    /// The Backspace key for deleting characters backward.
    Backspace,
    /// The Tab key for indentation and navigation.
    Tab,
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_event_serialization() {
        // Basic smoke test - empty test for now
    }
}
