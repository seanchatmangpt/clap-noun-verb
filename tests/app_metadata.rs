// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Regression test for `CommandRegistry::set_app_metadata`.
//!
//! Before this, `build_command()` hardcoded the literal name `"cli"` and
//! this crate's own compiled-in `CARGO_PKG_VERSION`, so a consuming binary's
//! `--help`/`--version` output never reflected its own name/version. This
//! is its own test file (not added to an existing one) because
//! `CommandRegistry` is a process-wide singleton (`OnceLock`) — putting it
//! in a shared test binary would make it order-dependent against any other
//! test that touches the registry.

use clap_noun_verb::cli::CommandRegistry;

#[test]
fn set_app_metadata_overrides_name_and_version() {
    CommandRegistry::set_app_metadata("ggen", "26.7.4");

    let registry = CommandRegistry::get();
    let reg = registry.lock().expect("lock registry");
    let cmd = reg.build_command();

    assert_eq!(cmd.get_name(), "ggen");
    assert_eq!(cmd.get_version(), Some("26.7.4"));
}
