//! # Zero-Allocation Hot Path for High-Throughput Invocations
//!
//! When CNV becomes the inner control plane for dense agent runtimes, it must handle
//! millions of invocations per second without drowning in allocations.
//!
//! ## Design Principles
//!
//! 1. **Arena Allocation**: Batch-scoped arenas for transient data
//! 2. **Lock-Free Queues**: MPSC/MPMC for invocation dispatch
//! 3. **Compact Handles**: Agent/tenant context as cache-friendly IDs
//! 4. **Zero-Copy Parsing**: Single-pass argument extraction
//! 5. **Branch Reduction**: Minimal decision trees in critical path

use super::{
    capability_id::CapabilityId,
    effects::EffectMetadata,
    tenancy::{AgentIdentity, TenantIdentity},
};
use crossbeam::queue::ArrayQueue;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

/// Compact handle for agent identity (avoiding clones)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AgentHandle(u64);

impl AgentHandle {
    /// Create a new agent handle
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// Get the raw ID
    pub fn id(&self) -> u64 {
        self.0
    }
}

/// Compact handle for tenant identity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TenantHandle(u64);

impl TenantHandle {
    /// Create a new tenant handle
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// Get the raw ID
    pub fn id(&self) -> u64 {
        self.0
    }
}

/// Compact invocation context for hot path
///
/// Uses handles instead of full identity structures to reduce cache pressure
#[derive(Debug, Clone, Copy)]
pub struct HotPathContext {
    /// Agent handle
    pub agent: AgentHandle,

    /// Tenant handle
    pub tenant: TenantHandle,

    /// Capability index (pre-resolved)
    pub capability_index: u32,

    /// Effect flags (bitfield for quick checks)
    pub effect_flags: EffectFlags,

    /// Correlation ID (hash, not full string)
    pub correlation_hash: u64,
}

impl HotPathContext {
    /// Create a new hot path context
    pub fn new(
        agent: AgentHandle,
        tenant: TenantHandle,
        capability_index: u32,
        effect_flags: EffectFlags,
    ) -> Self {
        Self { agent, tenant, capability_index, effect_flags, correlation_hash: 0 }
    }

    /// With correlation ID
    pub fn with_correlation(mut self, correlation_id: &str) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        correlation_id.hash(&mut hasher);
        self.correlation_hash = hasher.finish();
        self
    }
}

/// Bitfield for effect classification (enables branch-free checks)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectFlags(u16);

impl EffectFlags {
    pub const READ_ONLY: u16 = 1 << 0;
    pub const MUTATE_STATE: u16 = 1 << 1;
    pub const MUTATE_CONFIG: u16 = 1 << 2;
    pub const NETWORK: u16 = 1 << 3;
    pub const STORAGE: u16 = 1 << 4;
    pub const PRIVILEGED: u16 = 1 << 5;
    pub const IDEMPOTENT: u16 = 1 << 6;
    pub const ASYNC_CAPABLE: u16 = 1 << 7;

    /// Create empty flags
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Create from bits
    pub const fn from_bits(bits: u16) -> Self {
        Self(bits)
    }

    /// Add a flag
    pub const fn with(self, flag: u16) -> Self {
        Self(self.0 | flag)
    }

    /// Check if flag is set (branchless on modern CPUs)
    #[inline(always)]
    pub const fn has(self, flag: u16) -> bool {
        (self.0 & flag) != 0
    }

    /// Check if read-only
    #[inline(always)]
    pub const fn is_read_only(self) -> bool {
        self.has(Self::READ_ONLY)
    }

    /// Check if privileged
    #[inline(always)]
    pub const fn is_privileged(self) -> bool {
        self.has(Self::PRIVILEGED)
    }

    /// Combine with another flag set
    pub const fn merge(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

/// Parsed invocation ready for hot path execution
///
/// Uses string slices and avoids allocations where possible
#[derive(Debug)]
pub struct HotPathInvocation<'a> {
    /// Context
    pub context: HotPathContext,

    /// Capability path (borrowed)
    pub capability_path: &'a str,

    /// Arguments as key-value pairs (borrowed slices)
    pub args: &'a [(&'a str, &'a str)],

    /// Positional arguments
    pub positional: &'a [&'a str],

