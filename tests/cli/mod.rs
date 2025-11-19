//! Chicago TDD CLI Test Suite for clap-noun-verb
//!
//! This module contains comprehensive CLI tests covering:
//! - Plugin system integration
//! - Kernel capabilities
//! - Middleware execution
//! - I/O operations
//! - Telemetry and observability
//!
//! Test Philosophy (Chicago School TDD):
//! - No mocks - test real components
//! - State-based verification
//! - Integration over unit testing
//! - Arrange-Act-Assert (AAA) pattern

pub mod plugin_cli_tests;
pub mod kernel_cli_tests;
pub mod middleware_cli_tests;
pub mod io_cli_tests;
pub mod telemetry_cli_tests;
pub mod help_system_tests;
