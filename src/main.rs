use anyhow::Result;
use multishiva::cli;
use multishiva::core::config::{Config, ConfigMode};
use multishiva::core::focus::FocusManager;
use multishiva::core::network::Network;
use multishiva::core::permissions;
use multishiva::core::simulation::SimulationMode;
use multishiva::core::topology::{Edge, Position, Topology};
use tokio::signal;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging system with default configuration
    use multishiva::core::logging::{init_logging, LogConfig, LogLevel};

    let log_config = LogConfig {
        level: if cfg!(debug_assertions) {
            LogLevel::Debug
        } else {
            LogLevel::Info
        },
        enable_file: true,
        enable_console: true,
        log_dir: None, // Use default: ~/.local/share/multishiva/logs/
        filter: std::env::var("RUST_LOG").ok(),
    };

    init_logging(log_config)?;

    tracing::info!("🕉️  MultiShiva v{} starting...", env!("CARGO_PKG_VERSION"));

    // Parse and validate CLI arguments
    let args = cli::parse_and_validate()?;

    // Check if GUI mode is requested
    if args.gui {
        tracing::info!("🖥️  Launching GUI mode...");
        return multishiva::app::launch_gui();
    }

    // Load configuration
    let config_path = args.config.as_deref().unwrap_or("multishiva.yml");
    let config = Config::from_file(config_path).map_err(|e| {
        if config_path == "multishiva.yml" && !std::path::Path::new(config_path).exists() {
            anyhow::anyhow!(
                "Configuration file not found: {}\n\n\
                 To get started:\n\
                 1. Copy the example config: cp multishiva.yml.example multishiva.yml\n\
                 2. Edit the config file to match your setup\n\
                 3. For agent mode: cp multishiva-agent.yml.example multishiva-agent.yml\n\n\
                 Or specify a custom config: multishiva --config /path/to/config.yml\n\n\
                 Original error: {}",
                config_path,
                e
            )
        } else {
            e
        }
    })?;

    // Override config mode with CLI argument if provided
    let mut config = config;
    if let Some(cli_mode) = args.mode {
        let config_mode = match cli_mode {
            cli::Mode::Host => multishiva::core::config::ConfigMode::Host,
            cli::Mode::Agent => multishiva::core::config::ConfigMode::Agent,
        };
        tracing::info!("CLI mode override: {:?} -> {:?}", config.mode, config_mode);
        config.mode = config_mode;
    }

    // Override host address with CLI argument if provided
    if let Some(host_address) = args.host {
        tracing::info!(
            "CLI host address override: {:?} -> {}",
            config.host_address,
            host_address
        );
        config.host_address = Some(host_address);
    }

    config.validate()?;

    tracing::info!("Configuration loaded from: {}", config_path);
    tracing::info!("Running as: {:?} on port {}", config.mode, config.port);

    // Build topology from configuration
    let topology = build_topology(&config);
    tracing::info!(
        "Topology configured with {} machine(s)",
        topology.machine_count()
    );

    // Check if simulation mode is enabled
    if args.simulate {
        run_simulation_mode(config, topology).await?;
    } else {
        // Check system permissions before starting in production mode
        tracing::info!("Checking system permissions...");
        match permissions::check_permissions() {
            Ok(status) => {
                if status.is_granted() {
                    tracing::info!("✓ All required permissions granted");
                } else {
                    let missing = status.missing_permissions();
                    tracing::warn!("⚠️  Missing permissions: {}", missing.join(", "));
                    tracing::warn!("\n{}", permissions::get_permission_help());
                    tracing::warn!(
                        "MultiShiva may not function correctly without proper permissions."
                    );
                    tracing::warn!("Continuing anyway...");
                }
            }
            Err(e) => {
                tracing::warn!("Could not check permissions: {}", e);
                tracing::warn!("Continuing anyway...");
            }
        }

        run_production_mode(config, topology).await?;
    }

    Ok(())
}

