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

/// MultiShiva mDNS service type identifier.
///
/// This constant defines the service type used for mDNS service discovery
/// following the RFC 6763 DNS-SD standard. The format is `_service._proto.domain.`
/// where:
/// - `multishiva` is the service name
/// - `tcp` is the transport protocol
/// - `local` is the domain for link-local multicast DNS
pub const SERVICE_TYPE: &str = "_multishiva._tcp.local.";

/// Information about a discovered MultiShiva peer on the network.
///
/// This structure contains all the information needed to connect to and
/// verify a discovered MultiShiva instance, including network address,
/// security credentials, and additional metadata.
///
/// # Examples
///
/// ```
/// use std::net::IpAddr;
/// use multishiva::core::discovery::PeerInfo;
///
/// let peer = PeerInfo::new(
///     "agent-001".to_string(),
///     "192.168.1.100".parse::<IpAddr>().unwrap(),
///     53421,
/// );
///
/// println!("Peer {} is at {}", peer.name, peer.full_address());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerInfo {
    /// Machine name (instance name) of the discovered peer.
    ///
    /// This is the unique identifier used in mDNS service registration.
    pub name: String,

    /// IP address where the peer can be reached.
    ///
    /// This can be either IPv4 or IPv6 depending on network configuration.
    pub address: IpAddr,

    /// TCP port number where the MultiShiva service is listening.
    pub port: u16,

    /// Optional TLS PSK (Pre-Shared Key) hash for verification.
    ///
    /// When present, this hash can be used to verify that the peer
    /// shares the same security credentials before establishing a connection.
    pub psk_hash: Option<String>,

    /// Additional service properties published by the peer.
    ///
    /// These are key-value pairs that can contain arbitrary metadata
    /// about the peer's capabilities or configuration.
    pub properties: HashMap<String, String>,
}

impl PeerInfo {
    /// Creates a new `PeerInfo` instance with minimal information.
    ///
    /// The PSK hash and properties are initialized as empty and can be
    /// set later if needed.
    ///
    /// # Arguments
    ///
    /// * `name` - The machine name of the peer
    /// * `address` - The IP address of the peer
    /// * `port` - The port number where the service is listening
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::IpAddr;
    /// use multishiva::core::discovery::PeerInfo;
    ///
    /// let peer = PeerInfo::new(
    ///     "controller".to_string(),
    ///     "192.168.1.1".parse::<IpAddr>().unwrap(),
    ///     8080,
    /// );
    /// assert_eq!(peer.name, "controller");
    /// assert_eq!(peer.port, 8080);
    /// ```
    pub fn new(name: String, address: IpAddr, port: u16) -> Self {
        Self {
            name,
            address,
            port,
            psk_hash: None,
            properties: HashMap::new(),
        }
    }

    /// Returns the full network address in "IP:port" format.
    ///
    /// This is a convenience method for displaying or logging the peer's
    /// complete network address.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::IpAddr;
    /// use multishiva::core::discovery::PeerInfo;
    ///
    /// let peer = PeerInfo::new(
    ///     "agent".to_string(),
    ///     "10.0.0.5".parse::<IpAddr>().unwrap(),
    ///     3000,
    /// );
    /// assert_eq!(peer.full_address(), "10.0.0.5:3000");
    /// ```
    pub fn full_address(&self) -> String {
        format!("{}:{}", self.address, self.port)
    }
}

/// mDNS-based service discovery system for MultiShiva instances.
///
/// The `Discovery` struct manages both service registration (announcing this
/// instance's presence) and service browsing (finding other instances on the
/// local network). It uses mDNS (Multicast DNS) for zero-configuration networking.
///
/// # Examples
///
/// ```no_run
/// use multishiva::core::discovery::Discovery;
/// use std::collections::HashMap;
///
/// // Create a discovery instance
/// let discovery = Discovery::new("my-machine".to_string())?;
///
/// // Register this instance
/// discovery.register(8080, None, HashMap::new())?;
///
/// // Start discovering peers
/// discovery.start_browsing()?;
///
/// // Wait a bit for discovery
/// std::thread::sleep(std::time::Duration::from_secs(2));
///
/// // Get discovered peers
/// let peers = discovery.get_peers();
/// println!("Found {} peers", peers.len());
/// # Ok::<(), anyhow::Error>(())
/// ```
pub struct Discovery {
    daemon: ServiceDaemon,
    service_name: String,
    peers: Arc<Mutex<HashMap<String, PeerInfo>>>,
}

