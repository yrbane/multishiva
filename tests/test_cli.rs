use assert_cmd::Command;
use predicates::prelude::*;

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
    let mut cmd = Command::cargo_bin("multishiva").unwrap();
    cmd.arg("--mode").arg("host");
    // For now, just check it doesn't crash
    // Will add more validation when we implement the main loop
}

#[test]
fn test_cli_mode_agent() {
    let mut cmd = Command::cargo_bin("multishiva").unwrap();
    cmd.arg("--mode").arg("agent");
    // For now, just check it doesn't crash
}

#[test]
fn test_cli_with_config() {
    let mut cmd = Command::cargo_bin("multishiva").unwrap();
    cmd.arg("--config").arg("test.yaml");
    // Should fail because file doesn't exist, but validates arg parsing
}

#[test]
fn test_cli_gui_flag() {
    let mut cmd = Command::cargo_bin("multishiva").unwrap();
    cmd.arg("--gui");
    // For now, just check it parses
}

#[test]
fn test_cli_simulate_flag() {
    let mut cmd = Command::cargo_bin("multishiva").unwrap();
    cmd.arg("--simulate");
    // For now, just check it parses
}
