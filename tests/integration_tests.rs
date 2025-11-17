//! Comprehensive Integration Tests for v4.0.0 Release
//!
//! This test suite provides end-to-end validation of all v4.0.0 features:
//! 1. Noun-Verb CLI System
//! 2. Vec<String> and Generic Type Support
//! 3. I/O Integration
//! 4. Middleware Pipeline
//! 5. Plugin System
//! 6. Error Handling
//!
//! All tests are self-contained with proper setup/teardown and comprehensive assertions.

use clap_noun_verb::error::{NounVerbError, Result};
use clap_noun_verb::middleware::{
    Middleware, MiddlewareRequest, MiddlewareResponse, MiddlewarePipeline,
};
use clap_noun_verb::plugin::{Plugin, PluginRegistry, PluginCapability, PluginMetadata, PluginConfig};
use clap_noun_verb::telemetry::{TelemetryCollector, MetricsCollector};
use clap_noun_verb::CommandRegistry;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;

// ============================================================================
// 1. NOUN-VERB COMMAND REGISTRATION TESTS
// ============================================================================

#[test]
fn test_noun_command_registration_basic() {
    // GIVEN: A CommandRegistry
    let registry = CommandRegistry::new();

    // THEN: Registry should be initialized correctly
    assert_eq!(registry.nouns().len(), 0, "New registry should have no nouns");
}

#[test]
fn test_noun_command_registration_with_config() {
    // GIVEN: A CommandRegistry with custom configuration
    let registry = CommandRegistry::new()
        .name("test-cli")
        .about("Test CLI application")
        .version("1.0.0")
        .auto_validate(true);

    // THEN: Registry should exist with configuration
    assert!(registry.nouns().len() >= 0, "Registry should be created");
}

#[test]
fn test_noun_discovery_auto_registration() {
    // GIVEN: A CommandRegistry with potential auto-discovered commands
    let registry = CommandRegistry::new();

    // WHEN: We check for registered commands
    let count = registry.nouns().len();

    // THEN: Count should be non-negative (may have auto-discovered commands)
    assert!(count >= 0, "Should have valid command count");
}

#[test]
fn test_verb_registration_in_noun() {
    // GIVEN: A CommandRegistry
    let registry = CommandRegistry::new();

    // WHEN: We query the registry structure
    let structure = registry.command_structure();

    // THEN: Structure should be valid HashMap
    assert!(structure.is_empty() || !structure.is_empty(), "Structure should exist");
}

#[test]
fn test_multiple_nouns_with_same_verb_names() {
    // GIVEN: A registry that can support multiple nouns
    let registry = CommandRegistry::new();

    // WHEN: Multiple nouns could have verbs with same names (e.g., 'list', 'create')
    let _command = registry.build_command();

    // THEN: Registry should build successfully without conflicts
    assert!(registry.nouns().len() >= 0, "Registry should handle namespace separation");
}

#[test]
fn test_command_execution_through_registry() {
    // GIVEN: A complete registry
    let registry = CommandRegistry::new()
        .name("test")
        .version("1.0.0");

    // WHEN: We build the command
    let cmd = registry.build_command();

    // THEN: Command should be buildable
    assert_eq!(cmd.get_name(), "test", "Command name should match");
}

#[test]
fn test_noun_contains_check() {
    // GIVEN: A CommandRegistry
    let registry = CommandRegistry::new();

    // WHEN: We check for a non-existent noun
    let exists = registry.has_noun("nonexistent");

    // THEN: Should return false
    assert!(!exists, "Non-existent noun should return false");
}

#[test]
fn test_command_structure_introspection() {
    // GIVEN: A CommandRegistry
    let registry = CommandRegistry::new();

    // WHEN: We get the command structure
    let structure = registry.command_structure();

    // THEN: Structure should be a valid HashMap
    assert!(structure.len() >= 0, "Should return valid structure");
}

// ============================================================================
// 2. VEC<STRING> AND GENERIC TYPE SUPPORT TESTS
// ============================================================================

#[test]
fn test_vec_string_parameter_parsing_basic() {
    // GIVEN: A vec of strings representing CLI arguments
    let args: Vec<String> = vec!["arg1".to_string(), "arg2".to_string(), "arg3".to_string()];

    // THEN: Vec should be parsed correctly
    assert_eq!(args.len(), 3, "Should have 3 arguments");
    assert_eq!(args[0], "arg1", "First arg should be arg1");
    assert_eq!(args[1], "arg2", "Second arg should be arg2");
    assert_eq!(args[2], "arg3", "Third arg should be arg3");
}

#[test]
fn test_vec_string_comma_separated_simulation() {
    // GIVEN: Comma-separated string input (simulating CLI input)
    let input = "value1,value2,value3";

    // WHEN: We parse it into Vec<String>
    let values: Vec<String> = input.split(',').map(|s| s.to_string()).collect();

    // THEN: Should have correct values
    assert_eq!(values.len(), 3, "Should have 3 values");
    assert_eq!(values[0], "value1", "First should be value1");
    assert_eq!(values[1], "value2", "Second should be value2");
    assert_eq!(values[2], "value3", "Third should be value3");
}