impl Discovery {
    /// Creates a new `Discovery` instance.
    ///
    /// This initializes the underlying mDNS service daemon and prepares
    /// the discovery system for operation. No network activity occurs until
    /// `register()` or `start_browsing()` is called.
    ///
    /// # Arguments
    ///
    /// * `service_name` - The unique name for this service instance
    ///
    /// # Errors
    ///
    /// Returns an error if the mDNS service daemon cannot be initialized,
    /// which may happen if:
    /// - The system doesn't support multicast
    /// - Network interfaces are not available
    /// - Insufficient permissions to bind to multicast addresses
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use multishiva::core::discovery::Discovery;
    ///
    /// let discovery = Discovery::new("my-instance".to_string())?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn new(service_name: String) -> Result<Self> {
        let daemon = ServiceDaemon::new().context("Failed to create mDNS service daemon")?;

        Ok(Self {
            daemon,
            service_name,
            peers: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Registers this instance as a discoverable MultiShiva service on the network.
    ///
    /// This broadcasts the service's presence using mDNS, making it discoverable
    /// by other MultiShiva instances on the same local network. The service will
    /// remain advertised until `unregister()` is called or the `Discovery` instance
    /// is dropped.
    ///
    /// # Arguments
    ///
    /// * `port` - The TCP port number where this instance is listening
    /// * `psk_hash` - Optional Pre-Shared Key hash for TLS verification
    /// * `properties` - Additional key-value properties to advertise
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The hostname cannot be determined
    /// - The service information is invalid
    /// - The mDNS service registration fails
    /// - Another service is already registered with the same name
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use multishiva::core::discovery::Discovery;
    /// use std::collections::HashMap;
    ///
    /// let discovery = Discovery::new("my-machine".to_string())?;
    ///
    /// let mut props = HashMap::new();
    /// props.insert("version".to_string(), "1.0.0".to_string());
    ///
    /// discovery.register(8080, Some("abc123hash".to_string()), props)?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
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

        // mDNS requires hostname to end with .local.
        let mdns_hostname = if hostname.ends_with(".local.") {
            hostname
        } else {
            format!("{}.local.", hostname)
        };

        // Build properties including PSK hash
        let mut props = properties;
        if let Some(hash) = psk_hash {
            props.insert("psk_hash".to_string(), hash);
        }

        // Create service info
        let service_info = ServiceInfo::new(
            SERVICE_TYPE,
            &self.service_name,
            &mdns_hostname,
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

    /// Unregisters this instance's service from the network.
    ///
    /// This stops broadcasting the service's presence via mDNS. After calling
    /// this method, other instances will no longer discover this service
    /// (though it may take some time for the information to expire from their
    /// caches).
    ///
    /// # Errors
    ///
    /// Returns an error if the service cannot be unregistered, which may happen if:
    /// - The service was not previously registered
    /// - The mDNS daemon encountered an error
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use multishiva::core::discovery::Discovery;
    /// use std::collections::HashMap;
    ///
    /// let discovery = Discovery::new("my-machine".to_string())?;
    /// discovery.register(8080, None, HashMap::new())?;
    ///
    /// // Later, when shutting down
    /// discovery.unregister()?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn unregister(&self) -> Result<()> {
        let full_name = format!("{}.{}", self.service_name, SERVICE_TYPE);
        self.daemon
            .unregister(&full_name)
            .context("Failed to unregister mDNS service")?;

        tracing::info!("Unregistered mDNS service: {}", self.service_name);
        Ok(())
    }

    /// Starts actively browsing for MultiShiva services on the network.
    ///
    /// This spawns a background thread that continuously listens for mDNS
    /// service announcements. When peers are discovered, they are automatically
    /// added to the internal peer list and can be retrieved using `get_peers()`.
    /// The browsing continues until the `Discovery` instance is dropped or
    /// `shutdown()` is called.
    ///
    /// Services matching this instance's own name are automatically filtered out.
    ///
    /// # Errors
    ///
    /// Returns an error if browsing cannot be started, which may occur if:
    /// - The mDNS daemon is not running
    /// - Network interfaces are not available
    /// - The service type is invalid
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use multishiva::core::discovery::Discovery;
    ///
    /// let discovery = Discovery::new("my-machine".to_string())?;
    /// discovery.start_browsing()?;
    ///
    /// // Give it time to discover peers
    /// std::thread::sleep(std::time::Duration::from_secs(3));
    ///
    /// for peer in discovery.get_peers() {
    ///     println!("Found: {} at {}", peer.name, peer.full_address());
    /// }
    /// # Ok::<(), anyhow::Error>(())
    /// ```
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

    /// Returns a list of all currently discovered peers.
    ///
    /// This creates a snapshot of the current peer list at the time of the call.
    /// The list may change as new peers are discovered or existing peers are removed.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use multishiva::core::discovery::Discovery;
    ///
    /// let discovery = Discovery::new("my-machine".to_string())?;
    /// discovery.start_browsing()?;
    ///
    /// std::thread::sleep(std::time::Duration::from_secs(2));
    ///
    /// let peers = discovery.get_peers();
    /// println!("Discovered {} peer(s)", peers.len());
    ///
    /// for peer in peers {
    ///     println!("  - {} at {}", peer.name, peer.full_address());
    /// }
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn get_peers(&self) -> Vec<PeerInfo> {
        self.peers
            .lock()
            .map(|peers| peers.values().cloned().collect())
            .unwrap_or_default()
    }

    /// Retrieves information about a specific peer by name.
    ///
    /// Returns `Some(PeerInfo)` if a peer with the given name is currently
    /// discovered, or `None` if no such peer exists.
    ///
    /// # Arguments
    ///
    /// * `name` - The machine name of the peer to look up
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use multishiva::core::discovery::Discovery;
    ///
    /// let discovery = Discovery::new("my-machine".to_string())?;
    /// discovery.start_browsing()?;
    ///
    /// std::thread::sleep(std::time::Duration::from_secs(2));
    ///
    /// if let Some(peer) = discovery.get_peer("agent-001") {
    ///     println!("Agent found at {}", peer.full_address());
    /// } else {
    ///     println!("Agent not found");
    /// }
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn get_peer(&self, name: &str) -> Option<PeerInfo> {
        self.peers
            .lock()
            .ok()
            .and_then(|peers| peers.get(name).cloned())
    }

    /// Checks if a peer with the given name is currently discovered.
    ///
    /// This is a convenience method that returns `true` if the peer exists
    /// in the discovered peers list, `false` otherwise.
    ///
    /// # Arguments
    ///
    /// * `name` - The machine name of the peer to check
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use multishiva::core::discovery::Discovery;
    ///
    /// let discovery = Discovery::new("my-machine".to_string())?;
    /// discovery.start_browsing()?;
    ///
    /// std::thread::sleep(std::time::Duration::from_secs(2));
    ///
    /// if discovery.has_peer("agent-001") {
    ///     println!("Agent is online");
    /// }
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn has_peer(&self, name: &str) -> bool {
        self.peers
            .lock()
            .map(|peers| peers.contains_key(name))
            .unwrap_or(false)
    }

    /// Clears all discovered peers from the internal list.
    ///
    /// This removes all peer information from the discovery system. The browsing
    /// process continues to run if it was started, and peers will be rediscovered
    /// as they announce themselves.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use multishiva::core::discovery::Discovery;
    ///
    /// let discovery = Discovery::new("my-machine".to_string())?;
    /// discovery.start_browsing()?;
    ///
    /// std::thread::sleep(std::time::Duration::from_secs(2));
    /// println!("Found {} peers", discovery.get_peers().len());
    ///
    /// // Clear the list
    /// discovery.clear_peers();
    /// assert_eq!(discovery.get_peers().len(), 0);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn clear_peers(&self) {
        if let Ok(mut peers) = self.peers.lock() {
            peers.clear();
        }
    }

    /// Shuts down the discovery system gracefully.
    ///
    /// This stops all mDNS operations including service registration and browsing.
    /// The method blocks until the underlying mDNS daemon confirms shutdown.
    /// After calling this method, the `Discovery` instance should not be used
    /// for any further operations.
    ///
    /// Note: This is automatically called when the `Discovery` instance is dropped.
    ///
    /// # Errors
    ///
    /// Returns an error if the mDNS daemon cannot be shut down properly,
    /// though this is rare and usually indicates a system-level issue.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use multishiva::core::discovery::Discovery;
    ///
    /// let discovery = Discovery::new("my-machine".to_string())?;
    /// discovery.start_browsing()?;
    ///
    /// // Do some work...
    ///
    /// // Explicitly shutdown when done
    /// discovery.shutdown()?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
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
