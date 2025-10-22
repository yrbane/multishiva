# Changelog

All notable changes to MultiShiva will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.1.0] - 2025-10-22

### Added
- **Native Wayland Support via evdev**: Linux-specific input handling for Wayland and X11
  - New `input_evdev.rs` module with direct `/dev/input/event*` access
  - Auto-detection of mouse and keyboard devices
  - Relative event accumulation (REL_X, REL_Y) for accurate mouse tracking
  - Proper async/blocking bridge between evdev threads and Tokio runtime
  - Compatible with both Wayland and X11 display servers
  - Requires user to be in `input` group

- **Device Grabbing (Linux)**: Intelligent input blocking with EVIOCGRAB
  - Exclusive device grab via EVIOCGRAB ioctl (0x40044590)
  - Automatic grabbing when focus transfers to remote machine
  - Prevents local OS from processing events while controlling remote
  - Automatic ungrab when focus returns to host
  - `grab_devices()` and `ungrab_devices()` methods in EvdevInputHandler

- **Bidirectional Focus Control**: Seamless focus transfer in both directions
  - Agent-side edge detection for automatic focus return
  - New `FocusRelease` event sent from agent to host
  - Second channel pair for agent→host communication (`agent_tx`/`agent_rx`)
  - `send_event_to_host()` method for agent to send events back
  - Host listens for `FocusRelease` and resumes local control
  - Complete flow: Linux→Mac at left edge, Mac→Linux at right edge

- **Enhanced Network Communication**: Full bidirectional event flow
  - Host can now receive events from agents (not just send)
  - Length-prefixed messages support both directions (4-byte BE + data)
  - Heartbeat mechanism integrated with bidirectional channels
  - `input_event_tx` parameter in `start_host()` for event forwarding

- **Automatic Host Discovery via mDNS**: Zero-configuration agent mode
  - Agents can now auto-discover hosts on the network without specifying IP addresses
  - `discover_host_via_mdns()` function with 5-second timeout
  - Host automatically registers on mDNS at startup
  - Support for multiple hosts with warnings
  - Detailed troubleshooting in error messages

### Changed
- **Enhanced CLI Flexibility**: Command-line arguments now override config file
  - `--mode` flag now properly overrides config file mode setting
  - New `--host` argument for specifying agent host address
  - CLI overrides are logged for debugging
  - Improved first-run experience with helpful error messages

- **Linux Input System**: Platform-specific input handler selection
  - Linux (Wayland/X11): Uses `EvdevInputHandler` by default
  - macOS/Windows: Uses `RdevInputHandler` (unchanged)
  - Conditional compilation with `#[cfg(target_os = "linux")]`

- **Agent Mode**: Now monitors local input for focus return detection
  - Creates separate input handler for local capture
  - Detects right edge crossing when agent has focus
  - Sends `FocusRelease` to return control to host

### Fixed
- Fixed Tauri GUI module name reference (`app_lib` → `multishiva_gui_lib`)
- Fixed mDNS hostname format requirement (hostnames must end with `.local.`)
- Improved config file error message with step-by-step setup instructions
- Fixed dual cursor movement issue (events now properly blocked on host when focus is remote)
- Fixed evdev thread → tokio async boundary with proper channel bridging
- Updated all test files to pass new `input_event_tx` parameter to `start_host()`

### Security
- Device grabbing prevents local input leakage when controlling remote machine
- Proper cleanup of grabbed devices on focus return or application exit

### Documentation
- Completed comprehensive rustdoc documentation for all modules (closes #23)
- Added detailed bidirectional focus transfer flow diagram in README
- Documented Linux-specific requirements (input group membership)
- Explained device grabbing mechanism and behavior
- Updated roadmap: v1.1 marked as completed
- Added visual ASCII flow diagram for Host↔Agent transitions
- Enhanced architecture section with `input_evdev.rs` details

### Technical Details
- EVIOCGRAB ioctl constant: `0x40044590` (IOW('E', 0x90, int))
- Event bridge pattern: std::sync::mpsc → tokio::mpsc with polling loop
- Bidirectional network architecture with separate TX/RX channel pairs
- MessagePack serialization for cross-platform event transmission

## [1.0.0] - 2025-10-19

### Added
- **Complete Tauri GUI Implementation**: React + TypeScript interface
  - Interactive MachineGrid with drag-and-drop topology editor
  - Comprehensive SettingsPanel for configuration
  - Real-time StatusBar with connection monitoring
  - SecurityPanel for PSK management
- **Clipboard Synchronization**: Cross-machine clipboard sharing
- **Secure PSK Storage**: System keyring integration
- **mDNS Discovery**: Automatic peer detection
- **Config Persistence**: Automatic config saving with versioning
- **Comprehensive Logging**: Rotation and multi-level support

### Changed
- Project version bumped to 1.0.0
- Complete GUI workflow with all core features

## [0.3.0] - 2025-10-19

### Added
- **System Permissions Verification**: Multi-OS permission checking
  - macOS: Accessibility API permission detection
  - Linux: uinput and input group verification
  - Windows: Administrator privilege check
  - Automatic permission check on startup (production mode only)
  - Detailed help messages for fixing permissions on each OS
  - `permissions` module with `PermissionStatus` enum
  - 4 comprehensive tests for permission module

### Changed
- Main application now checks permissions before starting in production mode
- Helpful warning messages displayed if permissions are missing
- Application continues to run even with missing permissions (with warnings)

### Documentation
- Added OS-specific permission setup instructions
- Help text includes command-line examples for fixing permissions

## [0.2.0] - 2025-10-19

### Added
- **TLS Fingerprint Verification**: MITM attack detection
  - SHA-256 fingerprint calculation for PSK authentication
  - Persistent fingerprint storage in `~/.config/multishiva/fingerprints.json`
  - Automatic verification on each connection
  - Security warnings on fingerprint mismatch
  - First-connection fingerprint saving
- **Enhanced Network Security**:
  - Replaced DefaultHasher with cryptographically secure SHA-256
  - Machine name exchange during handshake
  - Fingerprint module with complete test coverage (14 tests)
- **Dependencies**:
  - Added `sha2` for secure hashing
  - Added `hex` for encoding
  - Added `chrono` for timestamps
  - Added `dirs` for config directory management
  - Added `hostname` for machine identification
  - Added `serde_json` for fingerprint persistence

### Changed
- Network handshake now includes machine name
- PSK hash computation uses SHA-256 instead of DefaultHasher
- Network module automatically loads and verifies fingerprints

### Security
- Protection against Man-in-the-Middle (MITM) attacks
- Cryptographically secure PSK hashing
- Fingerprint persistence and verification

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
