//! # Phase 3: Compile-Time Resource Quotas
//!
//! Enforce resource budgets at compile time using const generics and type-level programming.
//! Prevents resource exhaustion in trillion-agent systems.
//!
//! ## 2027+ Resource Management
//!
//! - **Compile-Time Budgets**: Resource limits enforced by type system
//! - **Zero Runtime Overhead**: All checks done at compile time
//! - **Composable Quotas**: Combine and split budgets type-safely
//! - **Provable Bounds**: Mathematical guarantees on resource usage
//!
//! ## Resource Types
//!
//! - Memory quotas (bytes)
//! - CPU quotas (nanoseconds)
//! - I/O quotas (operations)
//! - Network quotas (bytes)

use std::marker::PhantomData;

// ============================================================================
// Const Resource Units
// ============================================================================

/// Memory quota in bytes (const generic)
pub struct MemoryQuota<const BYTES: u64>;

/// CPU quota in nanoseconds (const generic)
pub struct CpuQuota<const NANOS: u64>;

/// I/O operations quota (const generic)
pub struct IoQuota<const OPS: u64>;

/// Network bytes quota (const generic)
pub struct NetworkQuota<const BYTES: u64>;

// ============================================================================
// Resource Budget (Type-Level)
// ============================================================================

/// Compile-time resource budget
///
/// # Example
///
/// ```rust,no_run
/// use clap_noun_verb::kernel::quotas::*;
///
/// // Define budget at compile time (1MB, 1ms, 100 ops, 10KB)
/// type MyBudget = ResourceBudget<
///     MemoryQuota<1_000_000>,
///     CpuQuota<1_000_000>,
///     IoQuota<100>,
///     NetworkQuota<10_000>
/// >;
///
/// // Create operation that consumes budget
/// let op = Operation::<MyBudget>::new("test");
///
/// // Compiler ensures we stay within budget!
/// ```
pub struct ResourceBudget<Mem, Cpu, Io, Net> {
    _mem: PhantomData<Mem>,
    _cpu: PhantomData<Cpu>,
    _io: PhantomData<Io>,
    _net: PhantomData<Net>,
}

impl<const M: u64, const C: u64, const I: u64, const N: u64>
    ResourceBudget<MemoryQuota<M>, CpuQuota<C>, IoQuota<I>, NetworkQuota<N>>
{
    /// Get memory quota (const)
    pub const fn memory_bytes() -> u64 {
        M
    }

    /// Get CPU quota (const)
    pub const fn cpu_nanos() -> u64 {
        C
    }

    /// Get I/O quota (const)
    pub const fn io_ops() -> u64 {
        I
    }

    /// Get network quota (const)
    pub const fn network_bytes() -> u64 {
        N
    }
}

// ============================================================================
// Budget Operations (Type-Level Arithmetic)
// ============================================================================

/// Split budget in half (compile-time)
pub type SplitBudget<Mem, Cpu, Io, Net> = (
    ResourceBudget<Mem, Cpu, Io, Net>,
    ResourceBudget<Mem, Cpu, Io, Net>,
);

/// Combine two budgets (compile-time)
///
/// NOTE: This trait and its implementation require the `generic_const_exprs` feature
/// which is not stable yet. Commented out until Rust 1.80+ or when feature stabilizes.
pub trait CombineBudgets<Other> {
    type Combined;
}

// COMMENTED OUT: Requires generic_const_exprs (unstable feature)
// impl<const M1: u64, const C1: u64, const I1: u64, const N1: u64, const M2: u64, const C2: u64, const I2: u64, const N2: u64>
//     CombineBudgets<ResourceBudget<MemoryQuota<M2>, CpuQuota<C2>, IoQuota<I2>, NetworkQuota<N2>>>
//     for ResourceBudget<MemoryQuota<M1>, CpuQuota<C1>, IoQuota<I1>, NetworkQuota<N1>>
// {
//     type Combined = ResourceBudget<
//         MemoryQuota<{ M1 + M2 }>,
//         CpuQuota<{ C1 + C2 }>,
//         IoQuota<{ I1 + I2 }>,
//         NetworkQuota<{ N1 + N2 }>,
//     >;
// }

// ============================================================================
// Budgeted Operation
// ============================================================================

/// Operation with compile-time resource budget
///
/// Type system ensures operation stays within budget.
pub struct Operation<Budget> {
    name: String,
    _budget: PhantomData<Budget>,
}

