use clap::Command;
use clap_complete::Generator;
use std::io::Write;

/// A trait wrapping `clap_complete::Generator` to satisfy interface contracts.
///
/// # Examples
///
/// ```
/// use clap_noun_verb_utils::completions::Shell;
///
/// fn check_shell<S: Shell>(_shell: S) {}
/// check_shell(clap_complete::Shell::Bash);
/// ```
pub trait Shell: Generator {}

impl Shell for clap_complete::Shell {}

/// Generate shell completions (Bash, Zsh, Fish, PowerShell) using `clap_complete`.
///
/// # Examples
///
/// ```
/// use clap::Command;
/// use clap_noun_verb_utils::completions::{generate_completions, Shell};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut cmd = Command::new("myapp");
/// let mut buffer = Vec::new();
/// generate_completions(&mut cmd, clap_complete::Shell::Bash, &mut buffer);
///
/// let output = String::from_utf8(buffer)?;
/// assert!(output.contains("myapp"));
/// # Ok(())
/// # }
/// ```
pub fn generate_completions<S: Shell>(cmd: &mut Command, shell: S, buf: &mut dyn Write) {
    let name = cmd.get_name().to_string();
    clap_complete::generate(shell, cmd, name, buf);
}
