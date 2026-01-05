#![cfg(feature = "autonomic")]

//! Comprehensive tests for zero-allocation hot path
//!
//! Critical 80/20 test coverage:
//! - Compact handle creation and usage
//! - Effect flags bitfield operations
//! - Arena allocation and reset
//! - Lock-free queue operations
//! - Zero-copy parsing
//! - Performance metrics tracking

use clap_noun_verb::autonomic::*;
use std::sync::Arc;
use std::thread;

#[test]
fn test_agent_handle_creation() {
    // GIVEN: Agent handle creation
    let handle1 = AgentHandle::new(1);
    let handle2 = AgentHandle::new(2);

    // THEN: Handles have correct IDs
    assert_eq!(handle1.id(), 1);
    assert_eq!(handle2.id(), 2);

    // AND: Different handles are not equal
    assert_ne!(handle1, handle2);

    // AND: Same ID produces equal handles
    let handle1_copy = AgentHandle::new(1);
    assert_eq!(handle1, handle1_copy);
}

#[test]
fn test_tenant_handle_creation() {
    // GIVEN: Tenant handle creation
    let handle1 = TenantHandle::new(100);
    let handle2 = TenantHandle::new(200);

    // THEN: Handles have correct IDs
    assert_eq!(handle1.id(), 100);
    assert_eq!(handle2.id(), 200);

    // AND: Handles are comparable
    assert_ne!(handle1, handle2);
}

#[test]
fn test_effect_flags_bitfield_operations() {
    // GIVEN: Empty effect flags
    let mut flags = EffectFlags::empty();

    // THEN: Initially no flags are set
    assert!(!flags.is_read_only());
    assert!(!flags.is_privileged());
    assert!(!flags.has(EffectFlags::NETWORK));

    // WHEN: We add flags
    flags = flags.with(EffectFlags::READ_ONLY).with(EffectFlags::NETWORK);

    // THEN: Those flags are set
    assert!(flags.is_read_only());
    assert!(flags.has(EffectFlags::NETWORK));
    assert!(!flags.is_privileged());
}

#[test]
fn test_effect_flags_merge() {
    // GIVEN: Two flag sets
    let flags1 = EffectFlags::empty().with(EffectFlags::READ_ONLY).with(EffectFlags::NETWORK);

    let flags2 = EffectFlags::empty().with(EffectFlags::PRIVILEGED).with(EffectFlags::STORAGE);

    // WHEN: We merge them
    let merged = flags1.merge(flags2);

    // THEN: All flags are present
    assert!(merged.is_read_only());
    assert!(merged.has(EffectFlags::NETWORK));
    assert!(merged.is_privileged());
    assert!(merged.has(EffectFlags::STORAGE));
}

#[test]
fn test_flag_set_operations() {
    // GIVEN: An empty flag set
    let mut flags = FlagSet::empty();

    // WHEN: We set various flags
    flags.set(0);
    flags.set(5);
    flags.set(10);
    flags.set(63); // Max index

    // THEN: Those flags are set
    assert!(flags.is_set(0));
    assert!(flags.is_set(5));
    assert!(flags.is_set(10));
    assert!(flags.is_set(63));

    // AND: Other flags are not set
    assert!(!flags.is_set(1));
    assert!(!flags.is_set(4));
    assert!(!flags.is_set(62));

    // AND: Count is correct
    assert_eq!(flags.count(), 4);
}

#[test]
fn test_hot_path_context_creation() {
    // GIVEN: Components for hot path context
    let agent = AgentHandle::new(1);
    let tenant = TenantHandle::new(100);
    let capability_index = 42;
    let effect_flags = EffectFlags::empty().with(EffectFlags::READ_ONLY);

    // WHEN: We create a context
    let ctx = HotPathContext::new(agent, tenant, capability_index, effect_flags);

    // THEN: All fields are set
    assert_eq!(ctx.agent.id(), 1);
    assert_eq!(ctx.tenant.id(), 100);
    assert_eq!(ctx.capability_index, 42);
    assert!(ctx.effect_flags.is_read_only());
}

