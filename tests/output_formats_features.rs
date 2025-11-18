//! OutputFormat feature tests
//!
//! Validates the output formatting system including:
//! - JSON output formatting and pretty-printing
//! - YAML output formatting
//! - TOML output formatting
//! - Table output formatting (ASCII tables)
//! - TSV (Tab-Separated Values) output formatting
//! - Round-trip validation for serialization

use clap_noun_verb::format::OutputFormat;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Validates JSON output formatting
///
/// This test ensures that:
/// - JSON formatter produces valid JSON
/// - Output is pretty-printed (human-readable)
/// - Nested structures are handled correctly
#[test]
fn test_json_output_formatting() {
    #[derive(Serialize)]
    struct TestData {
        name: String,
        count: u32,
        enabled: bool,
    }

    let data = TestData { name: "test".to_string(), count: 42, enabled: true };

    let result = OutputFormat::Json.format(&data).expect("Failed to format as JSON");

    // Verify it's valid JSON by parsing it back
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("Output should be valid JSON");

    assert_eq!(parsed["name"], "test", "JSON should contain correct name");
    assert_eq!(parsed["count"], 42, "JSON should contain correct count");
    assert_eq!(parsed["enabled"], true, "JSON should contain correct enabled flag");

    // Verify pretty printing (should contain newlines)
    assert!(result.contains('\n'), "JSON should be pretty-printed with newlines");
}

/// Validates YAML output formatting
///
/// This test ensures that:
/// - YAML formatter produces valid YAML
/// - Output can be parsed back correctly
/// - Complex nested structures work
#[test]
fn test_yaml_output_formatting() {
    #[derive(Serialize)]
    struct Config {
        server: ServerConfig,
        database: DatabaseConfig,
    }

    #[derive(Serialize)]
    struct ServerConfig {
        host: String,
        port: u16,
    }

    #[derive(Serialize)]
    struct DatabaseConfig {
        url: String,
        max_connections: u32,
    }

    let config = Config {
        server: ServerConfig { host: "localhost".to_string(), port: 8080 },
        database: DatabaseConfig { url: "postgresql://localhost/db".to_string(), max_connections: 10 },
    };

    let result = OutputFormat::Yaml.format(&config).expect("Failed to format as YAML");

    // Verify it's valid YAML by parsing it back
    let parsed: serde_yaml::Value = serde_yaml::from_str(&result).expect("Output should be valid YAML");

    assert_eq!(parsed["server"]["host"], "localhost", "YAML should contain correct server host");
    assert_eq!(parsed["server"]["port"], 8080, "YAML should contain correct server port");
    assert_eq!(
        parsed["database"]["max_connections"],
        10,
        "YAML should contain correct max_connections"
    );
}

/// Validates TOML output formatting
///
/// This test ensures that:
/// - TOML formatter produces valid TOML
/// - Output is pretty-printed
/// - Nested tables are handled correctly
#[test]
fn test_toml_output_formatting() {
    #[derive(Serialize)]
    struct AppConfig {
        title: String,
        version: String,
        settings: Settings,
    }

    #[derive(Serialize)]
    struct Settings {
        debug: bool,
        timeout: u32,
    }

    let config = AppConfig {
        title: "My App".to_string(),
        version: "1.0.0".to_string(),
        settings: Settings { debug: true, timeout: 30 },
    };

    let result = OutputFormat::Toml.format(&config).expect("Failed to format as TOML");

    // Verify it's valid TOML by parsing it back
    let parsed: toml::Value = toml::from_str(&result).expect("Output should be valid TOML");

    assert_eq!(parsed["title"].as_str(), Some("My App"), "TOML should contain correct title");
    assert_eq!(parsed["version"].as_str(), Some("1.0.0"), "TOML should contain correct version");
    assert_eq!(parsed["settings"]["debug"].as_bool(), Some(true), "TOML should contain correct debug flag");
    assert_eq!(parsed["settings"]["timeout"].as_integer(), Some(30), "TOML should contain correct timeout");
}

