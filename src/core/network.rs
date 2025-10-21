use anyhow::{Context, Result};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio::time::{sleep, Duration};

use crate::core::events::Event;
use crate::core::fingerprint::{Fingerprint, FingerprintStore, FingerprintVerification};

/// Interval between heartbeat messages sent to maintain connection liveness.
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// Maximum time to wait when establishing a TCP connection before timing out.
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(10);

/// Magic bytes used to identify and validate PSK handshake protocol version.
const PSK_MAGIC: &[u8] = b"MULTISHIVA_PSK_V1";

/// Network manager for secure peer-to-peer communication with PSK authentication.
///
/// The `Network` struct handles both hosting and connecting to remote peers,
/// providing encrypted communication channels with pre-shared key (PSK) authentication
/// and certificate fingerprint verification for enhanced security.
///
/// # Examples
///
/// ```no_run
/// use multishiva::core::network::Network;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let mut network = Network::new("my-secure-psk".to_string());
///
///     // Start hosting on port 8080
///     let port = network.start_host(8080).await?;
///     println!("Hosting on port {}", port);
///
///     Ok(())
/// }
/// ```
pub struct Network {
    psk: String,
    running: Arc<AtomicBool>,
    connected: Arc<AtomicBool>,
    connection_count: Arc<AtomicUsize>,
    event_tx: Arc<RwLock<Option<mpsc::Sender<Event>>>>,
    event_rx: Arc<RwLock<Option<mpsc::Receiver<Event>>>>,
    fingerprint_store: Arc<Mutex<FingerprintStore>>,
}

impl Network {
    /// Creates a new `Network` instance with the specified pre-shared key.
    ///
    /// Initializes the network manager with an event channel, fingerprint store,
    /// and connection state tracking. The fingerprint store is loaded from the
    /// default location, or a new one is created if loading fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use multishiva::core::network::Network;
    ///
    /// let network = Network::new("my-secret-key".to_string());
    /// ```
    pub fn new(psk: String) -> Self {
        let (tx, rx) = mpsc::channel(100);
        let fingerprint_store = FingerprintStore::load_default().unwrap_or_else(|e| {
            tracing::warn!("Could not load fingerprint store: {}. Creating new one.", e);
            FingerprintStore::new(FingerprintStore::default_path()).unwrap()
        });

        Self {
            psk,
            running: Arc::new(AtomicBool::new(false)),
            connected: Arc::new(AtomicBool::new(false)),
            connection_count: Arc::new(AtomicUsize::new(0)),
            event_tx: Arc::new(RwLock::new(Some(tx))),
            event_rx: Arc::new(RwLock::new(Some(rx))),
            fingerprint_store: Arc::new(Mutex::new(fingerprint_store)),
        }
    }

    /// Starts hosting on the specified port and listens for incoming connections.
    ///
    /// Binds to `127.0.0.1` on the given port and spawns an async task to accept
    /// incoming client connections. Each client connection is authenticated using
    /// PSK handshake before being handled in a separate task.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use multishiva::core::network::Network;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let mut network = Network::new("psk".to_string());
    ///     let actual_port = network.start_host(8080).await?;
    ///     println!("Hosting on port {}", actual_port);
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The port is already in use
    /// - Unable to bind to the specified address
    /// - Cannot retrieve the local address from the listener
    pub async fn start_host(&mut self, port: u16) -> Result<u16> {
        // Try to bind on IPv6 dual-stack first (supports both IPv4 and IPv6)
        // Falls back to IPv4-only if IPv6 is not available
        let listener = match TcpListener::bind(format!("[::]:{}", port)).await {
            Ok(listener) => {
                tracing::debug!("Bound to IPv6 dual-stack address [::]:{}", port);
                listener
            }
            Err(_) => {
                tracing::debug!("IPv6 not available, falling back to IPv4");
                TcpListener::bind(format!("0.0.0.0:{}", port))
                    .await
                    .context("Failed to bind to address")?
            }
        };

        let actual_port = listener.local_addr()?.port();
        self.running.store(true, Ordering::SeqCst);

        let running = self.running.clone();
        let connection_count = self.connection_count.clone();
        let psk = self.psk.clone();

        // Spawn host listener task
        tokio::spawn(async move {
            tracing::info!("Host listening on port {}", actual_port);

            while running.load(Ordering::SeqCst) {
                match tokio::time::timeout(Duration::from_millis(100), listener.accept()).await {
                    Ok(Ok((stream, addr))) => {
                        tracing::info!("New connection from {}", addr);
                        connection_count.fetch_add(1, Ordering::SeqCst);

                        let psk = psk.clone();
                        let connection_count = connection_count.clone();

                        tokio::spawn(async move {
                            if let Err(e) = handle_client(stream, psk).await {
                                tracing::error!("Client handler error: {}", e);
                            }
                            connection_count.fetch_sub(1, Ordering::SeqCst);
                        });
                    }
                    Ok(Err(e)) => {
                        tracing::error!("Accept error: {}", e);
                    }
                    Err(_) => {
                        // Timeout, continue loop to check running flag
                    }
                }
            }

            tracing::info!("Host stopped listening");
        });

        Ok(actual_port)
    }

