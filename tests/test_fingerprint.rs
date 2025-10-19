use multishiva::core::fingerprint::{Fingerprint, FingerprintStore};
use tempfile::TempDir;

#[test]
fn test_fingerprint_creation() {
    let fp = Fingerprint::new("test-machine", "abc123def456");
    assert_eq!(fp.machine_name(), "test-machine");
    assert_eq!(fp.hash(), "abc123def456");
}

#[test]
fn test_fingerprint_from_cert_data() {
    // Simulate certificate data
    let cert_data = b"test certificate data";
    let fp = Fingerprint::from_cert_data("machine1", cert_data);

    // Should create a SHA-256 hash
    assert_eq!(fp.machine_name(), "machine1");
    assert!(!fp.hash().is_empty());
    assert_eq!(fp.hash().len(), 64); // SHA-256 produces 64 hex chars
}

#[test]
fn test_fingerprint_equality() {
    let fp1 = Fingerprint::new("machine", "hash123");
    let fp2 = Fingerprint::new("machine", "hash123");
    let fp3 = Fingerprint::new("machine", "hash456");

    assert_eq!(fp1, fp2);
    assert_ne!(fp1, fp3);
}

#[test]
fn test_fingerprint_store_create() {
    let temp_dir = TempDir::new().unwrap();
    let store_path = temp_dir.path().join("fingerprints.json");

    let store = FingerprintStore::new(store_path.clone());
    assert!(store.is_ok());
}

#[test]
fn test_fingerprint_store_save_and_load() {
    let temp_dir = TempDir::new().unwrap();
    let store_path = temp_dir.path().join("fingerprints.json");

    let mut store = FingerprintStore::new(store_path.clone()).unwrap();

    // Save a fingerprint
    let fp = Fingerprint::new("machine1", "hash123abc");
    store.save("machine1", fp.clone()).unwrap();

    // Load it back
    let loaded = store.get("machine1");
    assert!(loaded.is_some());
    assert_eq!(loaded.unwrap(), &fp);
}

#[test]
fn test_fingerprint_store_get_nonexistent() {
    let temp_dir = TempDir::new().unwrap();
    let store_path = temp_dir.path().join("fingerprints.json");

    let store = FingerprintStore::new(store_path).unwrap();

    let result = store.get("nonexistent");
    assert!(result.is_none());
}

#[test]
fn test_fingerprint_store_update() {
    let temp_dir = TempDir::new().unwrap();
    let store_path = temp_dir.path().join("fingerprints.json");

    let mut store = FingerprintStore::new(store_path).unwrap();

    // Save initial fingerprint
    let fp1 = Fingerprint::new("machine1", "hash123");
    store.save("machine1", fp1).unwrap();

    // Update with new fingerprint
    let fp2 = Fingerprint::new("machine1", "hash456");
    store.save("machine1", fp2.clone()).unwrap();

    // Should have new fingerprint
    let loaded = store.get("machine1").unwrap();
    assert_eq!(loaded, &fp2);
}

#[test]
fn test_fingerprint_store_persistence() {
    let temp_dir = TempDir::new().unwrap();
    let store_path = temp_dir.path().join("fingerprints.json");

    // Create store and save fingerprint
    {
        let mut store = FingerprintStore::new(store_path.clone()).unwrap();
        let fp = Fingerprint::new("machine1", "persistent_hash");
        store.save("machine1", fp).unwrap();
    }

    // Load in new instance
    {
        let store = FingerprintStore::new(store_path).unwrap();
        let loaded = store.get("machine1");
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().hash(), "persistent_hash");
    }
}

#[test]
fn test_fingerprint_verification_match() {
    let fp = Fingerprint::new("machine1", "hash123");
    let cert_hash = "hash123";

    assert!(fp.verify(cert_hash));
}

#[test]
fn test_fingerprint_verification_mismatch() {
    let fp = Fingerprint::new("machine1", "hash123");
    let cert_hash = "different_hash";

    assert!(!fp.verify(cert_hash));
}

#[test]
fn test_fingerprint_store_remove() {
    let temp_dir = TempDir::new().unwrap();
    let store_path = temp_dir.path().join("fingerprints.json");

    let mut store = FingerprintStore::new(store_path).unwrap();

    let fp = Fingerprint::new("machine1", "hash123");
    store.save("machine1", fp).unwrap();

    assert!(store.get("machine1").is_some());

    store.remove("machine1").unwrap();
    assert!(store.get("machine1").is_none());
}

#[test]
fn test_fingerprint_store_list_all() {
    let temp_dir = TempDir::new().unwrap();
    let store_path = temp_dir.path().join("fingerprints.json");

    let mut store = FingerprintStore::new(store_path).unwrap();

    store
        .save("machine1", Fingerprint::new("machine1", "hash1"))
        .unwrap();
    store
        .save("machine2", Fingerprint::new("machine2", "hash2"))
        .unwrap();
    store
        .save("machine3", Fingerprint::new("machine3", "hash3"))
        .unwrap();

    let all = store.list_all();
    assert_eq!(all.len(), 3);
}

#[test]
fn test_fingerprint_default_store_path() {
    let default_path = FingerprintStore::default_path();

    // Should be in ~/.config/multishiva/fingerprints.json or equivalent
    assert!(default_path.to_str().unwrap().contains("multishiva"));
    assert!(default_path
        .to_str()
        .unwrap()
        .ends_with("fingerprints.json"));
}

#[test]
fn test_fingerprint_hash_consistency() {
    let cert_data = b"test certificate data";

    let fp1 = Fingerprint::from_cert_data("machine", cert_data);
    let fp2 = Fingerprint::from_cert_data("machine", cert_data);

    // Same data should produce same hash
    assert_eq!(fp1.hash(), fp2.hash());
}
