use anyhow::{Context, Result};
use evdev::{Device, EventType, InputEventKind, Key as EvdevKey};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::core::events::{Event, Key, MouseButton};
use crate::core::input::InputHandler;

/// Linux-specific input handler using evdev for native Wayland/X11 support.
///
/// This implementation reads directly from /dev/input/event* devices,
/// which works on both Wayland and X11. Requires the user to be in the
/// 'input' group or run with appropriate permissions.
pub struct EvdevInputHandler {
    capturing: Arc<AtomicBool>,
    devices: Vec<PathBuf>,
    mouse_position: Arc<std::sync::RwLock<(i32, i32)>>,
}

impl EvdevInputHandler {
    /// Creates a new evdev input handler.
    ///
    /// Automatically detects available input devices (mouse and keyboard).
    pub fn new() -> Result<Self> {
        let devices = Self::detect_input_devices()?;

        if devices.is_empty() {
            tracing::warn!("No input devices detected. You may need to:");
            tracing::warn!(
                "  1. Add your user to the 'input' group: sudo usermod -a -G input $USER"
            );
            tracing::warn!("  2. Log out and log back in");
            tracing::warn!("  3. Or run with sudo (not recommended for production)");
        } else {
            tracing::info!("Detected {} input device(s)", devices.len());
            for device in &devices {
                tracing::debug!("  - {:?}", device);
            }
        }

        Ok(Self {
            capturing: Arc::new(AtomicBool::new(false)),
            devices,
            // Initialize mouse at center of screen (will be updated by real events)
            mouse_position: Arc::new(std::sync::RwLock::new((960, 540))),
        })
    }

    /// Detects all available input devices (keyboard and mouse).
    ///
    /// Scans /dev/input/event* and filters for devices that support
    /// keyboard or mouse events.
    fn detect_input_devices() -> Result<Vec<PathBuf>> {
        let mut devices = Vec::new();

        // Scan /dev/input/event* files
        for entry in
            std::fs::read_dir("/dev/input").context("Failed to read /dev/input directory")?
        {
            let entry = entry?;
            let path = entry.path();

            // Only check eventX files
            if let Some(name) = path.file_name() {
                if !name.to_string_lossy().starts_with("event") {
                    continue;
                }
            }

            // Try to open the device
            match Device::open(&path) {
                Ok(device) => {
                    let has_keyboard = device.supported_events().contains(EventType::KEY);
                    let has_mouse = device.supported_events().contains(EventType::RELATIVE)
                        || device.supported_events().contains(EventType::ABSOLUTE);

                    if has_keyboard || has_mouse {
                        let name = device.name().unwrap_or("Unknown");
                        tracing::debug!(
                            "Found input device: {} ({:?}) - Keyboard: {}, Mouse: {}",
                            name,
                            path,
                            has_keyboard,
                            has_mouse
                        );
                        devices.push(path);
                    }
                }
                Err(e) => {
                    tracing::trace!("Skipping {:?}: {}", path, e);
                }
            }
        }

        Ok(devices)
    }

    /// Converts an evdev event to our internal Event type.
    fn convert_evdev_event(
        kind: InputEventKind,
        value: i32,
        mouse_pos: &Arc<std::sync::RwLock<(i32, i32)>>,
    ) -> Option<Event> {
        match kind {
            // Mouse movement (relative) - accumulate deltas
            InputEventKind::RelAxis(evdev::RelativeAxisType::REL_X) => {
                if let Ok(mut pos) = mouse_pos.write() {
                    pos.0 += value;
                    // Clamp to screen bounds (TODO: get actual screen size)
                    pos.0 = pos.0.clamp(0, 1920);
                    Some(Event::MouseMove { x: pos.0, y: pos.1 })
                } else {
                    None
                }
            }
            InputEventKind::RelAxis(evdev::RelativeAxisType::REL_Y) => {
                if let Ok(mut pos) = mouse_pos.write() {
                    pos.1 += value;
                    // Clamp to screen bounds (TODO: get actual screen size)
                    pos.1 = pos.1.clamp(0, 1080);
                    Some(Event::MouseMove { x: pos.0, y: pos.1 })
                } else {
                    None
                }
            }

            // Mouse movement (absolute) - for touchpads/tablets
            InputEventKind::AbsAxis(evdev::AbsoluteAxisType::ABS_X) => {
                if let Ok(mut pos) = mouse_pos.write() {
                    pos.0 = value;
                    Some(Event::MouseMove { x: pos.0, y: pos.1 })
                } else {
                    None
                }
            }
            InputEventKind::AbsAxis(evdev::AbsoluteAxisType::ABS_Y) => {
                if let Ok(mut pos) = mouse_pos.write() {
                    pos.1 = value;
                    Some(Event::MouseMove { x: pos.0, y: pos.1 })
                } else {
                    None
                }
            }

            // Mouse buttons
            InputEventKind::Key(key) => {
                match key {
                    // Mouse buttons
                    EvdevKey::BTN_LEFT => Some(Event::MouseButtonPress {
                        button: MouseButton::Left,
                    }),
                    EvdevKey::BTN_RIGHT => Some(Event::MouseButtonPress {
                        button: MouseButton::Right,
                    }),
                    EvdevKey::BTN_MIDDLE => Some(Event::MouseButtonPress {
                        button: MouseButton::Middle,
                    }),

                    // Keyboard keys
                    _ => convert_evdev_key(key).map(|our_key| Event::KeyPress { key: our_key }),
                }
            }

            // Mouse wheel
            InputEventKind::RelAxis(evdev::RelativeAxisType::REL_WHEEL) => {
                Some(Event::MouseScroll {
                    delta_x: 0,
                    delta_y: value as i64,
                })
            }
            InputEventKind::RelAxis(evdev::RelativeAxisType::REL_HWHEEL) => {
                Some(Event::MouseScroll {
                    delta_x: value as i64,
                    delta_y: 0,
                })
            }

            _ => None,
        }
    }
}

