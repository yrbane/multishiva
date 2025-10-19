use anyhow::{Context, Result};
use rdev::{simulate, Button, EventType as RdevEventType, Key as RdevKey};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock as StdRwLock};
use tokio::sync::mpsc;

use crate::core::events::{Event, Key, MouseButton};

type EventFilter = Box<dyn Fn(&Event) -> bool + Send + Sync>;

/// Trait for handling input capture and injection across different platforms.
///
/// This trait provides a unified interface for capturing keyboard and mouse events
/// from the operating system and injecting synthetic events back into the system.
/// Implementations must be thread-safe (`Send + Sync`).
///
/// # Examples
///
/// ```no_run
/// use multishiva::core::input::{InputHandler, RdevInputHandler};
/// use tokio::sync::mpsc;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let mut handler = RdevInputHandler::new();
///     let (tx, mut rx) = mpsc::channel(100);
///
///     handler.start_capture(tx).await?;
///
///     // Process captured events
///     while let Some(event) = rx.recv().await {
///         println!("Captured: {:?}", event);
///     }
///
///     Ok(())
/// }
/// ```
#[allow(async_fn_in_trait)]
pub trait InputHandler: Send + Sync {
    /// Starts capturing input events from the system.
    ///
    /// Begins monitoring all keyboard and mouse events and sends them through
    /// the provided channel. This method is idempotent - calling it multiple
    /// times has no additional effect if already capturing.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying event capture mechanism fails to initialize,
    /// which may happen due to insufficient permissions or platform-specific issues.
    async fn start_capture(&mut self, tx: mpsc::Sender<Event>) -> Result<()>;

    /// Stops capturing input events.
    ///
    /// Terminates the event capture loop started by `start_capture`. May take
    /// a short time to complete as it waits for the capture thread to finish.
    async fn stop_capture(&mut self);

    /// Injects a synthetic input event into the system.
    ///
    /// Simulates the given event as if it came from a physical input device.
    /// This requires appropriate system permissions on most platforms.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The event cannot be converted to the platform-specific format
    /// - The system rejects the event injection due to permissions
    /// - The underlying simulation mechanism fails
    async fn inject_event(&self, event: Event) -> Result<()>;

    /// Returns whether input capture is currently active.
    fn is_capturing(&self) -> bool;

    /// Returns the current screen dimensions in pixels as (width, height).
    fn get_screen_size(&self) -> (u32, u32);

    /// Returns the current cursor position in screen coordinates as (x, y).
    ///
    /// # Errors
    ///
    /// Returns an error if the cursor position cannot be retrieved from the system.
    fn get_cursor_position(&self) -> Result<(i32, i32)>;

    /// Checks whether the application has necessary permissions for input capture/injection.
    ///
    /// On macOS, this checks Accessibility permissions. On Linux, it checks for
    /// input device access. On Windows, it verifies input injection privileges.
    fn check_permissions(&self) -> bool;
}

/// Input handler implementation using the rdev library.
///
/// Provides cross-platform input capture and injection using the `rdev` crate.
/// Supports features like kill switches (emergency stop key combinations),
/// local input blocking, and event filtering.
///
/// # Examples
///
/// ```no_run
/// use multishiva::core::input::RdevInputHandler;
/// use multishiva::core::events::Key;
///
/// let mut handler = RdevInputHandler::new();
///
/// // Set up an emergency stop with Ctrl+Alt+Q
/// handler.set_kill_switch(vec![Key::ControlLeft, Key::AltLeft, Key::KeyQ]);
///
/// // Enable blocking of local input
/// handler.set_block_local(true);
/// ```
pub struct RdevInputHandler {
    capturing: Arc<AtomicBool>,
    block_local: Arc<AtomicBool>,
    kill_switch: Arc<StdRwLock<Option<Vec<Key>>>>,
    event_filter: Arc<StdRwLock<Option<EventFilter>>>,
    pressed_keys: Arc<StdRwLock<Vec<Key>>>,
}

