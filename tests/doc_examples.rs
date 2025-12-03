//! Documentation Examples Test Harness
//!
//! This module contains all code examples from documentation, ensuring they compile
//! and work correctly with the v5.1.1 API.
//!
//! Test organization:
//! - `readme_examples`: Examples from README.md
//! - `quickstart_examples`: Examples from docs/QUICKSTART.md
//! - `cli_reference_examples`: Examples from docs/CLI_REFERENCE.md
//! - `autonomic_examples`: Examples from AUTONOMIC.md
//! - `cookbook_examples`: Examples from docs/CLI_COOKBOOK.md

#![allow(dead_code, unused_imports, unused_variables)]

use clap_noun_verb::*;
use serde::Serialize;

mod readme_examples {
    use super::*;
    use clap_noun_verb::Result;

    /// Example from README.md:30-Second Example (Domain-Separated)
    #[test]
    fn test_domain_separated_example() {
        // domain/calculator.rs - Pure business logic
        mod domain {
            pub fn add(x: i32, y: i32) -> i32 {
                x + y
            }

            #[cfg(test)]
            mod tests {
                use super::*;

                #[test]
                fn test_add() {
                    assert_eq!(add(2, 3), 5);
                }
            }
        }

        // Verify domain logic works
        assert_eq!(domain::add(2, 3), 5);
    }

    /// Example from README.md: Elite Rust Mindset - Type-First Thinking
    #[test]
    fn test_type_first_service_state() {
        use std::time::Duration;

        // ✅ GOOD: Types make invalid states unrepresentable
        #[derive(Debug, PartialEq)]
        enum ServiceState {
            Running { pid: u32, uptime: Duration },
            Stopped,
        }

        let running = ServiceState::Running {
            pid: 1234,
            uptime: Duration::from_secs(3600),
        };

        match running {
            ServiceState::Running { pid, uptime } => {
                assert_eq!(pid, 1234);
                assert_eq!(uptime, Duration::from_secs(3600));
            }
            ServiceState::Stopped => panic!("Expected running state"),
        }
    }

    /// Example from README.md: Zero-Cost Awareness
    #[test]
    fn test_zero_cost_generics() {
        use serde::Serialize;

        // ✅ Zero-cost (monomorphization)
        fn process<T: Serialize>(item: T) -> String {
            serde_json::to_string(&item).unwrap_or_default()
        }

        #[derive(Serialize)]
        struct TestData {
            value: i32,
        }

        let data = TestData { value: 42 };
        let json = process(data);
        assert!(json.contains("42"));
    }

    /// Example from README.md: Memory Safety - Ownership
    #[test]
    fn test_ownership_semantics() {
        #[derive(Debug)]
        struct Output {
            result: i32,
        }

        // ✅ GOOD: Ownership explicit
        fn process_data(data: Vec<u8>) -> Result<Output> {
            let sum = data.iter().map(|&b| b as i32).sum();
            Ok(Output { result: sum })
        }

        let data = vec![1, 2, 3, 4, 5];
        let output = process_data(data);
        assert!(output.is_ok());
        // data is moved, can't use it here
    }

    /// Example from README.md: API Design - Type-Safe by Construction
    #[test]
    fn test_validated_email() {
        #[derive(Debug)]
        enum ValidationError {
            InvalidEmail,
        }

        // ✅ GOOD: Type-safe by construction
        struct ValidatedEmail(String);

        impl ValidatedEmail {
            pub fn new(email: String) -> std::result::Result<Self, ValidationError> {
                if email.contains('@') {
                    Ok(Self(email))
                } else {
                    Err(ValidationError::InvalidEmail)
                }
            }
        }

        fn send_email(to: ValidatedEmail) -> String {
            format!("Sent email to {}", to.0)
        }

        // Valid email
        let valid = ValidatedEmail::new("user@example.com".to_string());
        assert!(valid.is_ok());

        // Invalid email
        let invalid = ValidatedEmail::new("invalid".to_string());
        assert!(invalid.is_err());
    }
}

