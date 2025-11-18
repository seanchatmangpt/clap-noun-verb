//! AppContext feature tests
//!
//! Validates the AppContext state management system including:
//! - State isolation between different types
//! - Concurrent access patterns and thread-safety
//! - Data sharing between verbs in CLI execution
//! - Integration with actual CLI execution flow

use clap_noun_verb::context::{AppContext, ContextError};
use std::sync::{Arc, Barrier};
use std::thread;

/// Validates that AppContext correctly isolates state by type
///
/// This test ensures that:
/// - Different types can be stored independently
/// - Type retrieval is type-safe
/// - Values don't interfere with each other
#[test]
fn test_state_isolation_between_types() {
    let ctx = AppContext::new();

    // Insert multiple different types
    ctx.insert(42_i32).expect("Failed to insert i32");
    ctx.insert("hello world".to_string()).expect("Failed to insert String");
    ctx.insert(3.14_f64).expect("Failed to insert f64");
    ctx.insert(true).expect("Failed to insert bool");

    // Verify each type can be retrieved correctly
    let int_val: i32 = ctx.get().expect("Failed to get i32");
    assert_eq!(int_val, 42, "i32 value should be isolated and correct");

    let str_val: String = ctx.get().expect("Failed to get String");
    assert_eq!(str_val, "hello world", "String value should be isolated and correct");

    let float_val: f64 = ctx.get().expect("Failed to get f64");
    assert!((float_val - 3.14).abs() < f64::EPSILON, "f64 value should be isolated and correct");

    let bool_val: bool = ctx.get().expect("Failed to get bool");
    assert!(bool_val, "bool value should be isolated and correct");

    // Verify count
    assert_eq!(ctx.len().expect("Failed to get len"), 4, "Should have 4 distinct types stored");
}

