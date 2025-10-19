use multishiva::core::network::Network;

#[tokio::test]
async fn test_network_creation() {
    let network = Network::new();
    // Basic smoke test - just verify we can create a Network instance
    drop(network);
}

#[tokio::test]
async fn test_network_host_start() {
    let network = Network::new();
    // Placeholder test - will be implemented with actual network logic
    let result = network.start_host(53421).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_network_agent_connect() {
    let network = Network::new();
    // Placeholder test - will be implemented with actual network logic
    let result = network.connect_agent("localhost").await;
    assert!(result.is_ok());
}