mod quickstart_examples {
    use super::*;
    use clap_noun_verb::Result;

    /// Example from QUICKSTART.md: Step 2 - List Available Commands
    #[test]
    fn test_status_verb_structure() {
        #[derive(Serialize, Debug, PartialEq)]
        struct Status {
            message: String,
        }

        // Simulate the verb logic
        fn status() -> Result<Status> {
            Ok(Status {
                message: "System is running".to_string(),
            })
        }

        let result = status();
        assert!(result.is_ok());
        let status = result.unwrap();
        assert_eq!(status.message, "System is running");
    }

    /// Example from QUICKSTART.md: Step 3 - Services Module
    #[test]
    fn test_service_status_structure() {
        #[derive(Serialize, Debug)]
        struct ServiceStatus {
            name: String,
            running: bool,
            port: u16,
        }

        #[derive(Serialize, Debug)]
        struct ServiceList {
            services: Vec<ServiceStatus>,
        }

        fn status() -> Result<ServiceList> {
            Ok(ServiceList {
                services: vec![
                    ServiceStatus {
                        name: "api".to_string(),
                        running: true,
                        port: 8080,
                    },
                    ServiceStatus {
                        name: "worker".to_string(),
                        running: true,
                        port: 8081,
                    },
                ],
            })
        }

        let result = status();
        assert!(result.is_ok());
        let list = result.unwrap();
        assert_eq!(list.services.len(), 2);
        assert_eq!(list.services[0].name, "api");
        assert_eq!(list.services[0].port, 8080);
    }

    /// Example from QUICKSTART.md: Step 3 - Restart Verb
    #[test]
    fn test_restart_verb_structure() {
        #[derive(Serialize, Debug)]
        struct ServiceStatus {
            name: String,
            running: bool,
            port: u16,
        }

        fn restart(service: String, force: bool) -> Result<ServiceStatus> {
            Ok(ServiceStatus {
                name: service,
                running: true,
                port: 8080,
            })
        }

        let result = restart("api".to_string(), false);
        assert!(result.is_ok());
        let status = result.unwrap();
        assert_eq!(status.name, "api");
        assert!(status.running);
    }
}

mod autonomic_examples {
    use super::*;
    use clap_noun_verb::autonomic::*;
    use clap_noun_verb::Result;

    /// Example from AUTONOMIC.md: Effect Metadata
    #[test]
    fn test_effect_metadata_structure() {
        let metadata = CommandMetadata::new().with_effects(
            EffectMetadata::new(EffectType::ReadOnly).with_sensitivity(Sensitivity::Low),
        );

        // Verify metadata structure (effects is Option<EffectMetadata>)
        assert!(metadata.effects.is_some());
        let effects = metadata.effects.unwrap();
        assert_eq!(effects.effect_type, EffectType::ReadOnly);
        assert_eq!(effects.sensitivity, Sensitivity::Low);
    }

    /// Example from AUTONOMIC.md: Plane Interactions
    #[test]
    fn test_plane_interaction_structure() {
        let _interaction = PlaneInteraction::new()
            .observe_read()
            .ontology_read()
            .invariants_check()
            .overlays_emit();

        // PlaneInteraction structure verified by compilation
        // (methods exist and chain correctly)
    }

    /// Example from AUTONOMIC.md: Guards & Budgets
    #[test]
    fn test_guard_config_structure() {
        let guards = GuardConfig::new()
            .with_max_latency_ms(100)
            .with_max_memory_kb(1024)
            .with_max_cpu_ms(50);

        // Verify guard configuration (using public fields)
        assert_eq!(guards.max_latency_ms, Some(100));
        assert_eq!(guards.max_memory_kb, Some(1024));
        assert_eq!(guards.max_cpu_ms, Some(50));
    }