fn build_topology(config: &Config) -> Topology {
    let mut topology = Topology::new();

    // Add self machine
    topology.add_machine(config.self_name.clone(), Position { x: 0, y: 0 });

    // Add configured edges
    for (direction, target) in &config.edges {
        let edge = match direction.as_str() {
            "right" => Edge::Right,
            "left" => Edge::Left,
            "top" => Edge::Top,
            "bottom" => Edge::Bottom,
            _ => {
                tracing::warn!("Unknown edge direction: {}", direction);
                continue;
            }
        };
        topology.add_edge(config.self_name.clone(), edge, target.clone());
        tracing::debug!(
            "Added edge: {} -> {:?} -> {}",
            config.self_name,
            edge,
            target
        );
    }

    topology
}

async fn run_simulation_mode(config: Config, _topology: Topology) -> Result<()> {
    tracing::info!("🎭 Running in SIMULATION mode");

    let mut sim = SimulationMode::new();

    // Add host VM
    sim.add_virtual_machine(config.self_name.clone(), 1920, 1080);

    // Add VMs for each edge target
    for target in config.edges.values() {
        sim.add_virtual_machine(target.clone(), 1920, 1080);
    }

    tracing::info!("Created {} virtual machine(s)", sim.virtual_machine_count());

    // Run simulation until Ctrl+C
    tracing::info!("Press Ctrl+C to exit");
    signal::ctrl_c().await?;

    tracing::info!("Simulation stopping...");
    let stats = sim.get_statistics();
    tracing::info!("Total events sent: {}", stats.total_events_sent);

    Ok(())
}

/// Discover a MultiShiva host on the network using mDNS
///
/// This function starts mDNS service discovery and waits for up to 5 seconds
/// to find a host. If multiple hosts are found, it returns the first one.
async fn discover_host_via_mdns(config: &Config) -> Result<String> {
    use multishiva::core::discovery::Discovery;

    tracing::info!("Starting mDNS discovery...");
    let discovery = Discovery::new(config.self_name.clone())?;

    // Start browsing for MultiShiva services
    discovery.start_browsing()?;

    // Wait for discovery (check every 500ms for up to 5 seconds)
    let max_attempts = 10;
    for attempt in 1..=max_attempts {
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        let peers = discovery.get_peers();

        // Filter for hosts (not other agents - exclude self)
        let hosts: Vec<_> = peers
            .iter()
            .filter(|peer| peer.name != config.self_name)
            .collect();

        if !hosts.is_empty() {
            let peer_info = hosts[0];
            let address = peer_info.full_address();
            tracing::info!(
                "✓ Found host '{}' at {} (attempt {}/{})",
                peer_info.name,
                address,
                attempt,
                max_attempts
            );

            if hosts.len() > 1 {
                tracing::warn!(
                    "Multiple hosts found on network, using first one: {}",
                    peer_info.name
                );
                for info in hosts.iter().skip(1) {
                    tracing::warn!("  - Also found: {} at {}", info.name, info.full_address());
                }
            }

            return Ok(address);
        }

        if attempt < max_attempts {
            tracing::debug!(
                "No hosts found yet, waiting... (attempt {}/{})",
                attempt,
                max_attempts
            );
        }
    }

    anyhow::bail!(
        "No MultiShiva host found on the network after 5 seconds.\n\
         \n\
         Troubleshooting:\n\
         1. Make sure a host is running: `multishiva --mode host`\n\
         2. Check firewall settings (port {} should be open)\n\
         3. Verify both machines are on the same network\n\
         4. Manually specify host address: `multishiva --mode agent --host <address>`",
        config.port
    )
}

async fn run_production_mode(config: Config, _topology: Topology) -> Result<()> {
    tracing::info!("🚀 Running in PRODUCTION mode");

    let focus = FocusManager::new(config.self_name.clone());
    tracing::debug!("Focus manager initialized for: {}", config.self_name);

    match config.mode {
        ConfigMode::Host => run_host_mode(config, focus).await,
        ConfigMode::Agent => {
            // If host_address is not specified, try to discover it via mDNS
            let host_address = if let Some(addr) = config.host_address.clone() {
                addr
            } else {
                tracing::info!("🔍 No host address specified, using mDNS auto-discovery...");
                discover_host_via_mdns(&config).await?
            };
            run_agent_mode(config, focus, &host_address).await
        }
    }
}

