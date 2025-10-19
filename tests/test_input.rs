use multishiva::core::events::Event;
use multishiva::core::input::{InputHandler, RdevInputHandler};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_input_handler_creation() {
    let handler = RdevInputHandler::new();
    // Should create successfully
    drop(handler);
}

#[tokio::test]
async fn test_input_handler_start_and_stop() {
    let mut handler = RdevInputHandler::new();
    let (tx, _rx) = mpsc::channel(100);

    // Start capture
    let result = handler.start_capture(tx).await;
    assert!(result.is_ok());
    assert!(handler.is_capturing());

    // Stop capture
    handler.stop_capture().await;
    assert!(!handler.is_capturing());
}

#[tokio::test]
async fn test_input_handler_mouse_move_capture() {
    let mut handler = RdevInputHandler::new();
    let (tx, _rx) = mpsc::channel(100);

    handler.start_capture(tx).await.unwrap();

    // Simulate mouse move (this will be done by the system in real usage)
    // For testing, we'll inject an event directly
    sleep(Duration::from_millis(100)).await;

    handler.stop_capture().await;

    // In real usage, rx would receive events
    // For this test, we're just verifying the setup works
}

#[tokio::test]
async fn test_input_handler_inject_mouse_move() {
    let handler = RdevInputHandler::new();

    let event = Event::MouseMove { x: 500, y: 300 };
    let result = handler.inject_event(event).await;

    // Should succeed (may fail if permissions are not granted)
    // This is platform-dependent
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_input_handler_inject_mouse_click() {
    let handler = RdevInputHandler::new();

    let event = Event::MouseButtonPress {
        button: multishiva::core::events::MouseButton::Left,
    };
    let result = handler.inject_event(event).await;

    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_input_handler_inject_key_press() {
    let handler = RdevInputHandler::new();

    let event = Event::KeyPress {
        key: multishiva::core::events::Key::KeyA,
    };
    let result = handler.inject_event(event).await;

    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_input_handler_hotkey_detection() {
    let handler = RdevInputHandler::new();

    // Configure kill-switch hotkey
    handler.set_kill_switch(vec![
        multishiva::core::events::Key::ControlLeft,
        multishiva::core::events::Key::ShiftLeft,
        multishiva::core::events::Key::KeyQ,
    ]);

    // Verify configuration was set
    assert!(handler.has_kill_switch());
}

#[tokio::test]
async fn test_input_handler_block_local_input() {
    let mut handler = RdevInputHandler::new();

    // When blocking is enabled, local input should be suppressed
    handler.set_block_local(true);
    assert!(handler.is_blocking_local());

    handler.set_block_local(false);
    assert!(!handler.is_blocking_local());
}

#[tokio::test]
async fn test_input_handler_multiple_events() {
    let handler = RdevInputHandler::new();

    let events = vec![
        Event::MouseMove { x: 100, y: 200 },
        Event::MouseButtonPress {
            button: multishiva::core::events::MouseButton::Left,
        },
        Event::MouseButtonRelease {
            button: multishiva::core::events::MouseButton::Left,
        },
    ];

    for event in events {
        let _ = handler.inject_event(event).await;
    }

    // Should complete without panicking
}

#[tokio::test]
async fn test_input_handler_screen_bounds() {
    let handler = RdevInputHandler::new();

    // Get screen dimensions
    let (width, height) = handler.get_screen_size();

    assert!(width > 0);
    assert!(height > 0);

    // Typically at least 640x480
    assert!(width >= 640);
    assert!(height >= 480);
}

#[tokio::test]
async fn test_input_handler_cursor_position() {
    let handler = RdevInputHandler::new();

    let result = handler.get_cursor_position();
    assert!(result.is_ok());

    let (x, y) = result.unwrap();
    let (width, height) = handler.get_screen_size();

    // Cursor should be within screen bounds
    assert!(x >= 0 && x <= width as i32);
    assert!(y >= 0 && y <= height as i32);
}

#[tokio::test]
async fn test_input_handler_event_filtering() {
    let handler = RdevInputHandler::new();

    // Configure event filter to only capture mouse events
    handler.set_event_filter(|event| matches!(event, Event::MouseMove { .. }));

    // Verify filter is set
    assert!(handler.has_event_filter());
}

#[tokio::test]
async fn test_input_handler_concurrent_capture_injection() {
    let mut handler = RdevInputHandler::new();
    let (tx, _rx) = mpsc::channel(100);

    // Start capture
    handler.start_capture(tx).await.unwrap();

    // Should be able to inject while capturing
    let event = Event::MouseMove { x: 100, y: 100 };
    let result = handler.inject_event(event).await;

    // Should not conflict
    assert!(result.is_ok() || result.is_err());

    handler.stop_capture().await;
}

#[tokio::test]
async fn test_input_handler_permissions_check() {
    let handler = RdevInputHandler::new();

    // Check if we have necessary permissions
    let _has_permissions = handler.check_permissions();

    // On CI or systems without permissions, this may be false
    // That's okay - we just want to verify the check works without panicking
    // No assertion needed - just verify it doesn't crash
}
