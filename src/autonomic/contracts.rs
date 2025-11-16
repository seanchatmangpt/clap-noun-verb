//! # Temporal & Concurrency Contracts for Verbs
//!
//! First-class timing and concurrency constraints attached to verbs at compile time.
//! Contracts are const-evaluated and exposed to schedulers, policy engines, and agents.
//!
//! ## Design Principles
//!
//! 1. **Compile-Time Contracts**: Use const generics and const evaluation
//! 2. **Type-Level Guarantees**: Contracts are part of the verb's type signature
//! 3. **Zero Runtime Overhead**: Contract checks happen at compile time or scheduler init
//! 4. **Composable**: Contracts can be combined and refined

use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use std::time::Duration;

/// Duration class for expected execution time
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DurationClass {
    /// Microseconds - sub-millisecond operations
    FastPath,
    /// Milliseconds - interactive operations
    Interactive,
    /// Seconds - user-initiated operations
    UserInitiated,
    /// Minutes - batch processing
    Batch,
    /// Hours - long-running background tasks
    LongRunning,
}

impl DurationClass {
    /// Get typical duration for this class
    pub const fn typical_duration(&self) -> Duration {
        match self {
            DurationClass::FastPath => Duration::from_micros(100),
            DurationClass::Interactive => Duration::from_millis(100),
            DurationClass::UserInitiated => Duration::from_secs(5),
            DurationClass::Batch => Duration::from_secs(300),
            DurationClass::LongRunning => Duration::from_secs(3600),
        }
    }

    /// Get maximum duration for this class
    pub const fn max_duration(&self) -> Duration {
        match self {
            DurationClass::FastPath => Duration::from_millis(1),
            DurationClass::Interactive => Duration::from_millis(500),
            DurationClass::UserInitiated => Duration::from_secs(30),
            DurationClass::Batch => Duration::from_secs(1800),
            DurationClass::LongRunning => Duration::from_secs(86400),
        }
    }

    /// Check if this class is compatible with a deadline
    pub const fn meets_deadline(&self, deadline: Duration) -> bool {
        self.max_duration().as_nanos() <= deadline.as_nanos()
    }
}

/// Concurrency model for a verb
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConcurrencyModel {
    /// Only one instance per tenant, globally
    SingleTenantExclusive,

    /// Multiple instances per tenant, but shared resources
    TenantWideShared {
        /// Maximum concurrent instances per tenant
        max_concurrent: usize,
    },

    /// Global shared with rate limiting
    GlobalShared {
        /// Maximum concurrent instances globally
        max_concurrent: usize,
        /// Requests per second limit
        rate_limit_rps: Option<u32>,
    },

    /// Fully concurrent, no limits
    Unlimited,
}

/// Deadline specification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeadlineSpec {
    /// No deadline
    None,

    /// Hard deadline - must complete before this
    Hard { duration: Duration },

    /// Soft deadline - best effort
    Soft { duration: Duration },

    /// Must start by this time
    MustStartBy { window: Duration },
}

/// Temporal contract - timing guarantees and constraints
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemporalContract {
    /// Expected duration class
    pub duration_class: DurationClass,

    /// Deadline specification
    pub deadline: DeadlineSpec,

    /// Timeout - maximum allowed execution time
    pub timeout: Option<Duration>,

    /// Retry policy
    pub retry: RetryPolicy,

    /// Whether this operation is idempotent
    pub idempotent: bool,
}

impl TemporalContract {
    /// Create a new temporal contract
    pub const fn new(duration_class: DurationClass) -> Self {
        Self {
            duration_class,
            deadline: DeadlineSpec::None,
            timeout: None,
            retry: RetryPolicy::None,
            idempotent: false,
        }
    }

    /// Set deadline
    pub const fn with_deadline(mut self, deadline: DeadlineSpec) -> Self {
        self.deadline = deadline;
        self
    }