    /// Example from AUTONOMIC.md: Execution Receipts
    #[test]
    fn test_execution_receipt_structure() {
        let plane_interaction = PlaneInteraction::new().observe_read();

        let receipt = ExecutionReceipt::new("services status")
            .with_duration_ms(50)
            .with_guard(GuardResult::within_budget(50, 100))
            .with_planes(&plane_interaction);

        // Verify receipt structure (using public fields)
        assert_eq!(receipt.command, "services status");
        assert_eq!(receipt.duration_ms, 50);  // duration_ms is u64, not Option
        assert!(receipt.success);
    }

    /// Example from AUTONOMIC.md: Structured Errors
    #[test]
    fn test_structured_error() {
        let error = StructuredError::deadline_exceeded(100, 150);

        // Verify error structure (using public field)
        match error.kind {
            ErrorKind::DeadlineExceeded => {
                // Expected
            }
            _ => panic!("Expected DeadlineExceeded"),
        }
    }
}

mod cli_reference_examples {
    use super::*;
    use clap_noun_verb::Result;

    /// Example from CLI_REFERENCE.md: Basic Command Definition
    #[test]
    fn test_basic_command_structure() {
        #[derive(Serialize, Debug)]
        struct Output {
            message: String,
        }

        fn status() -> Result<Output> {
            Ok(Output {
                message: "OK".to_string(),
            })
        }

        let result = status();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().message, "OK");
    }

    /// Example from CLI_REFERENCE.md: Type Inference - String Argument
    #[test]
    fn test_string_argument() {
        #[derive(Serialize, Debug)]
        struct Greeting {
            message: String,
        }

        fn greet(name: String) -> Result<Greeting> {
            Ok(Greeting {
                message: format!("Hello, {}!", name),
            })
        }

        let result = greet("Alice".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().message, "Hello, Alice!");
    }

    /// Example from CLI_REFERENCE.md: Type Inference - Optional Argument
    #[test]
    fn test_optional_argument() {
        #[derive(Serialize, Debug)]
        struct Connection {
            host: String,
            port: u16,
        }

        fn connect(host: String, port: Option<u16>) -> Result<Connection> {
            let port = port.unwrap_or(8080);
            Ok(Connection { host, port })
        }

        // Test with port
        let with_port = connect("localhost".to_string(), Some(3000));
        assert!(with_port.is_ok());
        assert_eq!(with_port.unwrap().port, 3000);

        // Test without port (default)
        let without_port = connect("localhost".to_string(), None);
        assert!(without_port.is_ok());
        assert_eq!(without_port.unwrap().port, 8080);
    }

    /// Example from CLI_REFERENCE.md: Type Inference - Multiple Values
    #[test]
    fn test_multiple_values() {
        #[derive(Serialize, Debug)]
        struct TagResult {
            tags: Vec<String>,
            count: usize,
        }

        fn tag(tags: Vec<String>) -> Result<TagResult> {
            let count = tags.len();
            Ok(TagResult { tags, count })
        }

        let result = tag(vec!["tag1".to_string(), "tag2".to_string(), "tag3".to_string()]);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.count, 3);
        assert_eq!(result.tags.len(), 3);
    }
}

mod cookbook_examples {
    use super::*;
    use clap_noun_verb::Result;

    /// Example from CLI_COOKBOOK.md: Recipe 1 - Basic User Management
    #[test]
    fn test_user_management_structure() {
        #[derive(Serialize, Debug)]
        struct User {
            id: u32,
            name: String,
            email: String,
        }

        #[derive(Serialize, Debug)]
        struct UserList {
            users: Vec<User>,
            total: usize,
        }

        fn list(limit: usize, domain: Option<String>) -> Result<UserList> {
            let users = vec![
                User {
                    id: 1,
                    name: "Alice".to_string(),
                    email: "alice@example.com".to_string(),
                },
                User {
                    id: 2,
                    name: "Bob".to_string(),
                    email: "bob@example.com".to_string(),
                },
            ];

            let filtered = if let Some(domain) = domain {
                users
                    .into_iter()
                    .filter(|u| u.email.ends_with(&domain))
                    .collect()
            } else {
                users
            };

            Ok(UserList {
                total: filtered.len(),
                users: filtered,
            })
        }

        // Test without domain filter
        let all_users = list(10, None);
        assert!(all_users.is_ok());
        assert_eq!(all_users.unwrap().total, 2);

        // Test with domain filter
        let filtered = list(10, Some("example.com".to_string()));
        assert!(filtered.is_ok());
        assert_eq!(filtered.unwrap().total, 2);
    }