#[test]
fn test_hot_path_context_with_correlation() {
    // GIVEN: A context
    let ctx =
        HotPathContext::new(AgentHandle::new(1), TenantHandle::new(1), 0, EffectFlags::empty());

    // WHEN: We add a correlation ID
    let ctx = ctx.with_correlation("request-123");

    // THEN: Correlation hash is computed
    assert_ne!(ctx.correlation_hash, 0);

    // AND: Same correlation ID produces same hash
    let ctx2 =
        HotPathContext::new(AgentHandle::new(2), TenantHandle::new(2), 0, EffectFlags::empty())
            .with_correlation("request-123");

    assert_eq!(ctx.correlation_hash, ctx2.correlation_hash);
}

#[test]
fn test_invocation_arena_allocation() {
    // GIVEN: An arena with 1KB capacity
    let arena = InvocationArena::new(1024);

    // WHEN: We allocate values
    let val1 = arena.alloc(42u64).expect("Allocation should succeed");
    let val2 = arena.alloc(100u64).expect("Allocation should succeed");

    // THEN: Values are correctly stored
    assert_eq!(*val1, 42);
    assert_eq!(*val2, 100);

    // AND: Arena usage increased
    assert!(arena.usage() > 0);
    assert!(arena.utilization() > 0.0);
}

#[test]
fn test_invocation_arena_reset() {
    // GIVEN: An arena with allocations
    let mut arena = InvocationArena::new(1024);

    let _ = arena.alloc(42u64);
    let _ = arena.alloc(100u64);

    let usage_before = arena.usage();
    assert!(usage_before > 0);

    // WHEN: We reset the arena
    arena.reset();

    // THEN: Usage is zero
    assert_eq!(arena.usage(), 0);
    assert_eq!(arena.utilization(), 0.0);

    // AND: We can allocate again
    let val = arena.alloc(200u64).expect("Allocation should succeed");
    assert_eq!(*val, 200);
}

#[test]
fn test_invocation_arena_overflow() {
    // GIVEN: A small arena
    let arena = InvocationArena::new(16); // Only 16 bytes

    // WHEN: We allocate beyond capacity
    let _ = arena.alloc(1u64); // 8 bytes - OK
    let result = arena.alloc(2u64); // 8 more bytes - OK
    let overflow = arena.alloc(3u64); // Would exceed capacity

    // THEN: Final allocation fails
    assert!(overflow.is_none());
}

#[test]
fn test_invocation_queue_basic_operations() {
    // GIVEN: A queue with capacity 10
    let queue = InvocationQueue::new(10);

    // WHEN: We push items
    queue.try_push(1).expect("Push should succeed");
    queue.try_push(2).expect("Push should succeed");
    queue.try_push(3).expect("Push should succeed");

    // THEN: Length is correct
    assert_eq!(queue.len(), 3);
    assert!(!queue.is_empty());

    // AND: Total enqueued is tracked
    assert_eq!(queue.total_enqueued(), 3);

    // WHEN: We pop items
    assert_eq!(queue.try_pop(), Some(1));
    assert_eq!(queue.try_pop(), Some(2));

    // THEN: Queue length decreases
    assert_eq!(queue.len(), 1);
    assert_eq!(queue.total_dequeued(), 2);

    // AND: Last item is still there
    assert_eq!(queue.try_pop(), Some(3));
    assert!(queue.is_empty());
}

#[test]
fn test_invocation_queue_overflow() {
    // GIVEN: A small queue
    let queue = InvocationQueue::new(2);

    // WHEN: We fill it
    queue.try_push(1).expect("Push should succeed");
    queue.try_push(2).expect("Push should succeed");

    // THEN: Additional pushes fail
    let result = queue.try_push(3);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), 3); // Returns the value back
}

