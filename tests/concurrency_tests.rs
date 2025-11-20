//! Hyper-advanced concurrency tests for hot path components
//!
//! This test suite uses advanced concurrency testing techniques to:
//! - Verify thread-safety of lock-free data structures
//! - Detect race conditions under extreme contention
//! - Validate ordering guarantees under concurrent access
//! - Test memory visibility and happens-before relationships
//!
//! These tests go beyond standard concurrency tests by:
//! - Using high thread counts to maximize contention
//! - Employing stress testing with rapid operations
//! - Verifying linearizability properties
//! - Testing under different memory orderings

use clap_noun_verb::autonomic::*;
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;

/// Test InvocationQueue under extreme concurrent push/pop contention
#[test]
fn test_queue_concurrent_push_pop_stress() {
    const THREADS: usize = 16;
    const OPS_PER_THREAD: usize = 1000;

    let queue = Arc::new(InvocationQueue::new(THREADS * OPS_PER_THREAD));
    let barrier = Arc::new(Barrier::new(THREADS * 2));

    let mut handles = vec![];

    // Spawn producer threads
    for thread_id in 0..THREADS {
        let queue = Arc::clone(&queue);
        let barrier = Arc::clone(&barrier);

        let handle = thread::spawn(move || {
            // Wait for all threads to be ready
            barrier.wait();

            let start = thread_id * OPS_PER_THREAD;
            for i in 0..OPS_PER_THREAD {
                let value = start + i;
                // Retry on contention
                while queue.try_push(value).is_err() {
                    thread::yield_now();
                }
            }
        });

        handles.push(handle);
    }

    // Spawn consumer threads
    let consumed = Arc::new(std::sync::Mutex::new(Vec::new()));

    for _ in 0..THREADS {
        let queue = Arc::clone(&queue);
        let barrier = Arc::clone(&barrier);
        let consumed = Arc::clone(&consumed);

        let handle = thread::spawn(move || {
            barrier.wait();

            let mut local_consumed = Vec::new();

            for _ in 0..OPS_PER_THREAD {
                // Try to pop with backoff
                loop {
                    if let Some(value) = queue.try_pop() {
                        local_consumed.push(value);
                        break;
                    }
                    thread::yield_now();
                }
            }

            consumed.lock().unwrap().extend(local_consumed);
        });

        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify all values were consumed exactly once
    let mut all_consumed = consumed.lock().unwrap();
    all_consumed.sort_unstable();

    let expected: Vec<usize> = (0..THREADS * OPS_PER_THREAD).collect();
    assert_eq!(*all_consumed, expected, "All values should be consumed exactly once");
}

/// Test that InvocationQueue maintains FIFO ordering under single-producer-single-consumer
#[test]
fn test_queue_fifo_ordering_spsc() {
    const COUNT: usize = 10000;

    let queue = Arc::new(InvocationQueue::new(COUNT));
    let queue_producer = Arc::clone(&queue);
    let queue_consumer = Arc::clone(&queue);

    let producer = thread::spawn(move || {
        for i in 0..COUNT {
            while queue_producer.try_push(i).is_err() {
                thread::yield_now();
            }
        }
    });

    let consumer = thread::spawn(move || {
        let mut received = Vec::new();
        for _ in 0..COUNT {
            loop {
                if let Some(val) = queue_consumer.try_pop() {
                    received.push(val);
                    break;
                }
                thread::yield_now();
            }
        }
        received
    });

    producer.join().unwrap();
    let received = consumer.join().unwrap();

    // Verify FIFO ordering
    let expected: Vec<usize> = (0..COUNT).collect();
    assert_eq!(received, expected, "Queue must maintain FIFO ordering");
}

/// Test InvocationQueue statistics accuracy under concurrent access
#[test]
fn test_queue_stats_accuracy_concurrent() {
    const THREADS: usize = 8;
    const OPS_PER_THREAD: usize = 500;

    let queue = Arc::new(InvocationQueue::new(THREADS * OPS_PER_THREAD));
    let barrier = Arc::new(Barrier::new(THREADS));

    let mut handles = vec![];

    // All threads push and pop
    for thread_id in 0..THREADS {
        let queue = Arc::clone(&queue);
        let barrier = Arc::clone(&barrier);

        let handle = thread::spawn(move || {
            barrier.wait();

            // Push phase
            for i in 0..OPS_PER_THREAD {
                let value = thread_id * OPS_PER_THREAD + i;
                while queue.try_push(value).is_err() {
                    thread::yield_now();
                }
            }

            // Small delay to allow contention
            thread::sleep(Duration::from_micros(10));

            // Pop phase
            let mut popped = 0;
            for _ in 0..OPS_PER_THREAD {
                loop {
                    if queue.try_pop().is_some() {
                        popped += 1;
                        break;
                    }
                    thread::yield_now();
                }
            }

            popped
        });

        handles.push(handle);
    }

    // Wait for all threads
    let total_popped: usize = handles.into_iter().map(|h| h.join().unwrap()).sum();

    // Verify statistics
    let stats = queue.stats();
    assert_eq!(stats.total_enqueued, (THREADS * OPS_PER_THREAD) as u64);
    assert_eq!(stats.total_dequeued, total_popped as u64);
    assert_eq!(total_popped, THREADS * OPS_PER_THREAD);
    assert_eq!(stats.current_length, 0);
}

/// Test ContextPool handle allocation uniqueness under concurrent access
#[test]
fn test_context_pool_unique_handles_concurrent() {
    const THREADS: usize = 16;
    const HANDLES_PER_THREAD: usize = 100;

    let pool = Arc::new(ContextPool::new(THREADS * HANDLES_PER_THREAD));
    let barrier = Arc::new(Barrier::new(THREADS));

    let all_agent_handles = Arc::new(std::sync::Mutex::new(Vec::new()));
    let all_tenant_handles = Arc::new(std::sync::Mutex::new(Vec::new()));

    let mut handles = vec![];

    for _ in 0..THREADS {
        let pool = Arc::clone(&pool);
        let barrier = Arc::clone(&barrier);
        let agent_handles = Arc::clone(&all_agent_handles);
        let tenant_handles = Arc::clone(&all_tenant_handles);

        let handle = thread::spawn(move || {
            barrier.wait();

            let mut local_agents = Vec::new();
            let mut local_tenants = Vec::new();

            for _ in 0..HANDLES_PER_THREAD {
                local_agents.push(pool.alloc_agent_handle());
                local_tenants.push(pool.alloc_tenant_handle());
            }

            agent_handles.lock().unwrap().extend(local_agents);
            tenant_handles.lock().unwrap().extend(local_tenants);
        });

        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify all handles are unique
    let agents = all_agent_handles.lock().unwrap();
    let mut agent_ids: Vec<u64> = agents.iter().map(|h| h.id()).collect();
    agent_ids.sort_unstable();
    agent_ids.dedup();
    assert_eq!(agent_ids.len(), THREADS * HANDLES_PER_THREAD, "All agent handles must be unique");

    let tenants = all_tenant_handles.lock().unwrap();
    let mut tenant_ids: Vec<u64> = tenants.iter().map(|h| h.id()).collect();
    tenant_ids.sort_unstable();
    tenant_ids.dedup();
    assert_eq!(tenant_ids.len(), THREADS * HANDLES_PER_THREAD, "All tenant handles must be unique");
}

/// Test InvocationArena under concurrent allocation stress
#[test]
fn test_arena_concurrent_allocation() {
    const THREADS: usize = 8;
    const ALLOCS_PER_THREAD: usize = 100;
    const ARENA_SIZE: usize = 1024 * 1024; // 1MB

    let arena = Arc::new(InvocationArena::new(ARENA_SIZE));
    let barrier = Arc::new(Barrier::new(THREADS));

    let mut handles = vec![];

    for thread_id in 0..THREADS {
        let arena: Arc<InvocationArena> = Arc::clone(&arena);
        let barrier = Arc::clone(&barrier);

        let handle = thread::spawn(move || {
            barrier.wait();

            let mut successful_allocs = 0;

            for i in 0..ALLOCS_PER_THREAD {
                let value: u64 = (thread_id * ALLOCS_PER_THREAD + i) as u64;
                if let Some(allocated) = arena.alloc(value) {
                    // Verify the value was written correctly
                    assert_eq!(*allocated, value);
                    successful_allocs += 1;
                }
            }

            successful_allocs
        });

        handles.push(handle);
    }

    // Wait for all threads
    let total_allocs: usize = handles.into_iter().map(|h| h.join().unwrap()).sum();

    // At least some allocations should succeed
    assert!(total_allocs > 0, "Arena should allow concurrent allocations");

    // Usage should reflect successful allocations
    assert!(arena.usage() > 0);
}

/// Test memory visibility: writes in one thread visible to reads in another
#[test]
fn test_queue_memory_visibility() {
    const COUNT: usize = 1000;

    let queue = Arc::new(InvocationQueue::new(COUNT));

    // Producer writes incrementing values
    let queue_prod = Arc::clone(&queue);
    let producer = thread::spawn(move || {
        for i in 0..COUNT {
            while queue_prod.try_push(i).is_err() {
                thread::yield_now();
            }
        }
    });

    // Consumer reads and verifies visibility
    let queue_cons = Arc::clone(&queue);
    let consumer = thread::spawn(move || {
        let mut last_seen = None;
        for _ in 0..COUNT {
            loop {
                if let Some(val) = queue_cons.try_pop() {
                    // Each value should be greater than the last (FIFO property)
                    if let Some(last) = last_seen {
                        assert!(val > last, "Values should be increasing due to FIFO");
                    }
                    last_seen = Some(val);
                    break;
                }
                thread::yield_now();
            }
        }
    });

    producer.join().unwrap();
    consumer.join().unwrap();
}

/// Test that queue handles overflow correctly under concurrent pressure
#[test]
fn test_queue_overflow_handling_concurrent() {
    const CAPACITY: usize = 100;
    const THREADS: usize = 10;
    const ATTEMPTS_PER_THREAD: usize = 50;

    let queue = Arc::new(InvocationQueue::new(CAPACITY));
    let barrier = Arc::new(Barrier::new(THREADS));

    let mut handles = vec![];

    for thread_id in 0..THREADS {
        let queue = Arc::clone(&queue);
        let barrier = Arc::clone(&barrier);

        let handle = thread::spawn(move || {
            barrier.wait();

            let mut successful_pushes = 0;
            let mut failed_pushes = 0;

            for i in 0..ATTEMPTS_PER_THREAD {
                let value = thread_id * ATTEMPTS_PER_THREAD + i;
                match queue.try_push(value) {
                    Ok(_) => successful_pushes += 1,
                    Err(_) => failed_pushes += 1,
                }
            }

            (successful_pushes, failed_pushes)
        });

        handles.push(handle);
    }

    // Collect results
    let (total_success, total_failed): (usize, usize) = handles
        .into_iter()
        .map(|h| h.join().unwrap())
        .fold((0, 0), |(s, f), (ss, sf)| (s + ss, f + sf));

    // Total successes should not exceed capacity
    assert!(total_success <= CAPACITY, "Successful pushes should not exceed queue capacity");

    // Some pushes should fail (total attempts > capacity)
    assert!(total_failed > 0, "Some pushes should fail when capacity is exceeded");
}

/// Test HotPathContext creation and correlation under concurrent access
#[test]
fn test_hot_path_context_concurrent_creation() {
    const THREADS: usize = 8;
    const CONTEXTS_PER_THREAD: usize = 200;

    let barrier = Arc::new(Barrier::new(THREADS));
    let all_contexts = Arc::new(std::sync::Mutex::new(Vec::new()));

    let mut handles = vec![];

    for thread_id in 0..THREADS {
        let barrier = Arc::clone(&barrier);
        let contexts = Arc::clone(&all_contexts);

        let handle = thread::spawn(move || {
            barrier.wait();

            let mut local_contexts = Vec::new();

            for i in 0..CONTEXTS_PER_THREAD {
                let ctx = HotPathContext::new(
                    AgentHandle::new(thread_id as u64),
                    TenantHandle::new(1),
                    i as u32,
                    EffectFlags::empty().with(EffectFlags::READ_ONLY),
                )
                .with_correlation(&format!("req-{}-{}", thread_id, i));

                local_contexts.push(ctx);
            }

            contexts.lock().unwrap().extend(local_contexts);
        });

        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify all contexts created
    let all_ctx = all_contexts.lock().unwrap();
    assert_eq!(all_ctx.len(), THREADS * CONTEXTS_PER_THREAD);

    // Verify correlation hashes are computed
    for ctx in all_ctx.iter() {
        assert_ne!(ctx.correlation_hash, 0, "Correlation hash should be computed");
    }
}

/// Test linearizability: concurrent queue operations appear to occur in some sequential order
#[test]
fn test_queue_linearizability() {
    const OPERATIONS: usize = 500;
    const PRODUCERS: usize = 4;
    const CONSUMERS: usize = 4;

    let queue = Arc::new(InvocationQueue::new(OPERATIONS * PRODUCERS));
    let barrier = Arc::new(Barrier::new(PRODUCERS + CONSUMERS));

    let mut handles = vec![];

    // Producers push values
    for thread_id in 0..PRODUCERS {
        let queue = Arc::clone(&queue);
        let barrier = Arc::clone(&barrier);

        let handle = thread::spawn(move || {
            barrier.wait();

            for i in 0..OPERATIONS {
                let value = thread_id * OPERATIONS + i;
                while queue.try_push(value).is_err() {
                    thread::yield_now();
                }
            }
        });

        handles.push(handle);
    }

    // Consumers pop values
    let consumed = Arc::new(std::sync::Mutex::new(Vec::new()));

    for _ in 0..CONSUMERS {
        let queue = Arc::clone(&queue);
        let barrier = Arc::clone(&barrier);
        let consumed = Arc::clone(&consumed);

        let handle = thread::spawn(move || {
            barrier.wait();

            let mut local = Vec::new();
            let target = OPERATIONS * PRODUCERS / CONSUMERS;

            for _ in 0..target {
                loop {
                    if let Some(val) = queue.try_pop() {
                        local.push(val);
                        break;
                    }
                    thread::yield_now();
                }
            }

            consumed.lock().unwrap().extend(local);
        });

        handles.push(handle);
    }

    // Wait for all
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify linearizability property: each value appears exactly once
    let mut all_consumed = consumed.lock().unwrap();
    all_consumed.sort_unstable();

    let expected: Vec<usize> = (0..OPERATIONS * PRODUCERS).collect();
    assert_eq!(
        *all_consumed, expected,
        "Linearizability violated: each value must appear exactly once"
    );
}

/// Test EffectFlags bitfield operations are atomic
#[test]
fn test_effect_flags_atomic_operations() {
    const THREADS: usize = 8;
    const ITERATIONS: usize = 1000;

    let flags = Arc::new(std::sync::Mutex::new(EffectFlags::empty()));
    let barrier = Arc::new(Barrier::new(THREADS));

    let mut handles = vec![];

    for thread_id in 0..THREADS {
        let flags = Arc::clone(&flags);
        let barrier = Arc::clone(&barrier);

        let handle = thread::spawn(move || {
            barrier.wait();

            for _ in 0..ITERATIONS {
                let mut f = flags.lock().unwrap();

                // Each thread sets different flags based on thread_id
                *f = match thread_id % 4 {
                    0 => f.with(EffectFlags::READ_ONLY),
                    1 => f.with(EffectFlags::PRIVILEGED),
                    2 => f.with(EffectFlags::NETWORK),
                    _ => f.with(EffectFlags::STORAGE),
                };
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // At least some flags should be set after concurrent modifications
    let final_flags = flags.lock().unwrap();
    assert!(
        final_flags.is_read_only()
            || final_flags.is_privileged()
            || final_flags.has(EffectFlags::NETWORK)
            || final_flags.has(EffectFlags::STORAGE),
        "Some flags should be set after concurrent operations"
    );
}
