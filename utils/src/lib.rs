// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod adapters;
pub mod completions;
pub mod display_json;
pub mod help;
pub mod mangen;
pub mod markdown;
pub mod number_parsing;