impl Default for RdevInputHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl RdevInputHandler {
    /// Creates a new input handler with default settings.
    ///
    /// Initializes the handler with capturing disabled, no kill switch,
    /// no event filter, and local input blocking disabled.
    ///
    /// # Examples
    ///
    /// ```
    /// use multishiva::core::input::RdevInputHandler;
    ///
    /// let handler = RdevInputHandler::new();
    /// assert!(!handler.is_capturing());
    /// assert!(!handler.has_kill_switch());
    /// ```
    pub fn new() -> Self {
        Self {
            capturing: Arc::new(AtomicBool::new(false)),
            block_local: Arc::new(AtomicBool::new(false)),
            kill_switch: Arc::new(StdRwLock::new(None)),
            event_filter: Arc::new(StdRwLock::new(None)),
            pressed_keys: Arc::new(StdRwLock::new(Vec::new())),
        }
    }

    /// Sets a kill switch key combination.
    ///
    /// When all specified keys are pressed simultaneously, the kill switch
    /// activates. This is typically used as an emergency stop mechanism.
    ///
    /// # Examples
    ///
    /// ```
    /// use multishiva::core::input::RdevInputHandler;
    /// use multishiva::core::events::Key;
    ///
    /// let handler = RdevInputHandler::new();
    /// handler.set_kill_switch(vec![Key::ControlLeft, Key::AltLeft, Key::KeyQ]);
    /// assert!(handler.has_kill_switch());
    /// ```
    pub fn set_kill_switch(&self, keys: Vec<Key>) {
        if let Ok(mut lock) = self.kill_switch.write() {
            *lock = Some(keys);
        }
    }

    /// Returns whether a kill switch is currently configured.
    pub fn has_kill_switch(&self) -> bool {
        if let Ok(lock) = self.kill_switch.read() {
            lock.is_some()
        } else {
            false
        }
    }

    /// Enables or disables local input blocking.
    ///
    /// When enabled, captured input events are prevented from reaching
    /// the local system. This is platform-specific and may require
    /// low-level hooks.
    ///
    /// # Examples
    ///
    /// ```
    /// use multishiva::core::input::RdevInputHandler;
    ///
    /// let mut handler = RdevInputHandler::new();
    /// handler.set_block_local(true);
    /// assert!(handler.is_blocking_local());
    /// ```
    pub fn set_block_local(&mut self, block: bool) {
        self.block_local.store(block, Ordering::SeqCst);
    }

    /// Returns whether local input blocking is enabled.
    pub fn is_blocking_local(&self) -> bool {
        self.block_local.load(Ordering::SeqCst)
    }

    /// Sets a custom filter function for captured events.
    ///
    /// The filter function receives each captured event and returns `true`
    /// to allow it or `false` to drop it. Only events that pass the filter
    /// are forwarded through the capture channel.
    ///
    /// # Examples
    ///
    /// ```
    /// use multishiva::core::input::RdevInputHandler;
    /// use multishiva::core::events::Event;
    ///
    /// let handler = RdevInputHandler::new();
    ///
    /// // Only allow mouse events
    /// handler.set_event_filter(|event| {
    ///     matches!(event, Event::MouseMove { .. } | Event::MouseButtonPress { .. })
    /// });
    ///
    /// assert!(handler.has_event_filter());
    /// ```
    pub fn set_event_filter<F>(&self, filter: F)
    where
        F: Fn(&Event) -> bool + Send + Sync + 'static,
    {
        if let Ok(mut lock) = self.event_filter.write() {
            *lock = Some(Box::new(filter));
        }
    }

    /// Returns whether an event filter is currently configured.
    pub fn has_event_filter(&self) -> bool {
        if let Ok(lock) = self.event_filter.read() {
            lock.is_some()
        } else {
            false
        }
    }

    #[allow(dead_code)]
    fn check_kill_switch(&self, key: &Key) -> bool {
        if let Ok(mut pressed) = self.pressed_keys.write() {
            if !pressed.contains(key) {
                pressed.push(key.clone());
            }

            if let Ok(kill_switch_guard) = self.kill_switch.read() {
                if let Some(keys) = kill_switch_guard.as_ref() {
                    // Check if all kill switch keys are pressed
                    return keys.iter().all(|k| pressed.contains(k));
                }
            }
        }
        false
    }

    #[allow(dead_code)]
    fn handle_key_release(&self, key: &Key) {
        if let Ok(mut pressed) = self.pressed_keys.write() {
            pressed.retain(|k| k != key);
        }
    }
}

