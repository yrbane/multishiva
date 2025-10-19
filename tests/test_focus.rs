use multishiva::core::focus::FocusManager;
use std::time::Duration;

#[tokio::test]
async fn test_focus_manager_creation() {
    let manager = FocusManager::new("host".to_string());
    assert_eq!(manager.current(), "host");
    assert!(!manager.has_focus("agent1"));
    assert!(manager.has_focus("host"));
}

#[tokio::test]
async fn test_focus_transfer() {
    let mut manager = FocusManager::new("host".to_string());

    let result = manager.transfer_focus("agent1".to_string(), 100, 200).await;
    assert!(result.is_ok());
    assert_eq!(manager.current(), "agent1");
    assert!(manager.has_focus("agent1"));
    assert!(!manager.has_focus("host"));
}

#[tokio::test]
async fn test_focus_transfer_with_position() {
    let mut manager = FocusManager::new("host".to_string());

    manager
        .transfer_focus("agent1".to_string(), 500, 300)
        .await
        .unwrap();

    let (x, y) = manager.current_position();
    assert_eq!(x, 500);
    assert_eq!(y, 300);
}

#[tokio::test]
async fn test_focus_transfer_multiple() {
    let mut manager = FocusManager::new("host".to_string());

    manager
        .transfer_focus("agent1".to_string(), 0, 0)
        .await
        .unwrap();
    assert_eq!(manager.current(), "agent1");

    manager
        .transfer_focus("agent2".to_string(), 0, 0)
        .await
        .unwrap();
    assert_eq!(manager.current(), "agent2");

    manager
        .transfer_focus("host".to_string(), 0, 0)
        .await
        .unwrap();
    assert_eq!(manager.current(), "host");
}

#[tokio::test]
async fn test_focus_return_to_host() {
    let mut manager = FocusManager::new("host".to_string());

    manager
        .transfer_focus("agent1".to_string(), 0, 0)
        .await
        .unwrap();
    assert_eq!(manager.current(), "agent1");

    manager.return_to_host().await.unwrap();
    assert_eq!(manager.current(), "host");
}

#[tokio::test]
async fn test_focus_history() {
    let mut manager = FocusManager::new("host".to_string());

    manager
        .transfer_focus("agent1".to_string(), 0, 0)
        .await
        .unwrap();
    manager
        .transfer_focus("agent2".to_string(), 0, 0)
        .await
        .unwrap();

    let history = manager.focus_history();
    assert_eq!(history.len(), 3); // host, agent1, agent2
    assert_eq!(history[0], "host");
    assert_eq!(history[1], "agent1");
    assert_eq!(history[2], "agent2");
}

#[tokio::test]
async fn test_focus_transfer_with_friction() {
    let mut manager = FocusManager::new("host".to_string());
    manager.set_friction_ms(50);

    let start = std::time::Instant::now();
    manager
        .transfer_focus("agent1".to_string(), 0, 0)
        .await
        .unwrap();
    let elapsed = start.elapsed();

    // Should have waited at least friction_ms
    assert!(elapsed >= Duration::from_millis(50));
}

#[tokio::test]
async fn test_focus_no_self_transfer() {
    let mut manager = FocusManager::new("host".to_string());

    // Transferring to current focus should be a no-op
    let result = manager.transfer_focus("host".to_string(), 0, 0).await;
    assert!(result.is_ok());
    assert_eq!(manager.current(), "host");
}