    /// Set timeout
    pub const fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set retry policy
    pub const fn with_retry(mut self, retry: RetryPolicy) -> Self {
        self.retry = retry;
        self
    }

    /// Mark as idempotent
    pub const fn idempotent(mut self) -> Self {
        self.idempotent = true;
        self
    }

    /// Check if this contract is compatible with another
    pub fn is_compatible_with(&self, other: &TemporalContract) -> bool {
        // More restrictive duration class must be compatible
        self.duration_class <= other.duration_class
    }

    /// Get effective timeout considering deadline and class
    pub fn effective_timeout(&self) -> Option<Duration> {
        let timeout = self.timeout.or(Some(self.duration_class.max_duration()));

        match self.deadline {
            DeadlineSpec::Hard { duration } => {
                Some(timeout.map_or(duration, |t| t.min(duration)))
            }
            _ => timeout,
        }
    }
}

impl Default for TemporalContract {
    fn default() -> Self {
        Self::new(DurationClass::Interactive)
    }
}

/// Retry policy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetryPolicy {
    /// No retries
    None,

    /// Fixed number of retries with exponential backoff
    ExponentialBackoff {
        max_attempts: u32,
        initial_delay: Duration,
        max_delay: Duration,
    },

    /// Fixed number of retries with linear backoff
    LinearBackoff {
        max_attempts: u32,
        delay: Duration,
    },

    /// Retry forever with exponential backoff (for critical operations)
    Forever {
        initial_delay: Duration,
        max_delay: Duration,
    },
}

/// Concurrency contract - execution shape and resource limits
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConcurrencyContract {
    /// Concurrency model
    pub model: ConcurrencyModel,

    /// Required isolation level
    pub isolation: IsolationLevel,

    /// Resource limits
    pub resource_limits: ResourceLimits,
}

impl ConcurrencyContract {
    /// Create a new concurrency contract
    pub const fn new(model: ConcurrencyModel) -> Self {
        Self {
            model,
            isolation: IsolationLevel::Shared,
            resource_limits: ResourceLimits::unlimited(),
        }
    }

    /// Set isolation level
    pub const fn with_isolation(mut self, isolation: IsolationLevel) -> Self {
        self.isolation = isolation;
        self
    }

    /// Set resource limits
    pub const fn with_resource_limits(mut self, limits: ResourceLimits) -> Self {
        self.resource_limits = limits;
        self
    }
}

impl Default for ConcurrencyContract {
    fn default() -> Self {
        Self::new(ConcurrencyModel::Unlimited)
    }
}

/// Isolation level for execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IsolationLevel {
    /// Shared resources with other operations
    Shared,

    /// Isolated from other operations in the same tenant
    TenantIsolated,

    /// Fully isolated execution
    FullyIsolated,

    /// Sandboxed execution (separate process/container)
    Sandboxed,
}

/// Resource limits for execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory in bytes
    pub max_memory_bytes: Option<u64>,

    /// Maximum CPU time
    pub max_cpu_time: Option<Duration>,

    /// Maximum open file descriptors
    pub max_file_descriptors: Option<u32>,

    /// Maximum network connections
    pub max_network_connections: Option<u32>,
}

impl ResourceLimits {
    /// No limits
    pub const fn unlimited() -> Self {
        Self {
            max_memory_bytes: None,
            max_cpu_time: None,
            max_file_descriptors: None,
            max_network_connections: None,
        }
    }

    /// Strict limits for untrusted code
    pub const fn strict() -> Self {
        Self {
            max_memory_bytes: Some(100 * 1024 * 1024), // 100MB
            max_cpu_time: Some(Duration::from_secs(60)),
            max_file_descriptors: Some(10),
            max_network_connections: Some(5),
        }
    }
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self::unlimited()
    }
}

/// Complete execution contract combining temporal and concurrency
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionContract {
    /// Temporal contract
    pub temporal: TemporalContract,

    /// Concurrency contract
    pub concurrency: ConcurrencyContract,

    /// Contract metadata
    pub metadata: ContractMetadata,
}