    /// Flags (boolean arguments)
    pub flags: FlagSet,
}

/// Compact representation of boolean flags
#[derive(Debug, Clone, Copy, Default)]
pub struct FlagSet(u64);

impl FlagSet {
    /// Maximum number of flags
    pub const MAX_FLAGS: usize = 64;

    /// Create empty flag set
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Set a flag by index
    #[inline(always)]
    pub fn set(&mut self, index: usize) {
        debug_assert!(index < Self::MAX_FLAGS);
        self.0 |= 1 << index;
    }

    /// Check if flag is set
    #[inline(always)]
    pub const fn is_set(&self, index: usize) -> bool {
        (self.0 & (1 << index)) != 0
    }

    /// Count set flags (population count)
    #[inline(always)]
    pub const fn count(&self) -> u32 {
        self.0.count_ones()
    }
}

/// Arena for batch-scoped allocations
///
/// Reset between batches to avoid fragmentation
pub struct InvocationArena {
    /// Backing buffer
    buffer: Vec<u8>,

    /// Current offset
    offset: AtomicUsize,

    /// Total capacity
    capacity: usize,
}

impl InvocationArena {
    /// Create a new arena with specified capacity
    pub fn new(capacity: usize) -> Self {
        Self { buffer: vec![0u8; capacity], offset: AtomicUsize::new(0), capacity }
    }

    /// Allocate space for a value
    pub fn alloc<T>(&self, value: T) -> Option<&T> {
        let size = std::mem::size_of::<T>();
        let align = std::mem::align_of::<T>();

        // Align offset
        let current = self.offset.load(Ordering::Relaxed);
        let aligned = (current + align - 1) & !(align - 1);
        let new_offset = aligned + size;

        if new_offset > self.capacity {
            return None;
        }

        // Try to claim space
        match self.offset.compare_exchange(
            current,
            new_offset,
            Ordering::Release,
            Ordering::Relaxed,
        ) {
            Ok(_) => {
                // SAFETY: We have exclusive access to this region
                #[allow(unsafe_code)]
                unsafe {
                    let ptr = self.buffer.as_ptr().add(aligned) as *mut T;
                    std::ptr::write(ptr, value);
                    Some(&*ptr)
                }
            }
            Err(_) => {
                // Retry on contention
                self.alloc(value)
            }
        }
    }

    /// Reset the arena (must not be accessed concurrently)
    pub fn reset(&mut self) {
        self.offset.store(0, Ordering::Relaxed);
    }

    /// Get current usage
    pub fn usage(&self) -> usize {
        self.offset.load(Ordering::Relaxed)
    }

    /// Get utilization as percentage
    pub fn utilization(&self) -> f64 {
        (self.usage() as f64 / self.capacity as f64) * 100.0
    }
}

/// Lock-free invocation queue
pub struct InvocationQueue<T> {
    /// Bounded queue (lock-free)
    queue: Arc<ArrayQueue<T>>,

    /// Enqueue counter
    enqueued: AtomicU64,

    /// Dequeue counter
    dequeued: AtomicU64,
}

impl<T> InvocationQueue<T> {
    /// Create a new queue with specified capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            queue: Arc::new(ArrayQueue::new(capacity)),
            enqueued: AtomicU64::new(0),
            dequeued: AtomicU64::new(0),
        }
    }

    /// Try to enqueue an invocation
    #[inline(always)]
    pub fn try_push(&self, invocation: T) -> Result<(), T> {
        match self.queue.push(invocation) {
            Ok(()) => {
                self.enqueued.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }
            Err(value) => Err(value),
        }
    }

    /// Try to dequeue an invocation
    #[inline(always)]
    pub fn try_pop(&self) -> Option<T> {
        self.queue.pop().map(|value| {
            self.dequeued.fetch_add(1, Ordering::Relaxed);
            value
        })
    }

    /// Get current queue length
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Check if queue is empty
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Get total enqueued count
    pub fn total_enqueued(&self) -> u64 {
        self.enqueued.load(Ordering::Relaxed)
    }

    /// Get total dequeued count
    pub fn total_dequeued(&self) -> u64 {
        self.dequeued.load(Ordering::Relaxed)
    }

    /// Get queue statistics
    pub fn stats(&self) -> QueueStats {
        QueueStats {
            current_length: self.len(),
            total_enqueued: self.total_enqueued(),
            total_dequeued: self.total_dequeued(),
            capacity: self.queue.capacity(),
        }
    }
}

