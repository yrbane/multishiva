/// mDNS-based auto-discovery for MultiShiva machines
///
/// This module provides automatic discovery of MultiShiva instances
/// on the local network using mDNS (Multicast DNS) service discovery.
///
/// Features:
/// - Service registration (broadcast presence)
/// - Service browsing (discover peers)
/// - Automatic peer tracking
/// - Event-driven notifications
use anyhow::{Context, Result};
use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};

/// MultiShiva mDNS service type
pub const SERVICE_TYPE: &str = "_multishiva._tcp.local.";

/// Discovered peer information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerInfo {
    /// Machine name (instance name)
    pub name: String,
    /// IP address
    pub address: IpAddr,
    /// Port number
    pub port: u16,
    /// TLS PSK hash (for verification)
    pub psk_hash: Option<String>,
    /// Additional properties
    pub properties: HashMap<String, String>,
}

impl PeerInfo {
    /// Create a new PeerInfo
    pub fn new(name: String, address: IpAddr, port: u16) -> Self {
        Self {
            name,
            address,
            port,
            psk_hash: None,
            properties: HashMap::new(),
        }
    }

    /// Get the full address (IP:port)
    pub fn full_address(&self) -> String {
        format!("{}:{}", self.address, self.port)
    }
}

/// Discovery system for MultiShiva
pub struct Discovery {
    daemon: ServiceDaemon,
    service_name: String,
    peers: Arc<Mutex<HashMap<String, PeerInfo>>>,
}

