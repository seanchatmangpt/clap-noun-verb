#![cfg(feature = "config-formats")]
//! Tests for advanced configuration features
//! - Environment variable interpolation
//! - Profile-based merging

use clap_noun_verb::config::ConfigLoader;
use std::env;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_interpolation() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("test_config.toml");

    // Set environment variable
    env::set_var("TEST_PORT", "9999");
    env::set_var("TEST_HOST", "my-secret-host");

    let content = r#"
port = ${TEST_PORT}
host = "${TEST_HOST}"
nested.key = "${TEST_PORT}-value"
"#;
    fs::write(&config_path, content).unwrap();

    // Act
    let config = ConfigLoader::new().with_path(&config_path).load().expect("Should load config");

    // Assert
    assert_eq!(config.get_string("port"), Some("9999".to_string()));
    assert_eq!(config.get_string("host"), Some("my-secret-host".to_string()));
    let nested = config.get("nested").unwrap();
    assert_eq!(nested["key"], "9999-value");

    // Clean up
    env::remove_var("TEST_PORT");
    env::remove_var("TEST_HOST");
}

#[test]
fn test_profiles() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("profile_config.toml");

    let content = r#"
port = 8080
host = "localhost"

[production]
port = 80
host = "api.production.com"
verbose = true

[staging]
port = 8081
"#;
    fs::write(&config_path, content).unwrap();

    // 1. Test Default (no profile)
    let config_default =
        ConfigLoader::new().with_path(&config_path).load().expect("Should load config");

    assert_eq!(config_default.get_string("port"), Some("8080".to_string()));
    assert_eq!(config_default.get_string("host"), Some("localhost".to_string()));

    // 2. Test Production Profile
    let config_prod = ConfigLoader::new()
        .with_path(&config_path)
        .with_profile("production")
        .load()
        .expect("Should load production config");

    assert_eq!(config_prod.get_string("port"), Some("80".to_string()));
    assert_eq!(config_prod.get_string("host"), Some("api.production.com".to_string()));
    assert_eq!(config_prod.get_string("verbose"), Some("true".to_string()));

    // 3. Test Staging Profile (merging)
    let config_staging = ConfigLoader::new()
        .with_path(&config_path)
        .with_profile("staging")
        .load()
        .expect("Should load staging config");

    assert_eq!(config_staging.get_string("port"), Some("8081".to_string()));
    assert_eq!(config_staging.get_string("host"), Some("localhost".to_string()));
    // From base
}

#[test]
fn test_env_profile() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("env_profile_config.toml");

    let content = r#"
[prod]
host = "prod"
"#;
    fs::write(&config_path, content).unwrap();

    env::set_var("APP_ENV", "prod");

    // Act
    let config = ConfigLoader::new()
        .with_path(&config_path)
        .with_env_profile("APP_ENV")
        .load()
        .expect("Should load config");

    // Assert
    assert_eq!(config.get_string("host"), Some("prod".to_string()));

    env::remove_var("APP_ENV");
}

#[test]
fn test_nested_profiles() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("nested_profile_config.toml");

    let content = r#"
[database]
url = "localhost"
pool = 5

[prod]
[prod.database]
url = "db.prod.com"
"#;
    fs::write(&config_path, content).unwrap();

    // Act
    let config = ConfigLoader::new()
        .with_path(&config_path)
        .with_profile("prod")
        .load()
        .expect("Should load config");

    // Assert
    let db = config.get("database").unwrap();
    assert_eq!(db["url"], "db.prod.com");
    assert_eq!(db["pool"], "5"); // Flattened to string
}
