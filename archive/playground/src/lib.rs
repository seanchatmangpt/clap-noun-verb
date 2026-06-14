// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! MCPP CLI Library

pub mod commands;
pub mod domain;
pub mod integration;
pub mod outputs;

#[cfg(feature = "agent-sandbox")]
pub mod sandbox {
    pub use crate::integration::sandbox::{MockRegistryDatabase, SyntheticCommandExecutor, MOCK_REGISTRY};
}

pub fn init() {
    // Forces linking
}