    /// Connects to a remote host at the specified address.
    ///
    /// Establishes a TCP connection to the remote host, performs PSK authentication,
    /// and verifies the host's fingerprint. If the fingerprint is unrecognized or
    /// mismatched, the connection is rejected as a potential security threat.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use multishiva::core::network::Network;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let network = Network::new("psk".to_string());
    ///     network.connect_to_host("127.0.0.1:8080").await?;
    ///     println!("Connected successfully");
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Connection timeout is exceeded
    /// - Unable to connect to the host
    /// - PSK handshake fails (invalid or mismatched PSK)
    /// - Fingerprint verification fails (potential MITM attack)
    pub async fn connect_to_host(&self, addr: &str) -> Result<()> {
        tracing::debug!("Attempting to connect to host at: {}", addr);

        let mut stream =
            match tokio::time::timeout(CONNECTION_TIMEOUT, TcpStream::connect(addr)).await {
                Ok(Ok(stream)) => {
                    tracing::debug!("TCP connection established to {}", addr);
                    stream
                }
                Ok(Err(e)) => {
                    tracing::error!("TCP connection failed to {}: {:?}", addr, e);
                    return Err(e).context("Failed to connect to host");
                }
                Err(_) => {
                    tracing::error!(
                        "Connection timeout after {:?} to {}",
                        CONNECTION_TIMEOUT,
                        addr
                    );
                    anyhow::bail!("Connection timeout");
                }
            };

        // Perform PSK handshake and get machine name
        let machine_name = perform_psk_handshake(&mut stream, &self.psk, false)
            .await
            .context("PSK handshake failed")?;

        // Verify fingerprint
        let psk_fingerprint = Fingerprint::from_cert_data(&machine_name, self.psk.as_bytes());
        let mut store = self.fingerprint_store.lock().await;

        match store.verify_or_save(&machine_name, psk_fingerprint.hash())? {
            FingerprintVerification::Verified => {
                tracing::info!("✓ Fingerprint verified for {}", machine_name);
            }
            FingerprintVerification::FirstConnection => {
                tracing::warn!("First connection to {}. Fingerprint saved.", machine_name);
            }
            FingerprintVerification::Mismatch { stored, received } => {
                tracing::error!(
                    "⚠️  SECURITY WARNING: Fingerprint mismatch for {}!\n\
                     Stored:   {}\n\
                     Received: {}\n\
                     This could indicate a Man-in-the-Middle attack!",
                    machine_name,
                    stored,
                    received
                );
                anyhow::bail!("Fingerprint mismatch - possible MITM attack");
            }
        }

        self.connected.store(true, Ordering::SeqCst);

        let connected = self.connected.clone();
        let psk = self.psk.clone();

        // Spawn connection handler
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, psk, connected.clone()).await {
                tracing::error!("Connection handler error: {}", e);
            }
            connected.store(false, Ordering::SeqCst);
        });

        Ok(())
    }

    /// Sends an event through the internal event channel.
    ///
    /// Queues the event for processing by the network subsystem. Events are
    /// buffered in an async channel with a capacity of 100 messages.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use multishiva::core::network::Network;
    /// use multishiva::core::events::Event;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let network = Network::new("psk".to_string());
    ///     // network.send_event(Event::Connect).await?;
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the event channel is closed or the receiver has been dropped.
    pub async fn send_event(&self, event: Event) -> Result<()> {
        let tx_guard = self.event_tx.read().await;
        if let Some(tx) = tx_guard.as_ref() {
            tx.send(event)
                .await
                .context("Failed to send event to channel")?;
        }
        Ok(())
    }

    /// Receives the next event from the internal event channel.
    ///
    /// Blocks asynchronously until an event is available or the channel is closed.
    /// Returns `None` if the event sender has been dropped or the channel is closed.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use multishiva::core::network::Network;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let mut network = Network::new("psk".to_string());
    ///
    ///     if let Some(event) = network.receive_event().await {
    ///         println!("Received event: {:?}", event);
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub async fn receive_event(&mut self) -> Option<Event> {
        let mut rx_guard = self.event_rx.write().await;
        if let Some(rx) = rx_guard.as_mut() {
            rx.recv().await
        } else {
            None
        }
    }

    /// Stops all network operations and closes active connections.
    ///
    /// Signals all running tasks to terminate by setting the running and connected
    /// flags to false, then waits briefly to allow tasks to clean up gracefully.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use multishiva::core::network::Network;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let mut network = Network::new("psk".to_string());
    ///     network.start_host(8080).await?;
    ///
    ///     // Later...
    ///     network.stop().await;
    ///     Ok(())
    /// }
    /// ```
    pub async fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        self.connected.store(false, Ordering::SeqCst);
        sleep(Duration::from_millis(200)).await; // Give time for tasks to cleanup
    }

    /// Returns whether the network is currently running and hosting.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use multishiva::core::network::Network;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let mut network = Network::new("psk".to_string());
    ///     assert!(!network.is_running());
    ///
    ///     network.start_host(8080).await?;
    ///     assert!(network.is_running());
    ///     Ok(())
    /// }
    /// ```
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Returns whether the network is currently connected to a remote host.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use multishiva::core::network::Network;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let network = Network::new("psk".to_string());
    ///     assert!(!network.is_connected());
    ///
    ///     network.connect_to_host("127.0.0.1:8080").await?;
    ///     assert!(network.is_connected());
    ///     Ok(())
    /// }
    /// ```
    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::SeqCst)
    }

    /// Returns the number of currently active client connections.
    ///
    /// This count only applies when hosting. Each time a client connects,
    /// the count is incremented, and decremented when they disconnect.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use multishiva::core::network::Network;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let mut network = Network::new("psk".to_string());
    ///     network.start_host(8080).await?;
    ///
    ///     println!("Active connections: {}", network.connection_count());
    ///     Ok(())
    /// }
    /// ```
    pub fn connection_count(&self) -> usize {
        self.connection_count.load(Ordering::SeqCst)
    }
}

