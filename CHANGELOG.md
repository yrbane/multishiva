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
- **Comprehensive Testing**: 105 tests covering all modules
  - Unit tests for each module
  - Integration tests for complete workflows
  - Simulation mode tests
  - Network communication tests

### Technical Details
- Built with Rust 2021 edition
- Async runtime using Tokio
- TDD (Test-Driven Development) methodology
- CI/CD with GitHub Actions
- Pre-commit hooks for code quality
- Cross-platform support (Linux, macOS, Windows)

### Documentation
- Comprehensive README with quick start guide
- YAML configuration examples
- API documentation via rustdoc
- Contributing guidelines
- MIT License

[0.1.0]: https://github.com/yrbane/multishiva/releases/tag/v0.1.0
