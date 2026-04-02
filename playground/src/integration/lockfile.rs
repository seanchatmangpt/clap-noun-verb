//! Lockfile persistence integration layer
//!
//! Handles reading and writing lockfiles for sync state tracking with
//! workspace detection and version migration support.

use crate::domain::sync::Lockfile;
use crate::integration::workspace::WorkspaceDetector;
use std::path::{Path, PathBuf};

/// Lockfile storage manager with workspace detection and migration
pub struct LockfileStore {
    workspace_root: PathBuf,
}

impl LockfileStore {
    /// Default lockfile filename
    pub const FILENAME: &'static str = "ggen.lock";

    /// Supported lockfile versions for migration
    pub const SUPPORTED_VERSIONS: &[u32] = &[1, 2];

    /// Create new lockfile store by detecting workspace root
    pub fn detect() -> Result<Self, String> {
        let workspace_root = WorkspaceDetector::find_workspace_root()?;
        Ok(Self { workspace_root })
    }

    /// Create new lockfile store for specific workspace root
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }

    /// Get lockfile path
    pub fn path(&self) -> PathBuf {
        self.workspace_root.join(Self::FILENAME)
    }

    /// Get workspace root path
    pub fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }

    /// Load lockfile from workspace with migration
    pub fn load(&self) -> Result<Lockfile, String> {
        let path = self.path();

        if !path.exists() {
            return Err(format!("Lockfile not found: {}", path.display()));
        }

        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read lockfile: {}", e))?;

        // Parse JSON to check version before full deserialization
        let value: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse lockfile JSON: {}", e))?;

        let version = value.get("version")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32)
            .unwrap_or(1);

        // Migrate if needed
        let migrated_content = if !Self::SUPPORTED_VERSIONS.contains(&version) {
            return Err(format!(
                "Unsupported lockfile version {}. Supported versions: {:?}",
                version, Self::SUPPORTED_VERSIONS
            ));
        } else if version < Lockfile::CURRENT_VERSION {
            self.migrate_lockfile(&content, version)?
        } else {
            content
        };

        // Parse migrated content
        serde_json::from_str(&migrated_content)
            .map_err(|e| format!("Failed to parse lockfile: {}", e))
    }

    /// Save lockfile to workspace with atomic write
    pub fn save(&self, lockfile: &Lockfile) -> Result<(), String> {
        let path = self.path();

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create lockfile directory: {}", e))?;
            }
        }

        // Serialize to JSON
        let content = serde_json::to_string_pretty(lockfile)
            .map_err(|e| format!("Failed to serialize lockfile: {}", e))?;

        // Atomic write: write to temp file then rename
        let temp_path = path.with_extension("tmp");
        std::fs::write(&temp_path, content)
            .map_err(|e| format!("Failed to write lockfile: {}", e))?;

        // Atomic rename
        std::fs::rename(&temp_path, &path)
            .map_err(|e| format!("Failed to rename lockfile: {}", e))?;

        Ok(())
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

    /// Get lockfile version without full load
    pub fn get_version(&self) -> Result<Option<u32>, String> {
        let path = self.path();

        if !path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read lockfile: {}", e))?;

        let value: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse lockfile JSON: {}", e))?;

        Ok(value.get("version")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32))
    }

    /// Validate lockfile integrity
    pub fn validate(&self) -> Result<bool, String> {
        let lockfile = self.load()?;

        // Check version
        if lockfile.version > Lockfile::CURRENT_VERSION {
            return Ok(false);
        }

        // Validate required fields
        if lockfile.created_at.is_empty() {
            return Ok(false);
        }

        // Validate packs
        for pack in &lockfile.packs {
            if pack.identifier.is_empty() || pack.version.is_empty() {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Migrate lockfile from older version to current version
    fn migrate_lockfile(&self, content: &str, version: u32) -> Result<String, String> {
        match version {
            1 => {
                // Migration from v1 to v2
                let v1_lockfile: serde_json::Value = serde_json::from_str(content)
                    .map_err(|e| format!("Failed to parse v1 lockfile: {}", e))?;

                // Add new fields for v2
                let mut v2_lockfile = v1_lockfile;
                v2_lockfile["version"] = serde_json::json!(2);

                // Add default values for new fields if missing
                if v2_lockfile.get("git_sha").is_none() {
                    v2_lockfile["git_sha"] = serde_json::Value::Null;
                }

                serde_json::to_string_pretty(&v2_lockfile)
                    .map_err(|e| format!("Failed to serialize migrated lockfile: {}", e))
            }
            _ => Err(format!("No migration path from version {}", version))
        }
    }

    /// Backup lockfile before operations
    pub fn backup(&self) -> Result<PathBuf, String> {
        let path = self.path();

        if !path.exists() {
            return Err("Cannot backup non-existent lockfile".to_string());
        }

        let backup_path = path.with_extension(format!("backup.{}", chrono::Utc::now().timestamp()));
        std::fs::copy(&path, &backup_path)
            .map_err(|e| format!("Failed to backup lockfile: {}", e))?;

        Ok(backup_path)
    }

    /// Restore from backup
    pub fn restore_from_backup(&self, backup_path: &Path) -> Result<(), String> {
        if !backup_path.exists() {
            return Err(format!("Backup file not found: {}", backup_path.display()));
        }

        std::fs::copy(backup_path, self.path())
            .map_err(|e| format!("Failed to restore from backup: {}", e))?;

        Ok(())
    }
}

impl Default for LockfileStore {
    fn default() -> Self {
        Self::detect().unwrap_or_else(|_| {
            Self {
                workspace_root: std::env::current_dir()
                    .unwrap_or_else(|_| PathBuf::from("."))
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_lockfile_store_creates_path() {
        let temp_dir = TempDir::new().unwrap();
        let store = LockfileStore::new(temp_dir.path().to_path_buf());

        let path = store.path();
        assert!(path.ends_with("ggen.lock"));
    }

    #[test]
    fn test_lockfile_store_workspace_root() {
        let temp_dir = TempDir::new().unwrap();
        let store = LockfileStore::new(temp_dir.path().to_path_buf());

        assert_eq!(store.workspace_root(), temp_dir.path());
    }

    #[test]
    fn test_lockfile_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let store = LockfileStore::new(temp_dir.path().to_path_buf());

        let lockfile = Lockfile::new()
            .with_timestamp("2024-01-01T00:00:00Z".to_string())
            .with_policy_profile("test".to_string());

        store.save(&lockfile).unwrap();
        assert!(store.exists());

        let loaded = store.load().unwrap();
        assert_eq!(loaded.policy_profile, "test");
    }

    #[test]
    fn test_lockfile_delete() {
        let temp_dir = TempDir::new().unwrap();
        let store = LockfileStore::new(temp_dir.path().to_path_buf());

        let lockfile = Lockfile::new();
        store.save(&lockfile).unwrap();
        assert!(store.exists());

        store.delete().unwrap();
        assert!(!store.exists());
    }

    #[test]
    fn test_lockfile_validate() {
        let temp_dir = TempDir::new().unwrap();
        let store = LockfileStore::new(temp_dir.path().to_path_buf());

        let lockfile = Lockfile::new()
            .with_timestamp("2024-01-01T00:00:00Z".to_string());

        store.save(&lockfile).unwrap();
        assert!(store.validate().unwrap());
    }

    #[test]
    fn test_lockfile_backup_and_restore() {
        let temp_dir = TempDir::new().unwrap();
        let store = LockfileStore::new(temp_dir.path().to_path_buf());

        let lockfile = Lockfile::new()
            .with_timestamp("2024-01-01T00:00:00Z".to_string())
            .with_policy_profile("original".to_string());

        store.save(&lockfile).unwrap();

        let backup_path = store.backup().unwrap();
        assert!(backup_path.exists());

        // Modify and restore
        let modified = Lockfile::new()
            .with_timestamp("2024-01-02T00:00:00Z".to_string())
            .with_policy_profile("modified".to_string());
        store.save(&modified).unwrap();

        store.restore_from_backup(&backup_path).unwrap();
        let restored = store.load().unwrap();
        assert_eq!(restored.policy_profile, "original");
    }

    #[test]
    fn test_lockfile_get_version() {
        let temp_dir = TempDir::new().unwrap();
        let store = LockfileStore::new(temp_dir.path().to_path_buf());

        assert_eq!(store.get_version().unwrap(), None);

        let lockfile = Lockfile::new();
        store.save(&lockfile).unwrap();

        assert_eq!(store.get_version().unwrap(), Some(Lockfile::CURRENT_VERSION));
    }

    #[test]
    fn test_lockfile_load_nonexistent_returns_error() {
        let temp_dir = TempDir::new().unwrap();
        let store = LockfileStore::new(temp_dir.path().to_path_buf());

        let result = store.load();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_lockfile_default_uses_current_dir() {
        let store = LockfileStore::default();
        // Should not panic
        let _path = store.path();
    }
}
