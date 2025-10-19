//! # MultiShiva
//!
//! **Control multiple computers with one keyboard and mouse over LAN**
//!
//! MultiShiva is a cross-platform KVM (Keyboard, Video, Mouse) software solution
//! that allows seamless control of multiple computers using a single keyboard and
//! mouse. It operates over your local network using secure TLS connections.
//!
//! ## Features
//!
//! - **Seamless Control**: Move your mouse across screen edges to switch between machines
//! - **Secure Communication**: TLS-encrypted connections with pre-shared key authentication
//! - **Auto-Discovery**: mDNS-based automatic discovery of MultiShiva instances
//! - **Clipboard Sync**: Share clipboard content across connected machines
//! - **Secure Storage**: System keyring integration for PSK storage
//! - **Cross-Platform**: Works on Linux, Windows, and macOS
//! - **Flexible Topology**: Configure custom machine layouts with directional edges
//!
//! ## Architecture
//!
//! MultiShiva uses a client-server architecture:
//! - **Host**: The central machine that coordinates connections
//! - **Agents**: Client machines that connect to the host
//!
//! Each machine maintains its own configuration, including:
//! - Machine name and role (host or agent)
//! - Network settings (port, host address)
//! - Edge mappings (which machines are on which sides)
//! - Security settings (TLS PSK)
//! - Behavior tuning (thresholds, delays)
//!
//! ## Modules
//!
//! ### Core Functionality
//! - [`core::config`] - Configuration management with automatic persistence
//! - [`core::network`] - TLS-encrypted network communication
//! - [`core::events`] - Input event handling and forwarding
//! - [`core::focus`] - Focus management across machines
//! - [`core::topology`] - Machine layout and edge definitions
//!
//! ### Security
//! - [`core::fingerprint`] - TLS fingerprint verification
//! - [`core::keyring`] - Secure credential storage using system keyring
//! - [`core::permissions`] - System permission checks
//!
//! ### Features
//! - [`core::discovery`] - mDNS auto-discovery of peer machines
//! - [`core::clipboard`] - Cross-machine clipboard synchronization
//! - [`core::logging`] - Structured logging with rotation
//! - [`core::simulation`] - Testing mode for development
//!
//! ### User Interface
//! - [`cli`] - Command-line interface and argument parsing
//! - [`app`] - Application entry point and lifecycle
//!
//! ## Quick Start
//!
//! ### Host Configuration
//!
//! ```yaml
//! version: 1
//! self_name: "host"
//! mode: host
//! port: 53421
//! tls:
//!   psk: "your-secure-psk-here"
//! edges:
//!   right: "agent1"  # Agent to the right
//! ```
//!
//! ### Agent Configuration
//!
//! ```yaml
//! version: 1
//! self_name: "agent1"
//! mode: agent
//! port: 53421
//! host_address: "192.168.1.100:53421"
//! tls:
//!   psk: "same-psk-as-host"
//! edges:
//!   left: "host"  # Host is to the left
//! ```
//!
//! ## Examples
//!
//! ### Loading Configuration
//!
//! ```no_run
//! use multishiva::core::config::Config;
//!
//! let config = Config::from_file("multishiva.yml")?;
//! config.validate()?;
//! # Ok::<(), anyhow::Error>(())
//! ```
//!
//! ### Setting up mDNS Discovery
//!
//! ```no_run
//! use multishiva::core::discovery::Discovery;
//! use std::collections::HashMap;
//!
//! let discovery = Discovery::new("host".to_string())?;
//! discovery.register(53421, Some("psk-hash".to_string()), HashMap::new())?;
//! discovery.start_browsing()?;
//! # Ok::<(), anyhow::Error>(())
//! ```
//!
//! ### Secure PSK Storage
//!
//! ```no_run
//! use multishiva::core::keyring::KeyringManager;
//!
//! let keyring = KeyringManager::new();
//! keyring.set_psk("my-secure-psk")?;
//! let psk = keyring.get_psk()?;
//! # Ok::<(), anyhow::Error>(())
//! ```
//!
//! ## Version
//!
//! Current version: 1.0.0
//!
//! ## License
//!
//! MIT License - See LICENSE file for details

/// Application entry point and GUI launcher
pub mod app;

/// Command-line interface and argument parsing
pub mod cli;

/// Core functionality modules
pub mod core;