#[test]
fn test_invocation_queue_concurrent_access() {
    // GIVEN: A shared queue
    let queue = Arc::new(InvocationQueue::new(1000));

    // WHEN: Multiple threads push concurrently
    let mut handles = vec![];

    for i in 0..10 {
        let queue_clone = Arc::clone(&queue);
        let handle = thread::spawn(move || {
            for j in 0..100 {
                let value = i * 100 + j;
                queue_clone.try_push(value).ok();
            }
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // THEN: All items were enqueued (or queue filled)
    assert!(queue.total_enqueued() > 0);
    assert!(queue.total_enqueued() <= 1000);
}

#[test]
fn test_invocation_queue_stats() {
    // GIVEN: A queue with some activity
    let queue = InvocationQueue::new(100);

    queue.try_push(1).ok();
    queue.try_push(2).ok();
    queue.try_push(3).ok();

    queue.try_pop();

    // WHEN: We get stats
    let stats = queue.stats();

    // THEN: Stats are accurate
    assert_eq!(stats.current_length, 2);
    assert_eq!(stats.total_enqueued, 3);
    assert_eq!(stats.total_dequeued, 1);
    assert_eq!(stats.capacity, 100);
    assert_eq!(stats.throughput(), 1);
}

#[test]
fn test_context_pool_handle_allocation() {
    // GIVEN: A context pool
    let pool = ContextPool::new(10);

    // WHEN: We allocate handles
    let agent1 = pool.alloc_agent_handle();
    let agent2 = pool.alloc_agent_handle();
    let tenant1 = pool.alloc_tenant_handle();
    let tenant2 = pool.alloc_tenant_handle();

    // THEN: Each handle is unique
    assert_ne!(agent1.id(), agent2.id());
    assert_ne!(tenant1.id(), tenant2.id());

    // AND: IDs are sequential
    assert_eq!(agent2.id(), agent1.id() + 1);
    assert_eq!(tenant2.id(), tenant1.id() + 1);
}

#[test]
fn test_context_pool_acquire_release() {
    // GIVEN: A context pool
    let pool = ContextPool::new(5);

    let ctx =
        HotPathContext::new(AgentHandle::new(1), TenantHandle::new(1), 0, EffectFlags::empty());

    // WHEN: We release a context
    pool.release(ctx);

    // THEN: We can acquire it back
    let acquired = pool.try_acquire();
    assert!(acquired.is_some());

    // AND: Subsequent acquire fails (pool empty)
    assert!(pool.try_acquire().is_none());
}

#[test]
fn test_zero_copy_parser_basic() {
    // GIVEN: A command input
    let input = "user.create --name alice --age 30";
    let mut args_buffer = [("", ""); 10];
    let mut positional_buffer = [""; 10];

    // WHEN: We parse it
    let parsed = ZeroCopyParser::parse(input, &mut args_buffer, &mut positional_buffer)
        .expect("Parse should succeed");

    // THEN: Capability path is extracted
    assert_eq!(parsed.capability_path, "user.create");

    // AND: Named arguments are parsed
    assert_eq!(parsed.args.len(), 2);
    assert!(parsed.args.contains(&("name", "alice")));
    assert!(parsed.args.contains(&("age", "30")));
}

#[test]
fn test_zero_copy_parser_positional_args() {
    // GIVEN: A command with positional arguments
    let input = "file.copy source.txt dest.txt";
    let mut args_buffer = [("", ""); 10];
    let mut positional_buffer = [""; 10];

    // WHEN: We parse it
    let parsed = ZeroCopyParser::parse(input, &mut args_buffer, &mut positional_buffer)
        .expect("Parse should succeed");

    // THEN: Positional arguments are captured
    assert_eq!(parsed.positional.len(), 2);
    assert_eq!(parsed.positional[0], "source.txt");
    assert_eq!(parsed.positional[1], "dest.txt");
}

#[test]
fn test_zero_copy_parser_flags() {
    // GIVEN: A command with boolean flags
    let input = "git.commit -v";
    let mut args_buffer = [("", ""); 10];
    let mut positional_buffer = [""; 10];

    // WHEN: We parse it
    let parsed = ZeroCopyParser::parse(input, &mut args_buffer, &mut positional_buffer)
        .expect("Parse should succeed");

    // THEN: Flags are set
    assert!(parsed.flags.count() > 0);
}

#[test]
fn test_zero_copy_parser_mixed() {
    // GIVEN: A command with mixed argument types
    let input = "deploy.app myapp --region us-west --replicas 3 --force config.yaml";
    let mut args_buffer = [("", ""); 10];
    let mut positional_buffer = [""; 10];

    // WHEN: We parse it
    let parsed = ZeroCopyParser::parse(input, &mut args_buffer, &mut positional_buffer)
        .expect("Parse should succeed");

    // THEN: All argument types are handled
    assert_eq!(parsed.capability_path, "deploy.app");
    assert!(parsed.args.len() > 0);
    assert!(parsed.positional.len() > 0);
}

#[test]
fn test_zero_copy_parser_empty_input() {
    // GIVEN: Empty input
    let input = "";
    let mut args_buffer = [("", ""); 10];
    let mut positional_buffer = [""; 10];

    // WHEN: We parse it
    let result = ZeroCopyParser::parse(input, &mut args_buffer, &mut positional_buffer);

    // THEN: Parse fails
    assert!(result.is_err());
}

#[test]
fn test_zero_copy_parser_no_allocations() {
    // GIVEN: Input and buffers
    let input = "test.cmd --arg value pos1 pos2";
    let mut args_buffer = [("", ""); 10];
    let mut positional_buffer = [""; 10];

    // WHEN: We parse
    let parsed = ZeroCopyParser::parse(input, &mut args_buffer, &mut positional_buffer)
        .expect("Parse should succeed");

    // THEN: All slices point to original input (no allocations)
    // This is validated by the lifetimes - if any allocation happened,
    // the lifetimes wouldn't match
    assert!(input.contains(parsed.capability_path));
    for (key, value) in parsed.args {
        assert!(input.contains(key) || key.is_empty());
        assert!(input.contains(value) || value.is_empty());
    }
    for arg in parsed.positional {
        assert!(input.contains(arg) || arg.is_empty());
    }
}

#[test]
fn test_hot_path_metrics_calculation() {
    // GIVEN: Metrics with data
    let mut metrics = HotPathMetrics::default();

    metrics.invocations_processed = 1000;
    metrics.total_nanos = 100_000_000; // 100ms total
    metrics.cache_hits = 800;
    metrics.cache_misses = 200;

    // WHEN: We calculate derived metrics
    let avg_latency = metrics.avg_latency_nanos();
    let hit_rate = metrics.cache_hit_rate();
    let throughput = metrics.throughput_per_sec(1.0); // 1 second elapsed

    // THEN: Calculations are correct
    assert_eq!(avg_latency, 100_000.0); // 100μs average
    assert_eq!(hit_rate, 80.0); // 80% hit rate
    assert_eq!(throughput, 1000.0); // 1000 invocations/sec
}

#[test]
fn test_hot_path_metrics_edge_cases() {
    // GIVEN: Empty metrics
    let metrics = HotPathMetrics::default();

    // THEN: Calculations handle zero values
    assert_eq!(metrics.avg_latency_nanos(), 0.0);
    assert_eq!(metrics.cache_hit_rate(), 0.0);
    assert_eq!(metrics.throughput_per_sec(1.0), 0.0);
}

#[test]
fn test_parsed_invocation_lifetime() {
    // GIVEN: An input string
    let input = String::from("test.cmd --key value");
    let mut args_buffer = [("", ""); 10];
    let mut positional_buffer = [""; 10];

    // WHEN: We parse it
    let parsed = ZeroCopyParser::parse(&input, &mut args_buffer, &mut positional_buffer)
        .expect("Parse should succeed");

    // THEN: Parsed invocation borrows from input
    // (This is enforced by lifetimes - if we drop input, parsed becomes invalid)
    assert_eq!(parsed.capability_path, "test.cmd");
}
