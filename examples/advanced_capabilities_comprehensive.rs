//! Advanced Capabilities Framework Comprehensive Example
//!
//! Demonstrates all five advanced features:
//! 1. Type-Level Capability Enforcement
//! 2. Capability Introspection (--capabilities, --explain)
//! 3. Session Streaming Protocol
//! 4. Distributed Tracing
//! 5. Schema Registry

use clap_noun_verb::kernel::*;
use std::sync::Arc;
use tokio::runtime::Runtime;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rt = Runtime::new()?;

    rt.block_on(async {
        println!("═══════════════════════════════════════════════════════════");
        println!("Advanced Capabilities Framework Framework Capabilities Demonstration");
        println!("═══════════════════════════════════════════════════════════\n");

        // Feature 1: Type-Level Capability Enforcement
        println!("1. TYPE-LEVEL CAPABILITY ENFORCEMENT");
        println!("────────────────────────────────────");
        demo_type_level_enforcement();
        println!();

        // Feature 2: Capability Introspection
        println!("2. CAPABILITY INTROSPECTION");
        println!("─────────────────────────────");
        demo_introspection();
        println!();

        // Feature 3: Session Streaming
        println!("3. SESSION STREAMING PROTOCOL");
        println!("──────────────────────────────");
        demo_streaming().await?;
        println!();

        // Feature 4: Distributed Tracing
        println!("4. DISTRIBUTED TRACING");
        println!("───────────────────────");
        demo_tracing();
        println!();

        // Feature 5: Schema Registry
        println!("5. SCHEMA REGISTRY");
        println!("──────────────────");
        demo_schema_registry()?;
        println!();

        println!("═══════════════════════════════════════════════════════════");
        println!("All features demonstrated successfully!");
        println!("═══════════════════════════════════════════════════════════");

        Ok(())
    })
}

fn demo_type_level_enforcement() {
    println!("Creating capability contexts...");

    // Create different capability contexts
    let pure_context = TypedContext::<Pure>::new("compute-zone".to_string());
    let readonly_context = TypedContext::<ReadOnlyFS>::new("read-zone".to_string());
    let write_context = TypedContext::<ReadWriteFS>::new("admin-zone".to_string());

    // Create typed verbs
    let pure_verb = TypedVerb::<Pure, Safe>::new("calculate_hash".to_string())
        .with_description("Calculate hash of data".to_string());

    let readonly_verb = TypedVerb::<ReadOnlyFS, Safe>::new("read_config".to_string())
        .with_description("Read configuration file".to_string());

    let write_verb = TypedVerb::<ReadWriteFS, Safe>::new("update_database".to_string())
        .with_description("Update database".to_string());

    // Execute verbs in compatible contexts
    println!("✓ Pure verb in pure context: {}",
        pure_context.execute(&pure_verb, || Ok("executed")).unwrap());

    println!("✓ Pure verb in readonly context: {}",
        readonly_context.execute(&pure_verb, || Ok("executed")).unwrap());

    println!("✓ Readonly verb in readonly context: {}",
        readonly_context.execute(&readonly_verb, || Ok("executed")).unwrap());

    println!("✓ Write verb in write context: {}",
        write_context.execute(&write_verb, || Ok("executed")).unwrap());

    // Note: The following would be COMPILE ERRORS (shown in comments):
    // ✗ readonly_context.execute(&write_verb, || Ok("..."))?;  // Compile error!
    println!("✓ Type system prevents capability violations at compile time");

    // Demonstrate unsafe verb with approval
    let unsafe_verb = TypedVerb::<ReadWriteFS, UnsafeMeta>::new("delete_all".to_string());
    let approval = ApprovalToken::new(
        vec!["admin".to_string()],
        3600,
        "DBA approved bulk delete".to_string(),
    );

    let result = write_context.execute_unsafe(&unsafe_verb, approval, || {
        Ok("deleted 1000 records")
    });

    println!("✓ Unsafe verb with approval: {}", result.unwrap());
}

