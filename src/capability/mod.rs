// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Capability management, registry, packing, and evidence-backed standing.

pub mod impl_generated;
pub mod pack;
pub mod registry;

pub use impl_generated::{GeneratedHealthChecker, GeneratedRegistry, PackageMetadata};
pub use pack::{PackAddedOutput, PackRemovedOutput};
pub use registry::{CapabilityPackage, CapabilityRegistry, CapabilityStanding, ProofSurface};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capability_package_is_valid_when_identity_is_complete() {
        let package = CapabilityPackage::new("pkg-001", "TestPackage", "1.0.0", "A test package");
        assert!(package.validate().is_ok());

        let invalid = CapabilityPackage::new("", "Test", "1.0.0", "Desc");
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn registry_add_remove_round_trip() {
        let mut registry = CapabilityRegistry::new();
        let package = CapabilityPackage::new("pkg-001", "TestPkg", "1.0.0", "Test");

        registry.add_package(package).expect("package should be admitted");
        assert_eq!(registry.len(), 1);
        assert!(registry.contains("pkg-001"));

        let removed = registry.remove_package("pkg-001").expect("package should exist");
        assert_eq!(removed, "pkg-001");
        assert!(registry.is_empty());
    }
}
