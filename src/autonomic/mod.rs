//! Autonomic CLI Layer - Machine-grade interface for clap-noun-verb
//!
//! This module provides capabilities for building CLI applications that can be
//! introspected, analyzed, and driven by autonomic systems, MAPE-K loops, and agents.
//!
//! ## Key Features (2027 Swarm-Native)
//!
//! - **Introspection**: Discover commands, arguments, and metadata at runtime
//! - **Stable IDs**: Capability IDs that survive renames for protocol stability
//! - **Effect Modeling**: Declare read-only vs mutating operations with isolation requirements
//! - **Multi-Agent Tenancy**: Agent identity, tenant isolation, and QoS hints
//! - **Policy Hooks**: Pluggable governance for trillions of invocations
//! - **Capability Composition**: Input/output schemas for workflow composition
//! - **Streaming**: Events, sessions, and incremental receipts for long-running operations
//! - **Plane Integration**: O/Σ/Q/ΔΣ metadata for ontology-driven systems
//! - **Guards & Budgets**: Latency and resource constraints
//! - **Receipts**: Structured execution records for audit and analysis

pub mod async_runtime;
pub mod capability_id;
pub mod certificates;
pub mod cli;
pub mod contracts;
pub mod delegation;
pub mod effects;
pub mod errors;
pub mod governance;
pub mod graph;
pub mod guards;
pub mod hotpath;
pub mod introspection;
pub mod phase_observer;
pub mod phases;
pub mod planes;
pub mod policy;
pub mod protocol;
pub mod receipts;
pub mod schema;
pub mod simd;
pub mod streaming;
pub mod telemetry;
pub mod tenancy;
pub mod verification;

// Re-export key types
pub use capability_id::{
    CapabilityChange, CapabilityChangelog, CapabilityId, CapabilityVersion, ChangeType,
    DeprecationInfo,
};
pub use certificates::{
    Certificate, CertificateBuilder, CertificateError, CertificateId, CertifiedInvocation,
    PolicyTrace, SchemaHash,
};
pub use cli::{AutonomicCli, AutonomicNounCommand, AutonomicVerbCommand};
pub use contracts::{
    ConcurrencyContract, ConcurrencyModel, DeadlineSpec, DurationClass, ExecutionContract,
    HasContract, IsolationLevel, ResourceLimits, RetryPolicy, TemporalContract,
};
pub use delegation::{
    CapabilityConstraint, DelegationChain, DelegationError, DelegationRegistry, DelegationToken,
    EffectLevel, Principal, PrincipalType, TemporalConstraint, TokenId,
};
pub use effects::{
    DataSensitivityTag, EffectMetadata, EffectType, IsolationRequirement, Sensitivity,
};
pub use errors::{ErrorKind, ErrorResponse, StructuredError};
pub use governance::{
    EventId, EventType, GovernanceEvent, GovernanceLedger, OperatingMode, PolicyChangeType,
    ReplayEngine, ReplayResult, ViolationSeverity,
};
pub use graph::{
    CapabilityEdge, CapabilityGraph, CapabilityNode, EdgeId, EdgeType, GraphError, GraphStats,
    NodeId, SchemaCompatibility,
};
pub use guards::{GuardConfig, GuardResult, GuardStatus};
pub use hotpath::{
    AgentHandle, ContextPool, EffectFlags, FlagSet, HotPathContext, HotPathInvocation,
    HotPathMetrics, InvocationArena, InvocationQueue, TenantHandle, ZeroCopyParser,
};
pub use introspection::{
    AppMetadata, ArgumentMetadata, CommandCapabilities, CommandGraph, CommandMetadata, GraphEdge,
    GraphNode, IntrospectionResponse, NounMetadata, VerbMetadata,
};
pub use planes::{InteractionType, Plane, PlaneInteraction};
pub use policy::{
    PolicyDecision, PolicyEngine, PolicyRequest, PolicyResult, RuleBasedPolicyEngine,
};
pub use receipts::{ExecutionReceipt, ReceiptConfig, ReceiptWithOutput};
pub use schema::{
    CommandReference, CompositionMetadata, EquivalenceClass, EquivalenceRelationship, InputSchema,
    OutputSchema, PrimitiveType, Resource, TypeSchema,
};
pub use streaming::{
    IncrementalReceipt, SessionContext, SessionId, SessionManager, SessionState, StreamEvent,
    StreamEventType,
};
pub use tenancy::{
    AgentIdentity, EnforcementMode, InvocationContext, PolicyContext as TenantPolicyContext,
    PriorityClass, QoSHints, TenantIdentity,
};

/// Version of the autonomic CLI schema
pub const SCHEMA_VERSION: &str = "2.0.0";

/// Features supported by this implementation
pub const SUPPORTED_FEATURES: &[&str] = &[
    "introspect",
    "capabilities",
    "effects",
    "planes",
    "guards",
    "receipts",
    "errors",
    "stable_ids",
    "versioning",
    "tenancy",
    "policy",
    "composition",
    "streaming",
    "sessions",
    // 2027 Swarm-Native Features
    "certificates",
    "contracts",
    "delegation",
    "governance",
    "graph",
    "hotpath",
];
