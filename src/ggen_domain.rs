//! Stub module for ggen-domain (Future implementation)
//! This module provides placeholder types for the ggen integration feature.

/// Stub error type for ggen-domain
#[derive(Debug, Clone)]
pub struct Error(pub String);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ggen-domain error: {}", self.0)
    }
}

impl std::error::Error for Error {}

/// Stub generation module
pub mod generation {}

/// Stub project module
pub mod project {}

/// Stub template module
pub mod template {}
