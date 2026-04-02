#![cfg_attr(not(feature = "wizard"), allow(dead_code))]
//! Comprehensive session state machine tests for wizard module
//!
//! Tests type-safe state transitions, session lifecycle, SessionBuilder,
//! session data management, and compile-time guarantees.
//!
//! Chicago TDD Principles:
//! - State-based testing (verify session state and data)
//! - Behavior verification (test state transitions and invariants)
//! - AAA pattern (Arrange-Act-Assert)

#![cfg(feature = "wizard")]

#[cfg(feature = "wizard")]
use clap_noun_verb::wizard::session::*;
use serde_json::json;

// =============================================================================
// State Trait Tests - Verify all states implement State trait correctly
// =============================================================================

#[cfg(feature = "wizard")]
#[test]
fn test_init_state_name() {
    // Arrange + Act
    let state = Init;

    // Assert
    assert_eq!(state.name(), "Init");
    assert!(!state.is_terminal());
}

#[cfg(feature = "wizard")]
#[test]
fn test_active_state_name() {
    // Arrange + Act
    let state = Active;

    // Assert
    assert_eq!(state.name(), "Active");
    assert!(!state.is_terminal());
}

#[cfg(feature = "wizard")]
#[test]
fn test_paused_state_name() {
    // Arrange + Act
    let state = Paused;

    // Assert
    assert_eq!(state.name(), "Paused");
    assert!(!state.is_terminal());
}

#[cfg(feature = "wizard")]
#[test]
fn test_complete_state_name() {
    // Arrange + Act
    let state = Complete;

    // Assert
    assert_eq!(state.name(), "Complete");
    assert!(state.is_terminal());
}

#[cfg(feature = "wizard")]
#[test]
fn test_failed_state_name() {
    // Arrange + Act
    let state = Failed;

    // Assert
    assert_eq!(state.name(), "Failed");
    assert!(state.is_terminal());
}

// =============================================================================
// SessionData Tests - Test session data management
// =============================================================================

#[cfg(feature = "wizard")]
#[test]
fn test_session_data_new() {
    // Arrange
    let session_id = "session-123".to_string();

    // Act
    let data = SessionData::new(session_id.clone());

    // Assert
    assert_eq!(data.session_id, session_id);
    assert!(data.history.is_empty());
    assert_eq!(data.metadata, serde_json::Value::Null);
}

#[cfg(feature = "wizard")]
#[test]
fn test_session_data_with_capacity() {
    // Arrange
    let session_id = "session-456".to_string();
    let capacity = 50;

    // Act
    let data = SessionData::with_capacity(session_id.clone(), capacity);

    // Assert
    assert_eq!(data.session_id, session_id);
    assert_eq!(data.history.capacity(), capacity);
}

#[cfg(feature = "wizard")]
#[test]
fn test_session_data_add_interaction() {
    // Arrange
    let mut data = SessionData::new("test".to_string());
    let prompt = "What is Rust?".to_string();
    let response = "Rust is a systems programming language".to_string();

    // Act
    data.add_interaction(prompt.clone(), response.clone());

    // Assert
    assert_eq!(data.history.len(), 1);
    assert_eq!(data.history[0].0, prompt);
    assert_eq!(data.history[0].1, response);
}

#[cfg(feature = "wizard")]
#[test]
fn test_session_data_multiple_interactions() {
    // Arrange
    let mut data = SessionData::new("test".to_string());

    // Act
    for i in 0..10 {
        data.add_interaction(format!("prompt{}", i), format!("response{}", i));
    }

    // Assert
    assert_eq!(data.history.len(), 10);
    assert_eq!(data.history[5].0, "prompt5");
}

#[cfg(feature = "wizard")]
#[test]
fn test_session_data_last_interaction() {
    // Arrange
    let mut data = SessionData::new("test".to_string());

    // Act - no interactions yet
    let last_before = data.last_interaction();
    assert!(last_before.is_none());

    // Add interactions
    data.add_interaction("first".to_string(), "response1".to_string());
    data.add_interaction("second".to_string(), "response2".to_string());

    // Act - get last interaction
    let last_after = data.last_interaction();

    // Assert
    assert!(last_after.is_some());
    let (prompt, response) = last_after.unwrap();
    assert_eq!(prompt, "second");
    assert_eq!(response, "response2");
}

