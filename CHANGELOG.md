# Changelog

All notable changes to MultiShiva will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-01-09

### Added
- Initial release of MultiShiva core functionality
- **Configuration Module**: YAML-based configuration with validation
- **Event System**: Comprehensive event types with MessagePack serialization
  - Mouse events (move, click, scroll)
  - Keyboard events with full key mapping
  - Focus transfer events
  - Heartbeat for connection monitoring
- **Topology Management**: Spatial screen layout with edge detection
  - Support for all edges (top, bottom, left, right)
  - Configurable edge threshold
  - Neighbor detection
- **Focus Manager**: Intelligent focus transfer across machines
  - Focus history tracking
  - Friction delay support
  - Automatic return-to-host
- **Network Layer**: Secure TCP communication
  - PSK (Pre-Shared Key) authentication
  - Heartbeat mechanism
  - Connection management for multiple agents
  - Automatic reconnection handling
- **Input Handling**: Cross-platform keyboard/mouse capture and injection
  - Event capture using rdev
  - Event injection with permission checks
  - Kill-switch hotkey support
  - Local input blocking capability
- **Simulation Mode**: Testing without physical hardware
  - Virtual machine management
  - Network latency simulation
  - Event recording and replay
  - Statistics tracking
- **Command-Line Interface**: Flexible operation modes
  - Host and agent modes
  - Simulation mode flag
  - Configuration file support
  - Environment variable support (MULTISHIVA_MODE, MULTISHIVA_CONFIG, etc.)
  - Argument validation with conflict detection
  - GUI stub for future Tauri integration
- **Comprehensive Testing**: 101 tests covering all modules
  - 17 library tests (config, events, focus, network, etc.)
  - 12 CLI tests (validation, env vars, conflicts)
  - 10 integration tests (end-to-end workflows)
  - 9 network tests (host-agent communication)
  - 15 simulation tests (virtual machines, latency)
  - 11 topology tests (edge detection, positioning)
  - 8 config tests (YAML parsing, validation)
  - 8 event tests (serialization, types)
  - 8 focus tests (transfer, history)
  - 3 security tests (TLS, PSK authentication)

### Technical Details
- Built with Rust 2021 edition
- Async runtime using Tokio
- TDD (Test-Driven Development) methodology
- CI/CD with GitHub Actions (multi-OS testing + release builds)
- Pre-commit hooks for code quality (rustfmt + clippy)
- Code coverage >80% with tarpaulin
- Cross-platform support (Linux, macOS, Windows)
- Feature flags for optional GUI support

### Documentation
- Comprehensive README with quick start guide
- YAML configuration examples
- API documentation via rustdoc
- Contributing guidelines
- MIT License

[0.1.0]: https://github.com/yrbane/multishiva/releases/tag/v0.1.0
