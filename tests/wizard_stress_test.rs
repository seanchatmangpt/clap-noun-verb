#![cfg_attr(not(feature = "wizard"), allow(dead_code))]
//! Concurrent Stress Tests for Wizard v2
//!
//! Tests high-throughput scenarios, concurrent sessions, memory stability,
//! and resource exhaustion handling.
//! Follows Chicago TDD principles with real collaborators and state verification.

#[cfg(feature = "wizard")]
use clap_noun_verb::wizard::{ModelConfig, Prompt, SessionBuilder, WizardSession};
#[cfg(feature = "wizard")]
use std::sync::{Arc, Mutex};
#[cfg(feature = "wizard")]
use std::thread;
#[cfg(feature = "wizard")]
use std::time::{Duration, Instant};

// =============================================================================
// CONCURRENT SESSION TESTS (100+ sessions)
// =============================================================================

#[cfg(feature = "wizard")]
#[test]
fn test_100_concurrent_session_creation() {
    // Arrange: Prepare 100 session IDs
    let session_count = 100;

    // Act: Create 100 sessions concurrently
    let handles: Vec<_> = (0..session_count)
        .map(|i| thread::spawn(move || WizardSession::new(format!("stress-{}", i))))
        .collect();

    // Assert: All sessions created successfully
    let sessions: Vec<_> =
        handles.into_iter().map(|h| h.join().expect("Thread should not panic")).collect();

    assert_eq!(sessions.len(), session_count);

    // Verify unique session IDs
    for (i, session) in sessions.iter().enumerate() {
        assert_eq!(session.session_id(), format!("stress-{}", i));
    }
}

#[cfg(feature = "wizard")]
#[test]
fn test_concurrent_session_state_transitions() {
    // Arrange: Create shared counter for verification
    let success_count = Arc::new(Mutex::new(0));
    let thread_count = 50;

    // Act: Perform state transitions concurrently
    let handles: Vec<_> = (0..thread_count)
        .map(|i| {
            let counter = Arc::clone(&success_count);
            thread::spawn(move || {
                let session = WizardSession::new(format!("concurrent-{}", i));
                let mut session = session.start();
                session.add_interaction("test".to_string(), "response".to_string());
                let _completed = session.complete();

                // Increment success counter
                let mut count = counter.lock().unwrap();
                *count += 1;
            })
        })
        .collect();

    // Assert: All threads complete successfully
    for handle in handles {
        handle.join().expect("Thread should not panic");
    }

    let final_count = *success_count.lock().unwrap();
    assert_eq!(final_count, thread_count);
}

// =============================================================================
// HIGH-THROUGHPUT PROMPT PROCESSING
// =============================================================================

#[cfg(feature = "wizard")]
#[test]
fn test_high_throughput_prompt_creation() {
    // Arrange: Create many prompts rapidly
    let prompt_count = 10_000;
    let start = Instant::now();

    // Act: Create prompts in tight loop
    let prompts: Vec<_> = (0..prompt_count).map(|i| Prompt::new(format!("Prompt {}", i))).collect();

    let duration = start.elapsed();

    // Assert: All prompts created
    assert_eq!(prompts.len(), prompt_count);

    // Performance assertion: Should create prompts quickly (<1s)
    assert!(
        duration < Duration::from_secs(1),
        "Should create 10k prompts in <1s, took {:?}",
        duration
    );
}

#[cfg(feature = "wizard")]
#[test]
fn test_prompt_with_long_history() {
    // Arrange: Create prompt with extensive history
    let mut prompt = Prompt::new("Final question");

    // Act: Add 1000 history messages
    for i in 0..1000 {
        prompt = prompt.with_history(vec![
            crate::wizard::types::Message {
                role: crate::wizard::types::Role::User,
                content: format!("User message {}", i),
            },
            crate::wizard::types::Message {
                role: crate::wizard::types::Role::Assistant,
                content: format!("Assistant response {}", i),
            },
        ]);
    }

    // Assert: Prompt maintains all history
    assert_eq!(prompt.history.len(), 2000); // 1000 pairs
}

// =============================================================================
// MEMORY STABILITY OVER TIME
// =============================================================================

#[cfg(feature = "wizard")]
#[test]
fn test_memory_stability_with_large_sessions() {
    // Arrange: Create session with large interaction history
    let session = WizardSession::new("memory-stability".to_string());
    let mut session = session.start();

    // Act: Add 5000 interactions
    for i in 0..5000 {
        session.add_interaction(
            format!("Question {} with some additional text to increase memory usage", i),
            format!("Answer {} with detailed response content that takes up space", i),
        );
    }

    // Assert: All interactions stored correctly
    assert_eq!(session.history().len(), 5000);

    // Verify first and last interactions
    assert!(session.history()[0].0.starts_with("Question 0"));
    assert!(session.history()[4999].0.starts_with("Question 4999"));
}

#[cfg(feature = "wizard")]
#[test]
fn test_session_metadata_memory_stability() {
    // Arrange: Create session with large metadata
    let large_metadata = serde_json::json!({
        "data": vec!["item"; 10000],
        "nested": {
            "deep": {
                "structure": vec![1; 1000]
            }
        }
    });

    // Act: Create session with large metadata
    let session = SessionBuilder::new()
        .session_id("metadata-test".to_string())
        .metadata(large_metadata.clone())
        .build()
        .expect("Should build session");

    // Assert: Metadata preserved
    assert_eq!(session.metadata(), &large_metadata);
}

// =============================================================================
// LOCK CONTENTION DETECTION
// =============================================================================