fn demo_introspection() {
    println!("Building capability registry...");

    let mut registry = CapabilityRegistry::new();

    // Register various capabilities
    registry.register(CapabilityInfo {
        id: "list_services".to_string(),
        name: "List Services".to_string(),
        description: "List all running services".to_string(),
        side_effects: vec![SideEffect::ReadOnlyFS],
        resource_profile: ResourceProfile::Fast,
        stability: StabilityGuarantee::Stable,
        safety: SafetyProfile::AgentSafe,
        agent_safe: true,
        requires_approval: vec![],
    });

    registry.register(CapabilityInfo {
        id: "restart_service".to_string(),
        name: "Restart Service".to_string(),
        description: "Restart a running service".to_string(),
        side_effects: vec![SideEffect::Subprocess, SideEffect::Environment],
        resource_profile: ResourceProfile::Medium,
        stability: StabilityGuarantee::Stable,
        safety: SafetyProfile::AgentSafe,
        agent_safe: true,
        requires_approval: vec![],
    });

    registry.register(CapabilityInfo {
        id: "delete_data".to_string(),
        name: "Delete Data".to_string(),
        description: "Permanently delete data".to_string(),
        side_effects: vec![SideEffect::ReadWriteFS, SideEffect::Dangerous],
        resource_profile: ResourceProfile::Fast,
        stability: StabilityGuarantee::Stable,
        safety: SafetyProfile::HumanReviewRequired,
        agent_safe: false,
        requires_approval: vec!["dba".to_string(), "security".to_string()],
    });

    let handler = IntrospectionHandler::new(registry);

    // Handle --capabilities
    let output = handler.list_capabilities();
    println!("Total capabilities: {}", output.total_capabilities);
    println!("Agent-safe: {}", output.agent_safe_count);
    println!("Requires approval: {}", output.requires_approval_count);

    println!("\nAll capabilities:");
    for cap in &output.capabilities {
        println!("  {} - {}", cap.name, cap.description);
    }

    // Handle --explain
    println!("\nExplaining 'delete_data':");
    if let Ok(explanation) = handler.explain_capability("delete_data") {
        println!("  Name: {}", explanation.capability.name);
        println!("  Risk level: {}/100", explanation.risk_level);
        println!("  Implications:");
        for imp in explanation.implications {
            println!("    - {}", imp);
        }
        if !explanation.related_capabilities.is_empty() {
            println!("  Related: {}", explanation.related_capabilities.join(", "));
        }
    }
}

async fn demo_streaming() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating streaming session...");

    let handler = ServerStreamingHandler::new();

    // Create a streaming session
    let (session, sink) = handler
        .create_session(
            "demo-session-001".to_string(),
            "process_items".to_string(),
            vec!["--count".to_string(), "5".to_string()],
            100,
        )
        .await?;

    println!("Session created: {}", session.id);

    // Simulate processing items and sending frames
    for i in 0..5 {
        let frame = StreamFrame::Data {
            session_id: session.id.clone(),
            sequence: i,
            payload: serde_json::json!({
                "item": i,
                "status": "processed",
                "duration_ms": 50 + i * 10
            }),
        };

        sink.send(frame).await?;
        println!("  Sent frame {}/5", i + 1);
    }

    // Send log frames
    let log_frame = StreamFrame::Log {
        session_id: session.id.clone(),
        level: LogLevel::Info,
        message: "Processing completed successfully".to_string(),
        timestamp_ns: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64,
    };
    sink.send(log_frame).await?;

    // Send metrics frame
    let metrics_frame = StreamFrame::Metrics {
        session_id: session.id.clone(),
        cpu_us: 250000,
        memory_bytes: 10485760,
        io_read_bytes: 1048576,
        io_write_bytes: 524288,
    };
    sink.send(metrics_frame).await?;

    // Send completion frame
    let done_frame = StreamFrame::Done {
        session_id: session.id.clone(),
        exit_code: 0,
    };
    sink.send(done_frame).await?;
    println!("Session completed with exit code 0");

    // List active sessions
    let sessions = handler.list_sessions().await;
    println!("Active sessions: {}", sessions.len());

    Ok(())
}

