//! Schema Registry for Grammar Versions (CNV 4.1.0)
//!
//! Immutable schema versioning with merkle-tree based integrity checking.
//! Enables safe grammar evolution across trillion-agent deployments.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Schema version identifier
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SchemaVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl SchemaVersion {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch }
    }

    pub fn to_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }

    /// Semantic versioning comparison
    pub fn is_compatible_with(&self, other: &Self) -> bool {
        // Same major version means compatible
        self.major == other.major
    }
}

/// Merkle tree node for schema integrity verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleNode {
    pub hash: String,
    pub leaf_count: usize,
    pub children: Vec<MerkleNode>,
}

impl MerkleNode {
    pub fn new_leaf(hash: String) -> Self {
        Self {
            hash,
            leaf_count: 1,
            children: Vec::new(),
        }
    }

    pub fn new_branch(left: MerkleNode, right: MerkleNode) -> Self {
        let combined_hash = sha256_hash(&format!("{}{}", left.hash, right.hash));
        Self {
            hash: combined_hash,
            leaf_count: left.leaf_count + right.leaf_count,
            children: vec![left, right],
        }
    }

    pub fn root_hash(&self) -> &str {
        &self.hash
    }
}

/// Grammar schema entry in the registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaEntry {
    pub version: SchemaVersion,
    pub schema_json: String,
    pub root_hash: String,
    pub merkle_tree: MerkleNode,
    pub timestamp_ns: u64,
    pub author: String,
    pub changelog: String,
}

impl SchemaEntry {
    pub fn new(
        version: SchemaVersion,
        schema_json: String,
        author: String,
        changelog: String,
    ) -> Self {
        let root_hash = sha256_hash(&schema_json);
        let merkle_tree = MerkleNode::new_leaf(root_hash.clone());

        Self {
            version,
            schema_json,
            root_hash,
            merkle_tree,
            timestamp_ns: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
            author,
            changelog,
        }
    }

    /// Verify integrity of schema
    pub fn verify(&self) -> bool {
        let computed_hash = sha256_hash(&self.schema_json);
        computed_hash == self.root_hash
    }
}

/// Compatibility relationship between schema versions
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CompatibilityType {
    /// Fully backward compatible
    FullyCompatible,
    /// Breaking changes present
    Breaking,
    /// Requires migration path
    RequiresMigration,
    /// Incompatible
    Incompatible,
}

/// Schema evolution rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionRule {
    pub from_version: SchemaVersion,
    pub to_version: SchemaVersion,
    pub compatibility: CompatibilityType,
    pub migration_script: Option<String>,
    pub breaking_changes: Vec<String>,
}

/// Central schema registry
/// Thread-safe, append-only log of schema versions
pub struct SchemaRegistry {
    entries: Arc<std::sync::RwLock<HashMap<SchemaVersion, SchemaEntry>>>,
    evolution_rules: Arc<std::sync::RwLock<Vec<EvolutionRule>>>,
    merkle_root: Arc<std::sync::RwLock<Option<MerkleNode>>>,
}

