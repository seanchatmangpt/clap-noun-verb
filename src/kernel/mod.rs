//! Kernel module - Core system functionality

pub mod attestation;
pub mod capability;
pub mod quotas;
pub mod session;

// Re-export key types that exist in kernel::session
pub use attestation::{Attestation, AttestationManager};
pub use capability::{Capability, CapabilityManager};
pub use quotas::{QuotaManager, ResourceQuota};
pub use session::{Session, SessionManager};

// Note: SessionId, SessionState, SessionMetrics, etc. have moved to autonomic module
// They are re-exported from autonomic for convenience
pub use crate::autonomic::{SessionId, SessionManager as AutonomicSessionManager, SessionState};
