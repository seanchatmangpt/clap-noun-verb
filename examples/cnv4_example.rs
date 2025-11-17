//! CNV 4.0: Autonomic Command Fabric Example
//!
//! Demonstrates the three pillars of CNV 4.0:
//! 1. Capability Contracts
//! 2. Session Kernel
//! 3. Version Negotiation

use clap_noun_verb::kernel::*;
use clap_noun_verb::kernel::grammar::{GrammarNoun, GrammarVerb};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== CNV 4.0: Autonomic Command Fabric Demo ===\n");

    // Pillar 1: Capability Contracts
    demonstrate_capability_contracts()?;

    // Pillar 2: Session Kernel
    demonstrate_session_kernel()?;

    // Pillar 3: Version Negotiation
    demonstrate_version_negotiation()?;

    Ok(())
}

/// Demonstrate Capability Contracts (Pillar 1)
fn demonstrate_capability_contracts() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Pillar 1: Capability Contracts ===\n");

    // Create different capability contracts
    let pure_contract = CapabilityContract::pure();
    println!("Pure contract: {}", pure_contract);
    println!("  - Risk score: {}", pure_contract.risk_score());
    println!("  - Agent safe: {}", pure_contract.is_agent_safe());
    println!();

    let read_only_contract = CapabilityContract::read_only();
    println!("Read-only contract: {}", read_only_contract);
    println!("  - Risk score: {}", read_only_contract.risk_score());
    println!("  - Agent safe: {}", read_only_contract.is_agent_safe());
    println!();

    let dangerous_contract = CapabilityContract::dangerous();
    println!("Dangerous contract: {}", dangerous_contract);
    println!("  - Risk score: {}", dangerous_contract.risk_score());
    println!("  - Agent safe: {}", dangerous_contract.is_agent_safe());
    println!();

    // Check compatibility
    println!("Compatibility checks:");
    println!(
        "  Pure compatible with Read-only? {}",
        pure_contract.is_compatible_with(&read_only_contract)
    );
    println!(
        "  Read-only compatible with Pure? {}",
        read_only_contract.is_compatible_with(&pure_contract)
    );
    println!(
        "  Dangerous compatible with Pure? {}",
        dangerous_contract.is_compatible_with(&pure_contract)
    );
    println!();

    // Capability context
    let ctx = CapabilityContext::new(read_only_contract);
    println!("Capability context permissions:");
    println!("  - Can read FS: {}", ctx.can_read_fs());
    println!("  - Can write FS: {}", ctx.can_write_fs());
    println!("  - Can access network: {}", ctx.can_access_network());
    println!();

    Ok(())
}

/// Demonstrate Session Kernel (Pillar 2)
fn demonstrate_session_kernel() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Pillar 2: Session Kernel ===\n");

    // Create a session
    let mut session = SessionBuilder::new()
        .config(SessionConfig::default())
        .profile(TelemetryProfile::default())
        .capability(CapabilityContract::pure())
        .build();

    println!("Session created: {}", session.id());
    println!("  - Active: {}", session.is_active());
    println!("  - Capability: {}", session.capability());
    println!();

    // Yield some data frames
    println!("Yielding data frames:");
    for i in 0..3 {
        let frame = session.yield_data(
            StreamId::Stdout,
            serde_json::json!({
                "iteration": i,
                "message": format!("Frame {}", i)
            }),
        )?;
        println!("  - Frame {}: seq={}, stream={}", i, frame.sequence, frame.stream_id);
    }
    println!();

    // Yield a log frame
    println!("Yielding log frame:");
    let log_frame = session.yield_log("info", "Operation completed successfully", None)?;
    println!(
        "  - Log frame: seq={}, stream={}",
        log_frame.sequence, log_frame.stream_id
    );
    println!();

    // Check metrics
    let metrics = session.metrics();
    println!("Session metrics:");
    println!("  - Frames sent: {}", metrics.frames_sent);
    println!("  - Bytes sent: {}", metrics.bytes_sent);
    println!("  - Avg latency: {:.2}ms", metrics.avg_latency_ms);
    println!();

    // Demonstrate cancellation
    println!("Cancelling session...");
    session.cancel();
    println!("  - Active: {}", session.is_active());
    println!("  - Cancelled: {}", session.is_cancelled());
    println!();

    // Try to yield after cancellation
    match session.yield_data(StreamId::Stdout, serde_json::json!({"test": "data"})) {
        Ok(_) => println!("  - Unexpected success"),
        Err(e) => println!("  - Expected error: {}", e),
    }
    println!();

    Ok(())
}

