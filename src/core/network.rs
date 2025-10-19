use anyhow::{Context, Result};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, RwLock};
use tokio::time::{sleep, Duration};

use crate::core::events::Event;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(10);
const PSK_MAGIC: &[u8] = b"MULTISHIVA_PSK_V1";

pub struct Network {
    psk: String,
    running: Arc<AtomicBool>,
    connected: Arc<AtomicBool>,
    connection_count: Arc<AtomicUsize>,
    event_tx: Arc<RwLock<Option<mpsc::Sender<Event>>>>,
    event_rx: Arc<RwLock<Option<mpsc::Receiver<Event>>>>,
}

impl Network {
    pub fn new(psk: String) -> Self {
        let (tx, rx) = mpsc::channel(100);
        Self {
            psk,
            running: Arc::new(AtomicBool::new(false)),
            connected: Arc::new(AtomicBool::new(false)),
            connection_count: Arc::new(AtomicUsize::new(0)),
            event_tx: Arc::new(RwLock::new(Some(tx))),
            event_rx: Arc::new(RwLock::new(Some(rx))),
        }
    }

    pub async fn start_host(&mut self, port: u16) -> Result<u16> {
        let addr = format!("127.0.0.1:{}", port);
        let listener = TcpListener::bind(&addr)
            .await
            .context("Failed to bind to address")?;

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

    pub async fn connect_to_host(&self, addr: &str) -> Result<()> {
        let mut stream = tokio::time::timeout(CONNECTION_TIMEOUT, TcpStream::connect(addr))
            .await
            .context("Connection timeout")?
            .context("Failed to connect to host")?;

        // Perform PSK handshake
        if let Err(e) = perform_psk_handshake(&mut stream, &self.psk, false).await {
            return Err(e).context("PSK handshake failed");
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

    pub async fn send_event(&self, event: Event) -> Result<()> {
        let tx_guard = self.event_tx.read().await;
        if let Some(tx) = tx_guard.as_ref() {
            tx.send(event)
                .await
                .context("Failed to send event to channel")?;
        }
        Ok(())
    }

    pub async fn receive_event(&mut self) -> Option<Event> {
        let mut rx_guard = self.event_rx.write().await;
        if let Some(rx) = rx_guard.as_mut() {
            rx.recv().await
        } else {
            None
        }
    }

    pub async fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        self.connected.store(false, Ordering::SeqCst);
        sleep(Duration::from_millis(200)).await; // Give time for tasks to cleanup
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::SeqCst)
    }

    pub fn connection_count(&self) -> usize {
        self.connection_count.load(Ordering::SeqCst)
    }
}

async fn perform_psk_handshake(stream: &mut TcpStream, psk: &str, is_server: bool) -> Result<()> {
    if is_server {
        // Server: receive PSK hash and verify
        let mut buf = vec![0u8; 128];
        let n = stream.read(&mut buf).await?;

        if n < PSK_MAGIC.len() {
            anyhow::bail!("Invalid PSK handshake");
        }

        if &buf[0..PSK_MAGIC.len()] != PSK_MAGIC {
            anyhow::bail!("Invalid PSK magic");
        }

        let received_hash = &buf[PSK_MAGIC.len()..n];
        let expected_hash = compute_psk_hash(psk);

        if received_hash != expected_hash.as_bytes() {
            anyhow::bail!("PSK mismatch");
        }

        // Send acknowledgment
        stream.write_all(b"OK").await?;
    } else {
        // Client: send PSK hash
        let mut handshake = PSK_MAGIC.to_vec();
        handshake.extend_from_slice(compute_psk_hash(psk).as_bytes());

        stream.write_all(&handshake).await?;

        // Wait for acknowledgment
        let mut buf = [0u8; 2];
        let n = stream.read(&mut buf).await?;

        if n != 2 || &buf != b"OK" {
            anyhow::bail!("PSK handshake not acknowledged");
        }
    }

    Ok(())
}

fn compute_psk_hash(psk: &str) -> String {
    // Simple hash for now - in production use proper crypto
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    psk.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

async fn handle_client(mut stream: TcpStream, psk: String) -> Result<()> {
    // Perform PSK handshake
    if let Err(e) = perform_psk_handshake(&mut stream, &psk, true).await {
        tracing::warn!("PSK handshake failed: {}", e);
        return Err(e);
    }

    tracing::info!("Client authenticated successfully");

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
