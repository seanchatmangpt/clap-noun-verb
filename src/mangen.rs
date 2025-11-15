//! Man page generation for CLI applications
//!
//! This module provides utilities for generating Unix man pages from clap command definitions.
//!
//! # Example
//!
//! ```rust,ignore
//! use clap_noun_verb::mangen::generate_man_page;
//! use clap::Command;
//!
//! let cmd = Command::new("myapp");
//! let man_page = generate_man_page(&cmd, "myapp")?;
//! println!("{}", man_page);
//! ```

use crate::Result;
use clap::Command;
use std::io::Write;

/// Generate a man page from a clap Command
///
/// # Arguments
///
/// * `cmd` - The clap Command to generate documentation for
/// * `app_name` - The name of the application
///
/// # Returns
///
/// A String containing the man page in groff format
pub fn generate_man_page(cmd: &Command, app_name: &str) -> Result<String> {
    use clap_mangen::Man;

    let mut cmd_copy = cmd.clone();
    let mut buffer = Vec::new();

    let man = Man::new(cmd_copy);
    man.render(&mut buffer).map_err(|e| {
        crate::error::NounVerbError::execution_error(format!(
            "Failed to generate man page: {}",
            e
        ))
    })?;

    String::from_utf8(buffer).map_err(|e| {
        crate::error::NounVerbError::execution_error(format!(
            "Man page generation produced invalid UTF-8: {}",
            e
        ))
    })
}

/// Generate and write man page to a file
///
/// # Arguments
///
/// * `cmd` - The clap Command to document
/// * `app_name` - Application name
/// * `output_path` - Path where man page should be written
pub fn write_man_page(cmd: &Command, app_name: &str, output_path: &str) -> Result<()> {
    let man_content = generate_man_page(cmd, app_name)?;
    let mut file = std::fs::File::create(output_path).map_err(|e| {
        crate::error::NounVerbError::execution_error(format!(
            "Failed to create man page file: {}",
            e
        ))
    })?;

    file.write_all(man_content.as_bytes()).map_err(|e| {
        crate::error::NounVerbError::execution_error(format!(
            "Failed to write man page: {}",
            e
        ))
    })?;

    Ok(())
}

/// Generate man pages for all subcommands recursively
///
/// # Arguments
///
/// * `cmd` - The root command
/// * `app_name` - Application name
/// * `output_dir` - Directory to write man pages to
pub fn generate_all_man_pages(cmd: &Command, app_name: &str, output_dir: &str) -> Result<()> {
    // Create output directory if it doesn't exist
    std::fs::create_dir_all(output_dir).map_err(|e| {
        crate::error::NounVerbError::execution_error(format!(
            "Failed to create output directory: {}",
            e
        ))
    })?;

    // Generate main command man page
    let man_path = format!("{}/{}.1", output_dir, app_name);
    write_man_page(cmd, app_name, &man_path)?;

    // Generate man pages for subcommands
    for subcmd in cmd.get_subcommands() {
        generate_subcommand_man_pages(subcmd, app_name, output_dir)?;
    }

    Ok(())
}

/// Helper to recursively generate man pages for subcommands
fn generate_subcommand_man_pages(
    cmd: &Command,
    parent_name: &str,
    output_dir: &str,
) -> Result<()> {
    let cmd_name = cmd.get_name();
    let full_name = format!("{}-{}", parent_name, cmd_name);

    // Generate man page for this command
    let man_path = format!("{}/{}.1", output_dir, full_name);
    write_man_page(cmd, &full_name, &man_path)?;

    // Recursively process sub-subcommands
    for subcmd in cmd.get_subcommands() {
        generate_subcommand_man_pages(subcmd, &full_name, output_dir)?;
    }

    Ok(())
}

/// Get man page directory for the system
///
/// # Returns
///
/// Common man page directory path (e.g., /usr/local/share/man/man1)
pub fn get_man_dir() -> &'static str {
    if cfg!(target_os = "macos") {
        "/usr/local/share/man/man1"
    } else {
        "/usr/share/man/man1"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_man_dir() {
        let man_dir = get_man_dir();
        assert!(man_dir.contains("man1"));
    }

    #[test]
    fn test_generate_man_page() {
        let cmd = Command::new("test-app")
            .about("Test application")
            .subcommand(Command::new("test").about("Test subcommand"));

        let result = generate_man_page(&cmd, "test-app");
        assert!(result.is_ok());

        if let Ok(content) = result {
            // Basic sanity checks
            assert!(!content.is_empty());
            assert!(content.contains("test-app") || content.contains("Test application"));
        }
    }
}
