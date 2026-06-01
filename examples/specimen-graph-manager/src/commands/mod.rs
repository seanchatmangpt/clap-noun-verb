// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! CLI Command Handlers
//!
//! Each command module contains a single #[verb] handler function
//! that is auto-discovered at compile time via linkme distributed slices.

pub mod doctor_check;
pub mod graph_load;
pub mod graph_query;
pub mod graph_validate;
pub mod pack_add;
pub mod pack_remove;