impl ExecutionContract {
    /// Create a new execution contract
    pub fn new(temporal: TemporalContract, concurrency: ConcurrencyContract) -> Self {
        Self {
            temporal,
            concurrency,
            metadata: ContractMetadata::default(),
        }
    }

    /// Builder pattern
    pub fn builder() -> ExecutionContractBuilder {
        ExecutionContractBuilder::default()
    }

    /// Check if this contract can be satisfied by available resources
    pub fn can_satisfy(&self, available: &AvailableResources) -> bool {
        // Check concurrency limits
        match &self.concurrency.model {
            ConcurrencyModel::SingleTenantExclusive => available.tenant_slots > 0,
            ConcurrencyModel::TenantWideShared { max_concurrent } => {
                available.tenant_slots >= *max_concurrent
            }
            ConcurrencyModel::GlobalShared { max_concurrent, .. } => {
                available.global_slots >= *max_concurrent
            }
            ConcurrencyModel::Unlimited => true,
        }
    }

    /// Estimate resource consumption
    pub fn estimate_resources(&self) -> ResourceEstimate {
        ResourceEstimate {
            memory_bytes: self.concurrency.resource_limits.max_memory_bytes.unwrap_or(1024 * 1024),
            cpu_time: self.temporal.duration_class.typical_duration(),
            slots_required: match &self.concurrency.model {
                ConcurrencyModel::SingleTenantExclusive => 1,
                ConcurrencyModel::TenantWideShared { max_concurrent } => *max_concurrent,
                ConcurrencyModel::GlobalShared { max_concurrent, .. } => *max_concurrent,
                ConcurrencyModel::Unlimited => 1,
            },
        }
    }
}

/// Contract metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractMetadata {
    /// Contract version
    pub version: String,

    /// Whether this contract is experimental
    pub experimental: bool,

    /// Documentation
    pub description: Option<String>,
}

impl Default for ContractMetadata {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            experimental: false,
            description: None,
        }
    }
}

/// Available resources for contract satisfaction
#[derive(Debug, Clone)]
pub struct AvailableResources {
    /// Available tenant-level slots
    pub tenant_slots: usize,

    /// Available global slots
    pub global_slots: usize,

    /// Available memory
    pub available_memory_bytes: u64,

    /// Available CPU capacity
    pub available_cpu_percent: f32,
}

/// Resource consumption estimate
#[derive(Debug, Clone)]
pub struct ResourceEstimate {
    /// Estimated memory usage
    pub memory_bytes: u64,

    /// Estimated CPU time
    pub cpu_time: Duration,

    /// Required concurrency slots
    pub slots_required: usize,
}

/// Builder for execution contracts
#[derive(Debug, Default)]
pub struct ExecutionContractBuilder {
    duration_class: Option<DurationClass>,
    deadline: Option<DeadlineSpec>,
    timeout: Option<Duration>,
    retry: Option<RetryPolicy>,
    idempotent: bool,
    concurrency_model: Option<ConcurrencyModel>,
    isolation: Option<IsolationLevel>,
    resource_limits: Option<ResourceLimits>,
}

impl ExecutionContractBuilder {
    /// Set duration class
    pub fn duration_class(mut self, class: DurationClass) -> Self {
        self.duration_class = Some(class);
        self
    }

    /// Set deadline
    pub fn deadline(mut self, deadline: DeadlineSpec) -> Self {
        self.deadline = Some(deadline);
        self
    }

    /// Set timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set retry policy
    pub fn retry(mut self, retry: RetryPolicy) -> Self {
        self.retry = Some(retry);
        self
    }

    /// Mark as idempotent
    pub fn idempotent(mut self) -> Self {
        self.idempotent = true;
        self
    }

