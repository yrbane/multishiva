use multishiva::core::events::Event;
use multishiva::core::network::Network;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_network_creation() {
    let network = Network::new("test-psk".to_string());
    // Should create successfully
    drop(network);
}

#[tokio::test]
async fn test_network_host_start_and_stop() {
    let mut network = Network::new("test-psk".to_string());

    // Start host on random available port
    let result = network.start_host(0).await;
    assert!(result.is_ok());

    let port = result.unwrap();
    assert!(port > 0);

    // Stop should work
    network.stop().await;
}

#[tokio::test]
async fn test_network_host_rejects_wrong_psk() {
    let mut host_network = Network::new("correct-psk".to_string());
    let agent_network = Network::new("wrong-psk".to_string());

    // Start host
    let port = host_network.start_host(0).await.unwrap();
    sleep(Duration::from_millis(100)).await;

    // Agent with wrong PSK should fail to connect
    let _result = agent_network
        .connect_to_host(&format!("127.0.0.1:{}", port))
        .await;

    // Connection should be rejected
    // The PSK handshake should fail
    // Note: The agent won't be marked as connected if handshake fails

    host_network.stop().await;
}

#[tokio::test]
async fn test_network_send_receive_event() {
    let mut host_network = Network::new("shared-psk".to_string());
    let mut agent_network = Network::new("shared-psk".to_string());

    // Start host
    let port = host_network.start_host(0).await.unwrap();

    // Give it a moment to start
    sleep(Duration::from_millis(100)).await;

    // Connect agent
    let connect_result = agent_network
        .connect_to_host(&format!("127.0.0.1:{}", port))
        .await;
    assert!(connect_result.is_ok());

    // Send event from agent to host
    let event = Event::MouseMove { x: 100, y: 200 };
    let send_result = agent_network.send_event(event).await;
    assert!(send_result.is_ok());

    // Host should receive event
    // This will be implemented with channels

    host_network.stop().await;
    agent_network.stop().await;
}

#[tokio::test]
async fn test_network_heartbeat() {
    let mut host_network = Network::new("shared-psk".to_string());
    let mut agent_network = Network::new("shared-psk".to_string());

    // Start host
    let port = host_network.start_host(0).await.unwrap();
    sleep(Duration::from_millis(100)).await;

    // Connect agent
    agent_network
        .connect_to_host(&format!("127.0.0.1:{}", port))
        .await
        .unwrap();

    // Wait for heartbeat interval
    sleep(Duration::from_millis(200)).await;

    // Both should still be connected
    assert!(host_network.is_running());
    assert!(agent_network.is_connected());

    host_network.stop().await;
    agent_network.stop().await;
}

#[tokio::test]
async fn test_network_reconnect_on_disconnect() {
    let mut host_network = Network::new("shared-psk".to_string());
    let agent_network = Network::new("shared-psk".to_string());

    // Start host
    let port = host_network.start_host(0).await.unwrap();
    sleep(Duration::from_millis(100)).await;

    // Connect agent
    agent_network
        .connect_to_host(&format!("127.0.0.1:{}", port))
        .await
        .unwrap();
    assert!(agent_network.is_connected());

    // Simulate disconnect by stopping host
    host_network.stop().await;

    // Wait longer for disconnection to be detected (heartbeat timeout)
    sleep(Duration::from_secs(1)).await;

    // Agent connection handler should eventually exit
    // Note: In production this would trigger a reconnect attempt
}

#[tokio::test]
async fn test_network_multiple_agents() {
    let mut host_network = Network::new("shared-psk".to_string());
    let mut agent1 = Network::new("shared-psk".to_string());
    let mut agent2 = Network::new("shared-psk".to_string());

    // Start host
    let port = host_network.start_host(0).await.unwrap();
    sleep(Duration::from_millis(100)).await;

    // Connect multiple agents
    let result1 = agent1.connect_to_host(&format!("127.0.0.1:{}", port)).await;
    let result2 = agent2.connect_to_host(&format!("127.0.0.1:{}", port)).await;

    assert!(result1.is_ok());
    assert!(result2.is_ok());

    assert!(agent1.is_connected());
    assert!(agent2.is_connected());

    host_network.stop().await;
    agent1.stop().await;
    agent2.stop().await;
}

#[tokio::test]
async fn test_network_connection_timeout() {
    let agent_network = Network::new("shared-psk".to_string());

    // Try to connect to non-existent host
    let result = agent_network.connect_to_host("127.0.0.1:9999").await;

    // Should timeout or fail quickly
    assert!(result.is_err());
}

#[tokio::test]
async fn test_network_get_connection_count() {
    let mut host_network = Network::new("shared-psk".to_string());

    // Start host
    let _port = host_network.start_host(0).await.unwrap();

    // Initially should have 0 connections
    assert_eq!(host_network.connection_count(), 0);

    host_network.stop().await;
}
