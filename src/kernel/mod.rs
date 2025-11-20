//! Kernel module - Core system functionality

pub mod session;
pub mod attestation;
pub mod quotas;
pub mod capability;

// Re-export key types that exist in kernel::session
pub use session::{
    Session, SessionManager,
};
pub use attestation::{Attestation, AttestationManager};
pub use quotas::{QuotaManager, ResourceQuota};
pub use capability::{Capability, CapabilityManager};

// Note: SessionId, SessionState, SessionMetrics, etc. have moved to autonomic module
// They are re-exported from autonomic for convenience
pub use crate::autonomic::{SessionId, SessionState, SessionManager as AutonomicSessionManager};