impl Discovery {
    /// Create a new discovery instance
    pub fn new(service_name: String) -> Result<Self> {
        let daemon = ServiceDaemon::new().context("Failed to create mDNS service daemon")?;

        Ok(Self {
            daemon,
            service_name,
            peers: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Register this machine as a MultiShiva service
    ///
    /// # Arguments
    /// * `port` - Port number for the MultiShiva service
    /// * `psk_hash` - Optional TLS PSK hash for verification
    /// * `properties` - Additional service properties
    pub fn register(
        &self,
        port: u16,
        psk_hash: Option<String>,
        properties: HashMap<String, String>,
    ) -> Result<()> {
        let hostname = hostname::get()
            .context("Failed to get hostname")?
            .to_string_lossy()
            .to_string();

        // Build properties including PSK hash
        let mut props = properties;
        if let Some(hash) = psk_hash {
            props.insert("psk_hash".to_string(), hash);
        }

        // Create service info
        let service_info = ServiceInfo::new(
            SERVICE_TYPE,
            &self.service_name,
            &hostname,
            "", // Address will be auto-detected
            port,
            Some(props),
        )
        .context("Failed to create service info")?;

        // Register service
        self.daemon
            .register(service_info)
            .context("Failed to register mDNS service")?;

        tracing::info!(
            "Registered mDNS service: {} on port {}",
            self.service_name,
            port
        );

        Ok(())
    }

    /// Unregister this machine's service
    pub fn unregister(&self) -> Result<()> {
        let full_name = format!("{}.{}", self.service_name, SERVICE_TYPE);
        self.daemon
            .unregister(&full_name)
            .context("Failed to unregister mDNS service")?;

        tracing::info!("Unregistered mDNS service: {}", self.service_name);
        Ok(())
    }

    /// Start browsing for MultiShiva services
    ///
    /// This method starts a background task that continuously browses
    /// for MultiShiva services on the network and updates the peer list.
    pub fn start_browsing(&self) -> Result<()> {
        let receiver = self
            .daemon
            .browse(SERVICE_TYPE)
            .context("Failed to start browsing for services")?;

        let peers = Arc::clone(&self.peers);
        let service_name = self.service_name.clone();

        // Spawn background task to handle service events
        std::thread::spawn(move || {
            for event in receiver.iter() {
                match event {
                    ServiceEvent::ServiceResolved(info) => {
                        // Skip self
                        if info
                            .get_fullname()
                            .starts_with(&format!("{}.", service_name))
                        {
                            continue;
                        }

                        // Extract peer information
                        let name = info
                            .get_fullname()
                            .split('.')
                            .next()
                            .unwrap_or("unknown")
                            .to_string();

                        if let Some(address) = info.get_addresses().iter().next() {
                            let port = info.get_port();
                            let psk_hash =
                                info.get_property_val_str("psk_hash").map(|s| s.to_string());

                            let mut properties = HashMap::new();
                            for prop in info.get_properties().iter() {
                                let key = prop.key();
                                if key != "psk_hash" {
                                    let value = prop.val_str();
                                    properties.insert(key.to_string(), value.to_string());
                                }
                            }

                            let peer = PeerInfo {
                                name: name.clone(),
                                address: *address,
                                port,
                                psk_hash,
                                properties,
                            };

                            // Add to peers list
                            if let Ok(mut peers) = peers.lock() {
                                peers.insert(name.clone(), peer.clone());
                                tracing::info!(
                                    "Discovered peer: {} at {}",
                                    name,
                                    peer.full_address()
                                );
                            }
                        }
                    }
                    ServiceEvent::ServiceRemoved(_, fullname) => {
                        let name = fullname.split('.').next().unwrap_or("unknown");
                        if let Ok(mut peers) = peers.lock() {
                            if peers.remove(name).is_some() {
                                tracing::info!("Peer removed: {}", name);
                            }
                        }
                    }
                    _ => {}
                }
            }
        });

        tracing::info!("Started browsing for MultiShiva services");
        Ok(())
    }

    /// Get list of discovered peers
    pub fn get_peers(&self) -> Vec<PeerInfo> {
        self.peers
            .lock()
            .map(|peers| peers.values().cloned().collect())
            .unwrap_or_default()
    }

    /// Get a specific peer by name
    pub fn get_peer(&self, name: &str) -> Option<PeerInfo> {
        self.peers
            .lock()
            .ok()
            .and_then(|peers| peers.get(name).cloned())
    }

    /// Check if a peer is currently discovered
    pub fn has_peer(&self, name: &str) -> bool {
        self.peers
            .lock()
            .map(|peers| peers.contains_key(name))
            .unwrap_or(false)
    }

    /// Clear all discovered peers
    pub fn clear_peers(&self) {
        if let Ok(mut peers) = self.peers.lock() {
            peers.clear();
        }
    }

    /// Stop the discovery system
    pub fn shutdown(&self) -> Result<()> {
        let receiver = self
            .daemon
            .shutdown()
            .context("Failed to initiate mDNS daemon shutdown")?;

        // Wait for shutdown confirmation
        let _ = receiver.recv();
        Ok(())
    }
}

impl Drop for Discovery {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_info_creation() {
        let peer = PeerInfo::new(
            "agent1".to_string(),
            "192.168.1.100".parse().unwrap(),
            53421,
        );

        assert_eq!(peer.name, "agent1");
        assert_eq!(peer.port, 53421);
        assert_eq!(peer.full_address(), "192.168.1.100:53421");
        assert!(peer.psk_hash.is_none());
    }

    #[test]
    fn test_peer_info_with_properties() {
        let mut peer = PeerInfo::new(
            "agent2".to_string(),
            "192.168.1.101".parse().unwrap(),
            53421,
        );

        peer.psk_hash = Some("abc123".to_string());
        peer.properties
            .insert("mode".to_string(), "agent".to_string());

        assert_eq!(peer.psk_hash.as_deref(), Some("abc123"));
        assert_eq!(
            peer.properties.get("mode").map(|s| s.as_str()),
            Some("agent")
        );
    }

    #[test]
    fn test_discovery_creation() {
        let discovery = Discovery::new("test-host".to_string());
        assert!(discovery.is_ok());

        let discovery = discovery.unwrap();
        assert_eq!(discovery.service_name, "test-host");
        assert_eq!(discovery.get_peers().len(), 0);
    }

    #[test]
    fn test_peer_list_operations() {
        let discovery = Discovery::new("test-host".to_string()).unwrap();

        // Initially empty
        assert_eq!(discovery.get_peers().len(), 0);
        assert!(!discovery.has_peer("agent1"));
        assert!(discovery.get_peer("agent1").is_none());

        // Manually add a peer for testing
        {
            let mut peers = discovery.peers.lock().unwrap();
            let peer = PeerInfo::new(
                "agent1".to_string(),
                "192.168.1.100".parse().unwrap(),
                53421,
            );
            peers.insert("agent1".to_string(), peer);
        }

        // Check peer operations
        assert_eq!(discovery.get_peers().len(), 1);
        assert!(discovery.has_peer("agent1"));
        assert!(discovery.get_peer("agent1").is_some());

        // Clear peers
        discovery.clear_peers();
        assert_eq!(discovery.get_peers().len(), 0);
    }

    #[test]
    fn test_service_type_constant() {
        assert_eq!(SERVICE_TYPE, "_multishiva._tcp.local.");
    }

    // Note: Integration tests for actual mDNS registration/browsing
    // are difficult to test in CI environments without network access.
    // These should be tested manually on a local network.
}
