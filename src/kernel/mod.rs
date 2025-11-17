//! CNV Kernel Capabilities - Autonomic Command Fabric (v4.0)
//!
//! This module provides the core kernel capabilities for clap-noun-verb,
//! transforming it from a perfect CLI wrapper into a command fabric for
//! trillions of agents.
//!
//! ## Core Capabilities
//!
//! - **Telemetry Profile**: Standardized verbosity, color, and format control
//! - **Output Pipeline**: Deterministic, structured output for all verbs
//! - **Grammar Model**: Introspectable command structure for agents and tooling
//! - **Manpage Integration**: Thin wrapper around manpage generation
//! - **File IO**: Stream-based input/output abstraction
//! - **Test Harness**: API for robust testing of CNV apps
//!
//! ## CNV 4.0: Autonomic Command Fabric
//!
//! ### Pillar 1: Capability Contracts
//!
//! Machine-verifiable command guarantees:
//! - Side-effect classification (Pure, ReadOnlyFS, Network, etc.)
//! - Resource profiles (runtime, memory bands)
//! - Stability guarantees (Stable, Experimental, Deprecated)
//! - Safety profiles (AgentSafe, HumanReviewRequired)
//!
//! ### Pillar 2: Session Kernel
//!
//! Long-lived, multiplexed command streams:
//! - Session abstraction for stateful command execution
//! - Multiplexed framing protocol over stdio
//! - Backpressure and cancellation support
//! - Session-scoped telemetry and metrics
//!
//! ### Pillar 3: Version Negotiation
//!
//! Structured change management:
//! - Grammar delta computation (structural diffs)
//! - Change classification (breaking/non-breaking)
//! - Compatibility negotiation protocol
//! - Capability-aware change detection
//!
//! ## Design Philosophy
//!
//! The kernel layer provides deterministic, agent-grade CLI capabilities without
//! coupling to specific implementations. External libraries (clap_mangen, etc.)
//! are used as silent backends.
//!
//! ## Version
//!
//! - Kernel capabilities introduced in v3.8.0
//! - Autonomic Command Fabric (v4.0) - current version

pub mod ahi_policy;
pub mod advanced_quota_enforcement;
pub mod attestation;
pub mod broker;
pub mod broker_state;
pub mod capability;
pub mod capability_contracts;
pub mod clnrm;
pub mod concurrent;
pub mod const_caps;
pub mod contract_runtime_view;
pub mod dflss;
pub mod deterministic_execution;
pub mod execution_receipts;
pub mod frame_schema;
pub mod grammar;
pub mod grammar_dsl;
pub mod io;
pub mod manpage;
pub mod output;
pub mod pluggable_persistence;
pub mod policy_governance;
pub mod quotas;
pub mod replay_engine;
pub mod session;
pub mod session_log;
pub mod simd;
pub mod telemetry;
pub mod test_harness;
pub mod transport;
pub mod type_level_security;
pub mod typestate;
pub mod version;

