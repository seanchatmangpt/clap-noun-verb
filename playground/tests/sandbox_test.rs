//! Tests for agent-sandbox feature including SyntheticCommandExecutor and MockRegistryDatabase

#[cfg(feature = "agent-sandbox")]
#[test]
fn test_mock_registry_integration() {
    use mcpp_cli::sandbox::{MOCK_REGISTRY, MockRegistryDatabase};
    use mcpp_cli::integration::registry_client::{RegistryClient, RegistryInfo, RegistrySource};

    // 1. Populate and activate mock registry
    {
        let mut db = MOCK_REGISTRY.lock().unwrap();
        db.clear();
        db.set_active(true);
        db.register_package(RegistryInfo {
            name: "mock-pack".to_string(),
            description: "A sandbox mock package".to_string(),
            versions: vec!["1.0.0".to_string()],
            latest_version: "1.0.0".to_string(),
            dependencies: vec![],
            homepage: None,
            repository: None,
        });
        db.add_source(RegistrySource {
            name: "sandbox-source".to_string(),
            url: "https://sandbox.registry.dev".to_string(),
            priority: 200,
        });
        db.set_healthy(true);
    }

    let client = RegistryClient::default();

    // Test health check
    let health = client.health_check().unwrap();
    assert!(health.healthy);
    assert_eq!(health.version, "0.1.0-mock");

    // Test get info
    let info = client.get_info("mock-pack").unwrap();
    assert_eq!(info.name, "mock-pack");
    assert_eq!(info.description, "A sandbox mock package");

    // Test search
    let search_results = client.search("mock", None, 10).unwrap();
    assert_eq!(search_results.len(), 1);
    assert_eq!(search_results[0].name, "mock-pack");

    // Test list sources
    let sources = client.list_sources().unwrap();
    assert_eq!(sources.len(), 2); // default mock-default + sandbox-source
    assert!(sources.iter().any(|s| s.name == "sandbox-source"));

    // Test download pack
    let pack_bytes = client.download_pack("mock-pack", "1.0.0").unwrap();
    assert_eq!(pack_bytes, b"MOCK_PACK_BYTES");

    // 2. Deactivate mock registry
    {
        let mut db = MOCK_REGISTRY.lock().unwrap();
        db.set_active(false);
    }

    // Now it should fallback (e.g. download_pack fails because marketplace is disabled)
    let info_fallback = client.get_info("mock-pack");
    #[cfg(not(feature = "reqwest"))]
    {
        assert!(info_fallback.is_ok());
        assert_eq!(info_fallback.unwrap().description, "Marketplace feature is disabled");
    }
}

#[cfg(feature = "agent-sandbox")]
#[test]
fn test_synthetic_executor_dry_run_intercept() {
    use mcpp_cli::sandbox::SyntheticCommandExecutor;

    // papers generate is Mutating, papers list is ReadOnly
    let executor_dry = SyntheticCommandExecutor::new(true);

    // Act: run mutating command with dry_run = true
    let output_mutating = executor_dry.execute(vec![
        "papers".to_string(),
        "generate".to_string(),
        "IMRaD".to_string(),
    ]).unwrap();

    // Assert: should be intercepted and return the status dry_run_intercepted
    let val = output_mutating.data;
    assert_eq!(val["status"], "dry_run_intercepted");
    assert_eq!(val["noun"], "papers");
    assert_eq!(val["verb"], "generate");

    // Act: run read-only command with dry_run = true
    let output_readonly = executor_dry.execute(vec![
        "papers".to_string(),
        "list".to_string(),
    ]).unwrap();

    // Assert: should NOT be intercepted and returns Null data (since it prints directly to stdout)
    let val_ro = output_readonly.data;
    assert!(val_ro.is_null());
}
