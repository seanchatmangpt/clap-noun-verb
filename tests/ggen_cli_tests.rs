//! Comprehensive CLI Test Suite for clap-noun-verb (ggen subsystem)
//!
//! This is the main test harness that includes all Chicago TDD-style CLI tests
//! covering plugins, kernel, middleware, I/O, and telemetry subsystems.

// Include test modules
mod cli;

// Re-export test functions for organization
pub use cli::*;
