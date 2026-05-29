mod common;

use clap_noun_verb_utils::{completions, mangen, markdown};
use common::create_test_command;

#[test]
fn test_completions_generation() -> Result<(), String> {
    let mut cmd = create_test_command();
    let mut buf = Vec::new();

    completions::generate_completions(&mut cmd, clap_complete::Shell::Bash, &mut buf);
    let output = String::from_utf8(buf).map_err(|e| format!("UTF8 error: {}", e))?;

    assert!(output.contains("test-cli"));
    assert!(output.contains("complete"));
    Ok(())
}

#[test]
fn test_mangen_generation() -> Result<(), String> {
    let cmd = create_test_command();
    let mut buf = Vec::new();

    mangen::generate_manpage(&cmd, &mut buf)
        .map_err(|e| format!("Man page generation failed: {}", e))?;
    let output = String::from_utf8(buf).map_err(|e| format!("UTF8 error: {}", e))?;

    assert!(output.contains(".TH"));
    assert!(output.contains("test"));
    Ok(())
}

#[test]
fn test_markdown_generation() -> Result<(), String> {
    let cmd = create_test_command();
    let mut buf = Vec::new();

    markdown::generate_markdown(&cmd, &mut buf)
        .map_err(|e| format!("Markdown generation failed: {}", e))?;
    let output = String::from_utf8(buf).map_err(|e| format!("UTF8 error: {}", e))?;

    assert!(output.contains("# test-cli"));
    assert!(output.contains("# Usage"));
    assert!(output.contains("# Arguments / Options"));
    assert!(output.contains("# Subcommands"));
    Ok(())
}