async fn run_host_mode(config: Config, _focus: FocusManager) -> Result<()> {
    use multishiva::core::discovery::Discovery;
    use multishiva::core::input::InputHandler;
    use std::collections::HashMap;

    tracing::info!("Starting as HOST on port {}", config.port);

    let mut network = Network::new(config.tls.psk.clone());

    // Start host server
    let actual_port = network.start_host(config.port).await?;
    tracing::info!("✓ Host listening on port {}", actual_port);

    // Register this host on mDNS for auto-discovery
    tracing::info!("📡 Registering host on mDNS for auto-discovery...");
    let discovery = Discovery::new(config.self_name.clone())?;
    discovery.register(actual_port, None, HashMap::new())?;
    tracing::info!("✓ Host registered on mDNS as '{}'", config.self_name);

    // Log topology
    for (edge_name, neighbor_name) in &config.edges {
        tracing::info!("🔗 Topology: {} at edge {}", neighbor_name, edge_name);
    }

    // Start input capture - use evdev on Linux for Wayland/X11 support
    #[cfg(target_os = "linux")]
    let mut input_handler = {
        use multishiva::core::input_evdev::EvdevInputHandler;
        tracing::info!("Using native evdev backend (Wayland/X11 compatible)");
        EvdevInputHandler::new()?
    };

    #[cfg(not(target_os = "linux"))]
    let mut input_handler = {
        use multishiva::core::input::RdevInputHandler;
        tracing::info!("Using rdev backend");
        RdevInputHandler::new()
    };
    let (event_tx, mut event_rx) = tokio::sync::mpsc::channel(100);

    tracing::info!("🖱️  Starting mouse/keyboard capture...");
    input_handler.start_capture(event_tx).await?;
    tracing::info!("✓ Input capture started");

    let screen_size = input_handler.get_screen_size();
    tracing::info!("📺 Screen size: {}x{}", screen_size.0, screen_size.1);

    // Get edge threshold from config or use default
    let edge_threshold = config
        .behavior
        .as_ref()
        .and_then(|b| b.edge_threshold_px)
        .unwrap_or(10) as i32;
    tracing::info!("🎯 Edge threshold: {} pixels", edge_threshold);

    tracing::info!("Waiting for agents to connect...");
    tracing::info!("Press Ctrl+C to exit");

    // Track which machine has focus (None = local, Some(name) = remote)
    let mut focus_target: Option<String> = None;

    // Event processing loop
    let ctrl_c = signal::ctrl_c();
    tokio::pin!(ctrl_c);

    let mut event_count = 0u64;
    loop {
        tokio::select! {
            Some(event) = event_rx.recv() => {
                event_count += 1;

                // If focus is on remote machine, send ALL events there
                if let Some(ref target) = focus_target {
                    tracing::trace!("Forwarding event to {}: {:?}", target, event);
                    if let Err(e) = network.send_event(event).await {
                        tracing::error!("Failed to send event to {}: {}", target, e);
                    }
                    continue; // Don't process locally
                }

                // Process events locally when we have focus
                // Log mouse movement for debugging
                if let multishiva::core::events::Event::MouseMove { x, y } = &event {
                    // Log every 100th event to see if we're receiving them
                    if event_count % 100 == 0 {
                        tracing::info!("📊 Received {} events. Current mouse: ({}, {})", event_count, x, y);
                    }

                    // Log ALL mouse movements temporarily to debug
                    tracing::trace!("Mouse position: ({}, {})", x, y);

                    // Check edge proximity
                    let threshold = edge_threshold;
                    let at_left = *x < threshold;
                    let at_right = *x > (screen_size.0 as i32 - threshold);
                    let at_top = *y < threshold;
                    let at_bottom = *y > (screen_size.1 as i32 - threshold);

                    // DEBUG: Log the actual check values
                    if *x < 50 || *x > 1870 {
                        tracing::warn!("🔍 DEBUG: x={}, threshold={}, at_left={} (x < {}), at_right={} (x > {})",
                            x, threshold, at_left, threshold, at_right, screen_size.0 as i32 - threshold);
                    }

                    if at_left || at_right || at_top || at_bottom {
                        tracing::info!(
                            "🖱️  Mouse near edge at ({}, {}) - Left:{} Right:{} Top:{} Bottom:{} (screen: {}x{}, threshold: {})",
                            x, y, at_left, at_right, at_top, at_bottom, screen_size.0, screen_size.1, threshold
                        );

                        // Check if there's a neighbor on this edge
                        let edge = if at_left {
                            Some("left")
                        } else if at_right {
                            Some("right")
                        } else if at_top {
                            Some("top")
                        } else if at_bottom {
                            Some("bottom")
                        } else {
                            None
                        };

                        if let Some(edge_name) = edge {
                            if let Some(neighbor) = config.edges.get(edge_name) {
                                tracing::info!(
                                    "🚀 Edge crossed! Transferring focus to '{}'",
                                    neighbor
                                );

                                // Send FocusGrant event
                                use multishiva::core::events::Event;
                                let focus_event = Event::FocusGrant {
                                    target: neighbor.clone(),
                                    x: *x,
                                    y: *y,
                                };

                                if let Err(e) = network.send_event(focus_event).await {
                                    tracing::error!("Failed to send FocusGrant: {}", e);
                                } else {
                                    // Transfer focus to remote machine
                                    focus_target = Some(neighbor.clone());
                                    tracing::info!("✓ Focus transferred to '{}'", neighbor);
                                }
                            } else {
                                tracing::debug!("No neighbor configured on {} edge", edge_name);
                            }
                        }
                    }
                }
            }
            _ = &mut ctrl_c => {
                tracing::info!("Received Ctrl+C, stopping...");
                break;
            }
        }
    }

    tracing::info!("Host stopping...");
    input_handler.stop_capture().await;
    network.stop().await;
    tracing::info!("Host stopped");

    Ok(())
}