// Re-export key types for convenience
pub use ahi_policy::{
    AhiPolicyAdapter, PolicyDecision, PolicyUpdate, PolicyState, PolicyValidator,
    RiskActionType, CapabilityPolicy, DisabledCapabilityInfo, DeprecationInfo, RiskPolicy,
};
pub use broker::{
    BrokerKernel, BrokerRequest, BrokerResponse, BrokerResult, BrokerError,
    BrokerCapabilityRegistry, AdmissionPolicy, AdmissionController, BrokerLoad,
    TenantPolicy, FairShareScheduler, DeferredExecution,
};
pub use capability::{
    CapabilityClass, CapabilityContext, CapabilityContract, ResourceBand, SafetyProfile,
    StabilityProfile,
};
pub use capability_contracts::{
    CapabilityContractV2, CapabilitySchema, CapabilityConstraints, EffectsDeclaration,
    SideEffectType, IsolationRequirement, DataSensitivity, Invariant, InvariantSeverity,
    Guarantees, StabilityLevel, SafetyLevel, DeterminismGuarantee, IdempotencyGuarantee,
    ProofReferences, TestCoverage, StaticAnalysis, AuditReference, ProofObject,
    BuildMetadata, PathAccessPattern, AccessType,
};
pub use clnrm::{
    HermeticContainer, MockServices, MockResponse, RecordedSpan, QuotaBudget, DeterministicClock,
};
pub use dflss::{
    OptimizationObjective, Measurement, Candidate, Design, DeploymentPhase, DeploymentStatus,
    VerificationResult, DFLSSOptimizer, Change, Impact, DeploymentVerdict, SuccessCriteria,
};
pub use grammar::{Grammar, GrammarModel, GrammarNode};
pub use io::{FileIO, InputSource, OutputSink};
pub use manpage::ManpageGenerator;
pub use output::{OutputEnvelope, OutputPipeline, StructuredError, StructuredResult};
pub use execution_receipts::{
    CapabilityExecutionReceipt, EffectSummary, NetworkConnection, ReceiptMetadata,
    ReceiptStore, InMemoryReceiptStore, UsageStatistics,
};
pub use replay_engine::{
    ReplayEngine, VerifyReplayEngine, SimulateReplayEngine, AuditReplayEngine,
    DeterministicContext, ReplayEngineFactory, BatchReplayResult, BatchReplayExecutor,
    SideEffect, ReplayModeMarker, VerifyMode, SimulateMode, AuditMode, ReplayModeEnum,
};
pub use session::{
    ControlCommand, Frame, FramePayload, SessionBuilder, SessionConfig, SessionHandle,
    SessionId, SessionMetrics, SessionState, SessionVerb, StreamId,
};
pub use transport::{
    InvocationFrame, FrameType, FrameFlags, TransportError, InvocationTransport,
    StdioTransport, UnixSocketTransport, QuicTransport, TcpTransport,
};
pub use session_log::{
    SessionLogFrame, ReplayConfig, ReplayMode, ReplayResult,
    FrameDelta, SessionCompression, SessionLogStore, InMemorySessionLogStore,
    ExitCodeClass, LogicalClock, QuotaFootprint, ResultFrame, ErrorFrame,
    FrameMetadata, FrameOrderKey, TimingDrift, QuotaCheckResult,
    TimingPercentiles, ResourceStats, FrameValidationError, FRAME_SCHEMA_VERSION,
    MAX_CLOCK_SKEW_NS,
};
pub use frame_schema::{
    FrameSchemaVersion, SchemaV1, FrameSchemaCompat,
};
pub use broker_state::{
    BrokerState, BrokerTimeouts, QueueLimits, BackpressureError,
};
pub use contract_runtime_view::{
    ContractRuntimeView, UsageViolation, ReceiptVerifier, VerificationError,
};
pub use policy_governance::{
    PolicyDelta, PolicyState as GovernancePolicy, PolicySnapshot, PolicyTransitionValidator, TransitionError,
};
pub use advanced_quota_enforcement::{
    LockFreeQuotaBucket, QuotaReservation, QuotaExhausted,
};
pub use deterministic_execution::{
    DeterministicInstruction, DeterministicAuditTrail, DeterministicExecution,
    DeterministicReplayVerifier, AuditTrailFull, SyscallError, ReplayMismatch,
};
pub use pluggable_persistence::{
    PersistenceBackend, InMemoryBackend, ReplicatedBackend, PersistenceConstraint,
    ImmutableAfterWrite, EncryptedAtRest, AuditLogged,
};
pub use type_level_security::{
    Unverified, Verified, Encrypted, Signed, Executable, Immutable, Replicated,
    SecureContext, AllowedEffect, ReadFS, WriteFS, Network, Pure, ExecutionWithEffects,
    IsolationLevel, Shared, ProcessIsolated, ContainerIsolated, IsolatedInvocation,
};
pub use telemetry::{ColorPolicy, TelemetryProfile, VerbosityLevel};
pub use version::{
    ChangeType, ChangeSeverity, CompatibilityLevel, GrammarDelta, NegotiationRequest,
    NegotiationResponse, VersionNegotiator,
};
