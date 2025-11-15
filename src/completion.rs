//! Shell completion generation
//!
//! This module provides utilities to generate shell completions for supported shells.
//!
//! # Supported Shells
//!
//! - bash
//! - zsh
//! - fish
//! - powershell
//! - elvish
//!
//! # Example
//!
//! ```rust,ignore
//! use clap::Command;
//! use clap_noun_verb::completion::{generate_completion, Shell};
//!
//! let cmd = my_cli_command();
//! let completion = generate_completion(&cmd, Shell::Bash, "myapp");
//! println!("{}", completion);
//! ```

use clap::Command;
use std::fmt;
use std::io;

/// Supported shell types for completion generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shell {
    /// Bash shell completion
    Bash,
    /// Zsh shell completion
    Zsh,
    /// Fish shell completion
    Fish,
    /// PowerShell completion
    PowerShell,
    /// Elvish shell completion
    Elvish,
}

impl Shell {
    /// Get the file extension for completion files
    pub fn file_extension(self) -> &'static str {
        match self {
            Shell::Bash => "bash",
            Shell::Zsh => "zsh",
            Shell::Fish => "fish",
            Shell::PowerShell => "ps1",
            Shell::Elvish => "elv",
        }
    }

    /// Get the environment variable name for completion
    pub fn env_var_name(self) -> &'static str {
        match self {
            Shell::Bash => "BASH_COMPLETION_COMPAT_DIR",
            Shell::Zsh => "fpath",
            Shell::Fish => "fish_complete_path",
            Shell::PowerShell => "PSModulePath",
            Shell::Elvish => "ELVISH_LIBRARY_PATH",
        }
    }

    /// Get installation instructions for this shell
    pub fn install_instructions(self, app_name: &str) -> String {
        match self {
            Shell::Bash => format!(
                "# Bash\n\
                 # Add to ~/.bashrc or ~/.bash_profile:\n\
                 eval \"$({} --completions bash)\"\n\
                 # Or source directly:\n\
                 source <({} --completions bash))",
                app_name, app_name
            ),
            Shell::Zsh => format!(
                "# Zsh\n\
                 # Add to ~/.zshrc:\n\
                 eval \"$({} --completions zsh)\"\n\
                 # Or add completion directory to fpath:\n\
                 # mkdir -p ~/.zsh/completions\n\
                 # {} --completions zsh > ~/.zsh/completions/_{}\n\
                 # fpath+=(~/.zsh/completions)",
                app_name, app_name, app_name
            ),
            Shell::Fish => format!(
                "# Fish\n\
                 # Add to ~/.config/fish/config.fish:\n\
                 eval \"$({} --completions fish)\"\n\
                 # Or save to completions directory:\n\
                 # mkdir -p ~/.config/fish/completions\n\
                 # {} --completions fish > ~/.config/fish/completions/{}.fish",
                app_name, app_name, app_name
            ),
            Shell::PowerShell => format!(
                "# PowerShell\n\
                 # Add to PowerShell profile:\n\
                 # & ({{{} --completions powershell}}  | Out-String | Invoke-Expression)",
                app_name
            ),
            Shell::Elvish => format!(
                "# Elvish\n\
                 # Add to ~/.elvish/rc.elv:\n\
                 eval \"$({} --completions elvish)\"",
                app_name
            ),
        }
    }
}

impl fmt::Display for Shell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bash => write!(f, "bash"),
            Self::Zsh => write!(f, "zsh"),
            Self::Fish => write!(f, "fish"),
            Self::PowerShell => write!(f, "powershell"),
            Self::Elvish => write!(f, "elvish"),
        }
    }
}

impl std::str::FromStr for Shell {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bash" => Ok(Shell::Bash),
            "zsh" => Ok(Shell::Zsh),
            "fish" => Ok(Shell::Fish),
            "powershell" | "posh" => Ok(Shell::PowerShell),
            "elvish" => Ok(Shell::Elvish),
            _ => Err(format!(
                "Unknown shell '{}'. Supported: bash, zsh, fish, powershell, elvish",
                s
            )),
        }
    }
}

/// Generate shell completion for the given command
///
/// # Arguments
///
/// * `cmd` - The clap Command to generate completion for
/// * `shell` - Target shell type
/// * `app_name` - Name of the application
///
/// # Returns
///
/// String containing the shell-specific completion script
pub fn generate_completion(cmd: &mut Command, shell: Shell, app_name: &str) -> String {
    use clap_complete::generate;
    use clap_complete::shells;

    let mut completion_buffer = Vec::new();

    match shell {
        Shell::Bash => {
            generate(shells::Bash, cmd, app_name, &mut completion_buffer);
        }
        Shell::Zsh => {
            generate(shells::Zsh, cmd, app_name, &mut completion_buffer);
        }
        Shell::Fish => {
            generate(shells::Fish, cmd, app_name, &mut completion_buffer);
        }
        Shell::PowerShell => {
            generate(shells::PowerShell, cmd, app_name, &mut completion_buffer);
        }
        Shell::Elvish => {
            generate(shells::Elvish, cmd, app_name, &mut completion_buffer);
        }
    }

    String::from_utf8(completion_buffer).unwrap_or_default()
}

/// Generate completion and write to standard output
pub fn print_completion(cmd: &mut Command, shell: Shell, app_name: &str) -> io::Result<()> {
    let completion = generate_completion(cmd, shell, app_name);
    println!("{}", completion);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_display() {
        assert_eq!(Shell::Bash.to_string(), "bash");
        assert_eq!(Shell::Zsh.to_string(), "zsh");
        assert_eq!(Shell::Fish.to_string(), "fish");
    }

    #[test]
    fn test_shell_file_extension() {
        assert_eq!(Shell::Bash.file_extension(), "bash");
        assert_eq!(Shell::Fish.file_extension(), "fish");
        assert_eq!(Shell::PowerShell.file_extension(), "ps1");
    }

    #[test]
    fn test_shell_from_str() {
        assert_eq!("bash".parse::<Shell>().unwrap(), Shell::Bash);
        assert_eq!("FISH".parse::<Shell>().unwrap(), Shell::Fish);
        assert!("invalid".parse::<Shell>().is_err());
    }

    #[test]
    fn test_install_instructions() {
        let instructions = Shell::Bash.install_instructions("myapp");
        assert!(instructions.contains("myapp"));
        assert!(instructions.contains("bash"));
    }
}
