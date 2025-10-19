use multishiva::core::config::Config;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_config_load_valid() {
    let yaml_content = r#"
self_name: test-host
mode: host
port: 53421
tls:
  psk: test-key
edges: {}
"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(yaml_content.as_bytes()).unwrap();

    let config = Config::from_file(temp_file.path().to_str().unwrap());
    assert!(config.is_ok());

    let config = config.unwrap();
    assert_eq!(config.self_name, "test-host");
    assert_eq!(config.port, 53421);
}

#[test]
fn test_config_load_invalid_file() {
    let result = Config::from_file("/nonexistent/path.yaml");
    assert!(result.is_err());
}
