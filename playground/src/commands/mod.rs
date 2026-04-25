//! CLI command modules
//!
//! Organized by MCPP grammar: doctor, wizard, telco, powl8, receipt, spec, policy, ontology
//!
//! Following domain separation: CLI validates, domain computes, integration connects.
//! Each module contains thin CLI wrappers that delegate to domain logic.

pub mod doctor;
pub mod wizard;
pub mod telco;
pub mod powl8;
pub mod receipt;
pub mod spec;
pub mod policy;
pub mod ontology;
pub mod verify;
pub mod accept;

// Deprecated or legacy commands to be refactored
pub mod sync;
pub mod registry;
pub mod capability;
pub mod pack;
pub mod papers;
pub mod thesis;
pub mod config;
pub mod meta;