impl SchemaRegistry {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(std::sync::RwLock::new(HashMap::new())),
            evolution_rules: Arc::new(std::sync::RwLock::new(Vec::new())),
            merkle_root: Arc::new(std::sync::RwLock::new(None)),
        }
    }

    /// Register a new schema version (append-only)
    pub fn register_schema(&self, entry: SchemaEntry) -> Result<(), String> {
        // Verify integrity before registering
        if !entry.verify() {
            return Err("Schema integrity check failed".to_string());
        }

        let mut entries = self.entries.write().unwrap();

        // Check if version already registered (immutable)
        if entries.contains_key(&entry.version) {
            return Err(format!("Schema version {} already registered", entry.version.to_string()));
        }

        entries.insert(entry.version.clone(), entry);

        // Recompute merkle root
        self.update_merkle_root();

        Ok(())
    }

    /// Get schema entry by version
    pub fn get_schema(&self, version: &SchemaVersion) -> Option<SchemaEntry> {
        self.entries.read().unwrap().get(version).cloned()
    }

    /// List all registered schema versions
    pub fn list_versions(&self) -> Vec<SchemaVersion> {
        let mut versions: Vec<_> = self
            .entries
            .read()
            .unwrap()
            .keys()
            .cloned()
            .collect();
        versions.sort_by(|a, b| {
            (b.major, b.minor, b.patch).cmp(&(a.major, a.minor, a.patch))
        });
        versions
    }

    /// Define compatibility relationship
    pub fn add_evolution_rule(&self, rule: EvolutionRule) -> Result<(), String> {
        // Validate rule makes sense
        if !rule.from_version.is_compatible_with(&rule.to_version)
            && rule.compatibility != CompatibilityType::Breaking
        {
            return Err("Incompatible evolution must be marked as Breaking".to_string());
        }

        let mut rules = self.evolution_rules.write().unwrap();
        rules.push(rule);
        Ok(())
    }

    /// Check compatibility between two versions
    pub fn check_compatibility(
        &self,
        from: &SchemaVersion,
        to: &SchemaVersion,
    ) -> Option<CompatibilityType> {
        let rules = self.evolution_rules.read().unwrap();

        for rule in rules.iter() {
            if rule.from_version == *from && rule.to_version == *to {
                return Some(rule.compatibility);
            }
        }

        // Default compatibility logic
        if from == to {
            return Some(CompatibilityType::FullyCompatible);
        }

        if from.major == to.major {
            // Same major = compatible
            Some(CompatibilityType::FullyCompatible)
        } else {
            // Different major = breaking
            Some(CompatibilityType::Breaking)
        }
    }

    /// Get migration path from one version to another
    pub fn get_migration_path(
        &self,
        from: &SchemaVersion,
        to: &SchemaVersion,
    ) -> Option<Vec<EvolutionRule>> {
        let rules = self.evolution_rules.read().unwrap();

        // Simple path finding (could be enhanced with graph algorithms)
        let mut path = Vec::new();
        let mut current = from.clone();

        // Limit search to prevent infinite loops
        for _ in 0..20 {
            if current == *to {
                return Some(path);
            }

            let next = rules
                .iter()
                .find(|r| r.from_version == current)
                .cloned()?;

            path.push(next.clone());
            current = next.to_version;
        }

        None
    }

    /// Verify schema integrity against registered root hash
    pub fn verify_schema_integrity(&self, version: &SchemaVersion) -> bool {
        if let Some(entry) = self.get_schema(version) {
            entry.verify()
        } else {
            false
        }
    }

    /// Get merkle root for verification
    pub fn merkle_root(&self) -> Option<String> {
        self.merkle_root.read().unwrap().as_ref().map(|n| n.root_hash().to_string())
    }

    fn update_merkle_root(&self) {
        let entries = self.entries.read().unwrap();

        if entries.is_empty() {
            *self.merkle_root.write().unwrap() = None;
            return;
        }

        // Collect all hashes in version order
        let mut hashes: Vec<_> = entries.values().map(|e| &e.root_hash).collect();
        hashes.sort();

        // Build merkle tree from hashes
        let mut tree = vec![MerkleNode::new_leaf(hashes[0].clone())];

        for hash in hashes.iter().skip(1) {
            let prev = tree.pop().unwrap();
            let curr = MerkleNode::new_leaf(hash.to_string());
            tree.push(MerkleNode::new_branch(prev, curr));
        }

        if tree.len() == 1 {
            *self.merkle_root.write().unwrap() = tree.pop();
        }
    }
}

impl Default for SchemaRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple SHA256-like hash (production code would use real SHA256)
fn sha256_hash(data: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_version_comparison() {
        let v1 = SchemaVersion::new(1, 0, 0);
        let v2 = SchemaVersion::new(1, 1, 0);

        assert!(v1.is_compatible_with(&v2));
    }

    #[test]
    fn test_schema_version_incompatible() {
        let v1 = SchemaVersion::new(1, 0, 0);
        let v2 = SchemaVersion::new(2, 0, 0);

        assert!(!v1.is_compatible_with(&v2));
    }

    #[test]
    fn test_schema_entry_creation() {
        let entry = SchemaEntry::new(
            SchemaVersion::new(1, 0, 0),
            "{}".to_string(),
            "alice".to_string(),
            "Initial release".to_string(),
        );

        assert!(entry.verify());
    }

    #[test]
    fn test_schema_registry_registration() {
        let registry = SchemaRegistry::new();

        let entry = SchemaEntry::new(
            SchemaVersion::new(1, 0, 0),
            "{}".to_string(),
            "alice".to_string(),
            "Initial".to_string(),
        );

        let result = registry.register_schema(entry);
        assert!(result.is_ok());

        let versions = registry.list_versions();
        assert_eq!(versions.len(), 1);
    }

    #[test]
    fn test_schema_registry_duplicate_prevention() {
        let registry = SchemaRegistry::new();

        let entry = SchemaEntry::new(
            SchemaVersion::new(1, 0, 0),
            "{}".to_string(),
            "alice".to_string(),
            "Initial".to_string(),
        );

        registry.register_schema(entry.clone()).unwrap();

        // Try to register same version again
        let result = registry.register_schema(entry);
        assert!(result.is_err());
    }

    #[test]
    fn test_compatibility_check() {
        let registry = SchemaRegistry::new();

        let v1 = SchemaVersion::new(1, 0, 0);
        let v2 = SchemaVersion::new(1, 1, 0);

        let compat = registry.check_compatibility(&v1, &v2);
        assert_eq!(compat, Some(CompatibilityType::FullyCompatible));
    }

    #[test]
    fn test_merkle_tree_integrity() {
        let registry = SchemaRegistry::new();

        let entry = SchemaEntry::new(
            SchemaVersion::new(1, 0, 0),
            r#"{"verb":"test"}"#.to_string(),
            "alice".to_string(),
            "Test".to_string(),
        );

        registry.register_schema(entry).unwrap();

        // Merkle root should be set
        let root = registry.merkle_root();
        assert!(root.is_some());
    }
}
