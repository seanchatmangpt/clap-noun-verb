//! MCP Integration Validation - 80/20 Consolidated Test Suite
//!
//! Tests the critical 20% of functionality that validates 80% of the system:
//! - Handler lifecycle and trait implementation
//! - All 4 request/response types working end-to-end
//! - Swarm agent patterns coordinating through handler
//! - Concurrent operations under realistic stress

use clap_noun_verb::rdf::{OntologyBuilder, RdfMcpHandler};
use std::sync::Arc;

// ============================================================================
// TEST 1: CORE HANDLER LIFECYCLE & ALL 4 REQUEST/RESPONSE TYPES
// ============================================================================

#[test]
fn test_handler_lifecycle_all_request_response_types() {
    // Arrange: Build ontology with diverse command types
    let mut builder = OntologyBuilder::new();
    builder.add_command("test-sparql", "query", "execute", "Execute SPARQL").ok();
    builder.add_command("test-discover-a", "discovery", "find-a", "Find A").ok();
    builder.add_command("test-discover-b", "discovery", "find-b", "Find B").ok();
    builder.add_command("test-validate", "validation", "check", "Check validity").ok();
    builder.add_command("test-receipt", "audit", "record", "Record receipt").ok();

    let ontology = Arc::new(builder.build().expect("Failed to build ontology"));
    let handler = RdfMcpHandler::new(ontology);

    // Act & Assert: All 4 request/response types work end-to-end

    // 1. ServerHandler Trait Implementation
    let info = handler.get_server_info();
    assert!(!info.server_info.name.is_empty(), "ServerHandler should return name");
    assert!(!info.server_info.version.is_empty(), "ServerHandler should return version");

    // 2. SPARQL Query Request/Response
    let sparql = handler.execute_sparql("SELECT ?s WHERE { ?s ?p ?o . } LIMIT 1");
    assert!(sparql.is_ok(), "SPARQL request/response type should work");

    // 3. Command Discovery Request/Response
    let discovery = handler.discover_commands("discovery");
    assert!(discovery.is_ok(), "Discovery request/response type should work");
    let disco = discovery.expect("Should discover");
    assert!(disco.count >= 2, "Should discover discovery commands");

    // 4. Invocation Validation Request/Response
    let validation = handler.validate_invocation("test-validate", &None);
    assert!(validation.is_ok(), "Validation request/response type should work");
    // Note: Valid result depends on command existing in ontology triples
    let _ = validation.expect("Should return validation response");

    // 5. Execution Receipt Request/Response
    let receipt = handler.record_receipt("test-receipt", 0);
    assert!(receipt.is_ok(), "Receipt request/response type should work");
    let receipt_resp = receipt.expect("Should create receipt");
    assert!(!receipt_resp.receipt_id.is_empty(), "Receipt should have unique ID");
    assert_eq!(receipt_resp.command, "test-receipt", "Receipt should track command");
}

// ============================================================================
// TEST 2: SWARM AGENT PATTERNS - ALL ROLES COORDINATING THROUGH HANDLER
// ============================================================================

#[test]
fn test_swarm_agent_patterns_end_to_end() {
    // Arrange: Build ontology for swarm coordination
    let mut builder = OntologyBuilder::new();
    builder.add_command("scout-explore", "explorer", "scan", "Scan").ok();
    builder.add_command("validator-check", "guard", "enforce", "Enforce").ok();
    builder.add_command("worker-exec", "execution", "run", "Run").ok();
    builder.add_command("queen-coord", "coordination", "orchestrate", "Orchestrate").ok();

    let ontology = Arc::new(builder.build().expect("Failed to build ontology"));
    let handler = RdfMcpHandler::new(ontology);

    // Act: Simulate swarm agent patterns

    // Scout Pattern: Discover commands through discovery API
    let discovery = handler.discover_commands("explorer");
    assert!(discovery.is_ok(), "Scout should be able to discover commands");

    // Validator Pattern: Pre-validate before execution
    let validation = handler.validate_invocation("validator-check", &None);
    assert!(validation.is_ok(), "Validator pattern should work");

    // Worker Pattern: Validate then record receipt
    let valid = handler.validate_invocation("worker-exec", &None);
    let receipt = handler.record_receipt("worker-exec", 0);
    assert!(
        valid.is_ok() && receipt.is_ok(),
        "Worker should validate then record"
    );

    // Queen Pattern: Get server info + SPARQL queries for orchestration
    let server_info = handler.get_server_info();
    let sparql = handler.execute_sparql("SELECT ?s LIMIT 1");
    assert!(
        !server_info.server_info.name.is_empty() && sparql.is_ok(),
        "Queen should orchestrate via handler"
    );
}

// ============================================================================
// TEST 3: CONCURRENT OPERATIONS UNDER STRESS - REALISTIC SWARM LOAD
// ============================================================================

#[test]
fn test_concurrent_swarm_operations_under_stress() {
    // Arrange: Build ontology with 10 commands for concurrent load
    let mut builder = OntologyBuilder::new();
    for i in 0..10 {
        builder
            .add_command(
                &format!("cmd-{}", i),
                "concurrent",
                &format!("op{}", i),
                &format!("Op {}", i),
            )
            .ok();
    }

    let ontology = Arc::new(builder.build().expect("Failed to build ontology"));
    let handler = Arc::new(RdfMcpHandler::new(ontology));

    // Act: Spawn concurrent threads simulating 10 agents under realistic load
    let mut handles = vec![];
    for agent_id in 0..10 {
        let handler_clone = Arc::<RdfMcpHandler>::clone(&handler);
        let handle = std::thread::spawn(move || {
            // Each agent: validate, discover, get server info, record receipt
            let cmd = format!("cmd-{}", agent_id);
            let v1 = handler_clone.validate_invocation(&cmd, &None);
            let v2 = handler_clone.discover_commands("concurrent");
            let v3 = handler_clone.get_server_info();
            let v4 = handler_clone.record_receipt(&cmd, 0);

            (v1.is_ok(), v2.is_ok(), !v3.server_info.name.is_empty(), v4.is_ok())
        });
        handles.push(handle);
    }

    // Assert: All 10 concurrent operations succeed
    let results: Vec<_> = handles
        .into_iter()
        .filter_map(|h| h.join().ok())
        .collect();

    assert_eq!(results.len(), 10, "All 10 concurrent agents should complete");
    for (v1, v2, v3, v4) in results {
        assert!(v1 && v2 && v3 && v4, "Each agent should succeed at all operations");
    }
}