impl<T> Clone for InvocationQueue<T> {
    fn clone(&self) -> Self {
        Self {
            queue: Arc::clone(&self.queue),
            enqueued: AtomicU64::new(self.enqueued.load(Ordering::Relaxed)),
            dequeued: AtomicU64::new(self.dequeued.load(Ordering::Relaxed)),
        }
    }
}

/// Queue statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStats {
    /// Current queue length
    pub current_length: usize,

    /// Total items enqueued
    pub total_enqueued: u64,

    /// Total items dequeued
    pub total_dequeued: u64,

    /// Queue capacity
    pub capacity: usize,
}

impl QueueStats {
    /// Get utilization percentage
    pub fn utilization(&self) -> f64 {
        (self.current_length as f64 / self.capacity as f64) * 100.0
    }

    /// Get throughput (items processed)
    pub fn throughput(&self) -> u64 {
        self.total_dequeued
    }
}

/// Context pool for reusing context objects
pub struct ContextPool {
    /// Pool of available contexts
    pool: ArrayQueue<HotPathContext>,

    /// Next agent ID
    next_agent_id: AtomicU64,

    /// Next tenant ID
    next_tenant_id: AtomicU64,
}

impl ContextPool {
    /// Create a new context pool
    pub fn new(capacity: usize) -> Self {
        Self {
            pool: ArrayQueue::new(capacity),
            next_agent_id: AtomicU64::new(1),
            next_tenant_id: AtomicU64::new(1),
        }
    }

    /// Allocate an agent handle
    pub fn alloc_agent_handle(&self) -> AgentHandle {
        let id = self.next_agent_id.fetch_add(1, Ordering::Relaxed);
        AgentHandle::new(id)
    }

    /// Allocate a tenant handle
    pub fn alloc_tenant_handle(&self) -> TenantHandle {
        let id = self.next_tenant_id.fetch_add(1, Ordering::Relaxed);
        TenantHandle::new(id)
    }

    /// Try to get a context from pool
    pub fn try_acquire(&self) -> Option<HotPathContext> {
        self.pool.pop()
    }

    /// Return a context to pool
    pub fn release(&self, context: HotPathContext) {
        let _ = self.pool.push(context);
    }
}

/// Zero-copy argument parser
pub struct ZeroCopyParser;

impl ZeroCopyParser {
    /// Parse command line into invocation (single pass, no allocations)
    ///
    /// Returns borrowed slices into the input buffer
    pub fn parse<'a>(
        input: &'a str,
        args_buffer: &'a mut [(&'a str, &'a str)],
        positional_buffer: &'a mut [&'a str],
    ) -> Result<ParsedInvocation<'a>, ParseError> {
        let mut parts = input.split_whitespace();

        // First part is capability path
        let capability_path = parts.next().ok_or(ParseError::EmptyInput)?;

        let mut args_count = 0;
        let mut positional_count = 0;
        let mut flags = FlagSet::empty();

        // Parse remaining arguments
        while let Some(part) = parts.next() {
            if part.starts_with("--") {
                // Long option
                let name = &part[2..];
                if let Some(value) = parts.next() {
                    if args_count < args_buffer.len() {
                        args_buffer[args_count] = (name, value);
                        args_count += 1;
                    }
                } else {
                    // Boolean flag
                    if positional_count < FlagSet::MAX_FLAGS {
                        flags.set(positional_count);
                    }
                }
            } else if part.starts_with('-') && part.len() == 2 {
                // Short option
                let name = &part[1..];
                if let Some(value) = parts.next() {
                    if args_count < args_buffer.len() {
                        args_buffer[args_count] = (name, value);
                        args_count += 1;
                    }
                } else {
                    // Boolean flag
                    if positional_count < FlagSet::MAX_FLAGS {
                        flags.set(positional_count);
                    }
                }
            } else {
                // Positional argument
                if positional_count < positional_buffer.len() {
                    positional_buffer[positional_count] = part;
                    positional_count += 1;
                }
            }
        }

        Ok(ParsedInvocation {
            capability_path,
            args: &args_buffer[..args_count],
            positional: &positional_buffer[..positional_count],
            flags,
        })
    }
}