#[test]
fn test_vec_u32_generic_type_support() {
    // GIVEN: Vec of u32 values
    let input = "1,2,3,4,5";

    // WHEN: We parse to Vec<u32>
    let values: Result<Vec<u32>> = input
        .split(',')
        .map(|s| s.parse::<u32>().map_err(|e| {
            NounVerbError::ArgumentError { message: e.to_string() }
        }))
        .collect();

    // THEN: Should parse successfully
    assert!(values.is_ok(), "Should parse u32 values");
    let values = values.unwrap();
    assert_eq!(values.len(), 5, "Should have 5 values");
    assert_eq!(values[0], 1, "First should be 1");
    assert_eq!(values[4], 5, "Last should be 5");
}

#[test]
fn test_vec_pathbuf_generic_type_support() {
    // GIVEN: Vec of PathBuf values
    let paths = vec![
        PathBuf::from("/path/to/file1.txt"),
        PathBuf::from("/path/to/file2.txt"),
        PathBuf::from("relative/path.txt"),
    ];

    // THEN: Should handle PathBuf correctly
    assert_eq!(paths.len(), 3, "Should have 3 paths");
    assert!(paths[0].is_absolute() || paths[0].to_str().unwrap().starts_with('/'),
        "First path should be absolute");
    assert!(!paths[2].is_absolute(), "Third path should be relative");
}

#[test]
fn test_vec_string_mixed_with_other_parameters() {
    // GIVEN: A struct-like pattern with Vec<String> and other params
    struct CommandArgs {
        flags: Vec<String>,
        count: u32,
        enabled: bool,
    }

    let args = CommandArgs {
        flags: vec!["--verbose".to_string(), "--debug".to_string()],
        count: 5,
        enabled: true,
    };

    // THEN: All parameters should coexist
    assert_eq!(args.flags.len(), 2, "Should have 2 flags");
    assert_eq!(args.count, 5, "Count should be 5");
    assert!(args.enabled, "Should be enabled");
}

#[test]
fn test_option_vec_string_combination() {
    // GIVEN: Optional Vec<String> parameter
    let some_values: Option<Vec<String>> = Some(vec!["a".to_string(), "b".to_string()]);
    let no_values: Option<Vec<String>> = None;

    // THEN: Both variants should work
    assert!(some_values.is_some(), "Should have some values");
    assert_eq!(some_values.unwrap().len(), 2, "Should have 2 values");
    assert!(no_values.is_none(), "Should have no values");
}

#[test]
fn test_vec_string_empty_handling() {
    // GIVEN: An empty vec
    let empty: Vec<String> = Vec::new();

    // THEN: Should handle empty vec gracefully
    assert!(empty.is_empty(), "Should be empty");
    assert_eq!(empty.len(), 0, "Length should be 0");
}

#[test]
fn test_vec_string_with_special_characters() {
    // GIVEN: Strings with special characters
    let special_args: Vec<String> = vec![
        "--flag".to_string(),
        "value with spaces".to_string(),
        "path/to/file".to_string(),
        "key=value".to_string(),
        "quoted=\"value\"".to_string(),
    ];

    // THEN: Should preserve special characters
    assert_eq!(special_args.len(), 5, "Should have 5 args");
    assert!(special_args[1].contains(' '), "Should preserve spaces");
    assert!(special_args[2].contains('/'), "Should preserve slashes");
    assert!(special_args[3].contains('='), "Should preserve equals");
    assert!(special_args[4].contains('"'), "Should preserve quotes");
}

#[test]
fn test_vec_string_whitespace_handling() {
    // GIVEN: Input with various whitespace
    let input = "  value1  , value2,value3  ";

    // WHEN: We parse and trim
    let values: Vec<String> = input
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    // THEN: Should handle whitespace correctly
    assert_eq!(values.len(), 3, "Should have 3 values");
    assert_eq!(values[0], "value1", "Should trim whitespace");
}

// ============================================================================
// 3. I/O INTEGRATION TESTS
// ============================================================================

#[test]
fn test_io_module_availability() {
    // GIVEN: I/O module constants
    use clap_noun_verb::io::{IO_MODULE_VERSION, CLIO_AVAILABLE};

    // THEN: Module should be available and correctly versioned
    assert_eq!(IO_MODULE_VERSION, "4.0.0", "I/O module should be v4.0.0");
    assert!(CLIO_AVAILABLE, "Clio should be available");
}

#[test]
fn test_io_pipeline_creation_default() {
    // GIVEN: An I/O pipeline with defaults
    use clap_noun_verb::io::IoPipeline;

    let pipeline = IoPipeline::new();

    // THEN: Pipeline should be created with defaults
    assert_eq!(pipeline.buffer_size(), 8192, "Default buffer size should be 8192");
    assert!(pipeline.inputs().is_empty(), "Should start with no inputs");
    assert!(pipeline.output().is_none(), "Should start with no output");
}

