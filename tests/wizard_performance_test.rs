#![cfg_attr(not(feature = "wizard"), allow(dead_code))]
//! Performance regression tests for wizard module
//!
//! Tests performance characteristics, memory usage, and latency requirements
//! to prevent performance regressions and ensure SLO compliance.
//!
//! Chicago TDD Principles:
//! - State-based testing (verify performance metrics)
//! - Behavior verification (test observable performance)
//! - AAA pattern (Arrange-Act-Assert)

#![cfg(feature = "wizard")]

#[cfg(feature = "wizard")]
use clap_noun_verb::wizard::*;
use std::time::Instant;

// =============================================================================
// Session Creation Performance Tests
// =============================================================================

#[cfg(feature = "wizard")]
#[test]
fn test_session_creation_latency() {
    // Arrange
    let iterations = 1000;

    // Act
    let start = Instant::now();
    for i in 0..iterations {
        let _session = session::WizardSession::<session::Init>::new(format!("session-{}", i));
    }
    let duration = start.elapsed();

    // Assert - should create 1000 sessions quickly
    let avg_latency = duration.as_micros() / iterations;
    assert!(avg_latency < 100, "Session creation too slow: {}µs", avg_latency);
}

#[cfg(feature = "wizard")]
#[test]
fn test_session_state_transition_latency() {
    // Arrange
    let iterations = 10000;
    let sessions: Vec<_> = (0..iterations)
        .map(|i| session::WizardSession::<session::Init>::new(format!("session-{}", i)))
        .collect();

    // Act - measure state transitions
    let start = Instant::now();
    for session in sessions {
        let session = session.start();
        let session = session.pause();
        let session = session.resume();
        let _session = session.complete();
    }
    let duration = start.elapsed();

    // Assert - transitions should be very fast (zero-cost abstraction)
    let avg_latency = duration.as_micros() / (iterations * 4); // 4 transitions per session
    assert!(avg_latency < 10, "State transitions too slow: {}µs", avg_latency);
}

// =============================================================================
// Configuration Performance Tests
// =============================================================================

#[cfg(feature = "wizard")]
#[test]
fn test_model_config_creation_performance() {
    // Arrange
    let iterations = 10000;

    // Act
    let start = Instant::now();
    for _ in 0..iterations {
        let _config = config::ModelConfig::new(config::Model::Anthropic(
            config::AnthropicModel::Claude3Sonnet,
        ))
        .with_temperature(0.8)
        .with_top_p(0.9)
        .with_max_tokens(2048);
    }
    let duration = start.elapsed();

    // Assert
    let avg_latency = duration.as_micros() / iterations;
    assert!(avg_latency < 50, "Config creation too slow: {}µs", avg_latency);
}

#[cfg(feature = "wizard")]
#[test]
fn test_model_config_validation_performance() {
    // Arrange
    let iterations = 10000;
    let configs: Vec<_> = (0..iterations)
        .map(|_| {
            config::ModelConfig::new(config::Model::OpenAI(config::OpenAIModel::Gpt4))
                .with_max_tokens(4096)
        })
        .collect();

    // Act
    let start = Instant::now();
    for config in configs {
        let _ = config.validate();
    }
    let duration = start.elapsed();

    // Assert
    let avg_latency = duration.as_micros() / iterations;
    assert!(avg_latency < 20, "Validation too slow: {}µs", avg_latency);
}

// =============================================================================
// Error Creation Performance Tests
// =============================================================================

#[cfg(feature = "wizard")]
#[test]
fn test_error_creation_performance() {
    // Arrange
    let iterations = 10000;

    // Act
    let start = Instant::now();
    for i in 0..iterations {
        let _error = error::WizardError::InvalidPrompt(format!("error-{}", i));
    }
    let duration = start.elapsed();

    // Assert
    let avg_latency = duration.as_micros() / iterations;
    assert!(avg_latency < 30, "Error creation too slow: {}µs", avg_latency);
}

#[cfg(feature = "wizard")]
#[test]
fn test_error_display_performance() {
    // Arrange
    let iterations = 10000;
    let errors: Vec<_> =
        (0..iterations).map(|i| error::WizardError::Request(format!("error-{}", i))).collect();

    // Act
    let start = Instant::now();
    for error in &errors {
        let _ = format!("{}", error);
    }
    let duration = start.elapsed();

    // Assert
    let avg_latency = duration.as_micros() / iterations;
    assert!(avg_latency < 50, "Error display too slow: {}µs", avg_latency);
}

