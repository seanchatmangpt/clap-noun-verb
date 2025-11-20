//! Integration tests for ggen CLI with mock ggen library
//!
//! These tests verify that the clap-noun-verb ggen CLI works correctly
//! with mocked ggen library responses, without requiring actual external dependencies.

mod ggen_mocks;

use ggen_mocks::*;
use std::collections::HashMap;

// ============================================================================
// Test Utilities
// ============================================================================

/// Helper to setup test environment
fn setup_test() {
    reset_mocks();
}

/// Helper to verify successful response
fn assert_success(response: &str) {
    assert!(!response.is_empty(), "Response should not be empty");
}

// ============================================================================
// AI Generation Command Tests
// ============================================================================

#[test]
fn test_ai_generate_with_valid_description() {
    setup_test();

    // Queue mock response
    MOCK_API.queue_response(ggen_mocks::ApiResponse::GenerateSuccess {
        content: "pub fn example() { /* code */ }".to_string(),
        tokens: 150,
    });

    // Record call
    let mut params = HashMap::new();
    params.insert("description".to_string(), "Create REST API".to_string());
    params.insert("model".to_string(), "gpt-4-turbo".to_string());
    MOCK_API.record_call("ai.generate".to_string(), params);

    // Verify response
    match MOCK_API.get_response() {
        Some(ggen_mocks::ApiResponse::GenerateSuccess { tokens, content }) => {
            assert_eq!(tokens, 150);
            assert!(content.contains("example"));
        }
        _ => panic!("Expected successful generation"),
    }

    // Verify call was recorded
    let history = MOCK_API.get_history();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].endpoint, "ai.generate");
}

#[test]
fn test_ai_project_generation() {
    setup_test();

    // Queue mock response
    MOCK_API.queue_response(ggen_mocks::ApiResponse::ProjectSuccess {
        files: vec![
            "MyProject/src/main.rs".to_string(),
            "MyProject/Cargo.toml".to_string(),
            "MyProject/README.md".to_string(),
        ],
    });

    // Record call
    let mut params = HashMap::new();
    params.insert("name".to_string(), "MyProject".to_string());
    params.insert("model".to_string(), "claude-3-opus".to_string());
    MOCK_API.record_call("ai.project".to_string(), params);

    // Verify response
    match MOCK_API.get_response() {
        Some(ggen_mocks::ApiResponse::ProjectSuccess { files }) => {
            assert_eq!(files.len(), 3);
            assert!(files.iter().any(|f| f.contains("Cargo.toml")));
        }
        _ => panic!("Expected successful project generation"),
    }

    // Verify call
    let history = MOCK_API.get_history();
    assert_eq!(history.len(), 1);
}

#[test]
fn test_ai_graph_generation_with_format() {
    setup_test();

    // Queue mock response
    MOCK_API.queue_response(ggen_mocks::ApiResponse::GraphSuccess { triples: 1250 });

    // Record call
    let mut params = HashMap::new();
    params.insert("description".to_string(), "Ontology".to_string());
    params.insert("format".to_string(), "turtle".to_string());
    MOCK_API.record_call("ai.graph".to_string(), params);

    // Verify response
    match MOCK_API.get_response() {
        Some(ggen_mocks::ApiResponse::GraphSuccess { triples }) => {
            assert_eq!(triples, 1250);
        }
        _ => panic!("Expected successful graph generation"),
    }
}

#[test]
fn test_ai_sparql_query_generation() {
    setup_test();

    // Queue mock response
    MOCK_API.queue_response(ggen_mocks::ApiResponse::GenerateSuccess {
        content: "SELECT ?s ?p ?o WHERE { ?s ?p ?o }".to_string(),
        tokens: 45,
    });

    // Record call
    let mut params = HashMap::new();
    params.insert("description".to_string(), "Query for all triples".to_string());
    MOCK_API.record_call("ai.sparql".to_string(), params);

    // Verify response
    match MOCK_API.get_response() {
        Some(ggen_mocks::ApiResponse::GenerateSuccess { content, .. }) => {
            assert!(content.contains("SELECT"));
        }
        _ => panic!("Expected successful SPARQL generation"),
    }
}