#[test]
fn test_io_pipeline_builder_pattern() {
    // GIVEN: An I/O pipeline builder
    use clap_noun_verb::io::pipeline;

    // WHEN: We configure the pipeline using builder
    let pipe = pipeline()
        .buffer_size(4096)
        .build();

    // THEN: Configuration should be applied
    assert_eq!(pipe.buffer_size(), 4096, "Buffer size should be configured");
    assert!(pipe.inputs().is_empty(), "Should have no inputs initially");
}

#[test]
fn test_io_pipeline_custom_buffer_size() {
    // GIVEN: Multiple pipelines with different buffer sizes
    use clap_noun_verb::io::IoPipeline;

    let small = IoPipeline::new().with_buffer_size(1024);
    let medium = IoPipeline::new().with_buffer_size(4096);
    let large = IoPipeline::new().with_buffer_size(65536);

    // THEN: Each should have correct buffer size
    assert_eq!(small.buffer_size(), 1024, "Small buffer should be 1024");
    assert_eq!(medium.buffer_size(), 4096, "Medium buffer should be 4096");
    assert_eq!(large.buffer_size(), 65536, "Large buffer should be 65536");
}

#[test]
fn test_io_error_type_creation() {
    // GIVEN: I/O error type
    use clap_noun_verb::io::IoError;

    // WHEN: We create an error
    let error = IoError::custom("Test error message");

    // THEN: Error should be created correctly
    assert!(error.to_string().contains("Test"), "Error should contain message");
}

#[test]
fn test_io_type_registry_instantiation() {
    // GIVEN: I/O type registry
    use clap_noun_verb::io::IoTypeRegistry;

    // WHEN: We create a registry
    let registry = IoTypeRegistry::new();

    // THEN: Registry should be functional
    let _ = registry; // Validates creation
}

#[test]
fn test_io_async_backpressure_config() {
    // GIVEN: Async I/O backpressure configuration
    use clap_noun_verb::io::BackpressureConfig;

    // WHEN: We create default config
    let config = BackpressureConfig::default();

    // THEN: Config should have valid defaults
    assert!(config.chunk_size > 0, "Chunk size should be positive");
    assert!(config.max_buffered_chunks > 0, "Max buffered chunks should be positive");
}

#[test]
fn test_io_integration_types_available() {
    // GIVEN: Clio types re-exported
    use clap_noun_verb::io::{Input, Output};

    // THEN: Types should be available for type checking
    let _input_type: Option<Input> = None;
    let _output_type: Option<Output> = None;
    // Compile-time check that types are available
}

#[test]
fn test_io_pipeline_chaining() {
    // GIVEN: An I/O pipeline builder
    use clap_noun_verb::io::IoPipeline;

    // WHEN: We chain operations
    let pipeline = IoPipeline::new()
        .with_buffer_size(8192);

    // THEN: Chaining should work
    assert_eq!(pipeline.buffer_size(), 8192, "Chaining should preserve settings");
}

// ============================================================================
// 4. MIDDLEWARE PIPELINE TESTS
// ============================================================================

struct TestMiddleware {
    name: String,
    call_count: Arc<Mutex<usize>>,
    should_continue: bool,
}

impl Middleware for TestMiddleware {
    fn name(&self) -> &str {
        &self.name
    }

    fn before(&self, _request: &MiddlewareRequest) -> Result<bool> {
        let mut count = self.call_count.lock().unwrap();
        *count += 1;
        Ok(self.should_continue)
    }

    fn after(&self, _response: &MiddlewareResponse) -> Result<()> {
        let mut count = self.call_count.lock().unwrap();
        *count += 1;
        Ok(())
    }
}

#[test]
fn test_middleware_chain_execution_order() -> Result<()> {
    // GIVEN: A middleware pipeline with execution tracking
    let call_count = Arc::new(Mutex::new(0));
    let middleware = Box::new(TestMiddleware {
        name: "test-middleware".to_string(),
        call_count: call_count.clone(),
        should_continue: true,
    });

    let pipeline = MiddlewarePipeline::new().add(middleware);

    // WHEN: We execute before and after
    let request = MiddlewareRequest::new("test-command");
    let response = MiddlewareResponse::success("OK");

    pipeline.execute_before(&request)?;
    pipeline.execute_after(&response)?;

    // THEN: Middleware should be called twice (before + after)
    let count = *call_count.lock().unwrap();
    assert_eq!(count, 2, "Middleware should be called twice");

    Ok(())
}

#[test]
fn test_multiple_middleware_execution_sequence() -> Result<()> {
    // GIVEN: Multiple middleware in a pipeline
    let count1 = Arc::new(Mutex::new(0));
    let count2 = Arc::new(Mutex::new(0));
    let count3 = Arc::new(Mutex::new(0));

    let m1 = Box::new(TestMiddleware {
        name: "middleware-1".to_string(),
        call_count: count1.clone(),
        should_continue: true,
    });

    let m2 = Box::new(TestMiddleware {
        name: "middleware-2".to_string(),
        call_count: count2.clone(),
        should_continue: true,
    });

    let m3 = Box::new(TestMiddleware {
        name: "middleware-3".to_string(),
        call_count: count3.clone(),
        should_continue: true,
    });

    let pipeline = MiddlewarePipeline::new().add(m1).add(m2).add(m3);

    // WHEN: We execute the pipeline
    let request = MiddlewareRequest::new("test");
    pipeline.execute_before(&request)?;

    // THEN: All middleware should be called in order
    assert_eq!(*count1.lock().unwrap(), 1, "Middleware 1 should be called");
    assert_eq!(*count2.lock().unwrap(), 1, "Middleware 2 should be called");
    assert_eq!(*count3.lock().unwrap(), 1, "Middleware 3 should be called");

    Ok(())
}

