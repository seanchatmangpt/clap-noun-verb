// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Deterministic capability registry with evidence-backed standing.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Operational standing for a capability package.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CapabilityStanding {
    /// The capability has not been admitted or executed.
    #[default]
    Unknown,
    /// Some bounded proof surfaces execute, but the crown is incomplete.
    PartialAlive,
    /// All declared proof surfaces were observed and replayed.
    Alive,
    /// A typed dependency or environmental blocker prevents execution.
    Blocked,
    /// The admitted source exists but the build is broken.
    BuildBroken,
    /// The capability is outside the supported boundary.
    Unsupported,
}

/// One executable proof surface for a capability.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProofSurface {
    /// Stable proof-surface name.
    pub name: String,
    /// Verification rung such as unit, integration, e2e, chaos, stress, or replay.
    pub rung: String,
    /// Receipt identifier or content hash.
    pub receipt: String,
    /// Whether execution was directly observed.
    pub observed: bool,
    /// Whether replay reproduced the admitted consequence.
    pub replay_verified: bool,
}

impl ProofSurface {
    /// Construct one proof surface.
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        rung: impl Into<String>,
        receipt: impl Into<String>,
        observed: bool,
        replay_verified: bool,
    ) -> Self {
        Self {
            name: name.into(),
            rung: rung.into(),
            receipt: receipt.into(),
            observed,
            replay_verified,
        }
    }

    /// Whether this proof surface has standing.
    #[must_use]
    pub fn is_alive(&self) -> bool {
        self.observed && self.replay_verified && !self.receipt.trim().is_empty()
    }
}

/// Capability package metadata and its admitted proof closure.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityPackage {
    /// Unique package identifier.
    pub id: String,
    /// Package name.
    pub name: String,
    /// Package version.
    pub version: String,
    /// Package description.
    pub description: String,
    /// Ontology-owned noun default verb, when one exists.
    #[serde(default)]
    pub default_verb: Option<String>,
    /// Current evidence-derived standing.
    #[serde(default)]
    pub standing: CapabilityStanding,
    /// Executable proof surfaces.
    #[serde(default)]
    pub proof_surfaces: Vec<ProofSurface>,
    /// Capability IDs required by this package.
    #[serde(default)]
    pub dependencies: Vec<String>,
}

impl CapabilityPackage {
    /// Create a capability package from its id, name, version, and description.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        version: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            version: version.into(),
            description: description.into(),
            default_verb: None,
            standing: CapabilityStanding::Unknown,
            proof_surfaces: Vec::new(),
            dependencies: Vec::new(),
        }
    }

    /// Bind an ontology-owned default verb.
    #[must_use]
    pub fn with_default_verb(mut self, verb: impl Into<String>) -> Self {
        self.default_verb = Some(verb.into());
        self
    }

    /// Add one dependency if it is not already present.
    #[must_use]
    pub fn with_dependency(mut self, capability_id: impl Into<String>) -> Self {
        let capability_id = capability_id.into();
        if !self.dependencies.contains(&capability_id) {
            self.dependencies.push(capability_id);
            self.dependencies.sort();
        }
        self
    }

    /// Record or replace one proof surface by stable name and rung.
    pub fn record_proof(&mut self, proof: ProofSurface) -> Result<(), String> {
        if proof.name.trim().is_empty() || proof.rung.trim().is_empty() {
            return Err("Proof surface name and rung cannot be empty".to_string());
        }
        if let Some(existing) = self
            .proof_surfaces
            .iter_mut()
            .find(|item| item.name == proof.name && item.rung == proof.rung)
        {
            *existing = proof;
        } else {
            self.proof_surfaces.push(proof);
        }
        self.proof_surfaces
            .sort_by(|left, right| (&left.rung, &left.name).cmp(&(&right.rung, &right.name)));
        self.refresh_standing();
        Ok(())
    }

    /// Recompute standing strictly from observed proof surfaces.
    pub fn refresh_standing(&mut self) {
        let alive = self.proof_surfaces.iter().filter(|proof| proof.is_alive()).count();
        self.standing = if self.proof_surfaces.is_empty() {
            CapabilityStanding::Unknown
        } else if alive == self.proof_surfaces.len() {
            CapabilityStanding::Alive
        } else if alive > 0 {
            CapabilityStanding::PartialAlive
        } else {
            CapabilityStanding::Blocked
        };
    }

    /// Validate package metadata and standing invariants.
    pub fn validate(&self) -> Result<(), String> {
        if self.id.trim().is_empty() {
            return Err("Package ID cannot be empty".to_string());
        }
        if self.name.trim().is_empty() {
            return Err("Package name cannot be empty".to_string());
        }
        if self.version.trim().is_empty() {
            return Err("Package version cannot be empty".to_string());
        }
        if self.default_verb.as_deref().is_some_and(str::is_empty) {
            return Err("Default verb cannot be empty".to_string());
        }
        if self.dependencies.iter().any(|dependency| dependency.trim().is_empty()) {
            return Err("Capability dependencies cannot be empty".to_string());
        }
        if self.standing == CapabilityStanding::Alive
            && (self.proof_surfaces.is_empty()
                || self.proof_surfaces.iter().any(|proof| !proof.is_alive()))
        {
            return Err("ALIVE standing requires observed and replayed proof surfaces".to_string());
        }
        Ok(())
    }

    /// Number of declared proof surfaces.
    #[must_use]
    pub fn proof_surface_count(&self) -> usize {
        self.proof_surfaces.len()
    }
}