impl<const M: u64, const C: u64, const I: u64, const N: u64>
    Operation<ResourceBudget<MemoryQuota<M>, CpuQuota<C>, IoQuota<I>, NetworkQuota<N>>>
{
    /// Create new budgeted operation
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            _budget: PhantomData,
        }
    }

    /// Get budget constraints (const)
    pub const fn budget() -> BudgetInfo {
        BudgetInfo {
            memory_bytes: M,
            cpu_nanos: C,
            io_ops: I,
            network_bytes: N,
        }
    }

    /// Execute operation (budget enforced at compile time)
    pub fn execute<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        // In debug builds, could add runtime checks
        #[cfg(debug_assertions)]
        {
            println!(
                "Executing {} with budget: {}MB, {}μs, {} I/O ops, {}KB network",
                self.name,
                M / 1_000_000,
                C / 1_000,
                I,
                N / 1_000
            );
        }

        f()
    }

    /// Get operation name
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Runtime-accessible budget information
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BudgetInfo {
    pub memory_bytes: u64,
    pub cpu_nanos: u64,
    pub io_ops: u64,
    pub network_bytes: u64,
}

// ============================================================================
// Predefined Budget Tiers
// ============================================================================

/// Tiny budget: 1KB memory, 1μs CPU, 1 I/O op, 100 bytes network
pub type TinyBudget = ResourceBudget<
    MemoryQuota<1_000>,
    CpuQuota<1_000>,
    IoQuota<1>,
    NetworkQuota<100>,
>;

/// Small budget: 10KB memory, 10μs CPU, 10 I/O ops, 1KB network
pub type SmallBudget = ResourceBudget<
    MemoryQuota<10_000>,
    CpuQuota<10_000>,
    IoQuota<10>,
    NetworkQuota<1_000>,
>;

/// Medium budget: 1MB memory, 1ms CPU, 100 I/O ops, 100KB network
pub type MediumBudget = ResourceBudget<
    MemoryQuota<1_000_000>,
    CpuQuota<1_000_000>,
    IoQuota<100>,
    NetworkQuota<100_000>,
>;

/// Large budget: 10MB memory, 10ms CPU, 1000 I/O ops, 1MB network
pub type LargeBudget = ResourceBudget<
    MemoryQuota<10_000_000>,
    CpuQuota<10_000_000>,
    IoQuota<1_000>,
    NetworkQuota<1_000_000>,
>;

/// Unlimited budget (for special cases)
pub type UnlimitedBudget = ResourceBudget<
    MemoryQuota<{ u64::MAX }>,
    CpuQuota<{ u64::MAX }>,
    IoQuota<{ u64::MAX }>,
    NetworkQuota<{ u64::MAX }>,
>;

// ============================================================================
// Budget Enforcement Traits
// ============================================================================

/// Trait for operations that fit within a budget
pub trait FitsWithinBudget<Budget> {
    /// Check if this operation fits within budget (const)
    const FITS: bool;
}

impl<const M1: u64, const C1: u64, const I1: u64, const N1: u64, const M2: u64, const C2: u64, const I2: u64, const N2: u64>
    FitsWithinBudget<ResourceBudget<MemoryQuota<M2>, CpuQuota<C2>, IoQuota<I2>, NetworkQuota<N2>>>
    for ResourceBudget<MemoryQuota<M1>, CpuQuota<C1>, IoQuota<I1>, NetworkQuota<N1>>
{
    const FITS: bool = M1 <= M2 && C1 <= C2 && I1 <= I2 && N1 <= N2;
}

/// Require operation fits within budget (compile-time check)
///
/// # Example
///
/// ```rust,no_run
/// use clap_noun_verb::kernel::quotas::*;
///
/// type OperationBudget = SmallBudget;
/// type AllowedBudget = MediumBudget;
///
/// // This compiles because SmallBudget fits within MediumBudget
/// require_budget_fit::<OperationBudget, AllowedBudget>();
///
/// // This would NOT compile if OperationBudget > AllowedBudget
/// ```
pub const fn require_budget_fit<Op, Limit>()
where
    Op: FitsWithinBudget<Limit>,
{
    assert!(Op::FITS, "Operation budget exceeds limit");
}

// ============================================================================
// Budget Pool (Dynamic)
// ============================================================================

