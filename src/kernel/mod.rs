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

pub mod attestation;
pub mod capability;
pub mod concurrent;
pub mod const_caps;
pub mod grammar;
pub mod grammar_dsl;
pub mod io;
pub mod manpage;
pub mod output;
pub mod quotas;
pub mod session;
pub mod simd;
pub mod telemetry;
pub mod test_harness;
pub mod typestate;
pub mod version;

// Re-export key types for convenience
pub use capability::{
    CapabilityClass, CapabilityContext, CapabilityContract, ResourceBand, SafetyProfile,
    StabilityProfile,
};
pub use grammar::{Grammar, GrammarModel, GrammarNode};
pub use io::{FileIO, InputSource, OutputSink};
pub use manpage::ManpageGenerator;
pub use output::{OutputEnvelope, OutputPipeline, StructuredError, StructuredResult};
pub use session::{
    ControlCommand, Frame, FramePayload, SessionBuilder, SessionConfig, SessionHandle,
    SessionId, SessionMetrics, SessionState, SessionVerb, StreamId,
};
pub use telemetry::{ColorPolicy, TelemetryProfile, VerbosityLevel};
pub use version::{
    ChangeType, ChangeSeverity, CompatibilityLevel, GrammarDelta, NegotiationRequest,
    NegotiationResponse, VersionNegotiator,
};