/// Validates concurrent read access patterns from multiple threads
///
/// This test ensures that:
/// - Multiple threads can read simultaneously
/// - Values remain consistent across threads
/// - No data races occur during concurrent reads
#[test]
fn test_concurrent_read_access() {
    const NUM_THREADS: usize = 10;
    const EXPECTED_VALUE: i32 = 12345;

    let ctx = Arc::new(AppContext::new());
    ctx.insert(EXPECTED_VALUE).expect("Failed to insert value");

    let barrier = Arc::new(Barrier::new(NUM_THREADS));
    let mut handles = vec![];

    for _ in 0..NUM_THREADS {
        let ctx_clone = Arc::clone(&ctx);
        let barrier_clone = Arc::clone(&barrier);

        let handle = thread::spawn(move || {
            // Synchronize all threads to start at the same time
            barrier_clone.wait();

            // Each thread reads the value multiple times
            for _ in 0..100 {
                let value: i32 = ctx_clone.get().expect("Failed to read value");
                assert_eq!(value, EXPECTED_VALUE, "Value should be consistent across threads");
            }
        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().expect("Thread panicked");
    }
}

/// Validates concurrent write and read access patterns
///
/// This test ensures that:
/// - Multiple threads can write different types concurrently
/// - Writes don't corrupt other types' data
/// - Thread-safe locking works correctly
#[test]
fn test_concurrent_write_different_types() {
    const NUM_THREADS: usize = 8;

    let ctx = Arc::new(AppContext::new());
    let barrier = Arc::new(Barrier::new(NUM_THREADS));
    let mut handles = vec![];

    // Each thread writes a different type
    for thread_id in 0..NUM_THREADS {
        let ctx_clone = Arc::clone(&ctx);
        let barrier_clone = Arc::clone(&barrier);

        let handle = thread::spawn(move || {
            barrier_clone.wait();

            match thread_id % 4 {
                0 => {
                    ctx_clone.insert(thread_id as i32).expect("Failed to insert i32");
                }
                1 => {
                    ctx_clone.insert(format!("thread-{}", thread_id)).expect("Failed to insert String");
                }
                2 => {
                    ctx_clone.insert(thread_id as f64).expect("Failed to insert f64");
                }
                3 => {
                    ctx_clone.insert(thread_id % 2 == 0).expect("Failed to insert bool");
                }
                _ => unreachable!(),
            }

            // Give time for other threads to write
            thread::yield_now();

            // Verify that the value we wrote is still correct
            match thread_id % 4 {
                0 => {
                    let val: i32 = ctx_clone.get().expect("Failed to get i32");
                    // Note: Multiple threads write to i32, so we just verify we can read it
                    assert!(val >= 0, "i32 value should be valid");
                }
                1 => {
                    let val: String = ctx_clone.get().expect("Failed to get String");
                    assert!(val.starts_with("thread-"), "String value should have correct format");
                }
                2 => {
                    let val: f64 = ctx_clone.get().expect("Failed to get f64");
                    assert!(val >= 0.0, "f64 value should be valid");
                }
                3 => {
                    let _val: bool = ctx_clone.get().expect("Failed to get bool");
                }
                _ => unreachable!(),
            }
        });

        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    // Verify all types are present
    assert!(ctx.contains::<i32>().expect("Failed to check i32"), "Should contain i32");
    assert!(ctx.contains::<String>().expect("Failed to check String"), "Should contain String");
    assert!(ctx.contains::<f64>().expect("Failed to check f64"), "Should contain f64");
    assert!(ctx.contains::<bool>().expect("Failed to check bool"), "Should contain bool");
}

/// Validates data sharing between verbs in a simulated CLI execution
///
/// This test ensures that:
/// - Context can be used to share state between command handlers
/// - Data persists across multiple operations
/// - Integration pattern works as documented
#[test]
fn test_data_sharing_between_verbs() {
    #[derive(Clone, Debug, PartialEq)]
    struct AppState {
        counter: i32,
        last_operation: String,
    }

    let ctx = Arc::new(AppContext::new());

    // Simulate first verb: initialize state
    {
        let state = AppState { counter: 0, last_operation: "init".to_string() };
        ctx.insert(state).expect("Failed to initialize state");
    }

    // Simulate second verb: increment counter
    {
        let mut state: AppState = ctx.get().expect("Failed to get state");
        state.counter += 1;
        state.last_operation = "increment".to_string();
        ctx.insert(state).expect("Failed to update state");
    }

    // Simulate third verb: read and validate state
    {
        let state: AppState = ctx.get().expect("Failed to get state");
        assert_eq!(state.counter, 1, "Counter should have been incremented");
        assert_eq!(state.last_operation, "increment", "Last operation should be tracked");
    }

    // Simulate fourth verb: multiple increments
    {
        for _ in 0..5 {
            let mut state: AppState = ctx.get().expect("Failed to get state");
            state.counter += 1;
            state.last_operation = "batch_increment".to_string();
            ctx.insert(state).expect("Failed to update state");
        }
    }

    // Final verification
    {
        let state: AppState = ctx.get().expect("Failed to get final state");
        assert_eq!(state.counter, 6, "Counter should reflect all increments");
        assert_eq!(state.last_operation, "batch_increment", "Last operation should be latest");
    }
}

/// Validates the with() closure-based access pattern
///
/// This test ensures that:
/// - Closure-based access works correctly
/// - Return values are properly propagated
/// - Type inference works as expected
#[test]
fn test_closure_based_access() {
    let ctx = AppContext::new();

    ctx.insert("test data".to_string()).expect("Failed to insert string");

    // Use with() to transform the value
    let result = ctx
        .with::<String, _, _>(|s| format!("Result: {}", s))
        .expect("Failed to execute closure");

    assert_eq!(result, "Result: test data", "Closure should transform value correctly");

    // Use with() for computation
    ctx.insert(vec![1, 2, 3, 4, 5]).expect("Failed to insert vec");

    let sum = ctx.with::<Vec<i32>, _, _>(|v| v.iter().sum::<i32>()).expect("Failed to compute sum");

    assert_eq!(sum, 15, "Closure should compute sum correctly");
}

/// Validates error handling for missing types
///
/// This test ensures that:
/// - Attempting to get a non-existent type returns appropriate error
/// - Error messages are informative
/// - Error type is correct
#[test]
fn test_error_handling_missing_type() {
    let ctx = AppContext::new();

    // Try to get a type that was never inserted
    let result: Result<String, ContextError> = ctx.get();

    assert!(result.is_err(), "Should return error for missing type");

    match result.unwrap_err() {
        ContextError::TypeNotFound(type_name) => {
            assert!(
                type_name.contains("String") || type_name.contains("string"),
                "Error should mention the type name: {}",
                type_name
            );
        }
        ContextError::PoisonedLock => {
            panic!("Should not get PoisonedLock error for missing type");
        }
    }
}

/// Validates remove functionality
///
/// This test ensures that:
/// - Values can be removed from context
/// - Removal returns the correct value
/// - Removed values are no longer accessible
#[test]
fn test_remove_functionality() {
    let ctx = AppContext::new();

    // Insert multiple values
    ctx.insert(42_i32).expect("Failed to insert i32");
    ctx.insert("test".to_string()).expect("Failed to insert String");

    assert_eq!(ctx.len().expect("Failed to get len"), 2, "Should have 2 values");

    // Remove i32
    let removed: Option<i32> = ctx.remove().expect("Failed to remove i32");
    assert_eq!(removed, Some(42), "Should return removed value");

    // Verify i32 is gone
    assert!(!ctx.contains::<i32>().expect("Failed to check i32"), "i32 should be removed");
    assert_eq!(ctx.len().expect("Failed to get len"), 1, "Should have 1 value left");

    // String should still be present
    let str_val: String = ctx.get().expect("Failed to get String");
    assert_eq!(str_val, "test", "String should still be accessible");
}

/// Validates clear functionality
///
/// This test ensures that:
/// - Clear removes all values
/// - Context is empty after clear
/// - Length is correctly updated
#[test]
fn test_clear_functionality() {
    let ctx = AppContext::new();

    // Insert multiple values
    ctx.insert(42_i32).expect("Failed to insert i32");
    ctx.insert("test".to_string()).expect("Failed to insert String");
    ctx.insert(3.14_f64).expect("Failed to insert f64");

    assert_eq!(ctx.len().expect("Failed to get len"), 3, "Should have 3 values");
    assert!(!ctx.is_empty().expect("Failed to check empty"), "Should not be empty");

    // Clear all values
    ctx.clear().expect("Failed to clear context");

    assert_eq!(ctx.len().expect("Failed to get len"), 0, "Should have 0 values after clear");
    assert!(ctx.is_empty().expect("Failed to check empty"), "Should be empty after clear");

    // Verify all types are gone
    assert!(!ctx.contains::<i32>().expect("Failed to check i32"), "i32 should be gone");
    assert!(!ctx.contains::<String>().expect("Failed to check String"), "String should be gone");
    assert!(!ctx.contains::<f64>().expect("Failed to check f64"), "f64 should be gone");
}

/// Validates Clone implementation
///
/// This test ensures that:
/// - AppContext can be cloned
/// - Clones share the same underlying state
/// - Changes in one clone affect the other
#[test]
fn test_clone_shares_state() {
    let ctx1 = AppContext::new();
    ctx1.insert(100_i32).expect("Failed to insert value");

    // Clone the context
    let ctx2 = ctx1.clone();

    // Both should see the value
    let val1: i32 = ctx1.get().expect("Failed to get from ctx1");
    let val2: i32 = ctx2.get().expect("Failed to get from ctx2");

    assert_eq!(val1, 100, "Original context should have value");
    assert_eq!(val2, 100, "Cloned context should have value");

    // Modify through ctx2
    ctx2.insert(200_i32).expect("Failed to update value");

    // Both should see the update (shared Arc)
    let val1_updated: i32 = ctx1.get().expect("Failed to get updated value from ctx1");
    let val2_updated: i32 = ctx2.get().expect("Failed to get updated value from ctx2");

    assert_eq!(val1_updated, 200, "Original context should see update");
    assert_eq!(val2_updated, 200, "Cloned context should see update");
}
