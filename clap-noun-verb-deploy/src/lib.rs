//! Deployment adapters for `clap-noun-verb` applications.
//!
//! `clap-noun-verb-deploy` projects one admitted CLI command graph into multiple
//! serving and deployment surfaces without duplicating domain behavior.
//!
//! The crate deliberately separates three concerns:
//! - **SELECT**: inspect the CLI graph and select a callable command.
//! - **CONSTRUCT**: manufacture a validated [`Invocation`] or deployment manifest.
//! - **DO**: execute only through an explicit [`Executor`] supplied by the caller.
//!
//! Kubernetes and OCI modules only render artifacts. They never contact a
//! cluster, registry, container runtime, or network service.

mod executor;
mod gateway;
mod policy;
mod receipt;
mod schema;

#[cfg(feature = "container")]
pub mod container;
#[cfg(feature = "http")]
pub mod http;
#[cfg(feature = "kubernetes")]
pub mod kubernetes;
#[cfg(feature = "mcp")]
pub mod mcp;

pub use executor::{Execution, Executor, Invocation, ProcessExecutionError, ProcessExecutor};
pub use gateway::{Gateway, GatewayError};
pub use policy::{
    Admission, AdmissionError, AdmissionPolicy, AdmitValidated, CommandAllowList,
    EnvironmentAllowList,
};
pub use receipt::{ExecutionRecord, ReplayError, ReplayVerification};
pub use schema::{
    ArgumentBehavior, ArgumentKind, ArgumentSchema, CliSchema, CommandSchema, InvocationBuildError,
    ToolSchema,
};

use clap_noun_verb::CommandRegistry;

/// Transport-neutral deployment projection of a `clap-noun-verb` registry.
#[derive(Debug, Clone)]
pub struct Deploy {
    schema: CliSchema,
}

impl Deploy {
    /// Project an existing registry into an immutable deployable schema.
    ///
    /// This operation is read-only and cannot execute a verb.
    #[must_use]
    pub fn from_registry(registry: &CommandRegistry) -> Self {
        Self { schema: CliSchema::from_command(&registry.build_command()) }
    }

    /// Build a deployment projection from a raw Clap command graph.
    #[must_use]
    pub fn from_command(command: &clap_noun_verb::Command) -> Self {
        Self { schema: CliSchema::from_command(command) }
    }

    /// Return the immutable CLI schema shared by all transports.
    #[must_use]
    pub const fn schema(&self) -> &CliSchema {
        &self.schema
    }

    /// Consume this projection and return its schema.
    #[must_use]
    pub fn into_schema(self) -> CliSchema {
        self.schema
    }
}