impl Default for EvdevInputHandler {
    fn default() -> Self {
        Self::new().expect("Failed to create EvdevInputHandler")
    }
}

#[allow(async_fn_in_trait)]
impl InputHandler for EvdevInputHandler {
    async fn start_capture(&mut self, tx: mpsc::Sender<Event>) -> Result<()> {
        if self.capturing.load(Ordering::SeqCst) {
            return Ok(());
        }

        if self.devices.is_empty() {
            anyhow::bail!(
                "No input devices available. Please check permissions:\n\
                 1. sudo usermod -a -G input $USER\n\
                 2. Log out and log back in\n\
                 3. Or run with sudo (not recommended)"
            );
        }

        self.capturing.store(true, Ordering::SeqCst);
        let capturing = self.capturing.clone();
        let mouse_pos = self.mouse_position.clone();

        // Open devices
        let mut devices = Vec::new();
        for path in &self.devices {
            match Device::open(path) {
                Ok(device) => {
                    tracing::info!(
                        "Opened input device: {} ({:?})",
                        device.name().unwrap_or("Unknown"),
                        path
                    );
                    devices.push(device);
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to open {:?}: {}. Try: sudo usermod -a -G input $USER",
                        path,
                        e
                    );
                }
            }
        }

        if devices.is_empty() {
            anyhow::bail!("Could not open any input devices. Check permissions.");
        }

        // Create a standard channel for the evdev threads to communicate with tokio
        let (std_tx, std_rx) = std::sync::mpsc::channel::<Event>();

        // Spawn async task to forward events from std channel to tokio channel
        tokio::spawn(async move {
            tracing::debug!("evdev event bridge task started");
            loop {
                match std_rx.try_recv() {
                    Ok(event) => {
                        tracing::trace!("Bridge forwarding event: {:?}", event);
                        if tx.send(event).await.is_err() {
                            tracing::debug!("Receiver dropped, stopping bridge");
                            break;
                        }
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => {
                        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                    }
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        tracing::debug!("Std channel disconnected, stopping bridge");
                        break;
                    }
                }
            }
            tracing::debug!("evdev event bridge task exiting");
        });