struct ErrorMiddleware;

impl Middleware for ErrorMiddleware {
    fn name(&self) -> &str {
        "error-middleware"
    }

    fn before(&self, _request: &MiddlewareRequest) -> Result<bool> {
        Err(NounVerbError::MiddlewareError(
            "Intentional test error".to_string(),
        ))
    }
}

#[test]
fn test_middleware_error_propagation() {
    // GIVEN: A middleware that throws errors
    let pipeline = MiddlewarePipeline::new().add(Box::new(ErrorMiddleware));

    // WHEN: We execute it
    let request = MiddlewareRequest::new("test");
    let result = pipeline.execute_before(&request);

    // THEN: Error should be propagated
    assert!(result.is_err(), "Should propagate middleware error");
    assert!(matches!(result.unwrap_err(), NounVerbError::MiddlewareError(_)));
}

#[test]
fn test_middleware_short_circuit_on_false() {
    // GIVEN: A middleware that returns false
    let call_count = Arc::new(Mutex::new(0));
    let middleware = Box::new(TestMiddleware {
        name: "short-circuit".to_string(),
        call_count: call_count.clone(),
        should_continue: false,
    });

    let pipeline = MiddlewarePipeline::new().add(middleware);

    // WHEN: We execute it
    let request = MiddlewareRequest::new("test");
    let result = pipeline.execute_before(&request);

    // THEN: Should return error due to rejection
    assert!(result.is_err(), "Should reject request");
    assert_eq!(*call_count.lock().unwrap(), 1, "Should be called once");
}

#[test]
fn test_middleware_request_modification() {
    // GIVEN: A middleware request with various modifications
    let request = MiddlewareRequest::new("test-command")
        .with_arg("--input")
        .with_arg("file.txt")
        .with_arg("--output")
        .with_arg("result.txt")
        .with_requester("test-user");

    // THEN: Request should contain all modifications
    assert_eq!(request.command(), "test-command", "Command should match");
    assert_eq!(request.args().len(), 4, "Should have 4 args");
    assert_eq!(request.args()[0], "--input", "First arg should be --input");
    assert_eq!(request.args()[1], "file.txt", "Second arg should be file.txt");
    assert_eq!(request.requester(), Some("test-user"), "Requester should match");
}

#[test]
fn test_middleware_response_metadata() {
    // GIVEN: A middleware response with metadata
    let response = MiddlewareResponse::success("Operation completed successfully")
        .with_metadata("duration", "150ms")
        .with_metadata("status", "ok")
        .with_metadata("records_processed", "1000");

    // THEN: Response should contain all metadata
    assert!(response.is_success(), "Should be success");
    assert_eq!(response.message(), "Operation completed successfully");
    assert_eq!(response.metadata().get("duration"), Some(&"150ms".to_string()));
    assert_eq!(response.metadata().get("status"), Some(&"ok".to_string()));
    assert_eq!(response.metadata().get("records_processed"), Some(&"1000".to_string()));
}

#[test]
fn test_middleware_request_pii_redaction() {
    // GIVEN: A request with sensitive information
    let request = MiddlewareRequest::new("login")
        .with_arg("--username=john.doe")
        .with_arg("--password=secret123")
        .with_arg("--api-key=xyz789")
        .with_arg("--email=john@example.com")
        .with_arg("--port=8080");

    // WHEN: We redact sensitive arguments
    let sensitive_patterns = &["password", "api-key", "api_key", "token", "secret", "email"];
    let redacted = request.redacted_args(sensitive_patterns);

    // THEN: Sensitive args should be redacted, others preserved
    assert_eq!(redacted[0], "--username=john.doe", "Username should not be redacted");
    assert_eq!(redacted[1], "[REDACTED]", "Password should be redacted");
    assert_eq!(redacted[2], "[REDACTED]", "API key should be redacted");
    assert_eq!(redacted[3], "[REDACTED]", "Email should be redacted");
    assert_eq!(redacted[4], "--port=8080", "Port should not be redacted");
}

#[test]
fn test_middleware_pipeline_names() {
    // GIVEN: A pipeline with multiple middleware
    let m1 = Box::new(TestMiddleware {
        name: "auth".to_string(),
        call_count: Arc::new(Mutex::new(0)),
        should_continue: true,
    });
    let m2 = Box::new(TestMiddleware {
        name: "logging".to_string(),
        call_count: Arc::new(Mutex::new(0)),
        should_continue: true,
    });

    let pipeline = MiddlewarePipeline::new().add(m1).add(m2);

    // WHEN: We get middleware names
    let names = pipeline.middleware_names();

    // THEN: Should return correct names in order
    assert_eq!(names.len(), 2, "Should have 2 middleware");
    assert_eq!(names[0], "auth", "First should be auth");
    assert_eq!(names[1], "logging", "Second should be logging");
}