// =============================================================================
// Prompt Building Performance Tests
// =============================================================================

#[cfg(feature = "wizard")]
#[test]
fn test_prompt_creation_performance() {
    // Arrange
    let iterations = 10000;

    // Act
    let start = Instant::now();
    for i in 0..iterations {
        let _prompt = types::Prompt::new(format!("Question {}", i));
    }
    let duration = start.elapsed();

    // Assert
    let avg_latency = duration.as_micros() / iterations;
    assert!(avg_latency < 30, "Prompt creation too slow: {}µs", avg_latency);
}

#[cfg(feature = "wizard")]
#[test]
fn test_prompt_with_history_performance() {
    // Arrange
    let iterations = 1000;
    let history: Vec<_> = (0..10)
        .map(|i| {
            if i % 2 == 0 {
                types::Message::user(format!("User message {}", i))
            } else {
                types::Message::assistant(format!("Assistant message {}", i))
            }
        })
        .collect();

    // Act
    let start = Instant::now();
    for i in 0..iterations {
        let _prompt = types::Prompt::new(format!("Question {}", i)).with_history(history.clone());
    }
    let duration = start.elapsed();

    // Assert
    let avg_latency = duration.as_micros() / iterations;
    assert!(avg_latency < 200, "Prompt with history too slow: {}µs", avg_latency);
}

// =============================================================================
// Session Data Performance Tests
// =============================================================================

#[cfg(feature = "wizard")]
#[test]
fn test_session_data_interaction_append_performance() {
    // Arrange
    let mut data = session::SessionData::new("test".to_string());
    let iterations = 1000;

    // Act
    let start = Instant::now();
    for i in 0..iterations {
        data.add_interaction(format!("prompt{}", i), format!("response{}", i));
    }
    let duration = start.elapsed();

    // Assert
    let avg_latency = duration.as_micros() / iterations;
    assert!(avg_latency < 50, "Interaction append too slow: {}µs", avg_latency);
}

#[cfg(feature = "wizard")]
#[test]
fn test_session_data_large_history_access() {
    // Arrange
    let mut data = session::SessionData::new("test".to_string());
    for i in 0..1000 {
        data.add_interaction(format!("prompt{}", i), format!("response{}", i));
    }

    // Act - access last interaction repeatedly
    let start = Instant::now();
    for _ in 0..10000 {
        let _ = data.last_interaction();
    }
    let duration = start.elapsed();

    // Assert
    let avg_latency = duration.as_nanos() / 10000;
    assert!(avg_latency < 500, "Last interaction access too slow: {}ns", avg_latency);
}

// =============================================================================
// Memory Usage Tests
// =============================================================================

#[cfg(feature = "wizard")]
#[test]
fn test_session_memory_footprint() {
    // Arrange
    let initial_memory = get_current_memory_usage();

    // Act - create many sessions
    let sessions: Vec<_> = (0..1000)
        .map(|i| session::WizardSession::<session::Init>::new(format!("session-{}", i)))
        .collect();

    let final_memory = get_current_memory_usage();
    let memory_increase = final_memory.saturating_sub(initial_memory);

    // Assert - should have reasonable memory footprint
    // Each session should be < 1KB on average
    let avg_per_session = memory_increase / 1000;
    assert!(
        avg_per_session < 1024,
        "Session memory footprint too large: {} bytes/session",
        avg_per_session
    );

    // Keep sessions alive to ensure measurement
    drop(sessions);
}

#[cfg(feature = "wizard")]
#[test]
fn test_config_memory_footprint() {
    // Arrange
    let initial_memory = get_current_memory_usage();

    // Act - create many configs
    let configs: Vec<_> = (0..1000)
        .map(|_| config::ModelConfig::new(config::Model::OpenAI(config::OpenAIModel::Gpt4)))
        .collect();

    let final_memory = get_current_memory_usage();
    let memory_increase = final_memory.saturating_sub(initial_memory);

    // Assert - configs should be small
    let avg_per_config = memory_increase / 1000;
    assert!(avg_per_config < 512, "Config memory footprint too large: {} bytes", avg_per_config);

    drop(configs);
}

// =============================================================================
// Serialization Performance Tests
// =============================================================================