// =============================================================================
// WizardSession<Init> Tests - Test initial state
// =============================================================================

#[cfg(feature = "wizard")]
#[test]
fn test_wizard_session_new() {
    // Arrange
    let session_id = "session-789".to_string();

    // Act
    let session = WizardSession::<Init>::new(session_id.clone());

    // Assert
    assert_eq!(session.session_id(), session_id);
    assert_eq!(session.history().len(), 0);
}

#[cfg(feature = "wizard")]
#[test]
fn test_wizard_session_with_capacity() {
    // Arrange
    let session_id = "session-capacity".to_string();
    let capacity = 100;

    // Act
    let session = WizardSession::<Init>::with_capacity(session_id.clone(), capacity);

    // Assert
    assert_eq!(session.session_id(), session_id);
}

#[cfg(feature = "wizard")]
#[test]
fn test_wizard_session_metadata() {
    // Arrange
    let mut session = WizardSession::<Init>::new("test".to_string());
    let metadata = json!({
        "user_id": "user123",
        "created_at": "2024-01-01T00:00:00Z"
    });

    // Act
    session.set_metadata(metadata.clone());

    // Assert
    assert_eq!(session.metadata(), &metadata);
}

// =============================================================================
// State Transition Tests - Test all valid transitions
// =============================================================================

#[cfg(feature = "wizard")]
#[test]
fn test_init_to_active_transition() {
    // Arrange
    let session = WizardSession::<Init>::new("test".to_string());
    let session_id = session.session_id().to_string();

    // Act - transition to Active
    let session = session.start();

    // Assert - session_id should be preserved
    assert_eq!(session.session_id(), session_id);
}

#[cfg(feature = "wizard")]
#[test]
fn test_active_to_complete_transition() {
    // Arrange
    let session = WizardSession::<Init>::new("test".to_string());
    let mut session = session.start();

    // Add some interactions
    session.add_interaction("prompt1".to_string(), "response1".to_string());

    // Act - transition to Complete
    let session = session.complete();

    // Assert - history should be preserved
    assert_eq!(session.history().len(), 1);
}

#[cfg(feature = "wizard")]
#[test]
fn test_active_to_paused_transition() {
    // Arrange
    let session = WizardSession::<Init>::new("test".to_string());
    let session = session.start();

    // Act - transition to Paused
    let session = session.pause();

    // Assert
    assert_eq!(session.session_id(), "test");
}

#[cfg(feature = "wizard")]
#[test]
fn test_paused_to_active_transition() {
    // Arrange
    let session = WizardSession::<Init>::new("test".to_string());
    let session = session.start();
    let session = session.pause();

    // Act - resume (Paused -> Active)
    let session = session.resume();

    // Assert
    assert_eq!(session.session_id(), "test");
}

#[cfg(feature = "wizard")]
#[test]
fn test_active_to_failed_transition() {
    // Arrange
    let session = WizardSession::<Init>::new("test".to_string());
    let session = session.start();

    // Act - transition to Failed
    let session = session.fail();

    // Assert
    assert_eq!(session.session_id(), "test");
}

#[cfg(feature = "wizard")]
#[test]
fn test_paused_to_failed_transition() {
    // Arrange
    let session = WizardSession::<Init>::new("test".to_string());
    let session = session.start();
    let session = session.pause();

    // Act - transition to Failed from Paused
    let session = session.fail();

    // Assert
    assert_eq!(session.session_id(), "test");
}

// =============================================================================
// Session Lifecycle Tests - Test complete workflows
// =============================================================================

#[cfg(feature = "wizard")]
#[test]
fn test_complete_session_lifecycle_success() {
    // Arrange - create session
    let session = WizardSession::<Init>::new("workflow-test".to_string());
    assert_eq!(session.session_id(), "workflow-test");

    // Act - start session
    let mut session = session.start();
    session.add_interaction("Hello".to_string(), "Hi there!".to_string());

    // Act - complete session
    let session = session.complete();

    // Assert - history preserved
    assert_eq!(session.history().len(), 1);
}