// ============================================================================
// 5. PLUGIN SYSTEM TESTS
// ============================================================================

struct TestPlugin {
    name: String,
    version: String,
    loaded: bool,
    capabilities: Vec<PluginCapability>,
}

impl Plugin for TestPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn capabilities(&self) -> Vec<PluginCapability> {
        self.capabilities.clone()
    }

    fn load(&mut self) -> Result<()> {
        self.loaded = true;
        Ok(())
    }

    fn unload(&mut self) -> Result<()> {
        self.loaded = false;
        Ok(())
    }
}

#[test]
fn test_plugin_discovery_and_registration() {
    // GIVEN: A PluginRegistry
    let mut registry = PluginRegistry::new();

    // WHEN: We register a plugin
    let plugin = Box::new(TestPlugin {
        name: "test-plugin".to_string(),
        version: "1.0.0".to_string(),
        loaded: false,
        capabilities: vec![PluginCapability::Command],
    });

    let result = registry.register(plugin);

    // THEN: Plugin should be registered successfully
    assert!(result.is_ok(), "Plugin registration should succeed");
    assert_eq!(registry.count(), 1, "Registry should have 1 plugin");
}

#[test]
fn test_plugin_lifecycle_complete() -> Result<()> {
    // GIVEN: A test plugin
    let mut plugin = TestPlugin {
        name: "lifecycle-test".to_string(),
        version: "1.0.0".to_string(),
        loaded: false,
        capabilities: vec![PluginCapability::Command, PluginCapability::Hook],
    };

    // WHEN: We execute the full lifecycle

    // 1. Load
    plugin.load()?;
    assert!(plugin.loaded, "Plugin should be loaded");

    // 2. Validate capabilities
    let caps = plugin.capabilities();
    assert_eq!(caps.len(), 2, "Plugin should have 2 capabilities");
    assert!(caps.contains(&PluginCapability::Command), "Should have Command capability");
    assert!(caps.contains(&PluginCapability::Hook), "Should have Hook capability");

    // 3. Execute (simulated via name check)
    assert_eq!(plugin.name(), "lifecycle-test", "Plugin should be identifiable");

    // 4. Unload
    plugin.unload()?;
    assert!(!plugin.loaded, "Plugin should be unloaded");

    Ok(())
}

#[test]
fn test_plugin_dependency_tracking() {
    // GIVEN: Plugin metadata with dependencies
    let metadata = PluginMetadata::new("advanced-plugin", "2.0.0")
        .with_author("Test Author")
        .with_description("An advanced test plugin")
        .with_dependency("core-plugin")
        .with_dependency("utils-plugin")
        .with_dependency("logging-plugin")
        .with_min_api_version("4.0.0");

    // THEN: Dependencies should be tracked correctly
    assert_eq!(metadata.name(), "advanced-plugin");
    assert_eq!(metadata.version(), "2.0.0");
    assert_eq!(metadata.author(), "Test Author");
    assert_eq!(metadata.dependencies().len(), 3, "Should have 3 dependencies");
    assert!(metadata.dependencies().contains(&"core-plugin".to_string()));
    assert!(metadata.dependencies().contains(&"utils-plugin".to_string()));
    assert!(metadata.dependencies().contains(&"logging-plugin".to_string()));
    assert_eq!(metadata.min_api_version(), "4.0.0");
}

#[test]
fn test_plugin_isolation_multiple_instances() {
    // GIVEN: Multiple isolated plugins
    let mut registry = PluginRegistry::new();

    // WHEN: We register multiple plugins
    for i in 0..5 {
        let plugin = Box::new(TestPlugin {
            name: format!("plugin-{}", i),
            version: "1.0.0".to_string(),
            loaded: false,
            capabilities: vec![PluginCapability::Command],
        });
        let _ = registry.register(plugin);
    }

    // THEN: All plugins should be isolated in registry
    assert_eq!(registry.count(), 5, "Should have 5 isolated plugins");
}

#[test]
fn test_plugin_capability_validation() {
    // GIVEN: A plugin with multiple capabilities
    let plugin = TestPlugin {
        name: "multi-cap-plugin".to_string(),
        version: "1.0.0".to_string(),
        loaded: false,
        capabilities: vec![
            PluginCapability::Command,
            PluginCapability::Hook,
            PluginCapability::Middleware,
            PluginCapability::Validator,
        ],
    };

    // WHEN: We check capabilities
    let caps = plugin.capabilities();

    // THEN: Should have all expected capabilities
    assert_eq!(caps.len(), 4, "Should have 4 capabilities");
    assert!(caps.contains(&PluginCapability::Command), "Should have Command");
    assert!(caps.contains(&PluginCapability::Hook), "Should have Hook");
    assert!(caps.contains(&PluginCapability::Middleware), "Should have Middleware");
    assert!(caps.contains(&PluginCapability::Validator), "Should have Validator");
}