fn demo_tracing() {
    println!("Setting up distributed tracing...");

    // Create trace context
    let mut root_ctx = TraceContext::new();
    println!("Root trace ID: {}", root_ctx.trace_id);
    println!("W3C header: {}", root_ctx.to_trace_context_header());

    // Add baggage
    root_ctx.add_baggage("user".to_string(), "alice".to_string());
    root_ctx.add_baggage("request_id".to_string(), "req-123".to_string());

    // Create child span
    let child_ctx = root_ctx.child_span();
    println!("Child span ID: {}", child_ctx.span_id);

    // Create span with attributes
    let mut span = Span::new("api_call".to_string(), &child_ctx);
    span.add_attribute("http.method".to_string(), SpanAttribute::String("GET".to_string()));
    span.add_attribute("http.status_code".to_string(), SpanAttribute::Number(200.0));
    span.add_attribute("duration_ms".to_string(), SpanAttribute::Number(42.5));

    // Add events
    span.add_event(SpanEvent {
        name: "cache_hit".to_string(),
        timestamp_ns: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64,
        attributes: Default::default(),
    });

    span.set_ok();
    println!("✓ Span created and marked successful");

    // Setup sampling and export
    let sampler: Arc<dyn SamplingStrategy> = Arc::new(AlwaysSampler);
    let exporter = Arc::new(InMemoryExporter::new());
    let provider = TracingProvider::new(sampler, exporter.clone());

    provider.start_span(span);

    if let Ok(spans) = provider.flush() {
        println!("✓ {} span(s) exported", exporter.get_spans().len());
    }
}

fn demo_schema_registry() -> Result<(), Box<dyn std::error::Error>> {
    println!("Setting up schema registry...");

    let registry = SchemaRegistry::new();

    // Register v1.0.0
    let schema_v1 = SchemaEntry::new(
        SchemaVersion::new(1, 0, 0),
        r#"{"nouns":["services","config"],"verbs":["status","list"]}"#.to_string(),
        "alice".to_string(),
        "Initial release".to_string(),
    );
    registry.register_schema(schema_v1)?;
    println!("✓ Registered schema v1.0.0");

    // Register v1.1.0
    let schema_v1_1 = SchemaEntry::new(
        SchemaVersion::new(1, 1, 0),
        r#"{"nouns":["services","config"],"verbs":["status","list","restart"]}"#.to_string(),
        "bob".to_string(),
        "Added restart verb".to_string(),
    );
    registry.register_schema(schema_v1_1)?;
    println!("✓ Registered schema v1.1.0");

    // List versions
    let versions = registry.list_versions();
    println!("Registered versions: {}", versions.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", "));

    // Check compatibility
    let v1_0 = SchemaVersion::new(1, 0, 0);
    let v1_1 = SchemaVersion::new(1, 1, 0);
    let v2_0 = SchemaVersion::new(2, 0, 0);

    let compat_1_0_to_1_1 = registry.check_compatibility(&v1_0, &v1_1);
    println!("v1.0.0 → v1.1.0: {:?}", compat_1_0_to_1_1);

    let compat_1_0_to_2_0 = registry.check_compatibility(&v1_0, &v2_0);
    println!("v1.0.0 → v2.0.0: {:?}", compat_1_0_to_2_0);

    // Verify integrity
    let integrity_check = registry.verify_schema_integrity(&v1_0);
    println!("Schema v1.0.0 integrity: {}", if integrity_check { "✓ Valid" } else { "✗ Invalid" });

    // Get merkle root
    if let Some(root) = registry.merkle_root() {
        println!("Registry merkle root: {}", &root[..16]);
    }

    Ok(())
}
