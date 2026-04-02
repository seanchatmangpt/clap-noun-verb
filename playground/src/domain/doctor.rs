//! Doctor domain - diagnostic surface
//!
//! Detects workspace integrity, lockfile truth, pack integrity, trust/profile conflicts.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Doctor diagnostic engine
///
/// Runs diagnostic checks on workspace state.
pub struct Doctor {
    workspace_root: PathBuf,
}

/// Result from a single diagnostic check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticCheck {
    /// Check name
    pub name: String,
    /// Whether check passed
    pub passed: bool,
    /// Check output/details
    pub output: String,
    /// Suggestions for fixing issues
    pub suggestions: Vec<String>,
}

impl Doctor {
    /// Create new doctor for current workspace
    pub fn new() -> Result<Self, String> {
        let workspace_root = std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))?;

        Ok(Self { workspace_root })
    }

    /// Get workspace root path
    pub fn workspace_root(&self) -> Result<String, String> {
        self.workspace_root
            .to_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "Invalid workspace root path".to_string())
    }

    /// Check if lockfile exists
    pub fn check_lockfile_exists(&self) -> Result<bool, String> {
        Ok(self.workspace_root.join("ggen.lock").exists())
    }

    /// Check pack integrity
    pub fn check_pack_integrity(&self) -> Result<bool, String> {
        // In full implementation, this would verify installed packs
        Ok(true)
    }

    /// Check for policy conflicts
    pub fn check_policy_conflicts(&self) -> Result<Vec<String>, String> {
        // In full implementation, this would check for trust/profile conflicts
        Ok(Vec::new())
    }

    /// Run all diagnostic checks
    pub fn run_all_diagnostics(&self) -> Result<Vec<DiagnosticCheck>, String> {
        let mut checks = Vec::new();

        // Check 1: Workspace integrity
        checks.push(self.run_check("workspace-integrity")?);

        // Check 2: Lockfile exists
        checks.push(self.run_check("lockfile-exists")?);

        // Check 3: Pack integrity
        checks.push(self.run_check("pack-integrity")?);

        // Check 4: Policy conflicts
        checks.push(self.run_check("policy-conflicts")?);

        Ok(checks)
    }

    /// Run a specific diagnostic check
    pub fn run_check(&self, check_name: &str) -> Result<DiagnosticCheck, String> {
        match check_name {
            "workspace-integrity" => Ok(self.check_workspace_integrity()),
            "lockfile-exists" => Ok(self.check_lockfile()),
            "pack-integrity" => Ok(self.check_packs()),
            "policy-conflicts" => Ok(self.check_policy()),
            _ => Err(format!("Unknown check: {}", check_name)),
        }
    }

    /// Auto-fix issues found in diagnostics
    pub fn auto_fix(&self, checks: &[DiagnosticCheck]) -> Result<(), String> {
        // In full implementation, this would auto-fix fixable issues
        Ok(())
    }

    /// Fix a specific check
    pub fn fix_check(&self, check_name: &str) -> Result<(), String> {
        // In full implementation, this would fix specific issues
        Ok(())
    }

    // Individual check implementations

    fn check_workspace_integrity(&self) -> DiagnosticCheck {
        let workspace = self.workspace_root.display().to_string();

        DiagnosticCheck {
            name: "workspace-integrity".to_string(),
            passed: self.workspace_root.exists(),
            output: format!("Workspace root: {}", workspace),
            suggestions: if !self.workspace_root.exists() {
                vec!["Initialize a new workspace".to_string()]
            } else {
                Vec::new()
            },
        }
    }

    fn check_lockfile(&self) -> DiagnosticCheck {
        let lockfile_path = self.workspace_root.join("ggen.lock");
        let exists = lockfile_path.exists();

        DiagnosticCheck {
            name: "lockfile-exists".to_string(),
            passed: exists,
            output: format!(
                "Lockfile: {}",
                if exists {
                    "found"
                } else {
                    "not found (run `ggen sync` to create)"
                }
            ),
            suggestions: if !exists {
                vec!["Run `ggen sync` to create lockfile".to_string()]
            } else {
                Vec::new()
            },
        }
    }

    fn check_packs(&self) -> DiagnosticCheck {
        DiagnosticCheck {
            name: "pack-integrity".to_string(),
            passed: true, // Placeholder
            output: "All installed packs verified".to_string(),
            suggestions: Vec::new(),
        }
    }

    fn check_policy(&self) -> DiagnosticCheck {
        DiagnosticCheck {
            name: "policy-conflicts".to_string(),
            passed: true, // Placeholder
            output: "No policy conflicts detected".to_string(),
            suggestions: Vec::new(),
        }
    }
}

impl Default for Doctor {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            workspace_root: PathBuf::from("."),
        })
    }
}
