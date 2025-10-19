use multishiva::core::config::{Config, ConfigMode};
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_config_load_valid_host() {
    let yaml_content = r#"
self_name: test-host
mode: host
port: 53421
tls:
  psk: test-key
edges:
  right_of: laptop
  below: desktop2
"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(yaml_content.as_bytes()).unwrap();

    let config = Config::from_file(temp_file.path().to_str().unwrap());
    assert!(config.is_ok());

    let config = config.unwrap();
    assert_eq!(config.self_name, "test-host");
    assert_eq!(config.port, 53421);
    assert_eq!(config.tls.psk, "test-key");
    assert!(matches!(config.mode, ConfigMode::Host));
    assert_eq!(config.edges.get("right_of"), Some(&"laptop".to_string()));
    assert_eq!(config.edges.get("below"), Some(&"desktop2".to_string()));
}

#[test]
fn test_config_load_valid_agent() {
    let yaml_content = r#"
self_name: test-agent
mode: agent
port: 53421
tls:
  psk: agent-key
edges: {}
behavior:
  reconnect_delay_ms: 2000
"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(yaml_content.as_bytes()).unwrap();

    let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
    assert_eq!(config.self_name, "test-agent");
    assert!(matches!(config.mode, ConfigMode::Agent));
    assert_eq!(config.behavior.unwrap().reconnect_delay_ms, Some(2000));
}

#[test]
fn test_config_with_hotkeys() {
    let yaml_content = r#"
self_name: test-host
mode: host
port: 53421
tls:
  psk: test-key
edges: {}
hotkeys:
  focus_return: Ctrl+Ctrl
  kill_switch: Ctrl+Alt+Pause
"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(yaml_content.as_bytes()).unwrap();

    let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
    let hotkeys = config.hotkeys.unwrap();
    assert_eq!(hotkeys.focus_return, Some("Ctrl+Ctrl".to_string()));
    assert_eq!(hotkeys.kill_switch, Some("Ctrl+Alt+Pause".to_string()));
}

#[test]
fn test_config_with_behavior() {
    let yaml_content = r#"
self_name: test-host
mode: host
port: 53421
tls:
  psk: test-key
edges: {}
behavior:
  edge_threshold_px: 5
  friction_ms: 100
"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(yaml_content.as_bytes()).unwrap();

    let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
    let behavior = config.behavior.unwrap();
    assert_eq!(behavior.edge_threshold_px, Some(5));
    assert_eq!(behavior.friction_ms, Some(100));
}

#[test]
fn test_config_load_invalid_file() {
    let result = Config::from_file("/nonexistent/path.yaml");
    assert!(result.is_err());
}

#[test]
fn test_config_default_values() {
    let config = Config::default();
    assert_eq!(config.port, 53421);
    assert!(matches!(config.mode, ConfigMode::Host));
}

#[test]
fn test_config_validation_empty_psk() {
    let yaml_content = r#"
self_name: test-host
mode: host
port: 53421
tls:
  psk: ""
edges: {}
"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(yaml_content.as_bytes()).unwrap();

    let config = Config::from_file(temp_file.path().to_str().unwrap());
    assert!(config.is_ok()); // Should load but validation can happen later
}

#[test]
fn test_config_invalid_yaml() {
    let yaml_content = "invalid: yaml: content: [[[";

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(yaml_content.as_bytes()).unwrap();

    let result = Config::from_file(temp_file.path().to_str().unwrap());
    assert!(result.is_err());
}
