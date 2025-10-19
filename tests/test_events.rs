use multishiva::core::events::{Event, Key, MouseButton};

#[test]
fn test_event_mouse_move_serialization() {
    let event = Event::MouseMove { x: 100, y: 200 };
    let serialized = rmp_serde::to_vec(&event).unwrap();
    let deserialized: Event = rmp_serde::from_slice(&serialized).unwrap();

    match deserialized {
        Event::MouseMove { x, y } => {
            assert_eq!(x, 100);
            assert_eq!(y, 200);
        }
        _ => panic!("Wrong event type"),
    }
}

#[test]
fn test_event_mouse_click_serialization() {
    let event = Event::MouseClick {
        button: MouseButton::Left,
    };
    let serialized = rmp_serde::to_vec(&event).unwrap();
    let deserialized: Event = rmp_serde::from_slice(&serialized).unwrap();

    match deserialized {
        Event::MouseClick { button } => {
            assert!(matches!(button, MouseButton::Left));
        }
        _ => panic!("Wrong event type"),
    }
}

#[test]
fn test_event_key_press_serialization() {
    let event = Event::KeyPress { key: Key::KeyA };
    let serialized = rmp_serde::to_vec(&event).unwrap();
    let deserialized: Event = rmp_serde::from_slice(&serialized).unwrap();

    match deserialized {
        Event::KeyPress { key } => {
            assert_eq!(key, Key::KeyA);
        }
        _ => panic!("Wrong event type"),
    }
}

#[test]
fn test_event_focus_grant_serialization() {
    let event = Event::FocusGrant {
        target: "agent1".to_string(),
        x: 50,
        y: 100,
    };
    let serialized = rmp_serde::to_vec(&event).unwrap();
    let deserialized: Event = rmp_serde::from_slice(&serialized).unwrap();

    match deserialized {
        Event::FocusGrant { target, x, y } => {
            assert_eq!(target, "agent1");
            assert_eq!(x, 50);
            assert_eq!(y, 100);
        }
        _ => panic!("Wrong event type"),
    }
}

#[test]
fn test_event_heartbeat_serialization() {
    let event = Event::Heartbeat;
    let serialized = rmp_serde::to_vec(&event).unwrap();
    let deserialized: Event = rmp_serde::from_slice(&serialized).unwrap();

    assert!(matches!(deserialized, Event::Heartbeat));
}

#[test]
fn test_event_serialization_size() {
    // Verify events are compact (important for network efficiency)
    let event = Event::MouseMove { x: 100, y: 200 };
    let serialized = rmp_serde::to_vec(&event).unwrap();

    // MessagePack should be very compact (< 20 bytes for MouseMove)
    assert!(serialized.len() < 20);
}

#[test]
fn test_all_mouse_buttons() {
    for button in [MouseButton::Left, MouseButton::Right, MouseButton::Middle] {
        let event = Event::MouseClick {
            button: button.clone(),
        };
        let serialized = rmp_serde::to_vec(&event).unwrap();
        let _deserialized: Event = rmp_serde::from_slice(&serialized).unwrap();
    }
}

#[test]
fn test_event_focus_release() {
    let event = Event::FocusRelease;
    let serialized = rmp_serde::to_vec(&event).unwrap();
    let deserialized: Event = rmp_serde::from_slice(&serialized).unwrap();

    assert!(matches!(deserialized, Event::FocusRelease));
}
