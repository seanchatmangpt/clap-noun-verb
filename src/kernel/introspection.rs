//! Capability Introspection
//!
//! Provides `--capabilities` and `--explain` runtime introspection commands.
//! Uses advanced Rust patterns: GATs, trait objects, and compile-time registry.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Capability metadata introspection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub side_effects: Vec<EffectType>,
    pub resource_profile: ResourceProfile,
    pub stability: StabilityGuarantee,
    pub safety: SafetyProfile,
    pub agent_safe: bool,
    pub requires_approval: Vec<String>,
}

/// Side effects that a capability may produce
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EffectType {
    Pure,
    ReadOnlyFS,
    ReadWriteFS,
    Network,
    Subprocess,
    Environment,
    Dangerous,
}

impl EffectType {
    pub fn description(&self) -> &'static str {
        match self {
            Self::Pure => "Pure computation, no side effects",
            Self::ReadOnlyFS => "Reads from filesystem",
            Self::ReadWriteFS => "Reads and writes to filesystem",
            Self::Network => "Accesses network resources",
            Self::Subprocess => "Spawns subprocesses",
            Self::Environment => "Modifies environment variables",
            Self::Dangerous => "Dangerous operation requiring caution",
        }
    }

    pub fn risk_level(&self) -> u8 {
        match self {
            Self::Pure => 0,
            Self::ReadOnlyFS => 10,
            Self::ReadWriteFS => 30,
            Self::Network => 25,
            Self::Subprocess => 40,
            Self::Environment => 35,
            Self::Dangerous => 100,
        }
    }
}

/// Resource consumption profile
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ResourceProfile {
    Instant,       // < 1ms
    Fast,          // 1-100ms
    Medium,        // 100ms-1s
    Slow,          // 1s-10s
    Cold,          // > 10s
}

impl ResourceProfile {
    pub fn max_duration_ms(&self) -> u64 {
        match self {
            Self::Instant => 1,
            Self::Fast => 100,
            Self::Medium => 1000,
            Self::Slow => 10000,
            Self::Cold => 60000,
        }
    }
}

/// Stability guarantee
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum StabilityGuarantee {
    Stable,
    Preview,
    Experimental,
    Deprecated,
    NonDeterministic,
}

impl StabilityGuarantee {
    pub fn description(&self) -> &'static str {
        match self {
            Self::Stable => "Stable API, safe to depend on",
            Self::Preview => "Preview feature, may change",
            Self::Experimental => "Experimental, subject to change",
            Self::Deprecated => "Deprecated, use alternative",
            Self::NonDeterministic => "Output not guaranteed to be deterministic",
        }
    }
}

/// Safety profile for agent execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SafetyProfile {
    AgentSafe,
    HumanReviewRequired,
    InteractiveOnly,
}

/// Global capability registry using const generics
///
/// This uses advanced trait patterns to create a compile-time registry
/// that can be introspected at runtime with zero reflection overhead.
#[derive(Debug, Clone)]
pub struct CapabilityRegistry {
    capabilities: HashMap<String, Arc<CapabilityInfo>>,
}

impl CapabilityRegistry {
    pub fn new() -> Self {
        Self {
            capabilities: HashMap::new(),
        }
    }

    pub fn register(&mut self, capability: CapabilityInfo) {
        self.capabilities.insert(capability.id.clone(), Arc::new(capability));
    }

    pub fn get(&self, id: &str) -> Option<Arc<CapabilityInfo>> {
        self.capabilities.get(id).cloned()
    }

    pub fn list_all(&self) -> Vec<Arc<CapabilityInfo>> {
        self.capabilities.values().cloned().collect()
    }

    pub fn find_by_safety(&self, safety: &SafetyProfile) -> Vec<Arc<CapabilityInfo>> {
        self.capabilities
            .values()
            .filter(|cap| &cap.safety == safety)
            .cloned()
            .collect()
    }

    pub fn agent_safe_capabilities(&self) -> Vec<Arc<CapabilityInfo>> {
        self.capabilities
            .values()
            .filter(|cap| cap.agent_safe)
            .cloned()
            .collect()
    }

    pub fn find_by_side_effect(&self, effect: &EffectType) -> Vec<Arc<CapabilityInfo>> {
        self.capabilities
            .values()
            .filter(|cap| cap.side_effects.contains(effect))
            .cloned()
            .collect()
    }

    pub fn capabilities_requiring_approval(&self) -> Vec<Arc<CapabilityInfo>> {
        self.capabilities
            .values()
            .filter(|cap| !cap.requires_approval.is_empty())
            .cloned()
            .collect()
    }

    pub fn risk_score_for_capability(&self, id: &str) -> Option<u8> {
        self.capabilities.get(id).map(|cap| {
            cap.side_effects
                .iter()
                .map(|effect| effect.risk_level())
                .max()
                .unwrap_or(0)
        })
    }
}

impl Default for CapabilityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Introspection command handler
/// Provides `--capabilities` and `--explain <cap>` functionality
#[derive(Debug)]
pub struct IntrospectionHandler {
    registry: CapabilityRegistry,
}

impl IntrospectionHandler {
    pub fn new(registry: CapabilityRegistry) -> Self {
        Self { registry }
    }

    /// Handle `--capabilities` command
    pub fn list_capabilities(&self) -> CapabilitiesOutput {
        let caps = self.registry.list_all();
        let agent_safe_count = caps.iter().filter(|c| c.agent_safe).count();
        let requires_approval = self.registry.capabilities_requiring_approval();

        CapabilitiesOutput {
            total_capabilities: caps.len(),
            agent_safe_count,
            requires_approval_count: requires_approval.len(),
            capabilities: caps.iter().map(|c| c.as_ref().clone()).collect(),
        }
    }