#[cfg(feature = "wizard")]
#[test]
fn test_concurrent_access_without_deadlock() {
    // Arrange: Create shared sessions
    let sessions: Vec<_> = (0..10)
        .map(|i| Arc::new(Mutex::new(WizardSession::new(format!("lock-test-{}", i)).start())))
        .collect();

    // Act: Access sessions concurrently from multiple threads
    let handles: Vec<_> = (0..100)
        .map(|i| {
            let session = Arc::clone(&sessions[i % 10]);
            thread::spawn(move || {
                let mut s = session.lock().unwrap();
                s.add_interaction(format!("q-{}", i), format!("a-{}", i));
            })
        })
        .collect();

    // Assert: All threads complete without deadlock
    for handle in handles {
        handle.join().expect("Should not deadlock");
    }

    // Verify all interactions were recorded
    let total_interactions: usize =
        sessions.iter().map(|s| s.lock().unwrap().history().len()).sum();

    assert_eq!(total_interactions, 100);
}

// =============================================================================
// THREAD POOL EXHAUSTION HANDLING
// =============================================================================

#[cfg(feature = "wizard")]
#[test]
fn test_rapid_thread_spawn_and_join() {
    // Arrange: Spawn many threads rapidly
    let thread_count = 200;

    // Act: Create and join threads rapidly
    for batch in 0..10 {
        let handles: Vec<_> = (0..20)
            .map(|i| {
                let id = batch * 20 + i;
                thread::spawn(move || {
                    let session = WizardSession::new(format!("thread-{}", id));
                    let _started = session.start();
                    thread::sleep(Duration::from_millis(1));
                })
            })
            .collect();

        for handle in handles {
            handle.join().expect("Thread should complete");
        }
    }

    // Assert: Test completes without thread pool exhaustion
    assert!(true, "All threads completed successfully");
}

// =============================================================================
// RESOURCE EXHAUSTION RECOVERY
// =============================================================================

#[cfg(feature = "wizard")]
#[test]
fn test_model_config_concurrent_modification() {
    // Arrange: Create shared config
    let config: Arc<Mutex<ModelConfig>> = Arc::new(Mutex::new(ModelConfig::default()));
    let thread_count = 50;

    // Act: Modify config concurrently
    let handles: Vec<_> = (0..thread_count)
        .map(|i| {
            let cfg: Arc<Mutex<ModelConfig>> = Arc::clone(&config);
            thread::spawn(move || {
                let mut c = cfg.lock().unwrap();
                *c = c.clone().with_temperature(0.5 + (i as f32 / 100.0));
            })
        })
        .collect();

    // Assert: All modifications complete
    for handle in handles {
        handle.join().expect("Should not panic");
    }

    // Config should have final value
    let final_config = config.lock().unwrap();
    assert!(final_config.temperature >= 0.5);
}

#[cfg(feature = "wizard")]
#[test]
fn test_session_builder_under_load() {
    // Arrange: Build many sessions rapidly
    let session_count = 1000;
    let start = Instant::now();

    // Act: Build sessions rapidly
    let sessions: Vec<_> = (0..session_count)
        .map(|i| {
            SessionBuilder::new()
                .session_id(format!("builder-{}", i))
                .metadata(serde_json::json!({"index": i}))
                .build()
                .expect("Should build")
        })
        .collect();

    let duration = start.elapsed();

    // Assert: All sessions built successfully
    assert_eq!(sessions.len(), session_count);

    // Performance assertion: Should build quickly
    assert!(
        duration < Duration::from_secs(2),
        "Should build 1000 sessions in <2s, took {:?}",
        duration
    );
}

// =============================================================================
// PERFORMANCE BENCHMARKS
// =============================================================================

#[cfg(feature = "wizard")]
#[test]
fn test_session_creation_performance() {
    // Arrange: Prepare for performance test
    let iterations = 10_000;
    let start = Instant::now();

    // Act: Create sessions in tight loop
    for i in 0..iterations {
        let _session = WizardSession::new(format!("perf-{}", i));
    }

    let duration = start.elapsed();
    let per_iteration = duration / iterations;

    // Assert: Performance is acceptable
    assert!(
        per_iteration < Duration::from_micros(100),
        "Session creation should be <100μs, was {:?}",
        per_iteration
    );
}

#[cfg(feature = "wizard")]
#[test]
fn test_interaction_addition_performance() {
    // Arrange: Create session
    let session = WizardSession::new("interaction-perf".to_string());
    let mut session = session.start();
    let iterations = 10_000;
    let start = Instant::now();

    // Act: Add interactions rapidly
    for i in 0..iterations {
        session.add_interaction(format!("prompt {}", i), format!("response {}", i));
    }

    let duration = start.elapsed();
    let per_iteration = duration / iterations;

    // Assert: Performance is acceptable
    assert!(
        per_iteration < Duration::from_micros(50),
        "Interaction addition should be <50μs, was {:?}",
        per_iteration
    );
    assert_eq!(session.history().len(), iterations as usize);
}

#[cfg(feature = "wizard")]
#[test]
fn test_concurrent_prompt_processing() {
    // Arrange: Create prompts concurrently
    let thread_count = 100;
    let prompts_per_thread = 100;

    // Act: Process prompts across threads
    let handles: Vec<_> = (0..thread_count)
        .map(|thread_id| {
            thread::spawn(move || {
                let prompts: Vec<_> = (0..prompts_per_thread)
                    .map(|i| {
                        Prompt::new(format!("Thread {} prompt {}", thread_id, i))
                            .with_system("System context")
                    })
                    .collect();
                prompts.len()
            })
        })
        .collect();

    // Assert: All threads complete successfully
    let total: usize = handles.into_iter().map(|h| h.join().expect("Thread should complete")).sum();

    assert_eq!(total, thread_count * prompts_per_thread);
}
