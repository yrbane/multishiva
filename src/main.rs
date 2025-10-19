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

async fn run_production_mode(config: Config, _topology: Topology) -> Result<()> {
    tracing::info!("🚀 Running in PRODUCTION mode");

    let focus = FocusManager::new(config.self_name.clone());
    tracing::debug!("Focus manager initialized for: {}", config.self_name);

    match config.mode {
        ConfigMode::Host => run_host_mode(config, focus).await,
        ConfigMode::Agent => {
            let host_address = config.host_address.clone().ok_or_else(|| {
                anyhow::anyhow!("host_address must be specified in config for agent mode")
            })?;
            run_agent_mode(config, focus, &host_address).await
        }
    }
}

async fn run_host_mode(config: Config, mut _focus: FocusManager) -> Result<()> {
    tracing::info!("Starting as HOST on port {}", config.port);

    let mut network = Network::new(config.tls.psk.clone());

    // Start host server
    let actual_port = network.start_host(config.port).await?;
    tracing::info!("✓ Host listening on port {}", actual_port);

    tracing::info!("Waiting for agents to connect...");
    tracing::info!("Press Ctrl+C to exit");

    // Run until Ctrl+C
    signal::ctrl_c().await?;

    tracing::info!("Host stopping...");
    network.stop().await;
    tracing::info!("Host stopped");

    Ok(())
}

async fn run_agent_mode(
    config: Config,
    mut _focus: FocusManager,
    host_address: &str,
) -> Result<()> {
    tracing::info!("Starting as AGENT, connecting to: {}", host_address);

    let mut network = Network::new(config.tls.psk.clone());

    // Connect to host
    network.connect_to_host(host_address).await?;
    tracing::info!("✓ Connected to host at {}", host_address);

    tracing::info!("Press Ctrl+C to exit");

    // Run until Ctrl+C
    signal::ctrl_c().await?;

    tracing::info!("Agent stopping...");
    network.stop().await;
    tracing::info!("Agent stopped");

    Ok(())
}