#[cfg(feature = "wizard")]
#[test]
fn test_session_lifecycle_with_pause_resume() {
    // Arrange - create and start session
    let session = WizardSession::<Init>::new("pause-test".to_string());
    let mut session = session.start();

    // Act - add interaction, then pause
    session.add_interaction("prompt1".to_string(), "response1".to_string());
    let session = session.pause();

    // Act - resume and add more interactions
    let mut session = session.resume();
    session.add_interaction("prompt2".to_string(), "response2".to_string());

    // Act - complete session
    let session = session.complete();

    // Assert - all interactions preserved
    assert_eq!(session.history().len(), 2);
}

#[cfg(feature = "wizard")]
#[test]
fn test_session_lifecycle_failure() {
    // Arrange
    let session = WizardSession::<Init>::new("fail-test".to_string());
    let mut session = session.start();

    // Act - add some interactions before failure
    session.add_interaction("prompt".to_string(), "response".to_string());

    // Act - fail the session
    let session = session.fail();

    // Assert - data preserved even on failure
    assert_eq!(session.history().len(), 1);
}

// =============================================================================
// Session Builder Tests - Test builder pattern
// =============================================================================

#[cfg(feature = "wizard")]
#[test]
fn test_session_builder_success() {
    // Arrange + Act
    let session = SessionBuilder::new()
        .session_id("builder-test".to_string())
        .metadata(json!({"key": "value"}))
        .build();

    // Assert
    assert!(session.is_ok());
    let session = session.unwrap();
    assert_eq!(session.session_id(), "builder-test");
    assert_eq!(session.metadata(), &json!({"key": "value"}));
}

#[cfg(feature = "wizard")]
#[test]
fn test_session_builder_missing_session_id() {
    // Arrange + Act
    let result = SessionBuilder::new().metadata(json!({"key": "value"})).build();

    // Assert - should fail without session_id
    assert!(result.is_err());
}

#[cfg(feature = "wizard")]
#[test]
fn test_session_builder_default() {
    // Arrange + Act
    let builder = SessionBuilder::default();

    // Assert - default should work (but build will fail without session_id)
    let result = builder.build();
    assert!(result.is_err());
}

#[cfg(feature = "wizard")]
#[test]
fn test_session_builder_with_empty_metadata() {
    // Arrange + Act
    let session = SessionBuilder::new().session_id("test".to_string()).build();

    // Assert
    assert!(session.is_ok());
    let session = session.unwrap();
    assert_eq!(session.metadata(), &serde_json::Value::Null);
}

// =============================================================================
// Active Session Tests - Test interaction management
// =============================================================================

#[cfg(feature = "wizard")]
#[test]
fn test_active_session_add_multiple_interactions() {
    // Arrange
    let session = WizardSession::<Init>::new("multi-test".to_string());
    let mut session = session.start();

    // Act - add multiple interactions
    for i in 0..5 {
        session.add_interaction(format!("Question {}", i), format!("Answer {}", i));
    }

    // Assert
    assert_eq!(session.history().len(), 5);
}

#[cfg(feature = "wizard")]
#[test]
fn test_active_session_empty_interactions() {
    // Arrange
    let session = WizardSession::<Init>::new("empty-test".to_string());
    let mut session = session.start();

    // Act - add interaction with empty strings
    session.add_interaction(String::new(), String::new());

    // Assert - should allow empty strings
    assert_eq!(session.history().len(), 1);
}

#[cfg(feature = "wizard")]
#[test]
fn test_active_session_large_interaction() {
    // Arrange
    let session = WizardSession::<Init>::new("large-test".to_string());
    let mut session = session.start();

    // Act - add very large interaction
    let large_prompt = "x".repeat(100_000);
    let large_response = "y".repeat(100_000);
    session.add_interaction(large_prompt.clone(), large_response.clone());

    // Assert
    assert_eq!(session.history().len(), 1);
    assert_eq!(session.history()[0].0.len(), 100_000);
}

// =============================================================================
// Compile-Time Safety Tests - Document type-safe transitions
// =============================================================================

#[cfg(feature = "wizard")]
#[test]
fn test_compile_time_safety_init_cannot_pause() {
    // This test documents that certain transitions are impossible at compile time
    // For example, Init state cannot transition to Paused

    // Arrange
    let session = WizardSession::<Init>::new("test".to_string());

    // The following would not compile (documenting the type safety):
    // let session = session.pause(); // ERROR: no method `pause` on WizardSession<Init>

    // Assert - can only call start()
    let _session = session.start();
    // This is a compile-time guarantee - no runtime check needed
}

