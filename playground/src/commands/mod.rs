//! CLI command modules
//!
//! Organized by noun: papers, thesis, config, meta
//!
//! Following domain separation: CLI validates, domain computes, integration connects.
//! Each module contains thin CLI wrappers that delegate to domain logic.

pub mod papers;
pub mod thesis;
pub mod config;
pub mod meta;

// ggen v26.4.2 commands
pub mod sync;
pub mod receipt;
pub mod doctor;
pub mod policy;
pub mod registry;
pub mod capability;
pub mod pack;