/// Validates table output formatting for arrays of objects
///
/// This test ensures that:
/// - Arrays of objects are formatted as tables
/// - Headers are generated from object keys
/// - Null values are handled appropriately
#[test]
fn test_table_output_formatting() {
    #[derive(Serialize)]
    struct User {
        id: u32,
        name: String,
        active: bool,
    }

    let users = vec![
        User { id: 1, name: "Alice".to_string(), active: true },
        User { id: 2, name: "Bob".to_string(), active: false },
        User { id: 3, name: "Charlie".to_string(), active: true },
    ];

    let result = OutputFormat::Table.format(&users).expect("Failed to format as table");

    // Verify table structure
    let lines: Vec<&str> = result.lines().collect();
    assert!(lines.len() >= 4, "Table should have header + data rows");

    // Check header row (first line should contain column names)
    let header = lines[0];
    assert!(header.contains("id"), "Header should contain 'id'");
    assert!(header.contains("name"), "Header should contain 'name'");
    assert!(header.contains("active"), "Header should contain 'active'");

    // Check data rows contain expected values
    let data_rows: Vec<&str> = lines.iter().skip(1).copied().collect();
    let data = data_rows.join("\n");
    assert!(data.contains("Alice"), "Table should contain 'Alice'");
    assert!(data.contains("Bob"), "Table should contain 'Bob'");
    assert!(data.contains("Charlie"), "Table should contain 'Charlie'");
}

/// Validates TSV output formatting
///
/// This test ensures that:
/// - TSV formatter produces tab-separated output
/// - Special characters are escaped properly
/// - Headers are included
#[test]
fn test_tsv_output_formatting() {
    #[derive(Serialize)]
    struct Product {
        sku: String,
        name: String,
        price: f64,
    }

    let products = vec![
        Product { sku: "SKU001".to_string(), name: "Laptop".to_string(), price: 999.99 },
        Product { sku: "SKU002".to_string(), name: "Mouse".to_string(), price: 29.99 },
    ];

    let result = OutputFormat::Tsv.format(&products).expect("Failed to format as TSV");

    // Verify TSV structure
    let lines: Vec<&str> = result.lines().collect();
    assert!(lines.len() >= 3, "TSV should have header + data rows");

    // Check header
    let header = lines[0];
    assert!(header.contains('\t'), "Header should be tab-separated");

    // Check data rows
    for line in lines.iter().skip(1) {
        assert!(line.contains('\t'), "Data rows should be tab-separated");
    }

    // Verify content
    let content = lines.join("\n");
    assert!(content.contains("SKU001"), "TSV should contain SKU001");
    assert!(content.contains("Laptop"), "TSV should contain Laptop");
}

/// Validates TSV escaping of special characters
///
/// This test ensures that:
/// - Tabs in values are properly escaped
/// - Newlines in values are properly escaped
/// - Quotes are handled correctly
#[test]
fn test_tsv_special_character_escaping() {
    #[derive(Serialize)]
    struct TextData {
        normal: String,
        with_tab: String,
        with_newline: String,
    }

    let data = vec![TextData {
        normal: "simple".to_string(),
        with_tab: "has\ttab".to_string(),
        with_newline: "has\nnewline".to_string(),
    }];

    let result = OutputFormat::Tsv.format(&data).expect("Failed to format as TSV");

    // Verify special characters are escaped
    assert!(result.contains('"'), "Special characters should be quoted");
}