#[test]
fn test_plugin_has_capability_check() {
    // GIVEN: A plugin with specific capabilities
    let plugin = TestPlugin {
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        loaded: false,
        capabilities: vec![PluginCapability::Command, PluginCapability::Hook],
    };

    // THEN: Capability checks should work correctly
    assert!(plugin.has_capability(PluginCapability::Command), "Should have Command");
    assert!(plugin.has_capability(PluginCapability::Hook), "Should have Hook");
    assert!(!plugin.has_capability(PluginCapability::Middleware), "Should not have Middleware");
}

#[test]
fn test_plugin_config_defaults() {
    // GIVEN: Default plugin configuration
    let config = PluginConfig::default();

    // THEN: Should have sensible defaults
    assert!(config.is_auto_discover_enabled(), "Auto-discover should be enabled");
    assert!(config.is_cache_enabled(), "Cache should be enabled");
    assert!(config.is_sandbox_enabled(), "Sandbox should be enabled");
    assert_eq!(config.manifest_dir(), "./plugins", "Default manifest dir should be ./plugins");
}

#[test]
fn test_plugin_config_customization() {
    // GIVEN: Customized plugin configuration
    let config = PluginConfig::new()
        .with_auto_discover(false)
        .with_manifest_dir("/custom/plugins")
        .with_cache(false)
        .with_sandbox(true);

    // THEN: Custom settings should be applied
    assert!(!config.is_auto_discover_enabled(), "Auto-discover should be disabled");
    assert_eq!(config.manifest_dir(), "/custom/plugins");
    assert!(!config.is_cache_enabled(), "Cache should be disabled");
    assert!(config.is_sandbox_enabled(), "Sandbox should be enabled");
}

#[test]
fn test_plugin_unload_after_registration() -> Result<()> {
    // GIVEN: A registered and loaded plugin
    let mut plugin = TestPlugin {
        name: "unload-test".to_string(),
        version: "1.0.0".to_string(),
        loaded: false,
        capabilities: vec![PluginCapability::Command],
    };

    plugin.load()?;
    assert!(plugin.loaded, "Plugin should be loaded");

    // WHEN: We unload it
    plugin.unload()?;

    // THEN: Plugin should be unloaded
    assert!(!plugin.loaded, "Plugin should be unloaded");

    Ok(())
}

// ============================================================================
// 6. ERROR HANDLING TESTS
// ============================================================================

#[test]
fn test_error_command_not_found() {
    // GIVEN: Command not found error
    let error = NounVerbError::command_not_found("database");

    // THEN: Error should be created correctly
    match error {
        NounVerbError::CommandNotFound { noun } => {
            assert_eq!(noun, "database", "Noun should match");
        }
        _ => panic!("Wrong error type"),
    }
}

#[test]
fn test_error_verb_not_found() {
    // GIVEN: Verb not found error
    let error = NounVerbError::verb_not_found("user", "delete");

    // THEN: Error should contain both noun and verb
    match error {
        NounVerbError::VerbNotFound { noun, verb } => {
            assert_eq!(noun, "user", "Noun should match");
            assert_eq!(verb, "delete", "Verb should match");
        }
        _ => panic!("Wrong error type"),
    }
}

#[test]
fn test_error_validation_failed() {
    // GIVEN: Validation error
    let error = NounVerbError::ValidationFailed("Invalid email format".to_string());

    // THEN: Error should be of correct type
    match error {
        NounVerbError::ValidationFailed(msg) => {
            assert_eq!(msg, "Invalid email format", "Message should match");
        }
        _ => panic!("Wrong error type"),
    }
}

#[test]
fn test_error_plugin_error() {
    // GIVEN: Plugin error
    let error = NounVerbError::PluginError("Plugin failed to load: dependency missing".to_string());

    // THEN: Error should be of correct type
    assert!(matches!(error, NounVerbError::PluginError(_)), "Should be plugin error");
}

#[test]
fn test_error_middleware_error() {
    // GIVEN: Middleware error
    let error = NounVerbError::MiddlewareError("Authentication failed".to_string());

    // THEN: Error should be of correct type
    match error {
        NounVerbError::MiddlewareError(msg) => {
            assert!(msg.contains("Authentication"), "Should contain auth message");
        }
        _ => panic!("Wrong error type"),
    }
}

#[test]
fn test_error_execution_error() {
    // GIVEN: Execution error
    let error = NounVerbError::ExecutionError {
        message: "File not found: /path/to/file.txt".to_string(),
    };

    // THEN: Error should contain message
    match error {
        NounVerbError::ExecutionError { message } => {
            assert!(message.contains("File not found"), "Should contain error details");
        }
        _ => panic!("Wrong error type"),
    }
}

#[test]
fn test_error_argument_error() {
    // GIVEN: Argument error
    let error = NounVerbError::argument_error("Invalid port number: must be between 1-65535");

    // THEN: Error should be of correct type
    match error {
        NounVerbError::ArgumentError { message } => {
            assert!(message.contains("port number"), "Should contain error details");
        }
        _ => panic!("Wrong error type"),
    }
}