/// Ordered capability registry. Iteration and serialization are byte-stable.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CapabilityRegistry {
    packages: BTreeMap<String, CapabilityPackage>,
}

impl CapabilityRegistry {
    /// Create a new empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a package. Duplicate identifiers are refused rather than overwritten.
    pub fn add_package(&mut self, mut package: CapabilityPackage) -> Result<(), String> {
        package.refresh_standing();
        package.validate()?;
        if self.packages.contains_key(&package.id) {
            return Err(format!("Package already exists: {}", package.id));
        }
        self.packages.insert(package.id.clone(), package);
        Ok(())
    }

    /// Replace an existing package after validating its complete proof closure.
    pub fn update_package(&mut self, mut package: CapabilityPackage) -> Result<(), String> {
        package.refresh_standing();
        package.validate()?;
        if !self.packages.contains_key(&package.id) {
            return Err(format!("Package not found: {}", package.id));
        }
        self.packages.insert(package.id.clone(), package);
        Ok(())
    }

    /// Remove a package from the registry.
    pub fn remove_package(&mut self, id: &str) -> Result<String, String> {
        self.packages
            .remove(id)
            .map(|package| package.id)
            .ok_or_else(|| format!("Package not found: {id}"))
    }

    /// Resolve one package by exact ID.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&CapabilityPackage> {
        self.packages.get(id)
    }

    /// Get all packages in canonical ID order.
    #[must_use]
    pub fn packages(&self) -> Vec<CapabilityPackage> {
        self.packages.values().cloned().collect()
    }

    /// Return a dependency-closed package order or a typed cycle/missing refusal.
    pub fn dependency_order(&self) -> Result<Vec<String>, String> {
        fn visit(
            id: &str,
            packages: &BTreeMap<String, CapabilityPackage>,
            temporary: &mut Vec<String>,
            permanent: &mut Vec<String>,
            ordered: &mut Vec<String>,
        ) -> Result<(), String> {
            if permanent.iter().any(|item| item == id) {
                return Ok(());
            }
            if temporary.iter().any(|item| item == id) {
                return Err(format!("Capability dependency cycle detected at: {id}"));
            }
            let package =
                packages.get(id).ok_or_else(|| format!("Capability dependency not found: {id}"))?;
            temporary.push(id.to_string());
            for dependency in &package.dependencies {
                visit(dependency, packages, temporary, permanent, ordered)?;
            }
            temporary.retain(|item| item != id);
            permanent.push(id.to_string());
            ordered.push(id.to_string());
            Ok(())
        }

        let mut temporary = Vec::new();
        let mut permanent = Vec::new();
        let mut ordered = Vec::new();
        for id in self.packages.keys() {
            visit(id, &self.packages, &mut temporary, &mut permanent, &mut ordered)?;
        }
        Ok(ordered)
    }

    /// Get package count.
    #[must_use]
    pub fn len(&self) -> usize {
        self.packages.len()
    }

    /// Check whether the registry is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.packages.is_empty()
    }

    /// Check whether a package exists.
    #[must_use]
    pub fn contains(&self, id: &str) -> bool {
        self.packages.contains_key(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_requires_replay_before_alive() {
        let mut package = CapabilityPackage::new("pkg-001", "Graph", "1.0.0", "Graph operations");
        package
            .record_proof(ProofSurface::new("unit", "unit", "receipt-1", true, false))
            .expect("valid proof");
        assert_eq!(package.standing, CapabilityStanding::Blocked);
        package
            .record_proof(ProofSurface::new("unit", "unit", "receipt-1", true, true))
            .expect("valid proof");
        assert_eq!(package.standing, CapabilityStanding::Alive);
    }

    #[test]
    fn registry_refuses_duplicate_ids() {
        let mut registry = CapabilityRegistry::new();
        let package = CapabilityPackage::new("pkg-001", "TestPkg", "1.0.0", "Test");
        registry.add_package(package.clone()).expect("first insert");
        assert!(registry.add_package(package).is_err());
    }

    #[test]
    fn dependency_order_is_closed_and_stable() {
        let mut registry = CapabilityRegistry::new();
        registry.add_package(CapabilityPackage::new("core", "Core", "1", "Core")).expect("core");
        registry
            .add_package(CapabilityPackage::new("cli", "CLI", "1", "CLI").with_dependency("core"))
            .expect("cli");
        assert_eq!(registry.dependency_order().expect("closed"), vec!["core", "cli"]);
    }
}
