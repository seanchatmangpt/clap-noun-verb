// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Executable witness for deterministic shell policy and completion locations.

use clap_noun_verb::shell::{get_completions_dir, line_ending, ShellType};

fn main() {
    let shells = [
        ShellType::Bash,
        ShellType::Zsh,
        ShellType::Fish,
        ShellType::PowerShell,
        ShellType::Elvish,
        ShellType::Unknown,
    ];

    let names: Vec<_> = shells.iter().map(ShellType::as_str).collect();
    assert_eq!(names, vec!["bash", "zsh", "fish", "powershell", "elvish", "unknown"]);
    assert!(ShellType::Bash.supports_command_substitution());
    assert!(!ShellType::PowerShell.supports_command_substitution());
    assert!(ShellType::PowerShell.requires_special_escaping());
    assert_eq!(line_ending(ShellType::PowerShell), "\r\n");
    assert_eq!(line_ending(ShellType::Bash), "\n");
    assert!(get_completions_dir(ShellType::Unknown).is_none());

    println!("Shell policies admitted for {} variants", shells.len());
}