async fn run_agent_mode(
    config: Config,
    mut _focus: FocusManager,
    host_address: &str,
) -> Result<()> {
    use multishiva::core::input::InputHandler;

    tracing::info!("Starting as AGENT, connecting to: {}", host_address);

    let mut network = Network::new(config.tls.psk.clone());

    // Connect to host
    network.connect_to_host(host_address).await?;
    tracing::info!("✓ Connected to host at {}", host_address);

    // Create input handler for event injection
    let input_handler = {
        #[cfg(target_os = "linux")]
        {
            use multishiva::core::input_evdev::EvdevInputHandler;
            EvdevInputHandler::new()?
        }

        #[cfg(not(target_os = "linux"))]
        {
            use multishiva::core::input::RdevInputHandler;
            RdevInputHandler::new()
        }
    };

    tracing::info!("✓ Input injection ready");
    tracing::info!("Waiting for events from host...");

    // Event receiving loop
    let ctrl_c = signal::ctrl_c();
    tokio::pin!(ctrl_c);

    loop {
        tokio::select! {
            Some(event) = network.receive_event() => {
                tracing::debug!("Received event from host: {:?}", event);

                // Inject event locally
                if let Err(e) = input_handler.inject_event(event.clone()).await {
                    tracing::error!("Failed to inject event: {}", e);
                } else {
                    tracing::trace!("✓ Event injected: {:?}", event);
                }
            }
            _ = &mut ctrl_c => {
                tracing::info!("Received Ctrl+C, stopping...");
                break;
            }
        }
    }

    tracing::info!("Agent stopping...");
    network.stop().await;
    tracing::info!("Agent stopped");

    Ok(())
}
