// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Executable witness for `Deprecation` and semantic-version removal boundaries.

use clap_noun_verb::{Deprecation, DeprecationType};

fn main() {
    let deprecation = Deprecation::new(DeprecationType::Verb)
        .since("26.7.0")
        .removed_in("27.0.0")
        .note("The legacy route bypasses capability standing")
        .suggestion("Use `capability verify` instead");

    let warning = deprecation.warning_message("legacy-check");
    assert!(warning.contains("legacy-check"));
    assert!(warning.contains("26.7.0"));
    assert!(warning.contains("27.0.0"));
    assert!(warning.contains("capability verify"));

    assert!(!deprecation.is_removable("27.0.0-rc.1"));
    assert!(deprecation.is_removable("27.0.0"));
    assert!(deprecation.is_removable("27.1.0"));

    let help = deprecation.help_text("legacy-check");
    assert!(help.starts_with("[DEPRECATED since v26.7.0]"));
    println!("{help}");
}