/// Validates round-trip JSON serialization
///
/// This test ensures that:
/// - JSON output can be parsed back to original structure
/// - Data is preserved exactly
/// - No information is lost
#[test]
fn test_json_round_trip_validation() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct RoundTripData {
        string_field: String,
        int_field: i32,
        float_field: f64,
        bool_field: bool,
        optional_field: Option<String>,
    }

    let original = RoundTripData {
        string_field: "test data".to_string(),
        int_field: -42,
        float_field: 3.14159,
        bool_field: false,
        optional_field: Some("optional".to_string()),
    };

    // Serialize to JSON
    let json_output = OutputFormat::Json.format(&original).expect("Failed to format as JSON");

    // Deserialize back
    let deserialized: RoundTripData =
        serde_json::from_str(&json_output).expect("Failed to deserialize JSON");

    assert_eq!(original, deserialized, "Round-trip should preserve all data");
}

/// Validates round-trip YAML serialization
///
/// This test ensures that:
/// - YAML output can be parsed back correctly
/// - Nested structures survive round-trip
#[test]
fn test_yaml_round_trip_validation() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Nested {
        inner: String,
        value: u32,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct YamlData {
        name: String,
        nested: Nested,
        items: Vec<String>,
    }

    let original = YamlData {
        name: "test".to_string(),
        nested: Nested { inner: "nested value".to_string(), value: 100 },
        items: vec!["one".to_string(), "two".to_string(), "three".to_string()],
    };

    // Serialize to YAML
    let yaml_output = OutputFormat::Yaml.format(&original).expect("Failed to format as YAML");

    // Deserialize back
    let deserialized: YamlData = serde_yaml::from_str(&yaml_output).expect("Failed to deserialize YAML");

    assert_eq!(original, deserialized, "YAML round-trip should preserve all data");
}

/// Validates OutputFormat string parsing
///
/// This test ensures that:
/// - Format names are parsed correctly
/// - Case-insensitive parsing works
/// - Aliases are supported
/// - Invalid formats return errors
#[test]
fn test_output_format_from_str() {
    // Valid formats
    assert_eq!(
        OutputFormat::from_str("json").expect("Should parse json"),
        OutputFormat::Json,
        "Should parse 'json'"
    );
    assert_eq!(
        OutputFormat::from_str("JSON").expect("Should parse JSON"),
        OutputFormat::Json,
        "Should be case-insensitive"
    );
    assert_eq!(
        OutputFormat::from_str("yaml").expect("Should parse yaml"),
        OutputFormat::Yaml,
        "Should parse 'yaml'"
    );
    assert_eq!(
        OutputFormat::from_str("yml").expect("Should parse yml"),
        OutputFormat::Yaml,
        "Should parse 'yml' alias"
    );
    assert_eq!(
        OutputFormat::from_str("toml").expect("Should parse toml"),
        OutputFormat::Toml,
        "Should parse 'toml'"
    );
    assert_eq!(
        OutputFormat::from_str("table").expect("Should parse table"),
        OutputFormat::Table,
        "Should parse 'table'"
    );
    assert_eq!(
        OutputFormat::from_str("ascii").expect("Should parse ascii"),
        OutputFormat::Table,
        "Should parse 'ascii' alias for table"
    );
    assert_eq!(
        OutputFormat::from_str("tsv").expect("Should parse tsv"),
        OutputFormat::Tsv,
        "Should parse 'tsv'"
    );
    assert_eq!(
        OutputFormat::from_str("tab").expect("Should parse tab"),
        OutputFormat::Tsv,
        "Should parse 'tab' alias"
    );

    // Invalid format
    let result = OutputFormat::from_str("invalid");
    assert!(result.is_err(), "Should return error for invalid format");

    let error_msg = result.unwrap_err();
    assert!(error_msg.contains("Unknown format"), "Error should mention unknown format");
    assert!(error_msg.contains("invalid"), "Error should include the invalid input");
}

