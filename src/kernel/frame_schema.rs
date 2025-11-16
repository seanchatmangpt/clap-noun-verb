//! Frame schema version management and backward compatibility
//!
//! Ensures that all frame schema changes are backward compatible
//! and that replay engines only accept compatible versions.

use serde::{Deserialize, Serialize};

/// Frame schema version trait for compile-time schema binding
pub trait FrameSchemaVersion: Send + Sync {
    fn version(&self) -> u32;
    fn is_compatible_with(&self, other_version: u32) -> bool;
}

/// Current schema version (v1)
pub struct SchemaV1;

impl FrameSchemaVersion for SchemaV1 {
    fn version(&self) -> u32 {
        1
    }

    /// V1 can read frames with schema version <= 1
    fn is_compatible_with(&self, other_version: u32) -> bool {
        other_version <= 1
    }
}

/// Schema compatibility checker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameSchemaCompat {
    min_supported_version: u32,
    current_version: u32,
}

impl FrameSchemaCompat {
    /// Create a new compatibility checker
    pub fn new(current_version: u32) -> Self {
        Self {
            min_supported_version: 1,  // v1 is the minimum
            current_version,
        }
    }

    /// Check if a frame schema version is backward compatible
    pub fn is_backward_compatible(&self, frame_version: u32) -> bool {
        frame_version >= self.min_supported_version && frame_version <= self.current_version
    }

    /// Get the version requirements as a readable string
    pub fn version_requirements(&self) -> String {
        format!(
            "Frame schema must be between v{} and v{}",
            self.min_supported_version, self.current_version
        )
    }
}

impl Default for FrameSchemaCompat {
    fn default() -> Self {
        Self::new(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_v1_compatibility() {
        let v1 = SchemaV1;
        assert!(v1.is_compatible_with(1));
        assert!(!v1.is_compatible_with(2));
        assert!(!v1.is_compatible_with(0));
    }

    #[test]
    fn test_frame_schema_compat() {
        let compat = FrameSchemaCompat::new(1);
        assert!(compat.is_backward_compatible(1));
        assert!(!compat.is_backward_compatible(0));
        assert!(!compat.is_backward_compatible(2));
    }

    #[test]
    fn test_compat_version_requirements() {
        let compat = FrameSchemaCompat::new(1);
        let req = compat.version_requirements();
        assert!(req.contains("v1") && req.contains("v1"));
    }
}