    /// Set concurrency model
    pub fn concurrency_model(mut self, model: ConcurrencyModel) -> Self {
        self.concurrency_model = Some(model);
        self
    }

    /// Set isolation level
    pub fn isolation(mut self, isolation: IsolationLevel) -> Self {
        self.isolation = Some(isolation);
        self
    }

    /// Set resource limits
    pub fn resource_limits(mut self, limits: ResourceLimits) -> Self {
        self.resource_limits = Some(limits);
        self
    }

    /// Build the contract
    pub fn build(self) -> ExecutionContract {
        let duration_class = self.duration_class.unwrap_or(DurationClass::Interactive);

        let mut temporal = TemporalContract::new(duration_class);
        if let Some(deadline) = self.deadline {
            temporal = temporal.with_deadline(deadline);
        }
        if let Some(timeout) = self.timeout {
            temporal = temporal.with_timeout(timeout);
        }
        if let Some(retry) = self.retry {
            temporal = temporal.with_retry(retry);
        }
        if self.idempotent {
            temporal = temporal.idempotent();
        }

        let mut concurrency = ConcurrencyContract::new(
            self.concurrency_model.unwrap_or(ConcurrencyModel::Unlimited)
        );
        if let Some(isolation) = self.isolation {
            concurrency = concurrency.with_isolation(isolation);
        }
        if let Some(limits) = self.resource_limits {
            concurrency = concurrency.with_resource_limits(limits);
        }

        ExecutionContract::new(temporal, concurrency)
    }
}

/// Trait for types with execution contracts (attached at compile time)
pub trait HasContract {
    /// Get the execution contract for this type
    fn contract() -> ExecutionContract;
}

/// Macro to define contracts at compile time
#[macro_export]
macro_rules! define_contract {
    ($name:ident, $duration:expr, $concurrency:expr) => {
        impl $crate::autonomic::contracts::HasContract for $name {
            fn contract() -> $crate::autonomic::contracts::ExecutionContract {
                $crate::autonomic::contracts::ExecutionContract::new(
                    $crate::autonomic::contracts::TemporalContract::new($duration),
                    $crate::autonomic::contracts::ConcurrencyContract::new($concurrency),
                )
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duration_class_ordering() {
        assert!(DurationClass::FastPath < DurationClass::Interactive);
        assert!(DurationClass::Interactive < DurationClass::Batch);
    }

    #[test]
    fn test_temporal_contract_compatibility() {
        let fast = TemporalContract::new(DurationClass::FastPath);
        let interactive = TemporalContract::new(DurationClass::Interactive);

        assert!(fast.is_compatible_with(&interactive));
        assert!(!interactive.is_compatible_with(&fast));
    }

    #[test]
    fn test_contract_builder() {
        let contract = ExecutionContract::builder()
            .duration_class(DurationClass::FastPath)
            .deadline(DeadlineSpec::Hard {
                duration: Duration::from_millis(10),
            })
            .idempotent()
            .concurrency_model(ConcurrencyModel::TenantWideShared { max_concurrent: 5 })
            .build();

        assert_eq!(contract.temporal.duration_class, DurationClass::FastPath);
        assert!(contract.temporal.idempotent);
    }

    #[test]
    fn test_resource_satisfaction() {
        let contract = ExecutionContract::builder()
            .concurrency_model(ConcurrencyModel::TenantWideShared { max_concurrent: 3 })
            .build();

        let available = AvailableResources {
            tenant_slots: 5,
            global_slots: 10,
            available_memory_bytes: 1024 * 1024 * 1024,
            available_cpu_percent: 80.0,
        };

        assert!(contract.can_satisfy(&available));
    }

    struct FastOperation;
    define_contract!(FastOperation, DurationClass::FastPath, ConcurrencyModel::Unlimited);

    #[test]
    fn test_compile_time_contract() {
        let contract = FastOperation::contract();
        assert_eq!(contract.temporal.duration_class, DurationClass::FastPath);
    }
}