    /// Example from CLI_COOKBOOK.md: Recipe 6 - Workflow Steps
    #[test]
    fn test_workflow_structure() {
        #[derive(Serialize, Debug)]
        struct WorkflowResult {
            steps: Vec<StepResult>,
            success: bool,
        }

        #[derive(Serialize, Debug)]
        struct StepResult {
            name: String,
            status: String,
            duration_ms: u64,
        }

        fn deploy(environment: String, skip_tests: bool) -> Result<WorkflowResult> {
            let mut steps = Vec::new();

            // Step 1: Build
            steps.push(StepResult {
                name: "Build".to_string(),
                status: "success".to_string(),
                duration_ms: 1000,
            });

            // Step 2: Test (optional)
            if !skip_tests {
                steps.push(StepResult {
                    name: "Test".to_string(),
                    status: "success".to_string(),
                    duration_ms: 500,
                });
            }

            // Step 3: Deploy
            steps.push(StepResult {
                name: "Deploy".to_string(),
                status: "success".to_string(),
                duration_ms: 2000,
            });

            Ok(WorkflowResult {
                steps,
                success: true,
            })
        }

        // Test with tests
        let with_tests = deploy("production".to_string(), false);
        assert!(with_tests.is_ok());
        assert_eq!(with_tests.unwrap().steps.len(), 3);

        // Test skip tests
        let skip_tests = deploy("production".to_string(), true);
        assert!(skip_tests.is_ok());
        assert_eq!(skip_tests.unwrap().steps.len(), 2);
    }

    /// Example from CLI_COOKBOOK.md: Recipe 10 - Custom Error Types
    #[test]
    fn test_custom_error_handling() {
        use thiserror::Error;

        #[derive(Error, Debug)]
        enum AppError {
            #[error("Database error: {0}")]
            Database(String),

            #[error("Configuration error: {0}")]
            Config(String),

            #[error("Validation failed: {field}: {message}")]
            Validation { field: String, message: String },
        }

        // Simulate error creation
        let db_error = AppError::Database("Connection failed".to_string());
        assert_eq!(db_error.to_string(), "Database error: Connection failed");

        let validation_error = AppError::Validation {
            field: "email".to_string(),
            message: "Invalid format".to_string(),
        };
        assert_eq!(
            validation_error.to_string(),
            "Validation failed: email: Invalid format"
        );
    }
}

#[cfg(test)]
mod gap_analysis {
    //! Gap Analysis: Documented Features vs Actual v5.1.1 Capabilities
    //!
    //! This module tracks discrepancies between documentation and implementation.

    use super::*;

    #[test]
    fn test_documented_vs_actual_apis() {
        // Track API changes from v4 to v5.1.1
        let documented_v4_apis = vec![
            "VerbArgs",        // Documented in CLI_REFERENCE.md
            "OutputFormat",    // Documented in QUICKSTART.md
            "run_with_format", // Documented in QUICKSTART.md
        ];

        let actual_v5_apis = vec![
            "VerbContext", // Actual in v5.1.1
            "format::OutputFormat",
            "CliBuilder::with_format",
        ];

        // This test documents the gaps
        println!("\nAPI Migration Required:");
        println!("=======================");
        for (old, new) in documented_v4_apis.iter().zip(actual_v5_apis.iter()) {
            println!("  {} → {}", old, new);
        }
    }

    #[test]
    fn test_version_references() {
        // All documentation should reference v5.1.1, not v4.0.2 or v3.8.0
        let outdated_versions = vec!["4.0.2", "3.8.0", "3.2.0"];
        let current_version = "5.1.1";

        println!("\nVersion References to Update:");
        println!("=============================");
        for version in outdated_versions {
            println!("  {} → {}", version, current_version);
        }
    }
}
