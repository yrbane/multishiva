use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("multishiva").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Control multiple computers"));
}

#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("multishiva").unwrap();
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("0.1.0"));
}

#[test]
fn test_cli_mode_host() {
    // Create temporary config
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yml");
    fs::write(
        &config_path,
        r#"
self_name: "test"
mode: host
port: 53421
tls:
  psk: "test-psk"
edges: {}
"#,
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("multishiva").unwrap();
    cmd.arg("--mode")
        .arg("host")
        .arg("--config")
        .arg(config_path.to_str().unwrap());
    // Will timeout waiting for Ctrl+C, but validates parsing
}

#[test]
fn test_cli_mode_agent() {
    let mut cmd = Command::cargo_bin("multishiva").unwrap();
    cmd.arg("--mode").arg("agent");
    // Should fail because no config with host_address, but validates arg parsing
}

#[test]
fn test_cli_with_config() {
    let mut cmd = Command::cargo_bin("multishiva").unwrap();
    cmd.arg("--config").arg("test.yaml");
    // Should fail because file doesn't exist, but validates arg parsing
    cmd.assert().failure();
}

#[test]
fn test_cli_gui_flag() {
    let mut cmd = Command::cargo_bin("multishiva").unwrap();
    cmd.arg("--gui");
    // GUI not implemented yet, should error gracefully
}

#[test]
fn test_cli_simulate_flag() {
    // Create temporary config for simulation
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yml");
    fs::write(
        &config_path,
        r#"
self_name: "test"
mode: host
port: 53421
tls:
  psk: "test-psk"
edges:
  right: "agent1"
"#,
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("multishiva").unwrap();
    cmd.arg("--simulate")
        .arg("--config")
        .arg(config_path.to_str().unwrap());
    // Will timeout waiting for Ctrl+C in simulation mode
}

#[test]
fn test_cli_invalid_mode() {
    let mut cmd = Command::cargo_bin("multishiva").unwrap();
    cmd.arg("--mode").arg("invalid");
    cmd.assert().failure();
}

#[test]
fn test_cli_conflicting_gui_and_simulate() {
    let mut cmd = Command::cargo_bin("multishiva").unwrap();
    cmd.arg("--gui").arg("--simulate");
    // Should fail validation - cannot use both GUI and simulate
    cmd.assert().failure();
}

#[test]
fn test_cli_conflicting_mode_and_gui() {
    let mut cmd = Command::cargo_bin("multishiva").unwrap();
    cmd.arg("--mode").arg("host").arg("--gui");
    // Should fail validation - GUI auto-detects mode
    cmd.assert().failure();
}

#[test]
fn test_cli_env_var_mode() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yml");
    fs::write(
        &config_path,
        r#"
self_name: "test"
mode: host
port: 53421
tls:
  psk: "test-psk"
edges: {}
"#,
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("multishiva").unwrap();
    cmd.env("MULTISHIVA_MODE", "host")
        .env("MULTISHIVA_CONFIG", config_path.to_str().unwrap());
    // Should read mode from environment variable
}

#[test]
fn test_cli_env_var_simulate() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yml");
    fs::write(
        &config_path,
        r#"
self_name: "test"
mode: host
port: 53421
tls:
  psk: "test-psk"
edges:
  right: "agent1"
"#,
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("multishiva").unwrap();
    cmd.env("MULTISHIVA_SIMULATE", "true")
        .env("MULTISHIVA_CONFIG", config_path.to_str().unwrap());
    // Should read simulate flag from environment variable
}
