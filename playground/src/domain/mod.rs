//! Domain Logic Layer - Pure, Testable Business Logic
//!
//! This module contains pure functions that implement the core business logic.
//! NO CLI concerns, NO I/O, NO side effects - just pure computation.
//!
//! **The Golden Rule**: Domain logic is pure Rust functions that are:
//! - Testable in isolation (no mocks needed)
//! - Reusable across different interfaces (CLI, API, GUI)
//! - Free of side effects (no file I/O, no network, no printing)

pub mod papers;
pub mod thesis;
pub mod config;

// Re-export commonly used types (only what's actively used by CLI)
pub use papers::{Paper, PaperFamily};
pub use thesis::{ThesisFamily, ThesisSchedule, ThesisStructure};
pub use config::Config;