/// Parsed invocation with borrowed slices
#[derive(Debug)]
pub struct ParsedInvocation<'a> {
    pub capability_path: &'a str,
    pub args: &'a [(&'a str, &'a str)],
    pub positional: &'a [&'a str],
    pub flags: FlagSet,
}

/// Parse errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum ParseError {
    #[error("Empty input")]
    EmptyInput,

    #[error("Buffer overflow")]
    BufferOverflow,

    #[error("Invalid syntax: {0}")]
    InvalidSyntax(String),
}

/// Hot path metrics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct HotPathMetrics {
    /// Total invocations processed
    pub invocations_processed: u64,

    /// Total nanoseconds spent in hot path
    pub total_nanos: u64,

    /// Peak queue depth
    pub peak_queue_depth: usize,

    /// Arena utilization (percentage)
    pub arena_utilization: f64,

    /// Cache hits
    pub cache_hits: u64,

    /// Cache misses
    pub cache_misses: u64,
}

impl HotPathMetrics {
    /// Average latency per invocation (nanoseconds)
    pub fn avg_latency_nanos(&self) -> f64 {
        if self.invocations_processed == 0 {
            0.0
        } else {
            self.total_nanos as f64 / self.invocations_processed as f64
        }
    }

    /// Cache hit rate
    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            (self.cache_hits as f64 / total as f64) * 100.0
        }
    }

    /// Throughput (invocations per second)
    pub fn throughput_per_sec(&self, elapsed_secs: f64) -> f64 {
        if elapsed_secs == 0.0 {
            0.0
        } else {
            self.invocations_processed as f64 / elapsed_secs
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effect_flags() {
        let flags = EffectFlags::empty().with(EffectFlags::READ_ONLY).with(EffectFlags::NETWORK);

        assert!(flags.is_read_only());
        assert!(flags.has(EffectFlags::NETWORK));
        assert!(!flags.is_privileged());
    }

    #[test]
    fn test_flag_set() {
        let mut flags = FlagSet::empty();
        flags.set(0);
        flags.set(5);
        flags.set(10);

        assert!(flags.is_set(0));
        assert!(flags.is_set(5));
        assert!(flags.is_set(10));
        assert!(!flags.is_set(1));
        assert_eq!(flags.count(), 3);
    }

    #[test]
    fn test_invocation_arena() {
        let arena = InvocationArena::new(1024);

        let val1 = arena.alloc(42u64).unwrap();
        let val2 = arena.alloc(100u64).unwrap();

        assert_eq!(*val1, 42);
        assert_eq!(*val2, 100);
        assert!(arena.usage() > 0);
    }

    #[test]
    fn test_invocation_queue() {
        let queue = InvocationQueue::new(10);

        queue.try_push(1).unwrap();
        queue.try_push(2).unwrap();

        assert_eq!(queue.len(), 2);
        assert_eq!(queue.try_pop(), Some(1));
        assert_eq!(queue.try_pop(), Some(2));
        assert_eq!(queue.try_pop(), None);

        let stats = queue.stats();
        assert_eq!(stats.total_enqueued, 2);
        assert_eq!(stats.total_dequeued, 2);
    }

    #[test]
    fn test_zero_copy_parser() {
        let input = "user.create --name alice --age 30 active";
        let mut args_buffer = [("", ""); 10];
        let mut positional_buffer = [""; 10];

        let parsed =
            ZeroCopyParser::parse(input, &mut args_buffer, &mut positional_buffer).unwrap();

        assert_eq!(parsed.capability_path, "user.create");
        assert_eq!(parsed.args.len(), 2);
        assert_eq!(parsed.args[0], ("name", "alice"));
        assert_eq!(parsed.args[1], ("age", "30"));
        assert_eq!(parsed.positional.len(), 1);
        assert_eq!(parsed.positional[0], "active");
    }
}