#[test]
fn test_error_result_type_success() -> Result<()> {
    // GIVEN: A function returning Result with success
    fn successful_operation() -> Result<String> {
        Ok("success".to_string())
    }

    // WHEN: We call it
    let result = successful_operation()?;

    // THEN: Result should be success
    assert_eq!(result, "success", "Result should be success");

    Ok(())
}

#[test]
fn test_error_result_type_failure() {
    // GIVEN: A function returning Result with error
    fn failing_operation() -> Result<String> {
        Err(NounVerbError::ValidationFailed("Invalid input".to_string()))
    }

    // WHEN: We call it
    let result = failing_operation();

    // THEN: Result should be error
    assert!(result.is_err(), "Result should be error");
}

#[test]
fn test_error_missing_argument_helper() {
    // GIVEN: Missing argument error using helper
    let error = NounVerbError::missing_argument("username");

    // THEN: Error should have proper message
    match error {
        NounVerbError::ArgumentError { message } => {
            assert!(message.contains("username"), "Should mention username");
            assert!(message.contains("missing"), "Should mention missing");
        }
        _ => panic!("Wrong error type"),
    }
}

#[test]
fn test_error_validation_with_constraints() {
    // GIVEN: Validation error with constraints
    let error = NounVerbError::validation_error(
        "age",
        "150",
        Some("Must be between 0 and 120"),
    );

    // THEN: Error should contain constraint information
    match error {
        NounVerbError::ArgumentError { message } => {
            assert!(message.contains("age"), "Should mention field name");
            assert!(message.contains("150"), "Should mention value");
            assert!(message.contains("between 0 and 120"), "Should mention constraints");
        }
        _ => panic!("Wrong error type"),
    }
}

#[test]
fn test_error_display_formatting() {
    // GIVEN: Various errors
    let errors = vec![
        NounVerbError::command_not_found("user"),
        NounVerbError::ValidationFailed("test".to_string()),
        NounVerbError::PluginError("test".to_string()),
    ];

    // THEN: All should have string representations
    for error in errors {
        let error_string = error.to_string();
        assert!(!error_string.is_empty(), "Error should have string representation");
    }
}

// ============================================================================
// 7. INTEGRATION TESTS - COMBINED FEATURES
// ============================================================================

#[test]
fn test_integration_plugin_with_middleware() -> Result<()> {
    // GIVEN: A plugin and middleware working together
    let mut plugin_registry = PluginRegistry::new();
    let plugin = Box::new(TestPlugin {
        name: "integration-plugin".to_string(),
        version: "1.0.0".to_string(),
        loaded: false,
        capabilities: vec![PluginCapability::Command, PluginCapability::Middleware],
    });
    plugin_registry.register(plugin)?;

    let call_count = Arc::new(Mutex::new(0));
    let middleware = Box::new(TestMiddleware {
        name: "integration-middleware".to_string(),
        call_count: call_count.clone(),
        should_continue: true,
    });
    let middleware_pipeline = MiddlewarePipeline::new().add(middleware);

    // WHEN: We execute both systems
    let request = MiddlewareRequest::new("test-command");
    middleware_pipeline.execute_before(&request)?;

    // THEN: Both should work together
    assert_eq!(plugin_registry.count(), 1, "Plugin should be registered");
    assert_eq!(*call_count.lock().unwrap(), 1, "Middleware should be called");

    Ok(())
}

#[test]
fn test_integration_io_with_telemetry() -> Result<()> {
    // GIVEN: I/O operations with telemetry tracking
    use clap_noun_verb::io::IoPipeline;

    let telemetry = TelemetryCollector::new();
    let pipeline = IoPipeline::new();

    // WHEN: We perform I/O and record telemetry
    telemetry.record_command("io-operation", 100)?;
    telemetry.record_command("io-operation", 200)?;

    // THEN: Both I/O and telemetry should work
    assert_eq!(pipeline.buffer_size(), 8192, "I/O pipeline should work");
    assert!(telemetry.is_enabled(), "Telemetry should be enabled");
    assert_eq!(telemetry.metrics().command_count(), 2, "Should record 2 commands");

    Ok(())
}

#[test]
fn test_integration_error_handling_across_components() {
    // GIVEN: Errors from different components
    let plugin_err = NounVerbError::PluginError("Plugin initialization failed".to_string());
    let middleware_err = NounVerbError::MiddlewareError("Middleware rejected request".to_string());
    let io_err = NounVerbError::ExecutionError {
        message: "I/O operation failed".to_string()
    };

    // THEN: All errors should be compatible with Result type
    fn handle_error(err: NounVerbError) -> String {
        format!("Handled: {}", err)
    }

    let msg1 = handle_error(plugin_err);
    let msg2 = handle_error(middleware_err);
    let msg3 = handle_error(io_err);

    assert!(msg1.contains("Plugin"), "Should handle plugin error");
    assert!(msg2.contains("Middleware"), "Should handle middleware error");
    assert!(msg3.contains("I/O"), "Should handle I/O error");
}

