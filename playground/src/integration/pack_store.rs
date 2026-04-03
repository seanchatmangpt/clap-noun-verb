//! Pack storage integration layer
//!
//! Handles pack installation, storage, and management on the filesystem.
//! Packs are stored in `.ggen/packs/` within the workspace root.

use crate::domain::pack::Pack;
use crate::integration::workspace::WorkspaceDetector;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Pack storage manager with filesystem persistence
pub struct PackStoreIntegration {
    packs_dir: PathBuf,
}

/// Pack manifest file (pack.json) stored on disk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackManifest {
    pub name: String,
    pub version: String,
    pub dependencies: Vec<String>,
    pub checksum: String,
    pub installed_at: String,
}

impl PackStoreIntegration {
    /// Default packs directory name
    pub const DIRNAME: &'static str = "packs";

    /// Create by detecting workspace root
    pub fn detect() -> Result<Self, String> {
        let workspace_root = WorkspaceDetector::find_workspace_root()?;
        Self::new(workspace_root)
    }

    /// Create for specific workspace root
    pub fn new(workspace_root: PathBuf) -> Result<Self, String> {
        let packs_dir = workspace_root.join(".ggen").join(Self::DIRNAME);
        if !packs_dir.exists() {
            std::fs::create_dir_all(&packs_dir)
                .map_err(|e| format!("Failed to create packs directory: {}", e))?;
        }
        Ok(Self { packs_dir })
    }

    /// Install a pack (save manifest to disk)
    pub fn install_pack(&self, pack: &Pack) -> Result<(), String> {
        let manifest = PackManifest {
            name: pack.name.clone(),
            version: pack.version.clone(),
            dependencies: pack.dependencies.clone(),
            checksum: pack.checksum.clone(),
            installed_at: chrono::Utc::now().to_rfc3339(),
        };
        let manifest_path = self.pack_path(&pack.name);
        let content = serde_json::to_string_pretty(&manifest)
            .map_err(|e| format!("Failed to serialize pack manifest: {}", e))?;
        std::fs::write(&manifest_path, content)
            .map_err(|e| format!("Failed to write pack manifest: {}", e))?;
        Ok(())
    }

    /// Remove a pack by name
    pub fn remove_pack(&self, name: &str) -> Result<(), String> {
        let path = self.pack_path(name);
        if !path.exists() {
            return Err(format!("Pack not found: {}", name));
        }
        std::fs::remove_file(&path).map_err(|e| format!("Failed to remove pack: {}", e))?;
        Ok(())
    }

    /// List all installed packs
    pub fn list_installed(&self) -> Result<Vec<Pack>, String> {
        let mut packs = Vec::new();
        if !self.packs_dir.exists() {
            return Ok(packs);
        }
        for entry in std::fs::read_dir(&self.packs_dir)
            .map_err(|e| format!("Failed to read packs directory: {}", e))?
        {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(manifest) = serde_json::from_str::<PackManifest>(&content) {
                        packs.push(Pack {
                            name: manifest.name,
                            version: manifest.version,
                            dependencies: manifest.dependencies,
                            checksum: manifest.checksum,
                        });
                    }
                }
            }
        }
        Ok(packs)
    }

    /// Check whether a pack is installed
    pub fn is_installed(&self, name: &str) -> bool {
        self.pack_path(name).exists()
    }

    /// Get the path to a pack manifest file
    fn pack_path(&self, name: &str) -> PathBuf {
        self.packs_dir.join(format!("{}.json", name))
    }

    /// Get the packs directory path
    pub fn packs_dir(&self) -> &Path {
        &self.packs_dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_store() -> (PackStoreIntegration, TempDir) {
        let tmp = TempDir::new().unwrap();
        let store = PackStoreIntegration::new(tmp.path().to_path_buf()).unwrap();
        (store, tmp)
    }

    fn sample_pack(name: &str) -> Pack {
        Pack {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            dependencies: vec!["core".to_string()],
            checksum: "abc123".to_string(),
        }
    }

    #[test]
    fn test_install_and_list_packs() {
        let (store, _tmp) = make_store();
        let pack = sample_pack("test-pack");

        store.install_pack(&pack).unwrap();
        let installed = store.list_installed().unwrap();

        assert_eq!(installed.len(), 1);
        assert_eq!(installed[0].name, "test-pack");
        assert_eq!(installed[0].version, "1.0.0");
    }

    #[test]
    fn test_remove_pack() {
        let (store, _tmp) = make_store();
        let pack = sample_pack("removable");

        store.install_pack(&pack).unwrap();
        assert!(store.is_installed("removable"));

        store.remove_pack("removable").unwrap();
        assert!(!store.is_installed("removable"));

        let installed = store.list_installed().unwrap();
        assert!(installed.is_empty());
    }

    #[test]
    fn test_remove_nonexistent_pack_returns_error() {
        let (store, _tmp) = make_store();
        let result = store.remove_pack("no-such-pack");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Pack not found"));
    }

    #[test]
    fn test_is_installed() {
        let (store, _tmp) = make_store();

        assert!(!store.is_installed("missing"));
        store.install_pack(&sample_pack("present")).unwrap();
        assert!(store.is_installed("present"));
    }

    #[test]
    fn test_list_empty_when_no_packs() {
        let (store, _tmp) = make_store();
        let installed = store.list_installed().unwrap();
        assert!(installed.is_empty());
    }

    #[test]
    fn test_install_pack_creates_json_file() {
        let (store, _tmp) = make_store();
        let pack = sample_pack("file-check");

        store.install_pack(&pack).unwrap();

        let manifest_path = store.packs_dir().join("file-check.json");
        assert!(manifest_path.exists());

        let content = std::fs::read_to_string(&manifest_path).unwrap();
        let manifest: PackManifest = serde_json::from_str(&content).unwrap();
        assert_eq!(manifest.name, "file-check");
        assert_eq!(manifest.version, "1.0.0");
        assert!(!manifest.installed_at.is_empty());
    }

    #[test]
    fn test_packs_dir_path() {
        let (store, _tmp) = make_store();
        let dir = store.packs_dir();
        assert!(dir.to_string_lossy().contains(".ggen"));
        assert!(dir.ends_with("packs"));
    }
}