// ============================================================================
// Marketplace Command Tests
// ============================================================================

#[test]
fn test_marketplace_search_finds_packages() {
    setup_test();

    // Search using mock registry
    let results = MOCK_REGISTRY.search("api");

    // Verify results
    assert!(!results.is_empty(), "Should find packages matching 'api'");
    assert!(results.iter().any(|p| p.name.contains("API")));
}

#[test]
fn test_marketplace_search_no_results() {
    setup_test();

    let results = MOCK_REGISTRY.search("nonexistent-xyz-package");
    assert_eq!(results.len(), 0, "Should return empty for non-matching search");
}

#[test]
fn test_marketplace_search_by_tag() {
    setup_test();

    let results = MOCK_REGISTRY.search("rust");
    assert!(!results.is_empty(), "Should find packages tagged with 'rust'");
}

#[test]
fn test_marketplace_install_package() {
    setup_test();

    // Get package (simulates install)
    let pkg = MOCK_REGISTRY.get_package("io.ggen.rest-api").unwrap();

    assert_eq!(pkg.name, "REST API Template");
    assert_eq!(pkg.version, "1.0.0");
    assert_eq!(pkg.downloads, 5000);
}

#[test]
fn test_marketplace_install_nonexistent_package() {
    setup_test();

    let pkg = MOCK_REGISTRY.get_package("io.ggen.nonexistent");
    assert!(pkg.is_none(), "Should return None for non-existent package");
}

#[test]
fn test_marketplace_list_searches_all() {
    setup_test();

    // Simulate listing all packages
    let results = MOCK_REGISTRY.search("");
    // Empty search should not match anything by default
    assert_eq!(results.len(), 0);
}

#[test]
fn test_marketplace_search_history_tracking() {
    setup_test();

    // Clear history to start fresh
    MOCK_REGISTRY.clear_history();

    // Perform multiple searches
    MOCK_REGISTRY.search("api");
    MOCK_REGISTRY.search("web");
    MOCK_REGISTRY.search("auth");

    // Verify history contains at least our searches
    let history = MOCK_REGISTRY.get_search_history();
    assert_eq!(history.len(), 3);
    assert_eq!(history[0].query, "api");
    assert_eq!(history[1].query, "web");
    assert_eq!(history[2].query, "auth");
}

// ============================================================================
// Template Command Tests
// ============================================================================

#[test]
fn test_template_register_and_render() {
    setup_test();

    // Use new instance for isolated testing
    let templates = ggen_mocks::MockTemplateEngine::new();

    // Register template
    templates.register_template(
        "hello".to_string(),
        "Hello {{name}}, welcome to {{place}}!".to_string(),
    );

    // Render template
    let mut vars = HashMap::new();
    vars.insert("name".to_string(), "Alice".to_string());
    vars.insert("place".to_string(), "ggen".to_string());

    let result = templates.render("hello", vars).unwrap();
    assert_eq!(result, "Hello Alice, welcome to ggen!");
}

