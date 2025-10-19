use multishiva::core::focus::FocusManager;

#[tokio::test]
async fn test_focus_manager_creation() {
    let manager = FocusManager::new("host".to_string());
    assert_eq!(manager.current(), "host");
}

#[tokio::test]
async fn test_focus_transfer() {
    let mut manager = FocusManager::new("host".to_string());

    let result = manager.transfer_focus("agent1".to_string()).await;
    assert!(result.is_ok());
    assert_eq!(manager.current(), "agent1");
}

#[tokio::test]
async fn test_focus_transfer_multiple() {
    let mut manager = FocusManager::new("host".to_string());

    manager.transfer_focus("agent1".to_string()).await.unwrap();
    assert_eq!(manager.current(), "agent1");

    manager.transfer_focus("agent2".to_string()).await.unwrap();
    assert_eq!(manager.current(), "agent2");

    manager.transfer_focus("host".to_string()).await.unwrap();
    assert_eq!(manager.current(), "host");
}