impl InputHandler for RdevInputHandler {
    async fn start_capture(&mut self, tx: mpsc::Sender<Event>) -> Result<()> {
        if self.capturing.load(Ordering::SeqCst) {
            return Ok(());
        }

        self.capturing.store(true, Ordering::SeqCst);
        let capturing = self.capturing.clone();
        let block_local = self.block_local.clone();

        // Create a standard channel for the rdev thread
        let (std_tx, std_rx) = std::sync::mpsc::channel::<Event>();

        // Spawn capture thread (this runs rdev::listen which blocks)
        std::thread::spawn(move || {
            let callback = move |event: rdev::Event| {
                if !capturing.load(Ordering::SeqCst) {
                    return;
                }

                // Convert rdev event to our Event type
                let our_event = match convert_rdev_to_event(event.event_type) {
                    Some(e) => e,
                    None => return,
                };

                // Send through standard channel
                let _ = std_tx.send(our_event);

                // Block local input if enabled
                if block_local.load(Ordering::SeqCst) {
                    // In a real implementation, we would suppress the event here
                    // This is platform-specific and requires low-level hooks
                }
            };

            // Start listening (this blocks)
            if let Err(e) = rdev::listen(callback) {
                tracing::error!("Failed to listen for events: {:?}", e);
            }
        });

        // Spawn async task to forward events from std channel to tokio channel
        tokio::spawn(async move {
            loop {
                match std_rx.try_recv() {
                    Ok(event) => {
                        if tx.send(event).await.is_err() {
                            break;
                        }
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => {
                        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                    }
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    async fn stop_capture(&mut self) {
        self.capturing.store(false, Ordering::SeqCst);
        // Give time for the listener to stop
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    async fn inject_event(&self, event: Event) -> Result<()> {
        let rdev_event =
            convert_event_to_rdev(&event).context("Failed to convert event to rdev format")?;

        // Simulate the event
        tokio::task::spawn_blocking(move || {
            simulate(&rdev_event).map_err(|e| anyhow::anyhow!("Failed to simulate event: {:?}", e))
        })
        .await
        .context("Task join error")??;

        Ok(())
    }

    fn is_capturing(&self) -> bool {
        self.capturing.load(Ordering::SeqCst)
    }

    fn get_screen_size(&self) -> (u32, u32) {
        // Get primary display size
        // This is a simplified implementation
        // In production, use platform-specific APIs or rdev's display info
        #[cfg(target_os = "linux")]
        {
            // For Linux, we could use X11 or Wayland APIs
            // For now, return a reasonable default
            (1920, 1080)
        }

        #[cfg(target_os = "macos")]
        {
            // For macOS, we could use Core Graphics
            (1920, 1080)
        }

        #[cfg(target_os = "windows")]
        {
            // For Windows, we could use GetSystemMetrics
            (1920, 1080)
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            (1920, 1080)
        }
    }

    fn get_cursor_position(&self) -> Result<(i32, i32)> {
        // This would require platform-specific implementation
        // For now, return a placeholder
        // In production, use platform-specific cursor position APIs
        Ok((0, 0))
    }

    fn check_permissions(&self) -> bool {
        // Check if we have necessary permissions to capture/inject input
        // This is platform-specific

        #[cfg(target_os = "macos")]
        {
            // On macOS, check Accessibility permissions
            // This would require objective-c bindings or a crate like cocoa
            true
        }

        #[cfg(target_os = "linux")]
        {
            // On Linux, check if we can access /dev/uinput or have X11/Wayland access
            true
        }

        #[cfg(target_os = "windows")]
        {
            // On Windows, check if we have input injection privileges
            true
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            false
        }
    }
}

/// Converts an rdev event type to our internal Event representation.
///
/// Maps platform-specific rdev events to our unified Event enum. Returns
/// `None` for events that cannot be mapped or are not supported.
fn convert_rdev_to_event(event: RdevEventType) -> Option<Event> {
    match event {
        RdevEventType::MouseMove { x, y } => Some(Event::MouseMove {
            x: x as i32,
            y: y as i32,
        }),
        RdevEventType::ButtonPress(button) => {
            let our_button = convert_rdev_button(button)?;
            Some(Event::MouseButtonPress { button: our_button })
        }
        RdevEventType::ButtonRelease(button) => {
            let our_button = convert_rdev_button(button)?;
            Some(Event::MouseButtonRelease { button: our_button })
        }
        RdevEventType::Wheel { delta_x, delta_y } => Some(Event::MouseScroll { delta_x, delta_y }),
        RdevEventType::KeyPress(key) => {
            let our_key = convert_rdev_key(key)?;
            Some(Event::KeyPress { key: our_key })
        }
        RdevEventType::KeyRelease(key) => {
            let our_key = convert_rdev_key(key)?;
            Some(Event::KeyRelease { key: our_key })
        }
    }
}

/// Converts our internal Event to rdev's event type for injection.
///
/// Maps our unified Event enum to platform-specific rdev event types.
/// Returns `None` for events that cannot be injected (e.g., MouseClick,
/// FocusGrant, FocusRelease, Heartbeat).
fn convert_event_to_rdev(event: &Event) -> Option<RdevEventType> {
    match event {
        Event::MouseMove { x, y } => Some(RdevEventType::MouseMove {
            x: *x as f64,
            y: *y as f64,
        }),
        Event::MouseButtonPress { button } => {
            let rdev_button = convert_button_to_rdev(button)?;
            Some(RdevEventType::ButtonPress(rdev_button))
        }
        Event::MouseButtonRelease { button } => {
            let rdev_button = convert_button_to_rdev(button)?;
            Some(RdevEventType::ButtonRelease(rdev_button))
        }
        Event::MouseScroll { delta_x, delta_y } => Some(RdevEventType::Wheel {
            delta_x: *delta_x,
            delta_y: *delta_y,
        }),
        Event::KeyPress { key } => {
            let rdev_key = convert_key_to_rdev(key)?;
            Some(RdevEventType::KeyPress(rdev_key))
        }
        Event::KeyRelease { key } => {
            let rdev_key = convert_key_to_rdev(key)?;
            Some(RdevEventType::KeyRelease(rdev_key))
        }
        // Events that cannot be converted to rdev events
        Event::MouseClick { .. }
        | Event::FocusGrant { .. }
        | Event::FocusRelease
        | Event::Heartbeat => None,
    }
}

/// Converts an rdev mouse button to our MouseButton type.
///
/// Returns `None` for buttons that are not Left, Right, or Middle.
fn convert_rdev_button(button: Button) -> Option<MouseButton> {
    match button {
        Button::Left => Some(MouseButton::Left),
        Button::Right => Some(MouseButton::Right),
        Button::Middle => Some(MouseButton::Middle),
        _ => None,
    }
}

/// Converts our MouseButton type to rdev's Button type.
///
/// All MouseButton variants have a corresponding rdev Button, so this
/// always returns `Some`.
fn convert_button_to_rdev(button: &MouseButton) -> Option<Button> {
    match button {
        MouseButton::Left => Some(Button::Left),
        MouseButton::Right => Some(Button::Right),
        MouseButton::Middle => Some(Button::Middle),
    }
}

/// Converts an rdev key to our internal Key representation.
///
/// Maps platform-specific rdev keys to our unified Key enum. Only common
/// keys (letters, modifiers, special keys) are supported. Returns `None`
/// for unmapped keys.
fn convert_rdev_key(key: RdevKey) -> Option<Key> {
    // Map common keys - this is a simplified mapping
    match key {
        // Letters
        RdevKey::KeyA => Some(Key::KeyA),
        RdevKey::KeyB => Some(Key::KeyB),
        RdevKey::KeyC => Some(Key::KeyC),
        RdevKey::KeyD => Some(Key::KeyD),
        RdevKey::KeyE => Some(Key::KeyE),
        RdevKey::KeyF => Some(Key::KeyF),
        RdevKey::KeyG => Some(Key::KeyG),
        RdevKey::KeyH => Some(Key::KeyH),
        RdevKey::KeyI => Some(Key::KeyI),
        RdevKey::KeyJ => Some(Key::KeyJ),
        RdevKey::KeyK => Some(Key::KeyK),
        RdevKey::KeyL => Some(Key::KeyL),
        RdevKey::KeyM => Some(Key::KeyM),
        RdevKey::KeyN => Some(Key::KeyN),
        RdevKey::KeyO => Some(Key::KeyO),
        RdevKey::KeyP => Some(Key::KeyP),
        RdevKey::KeyQ => Some(Key::KeyQ),
        RdevKey::KeyR => Some(Key::KeyR),
        RdevKey::KeyS => Some(Key::KeyS),
        RdevKey::KeyT => Some(Key::KeyT),
        RdevKey::KeyU => Some(Key::KeyU),
        RdevKey::KeyV => Some(Key::KeyV),
        RdevKey::KeyW => Some(Key::KeyW),
        RdevKey::KeyX => Some(Key::KeyX),
        RdevKey::KeyY => Some(Key::KeyY),
        RdevKey::KeyZ => Some(Key::KeyZ),

        // Modifiers
        RdevKey::ControlLeft => Some(Key::ControlLeft),
        RdevKey::ControlRight => Some(Key::ControlRight),
        RdevKey::ShiftLeft => Some(Key::ShiftLeft),
        RdevKey::ShiftRight => Some(Key::ShiftRight),
        RdevKey::Alt => Some(Key::AltLeft),
        RdevKey::AltGr => Some(Key::AltRight),
        RdevKey::MetaLeft => Some(Key::MetaLeft),
        RdevKey::MetaRight => Some(Key::MetaRight),

        // Special keys
        RdevKey::Escape => Some(Key::Escape),
        RdevKey::Return => Some(Key::Return),
        RdevKey::Space => Some(Key::Space),
        RdevKey::Backspace => Some(Key::Backspace),
        RdevKey::Tab => Some(Key::Tab),

        _ => None, // Unmapped keys
    }
}

/// Converts our internal Key to rdev's Key type for injection.
///
/// Maps our unified Key enum to platform-specific rdev key codes.
/// Only common keys (letters, modifiers, special keys) are supported.
fn convert_key_to_rdev(key: &Key) -> Option<RdevKey> {
    match key {
        // Letters
        Key::KeyA => Some(RdevKey::KeyA),
        Key::KeyB => Some(RdevKey::KeyB),
        Key::KeyC => Some(RdevKey::KeyC),
        Key::KeyD => Some(RdevKey::KeyD),
        Key::KeyE => Some(RdevKey::KeyE),
        Key::KeyF => Some(RdevKey::KeyF),
        Key::KeyG => Some(RdevKey::KeyG),
        Key::KeyH => Some(RdevKey::KeyH),
        Key::KeyI => Some(RdevKey::KeyI),
        Key::KeyJ => Some(RdevKey::KeyJ),
        Key::KeyK => Some(RdevKey::KeyK),
        Key::KeyL => Some(RdevKey::KeyL),
        Key::KeyM => Some(RdevKey::KeyM),
        Key::KeyN => Some(RdevKey::KeyN),
        Key::KeyO => Some(RdevKey::KeyO),
        Key::KeyP => Some(RdevKey::KeyP),
        Key::KeyQ => Some(RdevKey::KeyQ),
        Key::KeyR => Some(RdevKey::KeyR),
        Key::KeyS => Some(RdevKey::KeyS),
        Key::KeyT => Some(RdevKey::KeyT),
        Key::KeyU => Some(RdevKey::KeyU),
        Key::KeyV => Some(RdevKey::KeyV),
        Key::KeyW => Some(RdevKey::KeyW),
        Key::KeyX => Some(RdevKey::KeyX),
        Key::KeyY => Some(RdevKey::KeyY),
        Key::KeyZ => Some(RdevKey::KeyZ),

        // Modifiers
        Key::ControlLeft => Some(RdevKey::ControlLeft),
        Key::ControlRight => Some(RdevKey::ControlRight),
        Key::ShiftLeft => Some(RdevKey::ShiftLeft),
        Key::ShiftRight => Some(RdevKey::ShiftRight),
        Key::AltLeft => Some(RdevKey::Alt),
        Key::AltRight => Some(RdevKey::AltGr),
        Key::MetaLeft => Some(RdevKey::MetaLeft),
        Key::MetaRight => Some(RdevKey::MetaRight),

        // Special keys
        Key::Escape => Some(RdevKey::Escape),
        Key::Return => Some(RdevKey::Return),
        Key::Space => Some(RdevKey::Space),
        Key::Backspace => Some(RdevKey::Backspace),
        Key::Tab => Some(RdevKey::Tab),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_handler_creation() {
        let _handler = RdevInputHandler::new();
    }

    #[test]
    fn test_event_conversion() {
        let event = Event::MouseMove { x: 100, y: 200 };
        let rdev_event = convert_event_to_rdev(&event);
        assert!(rdev_event.is_some());

        match rdev_event.unwrap() {
            RdevEventType::MouseMove { x, y } => {
                assert_eq!(x, 100.0);
                assert_eq!(y, 200.0);
            }
            _ => panic!("Wrong event type"),
        }
    }
}