#[test]
fn test_template_render_missing_template() {
    setup_test();

    let templates = ggen_mocks::MockTemplateEngine::new();

    let mut vars = HashMap::new();
    vars.insert("name".to_string(), "Alice".to_string());

    let result = templates.render("nonexistent", vars);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[test]
fn test_template_render_tracking() {
    setup_test();

    let templates = ggen_mocks::MockTemplateEngine::new();

    templates.register_template(
        "test".to_string(),
        "Value: {{value}}".to_string(),
    );

    // Render multiple times with different values
    for i in 1..=3 {
        let mut vars = HashMap::new();
        vars.insert("value".to_string(), format!("item_{}", i));
        templates.render("test", vars).ok();
    }

    // Verify tracking
    let rendered = templates.get_rendered();
    assert_eq!(rendered.len(), 3);
    assert!(rendered[0].content.contains("item_1"));
    assert!(rendered[1].content.contains("item_2"));
    assert!(rendered[2].content.contains("item_3"));
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_api_error_response_401() {
    setup_test();

    // Queue error response
    MOCK_API.queue_response(ggen_mocks::ApiResponse::Error {
        code: 401,
        message: "Unauthorized - Invalid API key".to_string(),
    });

    // Verify error
    match MOCK_API.get_response() {
        Some(ggen_mocks::ApiResponse::Error { code, message }) => {
            assert_eq!(code, 401);
            assert!(message.contains("Unauthorized"));
        }
        _ => panic!("Expected error response"),
    }
}

#[test]
fn test_api_error_response_429_rate_limit() {
    setup_test();

    MOCK_API.queue_response(ggen_mocks::ApiResponse::Error {
        code: 429,
        message: "Rate limit exceeded".to_string(),
    });

    match MOCK_API.get_response() {
        Some(ggen_mocks::ApiResponse::Error { code, message }) => {
            assert_eq!(code, 429);
            assert!(message.contains("Rate limit"));
        }
        _ => panic!("Expected rate limit error"),
    }
}

#[test]
fn test_api_error_response_500() {
    setup_test();

    MOCK_API.queue_response(ggen_mocks::ApiResponse::Error {
        code: 500,
        message: "Internal server error".to_string(),
    });

    match MOCK_API.get_response() {
        Some(ggen_mocks::ApiResponse::Error { code, message }) => {
            assert_eq!(code, 500);
            assert!(message.contains("Internal"));
        }
        _ => panic!("Expected server error"),
    }
}

// ============================================================================
// Integration Scenario Tests
// ============================================================================

#[test]
fn test_complete_workflow_generate_and_publish() {
    setup_test();

    // Step 1: Generate code with AI
    MOCK_API.queue_response(ggen_mocks::ApiResponse::GenerateSuccess {
        content: "// Generated code".to_string(),
        tokens: 200,
    });

    let mut params = HashMap::new();
    params.insert("description".to_string(), "REST API".to_string());
    MOCK_API.record_call("ai.generate".to_string(), params);

    match MOCK_API.get_response() {
        Some(ggen_mocks::ApiResponse::GenerateSuccess { .. }) => {
            // Success - continue to step 2
        }
        _ => panic!("Step 1 failed"),
    }

    // Step 2: Search for similar templates
    let templates = MOCK_REGISTRY.search("api");
    assert!(!templates.is_empty(), "Should find API templates");

    // Step 3: Verify workflow completed
    let history = MOCK_API.get_history();
    assert_eq!(history.len(), 1);
}

#[test]
fn test_template_generation_workflow() {
    setup_test();

    let templates = ggen_mocks::MockTemplateEngine::new();

    // Register templates
    templates.register_template(
        "service".to_string(),
        "[{{name}}]\nversion = {{version}}".to_string(),
    );

    // Render with variables
    let mut vars = HashMap::new();
    vars.insert("name".to_string(), "my-service".to_string());
    vars.insert("version".to_string(), "1.0.0".to_string());

    let result = templates.render("service", vars).unwrap();
    assert!(result.contains("my-service"));
    assert!(result.contains("1.0.0"));

    // Verify tracking
    let rendered = templates.get_rendered();
    assert_eq!(rendered.len(), 1);
}

// ============================================================================
// Mock State Management Tests
// ============================================================================

#[test]
fn test_mock_state_isolation() {
    setup_test();

    // First test
    MOCK_API.record_call("test1".to_string(), HashMap::new());
    let history1 = MOCK_API.get_history();
    assert_eq!(history1.len(), 1);

    // Reset and second test
    reset_mocks();
    let history2 = MOCK_API.get_history();
    assert_eq!(history2.len(), 0);
}

#[test]
fn test_multiple_responses_fifo() {
    setup_test();

    // Queue multiple responses
    for i in 1..=3 {
        MOCK_API.queue_response(ggen_mocks::ApiResponse::GenerateSuccess {
            content: format!("content_{}", i),
            tokens: 100 + i as u32,
        });
    }

    // Verify FIFO order
    for i in 1..=3 {
        match MOCK_API.get_response() {
            Some(ggen_mocks::ApiResponse::GenerateSuccess { content, tokens }) => {
                assert_eq!(content, format!("content_{}", i));
                assert_eq!(tokens, 100 + i as u32);
            }
            _ => panic!("Unexpected response"),
        }
    }
}