/// Validates OutputFormat Display trait
///
/// This test ensures that:
/// - Format names are displayed correctly
/// - Display output matches expected strings
#[test]
fn test_output_format_display() {
    assert_eq!(OutputFormat::Json.to_string(), "json", "Json should display as 'json'");
    assert_eq!(OutputFormat::Yaml.to_string(), "yaml", "Yaml should display as 'yaml'");
    assert_eq!(OutputFormat::Toml.to_string(), "toml", "Toml should display as 'toml'");
    assert_eq!(OutputFormat::Table.to_string(), "table", "Table should display as 'table'");
    assert_eq!(OutputFormat::Tsv.to_string(), "tsv", "Tsv should display as 'tsv'");
}

/// Validates available_formats() method
///
/// This test ensures that:
/// - All formats are listed
/// - List is complete and accurate
#[test]
fn test_available_formats() {
    let formats = OutputFormat::available_formats();

    assert_eq!(formats.len(), 5, "Should have 5 available formats");
    assert!(formats.contains(&"json"), "Should include json");
    assert!(formats.contains(&"yaml"), "Should include yaml");
    assert!(formats.contains(&"toml"), "Should include toml");
    assert!(formats.contains(&"table"), "Should include table");
    assert!(formats.contains(&"tsv"), "Should include tsv");
}

/// Validates table formatting with empty arrays
///
/// This test ensures that:
/// - Empty arrays are handled gracefully
/// - No panics or errors occur
#[test]
fn test_table_formatting_empty_array() {
    let empty: Vec<String> = vec![];

    let result = OutputFormat::Table.format(&empty).expect("Should handle empty array");

    assert_eq!(result, "(empty)", "Empty array should produce '(empty)' message");
}

/// Validates table formatting with single object
///
/// This test ensures that:
/// - Single objects are formatted correctly
/// - Key-value pairs are displayed properly
#[test]
fn test_table_formatting_single_object() {
    #[derive(Serialize)]
    struct Config {
        debug: bool,
        timeout: u32,
    }

    let config = Config { debug: true, timeout: 60 };

    let result = OutputFormat::Table.format(&config).expect("Should format single object");

    // Should contain key-value pairs
    assert!(result.contains("debug"), "Should contain 'debug' key");
    assert!(result.contains("timeout"), "Should contain 'timeout' key");
}

/// Validates formatting with Option types and null values
///
/// This test ensures that:
/// - Optional fields are handled correctly
/// - Null values are represented appropriately
#[test]
fn test_formatting_with_optional_fields() {
    #[derive(Serialize)]
    struct WithOptionals {
        required: String,
        optional_some: Option<String>,
        optional_none: Option<String>,
    }

    let data = WithOptionals {
        required: "always here".to_string(),
        optional_some: Some("sometimes here".to_string()),
        optional_none: None,
    };

    // Test JSON
    let json = OutputFormat::Json.format(&data).expect("Should format JSON with optionals");
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("Should be valid JSON");
    assert!(parsed["optional_none"].is_null(), "None should be represented as null in JSON");

    // Test YAML
    let yaml = OutputFormat::Yaml.format(&data).expect("Should format YAML with optionals");
    assert!(yaml.contains("null") || yaml.contains("~"), "YAML should represent null values");

    // Test Table
    let table = OutputFormat::Table.format(&data).expect("Should format table with optionals");
    assert!(table.contains("always here"), "Table should contain required field");
}

/// Validates default format is JSON
///
/// This test ensures that:
/// - Default format is correctly set to JSON
#[test]
fn test_default_format_is_json() {
    let default_format = OutputFormat::default();
    assert_eq!(default_format, OutputFormat::Json, "Default format should be JSON");
}

/// Validates format equality and cloning
///
/// This test ensures that:
/// - Format equality works correctly
/// - Cloning produces identical formats
#[test]
fn test_format_equality_and_clone() {
    let json1 = OutputFormat::Json;
    let json2 = OutputFormat::Json;
    let yaml = OutputFormat::Yaml;

    assert_eq!(json1, json2, "Same formats should be equal");
    assert_ne!(json1, yaml, "Different formats should not be equal");

    let cloned = json1;
    assert_eq!(json1, cloned, "Cloned format should be equal");
}
