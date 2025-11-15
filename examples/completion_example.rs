//! Example: Shell Completion Generation
//!
//! Demonstrates how to generate shell completions for supported shells
//! (bash, zsh, fish, powershell, elvish).

use clap_noun_verb_macros::{noun, verb};
use clap_noun_verb::{Result, generate_completion, Shell};
use clap::Command;
use serde::Serialize;

#[derive(Serialize)]
struct CompletionInfo {
    shell: String,
    installation_path: String,
    instructions: String,
}

/// Show available shells
#[noun("completion", "Shell completion generation")]
#[verb("available")]
fn show_available_shells() -> Result<String> {
    const SHELLS: &[&str] = &["bash", "zsh", "fish", "powershell", "elvish"];
    let info = format!(
        "Supported shells for completion:\n  - {}\n\nUse: myapp completion generate <shell>",
        SHELLS.join("\n  - ")
    );
    Ok(info)
}

/// Generate bash completion
#[noun("completion", "Shell completion generation")]
#[verb("bash")]
fn generate_bash() -> Result<String> {
    // In a real app, you'd create the actual command structure
    let mut cmd = Command::new("myapp")
        .about("Example CLI application")
        .subcommand(Command::new("services"))
        .subcommand(Command::new("users"))
        .subcommand(Command::new("config"));

    let completion = generate_completion(&mut cmd, Shell::Bash, "myapp");
    Ok(completion)
}

/// Generate zsh completion
#[noun("completion", "Shell completion generation")]
#[verb("zsh")]
fn generate_zsh() -> Result<String> {
    let mut cmd = Command::new("myapp")
        .about("Example CLI application")
        .subcommand(Command::new("services"))
        .subcommand(Command::new("users"))
        .subcommand(Command::new("config"));

    let completion = generate_completion(&mut cmd, Shell::Zsh, "myapp");
    Ok(completion)
}

/// Generate fish completion
#[noun("completion", "Shell completion generation")]
#[verb("fish")]
fn generate_fish() -> Result<String> {
    let mut cmd = Command::new("myapp")
        .about("Example CLI application")
        .subcommand(Command::new("services"))
        .subcommand(Command::new("users"))
        .subcommand(Command::new("config"));

    let completion = generate_completion(&mut cmd, Shell::Fish, "myapp");
    Ok(completion)
}

/// Show installation instructions
#[noun("completion", "Shell completion generation")]
#[verb("install")]
fn show_install_instructions(shell: String) -> Result<String> {
    let shell_type = match shell.as_str() {
        "bash" => Shell::Bash,
        "zsh" => Shell::Zsh,
        "fish" => Shell::Fish,
        "powershell" | "posh" => Shell::PowerShell,
        "elvish" => Shell::Elvish,
        _ => return Ok(format!("Unknown shell: {}. Use: bash, zsh, fish, powershell, or elvish", shell)),
    };

    Ok(shell_type.install_instructions("myapp"))
}

/// Show completion file extension for a shell
#[noun("completion", "Shell completion generation")]
#[verb("extension")]
fn show_extension(shell: String) -> Result<String> {
    let ext = match shell.as_str() {
        "bash" => Shell::Bash.file_extension(),
        "zsh" => Shell::Zsh.file_extension(),
        "fish" => Shell::Fish.file_extension(),
        "powershell" => Shell::PowerShell.file_extension(),
        "elvish" => Shell::Elvish.file_extension(),
        _ => return Ok(format!("Unknown shell: {}", shell)),
    };

    Ok(format!("Completion file extension for {}: {}", shell, ext))
}

fn main() -> Result<()> {
    clap_noun_verb::run()
}

// Usage examples:
// $ cargo run --example completion_example -- completion available
// $ cargo run --example completion_example -- completion bash
// $ cargo run --example completion_example -- completion zsh
// $ cargo run --example completion_example -- completion fish
// $ cargo run --example completion_example -- completion install bash
// $ cargo run --example completion_example -- completion extension fish