/// Demonstrate Version Negotiation (Pillar 3)
fn demonstrate_version_negotiation() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Pillar 3: Version Negotiation ===\n");

    // Create two grammar versions
    let v1 = create_grammar_v1();
    let v2 = create_grammar_v2();

    println!("Grammar v1: {}", v1.app_version.as_ref().unwrap());
    println!("  - Nouns: {}", v1.nouns.len());
    println!();

    println!("Grammar v2: {}", v2.app_version.as_ref().unwrap());
    println!("  - Nouns: {}", v2.nouns.len());
    println!();

    // Compute delta
    println!("Computing grammar delta...");
    let delta = GrammarDelta::compute(&v1, &v2)?;
    println!("  - From: {}", delta.from_version);
    println!("  - To: {}", delta.to_version);
    println!("  - Severity: {:?}", delta.severity);
    println!("  - Breaking changes: {}", delta.has_breaking_changes());
    println!();

    if !delta.noun_changes.is_empty() {
        println!("Noun changes:");
        for change in &delta.noun_changes {
            println!("  - {}: {:?}", change.name, change.change_type);
        }
        println!();
    }

    if !delta.verb_changes.is_empty() {
        println!("Verb changes:");
        for change in &delta.verb_changes {
            println!("  - {}.{}: {:?}", change.noun, change.name, change.change_type);
        }
        println!();
    }

    // Print change summary
    println!("Change summary:\n{}", delta.summary());
    println!();

    // Version negotiation
    println!("Version negotiation:");
    let mut negotiator = VersionNegotiator::new(v2.clone());
    negotiator.add_history("1.0.0".to_string(), v1.clone());

    let request = NegotiationRequest {
        known_version: "1.0.0".to_string(),
        required_capabilities: None,
        compatibility_level: CompatibilityLevel::Strict,
    };

    let response = negotiator.negotiate(&request)?;
    println!("  - Current version: {}", response.current_version);
    println!("  - Compatible: {}", response.compatible);
    println!("  - Warnings: {}", response.warnings.len());
    if !response.warnings.is_empty() {
        for warning in &response.warnings {
            println!("    - {}", warning);
        }
    }
    println!();

    Ok(())
}

/// Create grammar version 1.0.0
fn create_grammar_v1() -> GrammarModel {
    let mut grammar = GrammarModel::new("demo-app").with_version("1.0.0");

    let mut noun = GrammarNoun {
        name: "file".to_string(),
        help: Some("File operations".to_string()),
        long_help: None,
        verbs: Vec::new(),
        sub_nouns: Vec::new(),
        metadata: Default::default(),
    };

    // Add a read verb with capability
    noun.verbs.push(GrammarVerb {
        name: "read".to_string(),
        noun: "file".to_string(),
        help: Some("Read a file".to_string()),
        long_help: None,
        arguments: Vec::new(),
        deprecated: false,
        deprecation_message: None,
        capability: Some(CapabilityContract::read_only()),
        metadata: Default::default(),
    });

    grammar.add_noun(noun);
    grammar
}

/// Create grammar version 2.0.0 with changes
fn create_grammar_v2() -> GrammarModel {
    let mut grammar = GrammarModel::new("demo-app").with_version("2.0.0");

    let mut noun = GrammarNoun {
        name: "file".to_string(),
        help: Some("File operations".to_string()),
        long_help: None,
        verbs: Vec::new(),
        sub_nouns: Vec::new(),
        metadata: Default::default(),
    };

    // Keep the read verb
    noun.verbs.push(GrammarVerb {
        name: "read".to_string(),
        noun: "file".to_string(),
        help: Some("Read a file".to_string()),
        long_help: None,
        arguments: Vec::new(),
        deprecated: false,
        deprecation_message: None,
        capability: Some(CapabilityContract::read_only()),
        metadata: Default::default(),
    });

    // Add a write verb (new in v2)
    noun.verbs.push(GrammarVerb {
        name: "write".to_string(),
        noun: "file".to_string(),
        help: Some("Write a file".to_string()),
        long_help: None,
        arguments: Vec::new(),
        deprecated: false,
        deprecation_message: None,
        capability: Some(CapabilityContract::read_write()),
        metadata: Default::default(),
    });

    grammar.add_noun(noun);

    // Add a new noun (new in v2)
    let mut network_noun = GrammarNoun {
        name: "network".to_string(),
        help: Some("Network operations".to_string()),
        long_help: None,
        verbs: Vec::new(),
        sub_nouns: Vec::new(),
        metadata: Default::default(),
    };

    network_noun.verbs.push(GrammarVerb {
        name: "fetch".to_string(),
        noun: "network".to_string(),
        help: Some("Fetch data from URL".to_string()),
        long_help: None,
        arguments: Vec::new(),
        deprecated: false,
        deprecation_message: None,
        capability: Some(CapabilityContract::network()),
        metadata: Default::default(),
    });

    grammar.add_noun(network_noun);
    grammar
}
