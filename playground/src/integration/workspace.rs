//! Workspace detection for finding project roots
//!
//! This module provides utilities for finding workspace roots by searching upward
//! for marker files like `ggen.toml` or `.git`.

use std::path::{Path, PathBuf};

/// Workspace detector for finding project roots
pub struct WorkspaceDetector;

impl WorkspaceDetector {
    /// Find the workspace root by searching upward for `ggen.toml` or `.git`
    ///
    /// # Returns
    /// - `Ok(PathBuf)` - Path to the workspace root
    /// - `Err(String)` - Error message if workspace root cannot be found
    ///
    /// # Examples
    /// ```
    /// use crate::integration::workspace::WorkspaceDetector;
    ///
    /// let root = WorkspaceDetector::find_workspace_root().unwrap();
    /// println!("Workspace root: {:?}", root);
    /// ```
    pub fn find_workspace_root() -> Result<PathBuf, String> {
        // Start from current directory
        let current_dir = std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))?;

        Self::find_workspace_root_from(&current_dir)
    }

    /// Find the workspace root starting from a given directory
    ///
    /// # Arguments
    /// * `start_dir` - Directory to start searching from
    ///
    /// # Returns
    /// - `Ok(PathBuf)` - Path to the workspace root
    /// - `Err(String)` - Error message if workspace root cannot be found
    pub fn find_workspace_root_from(start_dir: &Path) -> Result<PathBuf, String> {
        let mut current = Some(start_dir);

        while let Some(dir) = current {
            // Check for ggen.toml (primary marker)
            let ggen_toml = dir.join("ggen.toml");
            if ggen_toml.exists() {
                return Ok(dir.to_path_buf());
            }

            // Check for .git (secondary marker)
            let git_dir = dir.join(".git");
            if git_dir.exists() {
                return Ok(dir.to_path_buf());
            }

            // Move to parent directory
            current = dir.parent();
        }

        Err("Could not find workspace root (no ggen.toml or .git found)".to_string())
    }

    /// Find the ggen.toml configuration file
    ///
    /// # Returns
    /// - `Ok(PathBuf)` - Path to ggen.toml
    /// - `Err(String)` - Error message if ggen.toml cannot be found
    pub fn find_ggen_config() -> Result<PathBuf, String> {
        let workspace_root = Self::find_workspace_root()?;
        let config_path = workspace_root.join("ggen.toml");

        if config_path.exists() {
            Ok(config_path)
        } else {
            Err(format!("ggen.toml not found in workspace root: {:?}", workspace_root))
        }
    }

    /// Get the .ggen directory path (for receipts, state, etc.)
    ///
    /// # Returns
    /// - `PathBuf` - Path to the .ggen directory
    pub fn ggen_dir() -> PathBuf {
        let workspace_root = Self::find_workspace_root()
            .unwrap_or_else(|_| std::env::current_dir().unwrap());
        workspace_root.join(".ggen")
    }

    /// Get the receipts directory path
    ///
    /// # Returns
    /// - `PathBuf` - Path to the receipts directory
    pub fn receipts_dir() -> PathBuf {
        Self::ggen_dir().join("receipts")
    }

    /// Get the packs directory path
    ///
    /// # Returns
    /// - `PathBuf` - Path to the packs directory
    pub fn packs_dir() -> PathBuf {
        Self::ggen_dir().join("packs")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_workspace_root_from_current_dir() {
        // This test assumes we're in a git repository
        let result = WorkspaceDetector::find_workspace_root();
        assert!(result.is_ok(), "Should find workspace root from current directory");

        let root = result.unwrap();
        assert!(root.exists(), "Workspace root should exist");
    }

    #[test]
    fn test_ggen_dir_returns_path() {
        let ggen_dir = WorkspaceDetector::ggen_dir();
        assert!(ggen_dir.ends_with(".ggen"));
    }

    #[test]
    fn test_receipts_dir_returns_path() {
        let receipts_dir = WorkspaceDetector::receipts_dir();
        let dir_str = receipts_dir.to_string_lossy();
        assert!(dir_str.contains(".ggen"));
        assert!(dir_str.contains("receipts"));
    }

    #[test]
    fn test_packs_dir_returns_path() {
        let packs_dir = WorkspaceDetector::packs_dir();
        let dir_str = packs_dir.to_string_lossy();
        assert!(dir_str.contains(".ggen"));
        assert!(dir_str.contains("packs"));
    }
}