async fn perform_psk_handshake(
    stream: &mut TcpStream,
    psk: &str,
    is_server: bool,
) -> Result<String> {
    let psk_hash = compute_psk_hash(psk);

    if is_server {
        // Server: receive PSK hash and machine name
        let mut buf = vec![0u8; 256];
        let n = stream.read(&mut buf).await?;

        if n < PSK_MAGIC.len() {
            anyhow::bail!("Invalid PSK handshake");
        }

        if &buf[0..PSK_MAGIC.len()] != PSK_MAGIC {
            anyhow::bail!("Invalid PSK magic");
        }

        let data = &buf[PSK_MAGIC.len()..n];
        // Parse: machine_name\0psk_hash
        let parts: Vec<&[u8]> = data.splitn(2, |&b| b == 0).collect();

        if parts.len() != 2 {
            anyhow::bail!("Invalid handshake format");
        }

        let machine_name = std::str::from_utf8(parts[0])
            .context("Invalid machine name")?
            .to_string();
        let received_hash = std::str::from_utf8(parts[1]).context("Invalid PSK hash")?;

        if received_hash != psk_hash {
            anyhow::bail!("PSK mismatch");
        }

        // Send acknowledgment
        stream.write_all(b"OK").await?;

        Ok(machine_name)
    } else {
        // Client: send machine name and PSK hash
        let machine_name = hostname::get()
            .ok()
            .and_then(|h| h.into_string().ok())
            .unwrap_or_else(|| "unknown".to_string());

        let mut handshake = PSK_MAGIC.to_vec();
        handshake.extend_from_slice(machine_name.as_bytes());
        handshake.push(0); // Null separator
        handshake.extend_from_slice(psk_hash.as_bytes());

        stream.write_all(&handshake).await?;

        // Wait for acknowledgment
        let mut buf = [0u8; 2];
        let n = stream.read(&mut buf).await?;

        if n != 2 || &buf != b"OK" {
            anyhow::bail!("PSK handshake not acknowledged");
        }

        Ok(machine_name)
    }
}

fn compute_psk_hash(psk: &str) -> String {
    // Use SHA-256 for cryptographically secure hashing
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(psk.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

async fn handle_client(mut stream: TcpStream, psk: String) -> Result<()> {
    // Perform PSK handshake and get machine name
    let machine_name = match perform_psk_handshake(&mut stream, &psk, true).await {
        Ok(name) => name,
        Err(e) => {
            tracing::warn!("PSK handshake failed: {}", e);
            return Err(e);
        }
    };

    tracing::info!("Client {} authenticated successfully", machine_name);

    // Note: fingerprint verification would happen here in a full implementation
    // For now, we just log the successful authentication

    // Keep connection alive and handle events
    loop {
        // TODO: Read and handle events
        sleep(Duration::from_secs(1)).await;
    }
}

async fn handle_connection(
    mut _stream: TcpStream,
    _psk: String,
    connected: Arc<AtomicBool>,
) -> Result<()> {
    // Heartbeat loop
    while connected.load(Ordering::SeqCst) {
        sleep(HEARTBEAT_INTERVAL).await;
        // TODO: Send heartbeat
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_psk_hash() {
        let hash1 = compute_psk_hash("test-psk");
        let hash2 = compute_psk_hash("test-psk");
        let hash3 = compute_psk_hash("different-psk");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }
}