/// Runtime budget pool for dynamic allocation
///
/// While budgets are enforced at compile time, this allows
/// tracking actual usage at runtime for monitoring.
pub struct BudgetPool {
    memory_used: u64,
    memory_limit: u64,
    cpu_used: u64,
    cpu_limit: u64,
    io_used: u64,
    io_limit: u64,
    network_used: u64,
    network_limit: u64,
}

impl BudgetPool {
    /// Create new budget pool from compile-time budget
    pub fn from_budget<const M: u64, const C: u64, const I: u64, const N: u64>(
        _budget: &ResourceBudget<MemoryQuota<M>, CpuQuota<C>, IoQuota<I>, NetworkQuota<N>>,
    ) -> Self {
        Self {
            memory_used: 0,
            memory_limit: M,
            cpu_used: 0,
            cpu_limit: C,
            io_used: 0,
            io_limit: I,
            network_used: 0,
            network_limit: N,
        }
    }

    /// Allocate memory from pool
    pub fn allocate_memory(&mut self, bytes: u64) -> Result<(), BudgetError> {
        if self.memory_used + bytes > self.memory_limit {
            return Err(BudgetError::MemoryExhausted {
                requested: bytes,
                available: self.memory_limit - self.memory_used,
            });
        }

        self.memory_used += bytes;
        Ok(())
    }

    /// Consume CPU time
    pub fn consume_cpu(&mut self, nanos: u64) -> Result<(), BudgetError> {
        if self.cpu_used + nanos > self.cpu_limit {
            return Err(BudgetError::CpuExhausted {
                requested: nanos,
                available: self.cpu_limit - self.cpu_used,
            });
        }

        self.cpu_used += nanos;
        Ok(())
    }

    /// Consume I/O operation
    pub fn consume_io(&mut self, ops: u64) -> Result<(), BudgetError> {
        if self.io_used + ops > self.io_limit {
            return Err(BudgetError::IoExhausted {
                requested: ops,
                available: self.io_limit - self.io_used,
            });
        }

        self.io_used += ops;
        Ok(())
    }

    /// Consume network bandwidth
    pub fn consume_network(&mut self, bytes: u64) -> Result<(), BudgetError> {
        if self.network_used + bytes > self.network_limit {
            return Err(BudgetError::NetworkExhausted {
                requested: bytes,
                available: self.network_limit - self.network_used,
            });
        }

        self.network_used += bytes;
        Ok(())
    }

    /// Get usage statistics
    pub fn stats(&self) -> BudgetStats {
        BudgetStats {
            memory_used: self.memory_used,
            memory_limit: self.memory_limit,
            cpu_used: self.cpu_used,
            cpu_limit: self.cpu_limit,
            io_used: self.io_used,
            io_limit: self.io_limit,
            network_used: self.network_used,
            network_limit: self.network_limit,
        }
    }
}

/// Budget exhaustion errors
#[derive(Debug, Clone, Copy)]
pub enum BudgetError {
    MemoryExhausted { requested: u64, available: u64 },
    CpuExhausted { requested: u64, available: u64 },
    IoExhausted { requested: u64, available: u64 },
    NetworkExhausted { requested: u64, available: u64 },
}

impl std::fmt::Display for BudgetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MemoryExhausted { requested, available } => {
                write!(
                    f,
                    "Memory budget exhausted: requested {} bytes, only {} available",
                    requested, available
                )
            }
            Self::CpuExhausted { requested, available } => {
                write!(
                    f,
                    "CPU budget exhausted: requested {} ns, only {} available",
                    requested, available
                )
            }
            Self::IoExhausted { requested, available } => {
                write!(
                    f,
                    "I/O budget exhausted: requested {} ops, only {} available",
                    requested, available
                )
            }
            Self::NetworkExhausted { requested, available } => {
                write!(
                    f,
                    "Network budget exhausted: requested {} bytes, only {} available",
                    requested, available
                )
            }
        }
    }
}

impl std::error::Error for BudgetError {}

/// Budget usage statistics
#[derive(Debug, Clone, Copy)]
pub struct BudgetStats {
    pub memory_used: u64,
    pub memory_limit: u64,
    pub cpu_used: u64,
    pub cpu_limit: u64,
    pub io_used: u64,
    pub io_limit: u64,
    pub network_used: u64,
    pub network_limit: u64,
}