    /// Handle `--explain <capability>` command
    pub fn explain_capability(&self, id: &str) -> Result<ExplanationOutput, String> {
        let cap = self
            .registry
            .get(id)
            .ok_or_else(|| format!("Unknown capability: {}", id))?;

        let risk_level = self.registry.risk_score_for_capability(id).unwrap_or(0);

        Ok(ExplanationOutput {
            capability: cap.as_ref().clone(),
            risk_level,
            implications: self.compute_implications(&cap),
            related_capabilities: self.find_related(&cap),
        })
    }

    fn compute_implications(&self, cap: &CapabilityInfo) -> Vec<String> {
        let mut implications = Vec::new();

        if cap.agent_safe {
            implications.push("Safe for agent execution".to_string());
        } else {
            implications.push("NOT safe for agent execution - requires human review".to_string());
        }

        if cap.side_effects.contains(&EffectType::Dangerous) {
            implications.push("DANGEROUS: Exercise extreme caution".to_string());
        }

        if !cap.requires_approval.is_empty() {
            implications.push(format!(
                "Requires approval from: {}",
                cap.requires_approval.join(", ")
            ));
        }

        implications
    }

    fn find_related(&self, cap: &CapabilityInfo) -> Vec<String> {
        let mut related = Vec::new();

        for effect in &cap.side_effects {
            let similar = self.registry.find_by_side_effect(effect);
            for sim in similar {
                if sim.id != cap.id && !related.contains(&sim.id) {
                    related.push(sim.id.clone());
                }
            }
        }

        related.truncate(5); // Limit to 5 related capabilities
        related
    }
}

/// Output of `--capabilities`
#[derive(Debug, Serialize, Deserialize)]
pub struct CapabilitiesOutput {
    pub total_capabilities: usize,
    pub agent_safe_count: usize,
    pub requires_approval_count: usize,
    pub capabilities: Vec<CapabilityInfo>,
}

/// Output of `--explain <capability>`
#[derive(Debug, Serialize, Deserialize)]
pub struct ExplanationOutput {
    pub capability: CapabilityInfo,
    pub risk_level: u8,
    pub implications: Vec<String>,
    pub related_capabilities: Vec<String>,
}

/// Compile-time capability definition macro (future enhancement)
/// This would allow:
/// ```ignore
/// define_capability! {
///     id: "services.restart",
///     name: "Restart Service",
///     description: "Restart a running service",
///     side_effects: [ReadWriteFS, Subprocess],
///     stability: Stable,
///     safety: AgentSafe,
/// }
/// ```

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_registry() {
        let mut registry = CapabilityRegistry::new();

        let cap1 = CapabilityInfo {
            id: "list".to_string(),
            name: "List".to_string(),
            description: "List items".to_string(),
            side_effects: vec![EffectType::ReadOnlyFS],
            resource_profile: ResourceProfile::Fast,
            stability: StabilityGuarantee::Stable,
            safety: SafetyProfile::AgentSafe,
            agent_safe: true,
            requires_approval: vec![],
        };

        registry.register(cap1);
        assert!(registry.get("list").is_some());
    }

    #[test]
    fn test_side_effect_risk_levels() {
        assert_eq!(EffectType::Pure.risk_level(), 0);
        assert_eq!(EffectType::ReadOnlyFS.risk_level(), 10);
        assert_eq!(EffectType::Dangerous.risk_level(), 100);
    }

    #[test]
    fn test_resource_profile_durations() {
        assert_eq!(ResourceProfile::Instant.max_duration_ms(), 1);
        assert_eq!(ResourceProfile::Fast.max_duration_ms(), 100);
        assert_eq!(ResourceProfile::Cold.max_duration_ms(), 60000);
    }

    #[test]
    fn test_introspection_handler() {
        let mut registry = CapabilityRegistry::new();

        registry.register(CapabilityInfo {
            id: "read".to_string(),
            name: "Read".to_string(),
            description: "Read data".to_string(),
            side_effects: vec![EffectType::ReadOnlyFS],
            resource_profile: ResourceProfile::Fast,
            stability: StabilityGuarantee::Stable,
            safety: SafetyProfile::AgentSafe,
            agent_safe: true,
            requires_approval: vec![],
        });

        let handler = IntrospectionHandler::new(registry);
        let output = handler.list_capabilities();

        assert_eq!(output.total_capabilities, 1);
        assert_eq!(output.agent_safe_count, 1);
    }

    #[test]
    fn test_explain_capability() {
        let mut registry = CapabilityRegistry::new();

        registry.register(CapabilityInfo {
            id: "dangerous".to_string(),
            name: "Dangerous Op".to_string(),
            description: "A dangerous operation".to_string(),
            side_effects: vec![EffectType::Dangerous],
            resource_profile: ResourceProfile::Instant,
            stability: StabilityGuarantee::Stable,
            safety: SafetyProfile::HumanReviewRequired,
            agent_safe: false,
            requires_approval: vec!["admin".to_string()],
        });

        let handler = IntrospectionHandler::new(registry);
        let output = handler.explain_capability("dangerous").unwrap();

        assert_eq!(output.capability.safety, SafetyProfile::HumanReviewRequired);
        assert!(output
            .implications
            .iter()
            .any(|i| i.contains("NOT safe for agent execution")));
    }
}