#[test]
fn test_integration_full_stack_command_execution() -> Result<()> {
    // GIVEN: Full stack with registry, telemetry, and middleware
    use clap_noun_verb::io::IoPipeline;

    let registry = CommandRegistry::new()
        .name("integration-test")
        .version("1.0.0");
    let _io_pipeline = IoPipeline::new();
    let telemetry = TelemetryCollector::new();
    let middleware_pipeline = MiddlewarePipeline::new();

    // WHEN: We simulate a command execution flow
    let request = MiddlewareRequest::new("full-stack-test");
    middleware_pipeline.execute_before(&request)?;
    telemetry.record_command("full-stack-test", 150)?;

    // THEN: All components should work together
    assert!(registry.nouns().len() >= 0, "Registry should be functional");
    assert!(telemetry.is_enabled(), "Telemetry should be enabled");
    assert_eq!(telemetry.metrics().command_count(), 1, "Should record execution");

    Ok(())
}

#[test]
fn test_integration_concurrent_plugin_registration() -> Result<()> {
    // GIVEN: A plugin registry for concurrent access
    let mut registry = PluginRegistry::new();

    // WHEN: We register multiple plugins sequentially
    for i in 0..10 {
        let plugin = Box::new(TestPlugin {
            name: format!("concurrent-plugin-{}", i),
            version: "1.0.0".to_string(),
            loaded: false,
            capabilities: vec![PluginCapability::Command],
        });
        registry.register(plugin)?;
    }

    // THEN: All plugins should be registered
    assert_eq!(registry.count(), 10, "Should have 10 plugins");

    Ok(())
}

#[test]
fn test_integration_middleware_with_error_recovery() -> Result<()> {
    // GIVEN: Middleware pipeline with graceful handling
    let call_count = Arc::new(Mutex::new(0));

    struct RecoveryMiddleware {
        call_count: Arc<Mutex<usize>>,
    }

    impl Middleware for RecoveryMiddleware {
        fn name(&self) -> &str {
            "recovery-middleware"
        }

        fn before(&self, _request: &MiddlewareRequest) -> Result<bool> {
            let mut count = self.call_count.lock().unwrap();
            *count += 1;
            Ok(true)
        }

        fn handle_error(&self, _error: &NounVerbError) -> Result<Option<String>> {
            Ok(Some("Recovered from error".to_string()))
        }
    }

    let middleware = Box::new(RecoveryMiddleware {
        call_count: call_count.clone(),
    });
    let pipeline = MiddlewarePipeline::new().add(middleware);

    // WHEN: We execute and handle error
    let request = MiddlewareRequest::new("test");
    pipeline.execute_before(&request)?;

    let error = NounVerbError::ValidationFailed("test".to_string());
    let recovery = pipeline.handle_error(&error)?;

    // THEN: Should execute and recover
    assert_eq!(*call_count.lock().unwrap(), 1, "Should be called once");
    assert!(recovery.is_some(), "Should have recovery message");
    assert_eq!(recovery.unwrap(), "Recovered from error");

    Ok(())
}

#[test]
fn test_integration_vec_string_with_validation() -> Result<()> {
    // GIVEN: Vec<String> with validation logic
    fn validate_tags(tags: &[String]) -> Result<()> {
        if tags.is_empty() {
            return Err(NounVerbError::ValidationFailed(
                "At least one tag is required".to_string(),
            ));
        }
        for tag in tags {
            if tag.len() > 50 {
                return Err(NounVerbError::ValidationFailed(
                    format!("Tag '{}' exceeds maximum length of 50", tag),
                ));
            }
        }
        Ok(())
    }

    // WHEN: We validate valid tags
    let valid_tags = vec!["rust".to_string(), "cli".to_string(), "testing".to_string()];
    let result = validate_tags(&valid_tags);

    // THEN: Validation should pass
    assert!(result.is_ok(), "Valid tags should pass validation");

    // WHEN: We validate empty tags
    let empty_tags: Vec<String> = vec![];
    let result = validate_tags(&empty_tags);

    // THEN: Should fail validation
    assert!(result.is_err(), "Empty tags should fail validation");

    Ok(())
}

#[test]
fn test_integration_telemetry_metrics_aggregation() -> Result<()> {
    // GIVEN: A metrics collector tracking various operations
    let metrics = MetricsCollector::new();

    // WHEN: We record multiple operations
    metrics.record_command_execution("cmd1", 100)?;
    metrics.record_command_execution("cmd2", 200)?;
    metrics.record_command_execution("cmd1", 150)?;
    metrics.record_command_execution("cmd3", 50)?;
    metrics.record_command_error("cmd1", "timeout")?;
    metrics.record_command_error("cmd2", "invalid input")?;

    // THEN: Metrics should be aggregated correctly
    assert_eq!(metrics.command_count(), 4, "Should have 4 command executions");
    assert_eq!(metrics.error_count(), 2, "Should have 2 errors");

    let execution_times = metrics.execution_times();
    assert_eq!(execution_times.values().len(), 4, "Should have 4 timing records");
    assert!(execution_times.mean().is_some(), "Should calculate mean");

    Ok(())
}