#[cfg(feature = "wizard")]
#[test]
fn test_compile_time_safety_complete_is_terminal() {
    // Arrange
    let session = WizardSession::<Init>::new("test".to_string());
    let session = session.start();
    let session = session.complete();

    // The following would not compile:
    // let session = session.start(); // ERROR: no method `start` on WizardSession<Complete>
    // let session = session.pause(); // ERROR: no method `pause` on WizardSession<Complete>

    // Assert - Complete is terminal (documented by lack of transition methods)
    assert_eq!(session.session_id(), "test");
}

#[cfg(feature = "wizard")]
#[test]
fn test_compile_time_safety_failed_is_terminal() {
    // Arrange
    let session = WizardSession::<Init>::new("test".to_string());
    let session = session.start();
    let session = session.fail();

    // The following would not compile:
    // let session = session.resume(); // ERROR: no method `resume` on WizardSession<Failed>

    // Assert - Failed is terminal
    assert_eq!(session.session_id(), "test");
}

// =============================================================================
// SessionData Serialization Tests
// =============================================================================

#[cfg(feature = "wizard")]
#[test]
fn test_session_data_serialization() {
    // Arrange
    let mut data = SessionData::new("test".to_string());
    data.add_interaction("prompt".to_string(), "response".to_string());
    data.metadata = json!({"key": "value"});

    // Act
    let json = serde_json::to_string(&data);

    // Assert
    assert!(json.is_ok());
    let json_str = json.unwrap();
    assert!(json_str.contains("session_id"));
    assert!(json_str.contains("history"));
    assert!(json_str.contains("metadata"));
}

#[cfg(feature = "wizard")]
#[test]
fn test_session_data_deserialization() {
    // Arrange
    let json = r#"{
        "session_id": "test",
        "history": [["prompt", "response"]],
        "metadata": {"key": "value"}
    }"#;

    // Act
    let data: Result<SessionData, _> = serde_json::from_str(json);

    // Assert
    assert!(data.is_ok());
    let data = data.unwrap();
    assert_eq!(data.session_id, "test");
    assert_eq!(data.history.len(), 1);
}

// =============================================================================
// Edge Cases and Boundary Tests
// =============================================================================

#[cfg(feature = "wizard")]
#[test]
fn test_session_with_unicode_characters() {
    // Arrange
    let session = WizardSession::<Init>::new("session-🦀".to_string());

    // Act
    let session_id = session.session_id();

    // Assert
    assert_eq!(session_id, "session-🦀");
}

#[cfg(feature = "wizard")]
#[test]
fn test_session_with_very_long_id() {
    // Arrange
    let long_id = "x".repeat(10_000);

    // Act
    let session = WizardSession::<Init>::new(long_id.clone());

    // Assert
    assert_eq!(session.session_id(), long_id);
}

#[cfg(feature = "wizard")]
#[test]
fn test_session_with_zero_capacity() {
    // Arrange + Act
    let session = WizardSession::<Init>::with_capacity("test".to_string(), 0);

    // Assert - should work with zero capacity (will reallocate as needed)
    assert_eq!(session.session_id(), "test");
}

#[cfg(feature = "wizard")]
#[test]
fn test_session_metadata_overwrite() {
    // Arrange
    let mut session = WizardSession::<Init>::new("test".to_string());

    // Act - set metadata multiple times
    session.set_metadata(json!({"version": 1}));
    session.set_metadata(json!({"version": 2}));

    // Assert - latest metadata wins
    assert_eq!(session.metadata(), &json!({"version": 2}));
}

// =============================================================================
// Zero-Cost Abstraction Tests - Document performance characteristics
// =============================================================================

#[cfg(feature = "wizard")]
#[test]
fn test_state_transitions_are_zero_cost() {
    // This test documents that state transitions use PhantomData
    // and should compile to no-ops (zero runtime cost)

    // Arrange
    let session = WizardSession::<Init>::new("test".to_string());

    // Act - multiple zero-cost transitions
    let session = session.start();
    let session = session.pause();
    let session = session.resume();
    let session = session.complete();

    // Assert - all data preserved through transitions
    assert_eq!(session.session_id(), "test");

    // Note: The actual zero-cost nature would be verified through
    // assembly inspection or benchmarks, not runtime tests
}
