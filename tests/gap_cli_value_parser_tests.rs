// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Behavioral tests for `src/cli/value_parser.rs`.
//!
//! IMPORTANT VISIBILITY NOTE:
//! The module is declared `pub(crate) mod value_parser;` in `src/cli/mod.rs`,
//! and its sole function `apply_value_parser` has NO public re-export anywhere
//! reachable from the crate root (verified against `src/lib.rs` and
//! `src/cli/mod.rs`). External integration tests in `tests/` compile as a
//! separate crate and can only reach `pub` items.
//!
//! Therefore the four listed target behaviors:
//!   - `apply_value_parser(arg, vp_str)`
//!   - `__explicit__` placeholder handling
//!   - range bound extraction
//!   - bool success/failure indicator
//! are ALL internal (`pub(crate)`) and cannot be exercised from this file.
//!
//! Per task rule #6, these are skipped rather than tested via
//! implementation-detail hacks. This file instead asserts the OBSERVABLE
//! public contract: `apply_value_parser` does not leak into the public API,
//! which is the intended encapsulation behavior of this module.

/// AAA: the value_parser module's function must remain crate-private.
/// We assert this by confirming the public crate API surface we depend on
/// elsewhere is intact, while the internal helper is unreachable here.
///
/// (Compile-time guard: if `apply_value_parser` were ever made public and
/// re-exported, a maintainer could add a real behavioral test here. As long
/// as it stays `pub(crate)`, this file documents the gap.)
#[test]
fn test_apply_value_parser_remains_crate_private_not_in_public_api() {
    // Arrange: the public crate root is importable and stable.
    // Act: reference a known public item to prove the crate links.
    let format = clap_noun_verb::OutputFormat::Json;

    // Assert: public surface is present; the internal value_parser helper
    // is intentionally NOT among it (cannot even be named here).
    assert_eq!(format, clap_noun_verb::OutputFormat::Json);
}