        // Spawn capture thread for each device
        for mut device in devices {
            let std_tx = std_tx.clone();
            let capturing = capturing.clone();
            let mouse_pos = mouse_pos.clone();

            std::thread::spawn(move || {
                tracing::debug!(
                    "evdev capture thread started for {}",
                    device.name().unwrap_or("Unknown")
                );

                loop {
                    if !capturing.load(Ordering::SeqCst) {
                        tracing::debug!("Stopping evdev capture thread");
                        break;
                    }

                    match device.fetch_events() {
                        Ok(events) => {
                            for event in events {
                                tracing::trace!(
                                    "evdev raw event: {:?} value={}",
                                    event.kind(),
                                    event.value()
                                );

                                if let Some(our_event) = Self::convert_evdev_event(
                                    event.kind(),
                                    event.value(),
                                    &mouse_pos,
                                ) {
                                    tracing::debug!("Converted evdev event: {:?}", our_event);

                                    // Send through standard channel (non-async)
                                    if let Err(e) = std_tx.send(our_event) {
                                        tracing::error!(
                                            "Failed to send event through channel: {:?}",
                                            e
                                        );
                                        break;
                                    }
                                }
                            }
                        }
                        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            // No events available, sleep briefly
                            std::thread::sleep(std::time::Duration::from_millis(10));
                        }
                        Err(e) => {
                            tracing::error!("Error fetching evdev events: {:?}", e);
                            break;
                        }
                    }
                }

                tracing::debug!("evdev capture thread exiting");
            });
        }

        tracing::info!("✓ evdev input capture started");
        Ok(())
    }

    async fn stop_capture(&mut self) {
        self.capturing.store(false, Ordering::SeqCst);
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    async fn inject_event(&self, event: Event) -> Result<()> {
        // TODO: Implement uinput-based event injection
        tracing::warn!(
            "Event injection not yet implemented for evdev backend: {:?}",
            event
        );
        Ok(())
    }

    fn is_capturing(&self) -> bool {
        self.capturing.load(Ordering::SeqCst)
    }

    fn get_screen_size(&self) -> (u32, u32) {
        // Get screen size from environment variables or default
        // This is a simplified implementation
        // In production, query the display server (X11/Wayland)

        // Try to get from X11 if available
        #[cfg(target_os = "linux")]
        {
            // TODO: Query actual screen size from X11/Wayland
            (1920, 1080)
        }

        #[cfg(not(target_os = "linux"))]
        {
            (1920, 1080)
        }
    }

    fn get_cursor_position(&self) -> Result<(i32, i32)> {
        if let Ok(pos) = self.mouse_position.read() {
            Ok(*pos)
        } else {
            Ok((0, 0))
        }
    }

    fn check_permissions(&self) -> bool {
        // Check if we can access /dev/input devices
        !self.devices.is_empty()
    }
}

/// Converts an evdev key code to our internal Key representation.
fn convert_evdev_key(key: EvdevKey) -> Option<Key> {
    match key {
        // Letters
        EvdevKey::KEY_A => Some(Key::KeyA),
        EvdevKey::KEY_B => Some(Key::KeyB),
        EvdevKey::KEY_C => Some(Key::KeyC),
        EvdevKey::KEY_D => Some(Key::KeyD),
        EvdevKey::KEY_E => Some(Key::KeyE),
        EvdevKey::KEY_F => Some(Key::KeyF),
        EvdevKey::KEY_G => Some(Key::KeyG),
        EvdevKey::KEY_H => Some(Key::KeyH),
        EvdevKey::KEY_I => Some(Key::KeyI),
        EvdevKey::KEY_J => Some(Key::KeyJ),
        EvdevKey::KEY_K => Some(Key::KeyK),
        EvdevKey::KEY_L => Some(Key::KeyL),
        EvdevKey::KEY_M => Some(Key::KeyM),
        EvdevKey::KEY_N => Some(Key::KeyN),
        EvdevKey::KEY_O => Some(Key::KeyO),
        EvdevKey::KEY_P => Some(Key::KeyP),
        EvdevKey::KEY_Q => Some(Key::KeyQ),
        EvdevKey::KEY_R => Some(Key::KeyR),
        EvdevKey::KEY_S => Some(Key::KeyS),
        EvdevKey::KEY_T => Some(Key::KeyT),
        EvdevKey::KEY_U => Some(Key::KeyU),
        EvdevKey::KEY_V => Some(Key::KeyV),
        EvdevKey::KEY_W => Some(Key::KeyW),
        EvdevKey::KEY_X => Some(Key::KeyX),
        EvdevKey::KEY_Y => Some(Key::KeyY),
        EvdevKey::KEY_Z => Some(Key::KeyZ),

        // Modifiers
        EvdevKey::KEY_LEFTCTRL => Some(Key::ControlLeft),
        EvdevKey::KEY_RIGHTCTRL => Some(Key::ControlRight),
        EvdevKey::KEY_LEFTSHIFT => Some(Key::ShiftLeft),
        EvdevKey::KEY_RIGHTSHIFT => Some(Key::ShiftRight),
        EvdevKey::KEY_LEFTALT => Some(Key::AltLeft),
        EvdevKey::KEY_RIGHTALT => Some(Key::AltRight),
        EvdevKey::KEY_LEFTMETA => Some(Key::MetaLeft),
        EvdevKey::KEY_RIGHTMETA => Some(Key::MetaRight),

        // Special keys
        EvdevKey::KEY_ESC => Some(Key::Escape),
        EvdevKey::KEY_ENTER => Some(Key::Return),
        EvdevKey::KEY_SPACE => Some(Key::Space),
        EvdevKey::KEY_BACKSPACE => Some(Key::Backspace),
        EvdevKey::KEY_TAB => Some(Key::Tab),

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evdev_handler_creation() {
        // This test may fail if not in input group
        match EvdevInputHandler::new() {
            Ok(handler) => {
                assert!(!handler.is_capturing());
            }
            Err(e) => {
                println!(
                    "Note: EvdevInputHandler creation failed (expected if not in input group): {}",
                    e
                );
            }
        }
    }
}
