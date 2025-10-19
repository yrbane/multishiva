use anyhow::{Context, Result};
use rdev::{simulate, Button, EventType as RdevEventType, Key as RdevKey};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock as StdRwLock};
use tokio::sync::mpsc;

use crate::core::events::{Event, Key, MouseButton};

type EventFilter = Box<dyn Fn(&Event) -> bool + Send + Sync>;

#[allow(async_fn_in_trait)]
pub trait InputHandler: Send + Sync {
    async fn start_capture(&mut self, tx: mpsc::Sender<Event>) -> Result<()>;
    async fn stop_capture(&mut self);
    async fn inject_event(&self, event: Event) -> Result<()>;
    fn is_capturing(&self) -> bool;
    fn get_screen_size(&self) -> (u32, u32);
    fn get_cursor_position(&self) -> Result<(i32, i32)>;
    fn check_permissions(&self) -> bool;
}

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
    pub fn new() -> Self {
        Self {
            capturing: Arc::new(AtomicBool::new(false)),
            block_local: Arc::new(AtomicBool::new(false)),
            kill_switch: Arc::new(StdRwLock::new(None)),
            event_filter: Arc::new(StdRwLock::new(None)),
            pressed_keys: Arc::new(StdRwLock::new(Vec::new())),
        }
    }

    pub fn set_kill_switch(&self, keys: Vec<Key>) {
        if let Ok(mut lock) = self.kill_switch.write() {
            *lock = Some(keys);
        }
    }

    pub fn has_kill_switch(&self) -> bool {
        if let Ok(lock) = self.kill_switch.read() {
            lock.is_some()
        } else {
            false
        }
    }

    pub fn set_block_local(&mut self, block: bool) {
        self.block_local.store(block, Ordering::SeqCst);
    }

    pub fn is_blocking_local(&self) -> bool {
        self.block_local.load(Ordering::SeqCst)
    }

    pub fn set_event_filter<F>(&self, filter: F)
    where
        F: Fn(&Event) -> bool + Send + Sync + 'static,
    {
        if let Ok(mut lock) = self.event_filter.write() {
            *lock = Some(Box::new(filter));
        }
    }

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

fn convert_rdev_button(button: Button) -> Option<MouseButton> {
    match button {
        Button::Left => Some(MouseButton::Left),
        Button::Right => Some(MouseButton::Right),
        Button::Middle => Some(MouseButton::Middle),
        _ => None,
    }
}

fn convert_button_to_rdev(button: &MouseButton) -> Option<Button> {
    match button {
        MouseButton::Left => Some(Button::Left),
        MouseButton::Right => Some(Button::Right),
        MouseButton::Middle => Some(Button::Middle),
    }
}

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