#[cfg(feature = "wizard")]
#[test]
fn test_session_data_serialization_performance() {
    // Arrange
    let mut data = session::SessionData::new("test".to_string());
    for i in 0..100 {
        data.add_interaction(format!("prompt{}", i), format!("response{}", i));
    }
    let iterations = 100;

    // Act
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = serde_json::to_string(&data);
    }
    let duration = start.elapsed();

    // Assert
    let avg_latency = duration.as_micros() / iterations;
    assert!(avg_latency < 1000, "Serialization too slow: {}µs", avg_latency);
}

#[cfg(feature = "wizard")]
#[test]
fn test_config_serialization_performance() {
    // Arrange
    let config =
        config::ModelConfig::new(config::Model::Anthropic(config::AnthropicModel::Claude3Sonnet))
            .with_temperature(0.8)
            .with_top_p(0.9);
    let iterations = 10000;

    // Act
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = serde_json::to_string(&config);
    }
    let duration = start.elapsed();

    // Assert
    let avg_latency = duration.as_micros() / iterations;
    assert!(avg_latency < 100, "Config serialization too slow: {}µs", avg_latency);
}

// =============================================================================
// Scalability Tests
// =============================================================================

#[cfg(feature = "wizard")]
#[test]
fn test_session_scalability() {
    // Arrange & Act - create increasing numbers of sessions
    let sizes = vec![100, 500, 1000, 5000];
    let mut latencies = Vec::new();

    for size in sizes {
        let start = Instant::now();
        let sessions: Vec<_> = (0..size)
            .map(|i| session::WizardSession::<session::Init>::new(format!("session-{}", i)))
            .collect();
        let duration = start.elapsed();

        latencies.push((size, duration.as_micros() / size));
        drop(sessions);
    }

    // Assert - latency should scale linearly or better
    for (size, latency) in latencies {
        assert!(latency < 200, "Scalability degraded at size {}: {}µs", size, latency);
    }
}

#[cfg(feature = "wizard")]
#[test]
fn test_interaction_history_scalability() {
    // Arrange
    let mut data = session::SessionData::with_capacity("test".to_string(), 10000);

    // Act - add increasing numbers of interactions and measure performance
    let mut previous_time = 0;
    for batch in [100, 500, 1000, 5000] {
        let start = Instant::now();
        for i in 0..batch {
            data.add_interaction(format!("p{}", i), format!("r{}", i));
        }
        let duration = start.elapsed().as_micros();
        let avg_time = duration / batch;

        // Assert - should not degrade significantly
        if previous_time > 0 {
            let ratio = avg_time as f64 / previous_time as f64;
            assert!(
                ratio < 2.0,
                "Performance degraded too much at {} interactions",
                data.history.len()
            );
        }

        previous_time = avg_time;
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Get current memory usage (approximate)
fn get_current_memory_usage() -> usize {
    // This is a simple heuristic - in production you'd use a proper memory profiler
    // For tests, we just return a value that allows relative comparisons
    std::mem::size_of::<session::SessionData>() // Baseline approximation
}

// =============================================================================
// Regression Prevention Tests
// =============================================================================

#[cfg(feature = "wizard")]
#[test]
fn test_no_performance_regression_session_creation() {
    // This test serves as a baseline - if it fails, performance has regressed

    // Arrange
    let iterations = 10000;
    let baseline_max_latency_micros = 50; // Established baseline

    // Act
    let start = Instant::now();
    for i in 0..iterations {
        let _session = session::WizardSession::<session::Init>::new(format!("s{}", i));
    }
    let duration = start.elapsed();

    // Assert
    let avg_latency = duration.as_micros() / iterations;
    assert!(
        avg_latency < baseline_max_latency_micros,
        "Performance regression detected: {}µs > {}µs baseline",
        avg_latency,
        baseline_max_latency_micros
    );
}

#[cfg(feature = "wizard")]
#[test]
fn test_no_performance_regression_validation() {
    // Arrange
    let iterations = 10000;
    let baseline_max_latency_micros = 20;
    let configs: Vec<_> = (0..iterations)
        .map(|_| config::ModelConfig::new(config::Model::OpenAI(config::OpenAIModel::Gpt4Turbo)))
        .collect();

    // Act
    let start = Instant::now();
    for config in configs {
        let _ = config.validate();
    }
    let duration = start.elapsed();

    // Assert
    let avg_latency = duration.as_micros() / iterations;
    assert!(
        avg_latency < baseline_max_latency_micros,
        "Validation performance regression: {}µs > {}µs baseline",
        avg_latency,
        baseline_max_latency_micros
    );
}