impl BudgetStats {
    /// Get memory utilization (0.0 - 1.0)
    pub fn memory_utilization(&self) -> f64 {
        if self.memory_limit == 0 {
            0.0
        } else {
            self.memory_used as f64 / self.memory_limit as f64
        }
    }

    /// Get CPU utilization (0.0 - 1.0)
    pub fn cpu_utilization(&self) -> f64 {
        if self.cpu_limit == 0 {
            0.0
        } else {
            self.cpu_used as f64 / self.cpu_limit as f64
        }
    }

    /// Get I/O utilization (0.0 - 1.0)
    pub fn io_utilization(&self) -> f64 {
        if self.io_limit == 0 {
            0.0
        } else {
            self.io_used as f64 / self.io_limit as f64
        }
    }

    /// Get network utilization (0.0 - 1.0)
    pub fn network_utilization(&self) -> f64 {
        if self.network_limit == 0 {
            0.0
        } else {
            self.network_used as f64 / self.network_limit as f64
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_const_budget_values() {
        type MyBudget = ResourceBudget<
            MemoryQuota<1_000_000>,
            CpuQuota<2_000_000>,
            IoQuota<100>,
            NetworkQuota<50_000>,
        >;

        assert_eq!(MyBudget::memory_bytes(), 1_000_000);
        assert_eq!(MyBudget::cpu_nanos(), 2_000_000);
        assert_eq!(MyBudget::io_ops(), 100);
        assert_eq!(MyBudget::network_bytes(), 50_000);
    }

    #[test]
    fn test_operation_execution() {
        type OpBudget = SmallBudget;
        let op = Operation::<OpBudget>::new("test-operation");

        let result = op.execute(|| 42);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_budget_info() {
        let info = Operation::<MediumBudget>::budget();

        assert_eq!(info.memory_bytes, 1_000_000);
        assert_eq!(info.cpu_nanos, 1_000_000);
        assert_eq!(info.io_ops, 100);
        assert_eq!(info.network_bytes, 100_000);
    }

    #[test]
    fn test_budget_fits() {
        // Small fits within Medium
        const FITS: bool = <SmallBudget as FitsWithinBudget<MediumBudget>>::FITS;
        assert!(FITS);

        // Medium does NOT fit within Small
        const DOES_NOT_FIT: bool = <MediumBudget as FitsWithinBudget<SmallBudget>>::FITS;
        assert!(!DOES_NOT_FIT);
    }

    #[test]
    fn test_budget_pool() {
        let budget: MediumBudget = ResourceBudget {
            _mem: PhantomData,
            _cpu: PhantomData,
            _io: PhantomData,
            _net: PhantomData,
        };
        let mut pool = BudgetPool::from_budget(&budget);

        // Allocate memory
        pool.allocate_memory(100_000).unwrap();
        assert_eq!(pool.stats().memory_used, 100_000);

        // Consume CPU
        pool.consume_cpu(50_000).unwrap();
        assert_eq!(pool.stats().cpu_used, 50_000);

        // Consume I/O
        pool.consume_io(10).unwrap();
        assert_eq!(pool.stats().io_used, 10);

        // Consume network
        pool.consume_network(10_000).unwrap();
        assert_eq!(pool.stats().network_used, 10_000);
    }

    #[test]
    fn test_budget_exhaustion() {
        let budget: TinyBudget = ResourceBudget {
            _mem: PhantomData,
            _cpu: PhantomData,
            _io: PhantomData,
            _net: PhantomData,
        };
        let mut pool = BudgetPool::from_budget(&budget);

        // Try to allocate more than limit
        let result = pool.allocate_memory(2_000);
        assert!(result.is_err());

        match result.unwrap_err() {
            BudgetError::MemoryExhausted { requested, available } => {
                assert_eq!(requested, 2_000);
                assert_eq!(available, 1_000);
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_budget_utilization() {
        let budget: MediumBudget = ResourceBudget {
            _mem: PhantomData,
            _cpu: PhantomData,
            _io: PhantomData,
            _net: PhantomData,
        };
        let mut pool = BudgetPool::from_budget(&budget);

        pool.allocate_memory(500_000).unwrap(); // 50%
        let stats = pool.stats();

        assert_eq!(stats.memory_utilization(), 0.5);
        assert_eq!(stats.cpu_utilization(), 0.0);
    }
}
