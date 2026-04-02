//! Lockfile persistence integration layer
//!
//! Handles reading and writing lockfiles for sync state tracking.

use crate::domain::sync::Lockfile;
use std::path::{Path, PathBuf};

/// Lockfile storage manager
pub struct LockfileStore {
    workspace_root: PathBuf,
}

impl LockfileStore {
    /// Default lockfile filename
    pub const FILENAME: &'static str = "ggen.lock";

    /// Create new lockfile store for workspace
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }

    /// Get lockfile path
    pub fn path(&self) -> PathBuf {
        self.workspace_root.join(Self::FILENAME)
    }

    /// Load lockfile from workspace
    pub fn load(&self) -> Result<Lockfile, String> {
        Lockfile::load(&self.path())
    }

    /// Save lockfile to workspace
    pub fn save(&self, lockfile: &Lockfile) -> Result<(), String> {
        lockfile.save(&self.path())
    }

    /// Check if lockfile exists
    pub fn exists(&self) -> bool {
        self.path().exists()
    }

    /// Delete lockfile
    pub fn delete(&self) -> Result<(), String> {
        if self.exists() {
            std::fs::remove_file(&self.path())
                .map_err(|e| format!("Failed to delete lockfile: {}", e))?;
        }
        Ok(())
    }
}
