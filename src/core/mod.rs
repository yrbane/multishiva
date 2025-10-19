/// Clipboard synchronization across machines
pub mod clipboard;

/// Configuration management with persistence and validation
pub mod config;

/// mDNS-based auto-discovery of MultiShiva instances
pub mod discovery;

/// Input event types and handling
pub mod events;

/// TLS fingerprint generation and verification
pub mod fingerprint;

/// Focus management across multiple machines
pub mod focus;

/// Input capture and injection (keyboard/mouse)
pub mod input;

/// Secure credential storage using system keyring
pub mod keyring;

/// Structured logging with rotation
pub mod logging;

/// TLS-encrypted network communication
pub mod network;

/// System permission checks and requirements
pub mod permissions;

/// Simulation mode for testing without hardware
pub mod simulation;

/// Machine topology and edge mapping
pub mod topology;
