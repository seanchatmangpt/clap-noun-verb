//! Multi-agent identity and tenancy support for swarm-native CLIs
//!
//! In a world with trillions of agents, every invocation needs:
//! - Agent identity (who is executing)
//! - Tenant/project identity (what context)
//! - Policy context (what rules apply)
//! - Priority and QoS hints (how important is this)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Agent identity for command invocation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentIdentity {
    /// Unique agent identifier
    pub agent_id: String,
    /// Agent type (e.g., "code-assistant", "build-bot", "monitor")
    pub agent_type: String,
    /// Agent version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_version: Option<String>,
    /// Additional agent metadata
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub metadata: HashMap<String, String>,
}

impl AgentIdentity {
    /// Create a new agent identity
    pub fn new(agent_id: impl Into<String>, agent_type: impl Into<String>) -> Self {
        Self {
            agent_id: agent_id.into(),
            agent_type: agent_type.into(),
            agent_version: None,
            metadata: HashMap::new(),
        }
    }

    /// Set agent version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.agent_version = Some(version.into());
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Create an anonymous agent identity
    pub fn anonymous() -> Self {
        Self::new("anonymous", "unknown")
    }

    /// Create a human user identity
    pub fn human(user_id: impl Into<String>) -> Self {
        Self::new(user_id, "human")
    }
}

// Manual Hash impl to skip metadata HashMap
impl std::hash::Hash for AgentIdentity {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.agent_id.hash(state);
        self.agent_type.hash(state);
        self.agent_version.hash(state);
        // Skip metadata as HashMap doesn't implement Hash
    }
}

/// Tenant or project identity
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TenantIdentity {
    /// Unique tenant/project identifier
    pub tenant_id: String,
    /// Tenant name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_name: Option<String>,
    /// Organization ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<String>,
    /// Environment (e.g., "production", "staging", "development")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
}

impl TenantIdentity {
    /// Create a new tenant identity
    pub fn new(tenant_id: impl Into<String>) -> Self {
        Self {
            tenant_id: tenant_id.into(),
            tenant_name: None,
            organization_id: None,
            environment: None,
        }
    }

    /// Set tenant name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.tenant_name = Some(name.into());
        self
    }

    /// Set organization ID
    pub fn with_organization(mut self, org_id: impl Into<String>) -> Self {
        self.organization_id = Some(org_id.into());
        self
    }

    /// Set environment
    pub fn with_environment(mut self, env: impl Into<String>) -> Self {
        self.environment = Some(env.into());
        self
    }

    /// Create a default tenant
    pub fn default_tenant() -> Self {
        Self::new("default")
    }
}

/// Policy context for command invocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyContext {
    /// Policy token or identifier
    pub policy_id: String,
    /// Policy version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_version: Option<String>,
    /// Applied rules
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub applied_rules: Vec<String>,
    /// Policy enforcement mode
    pub enforcement_mode: EnforcementMode,
}

impl PolicyContext {
    /// Create a new policy context
    pub fn new(policy_id: impl Into<String>) -> Self {
        Self {
            policy_id: policy_id.into(),
            policy_version: None,
            applied_rules: Vec::new(),
            enforcement_mode: EnforcementMode::Enforce,
        }
    }

    /// Set policy version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.policy_version = Some(version.into());
        self
    }

    /// Add an applied rule
    pub fn with_rule(mut self, rule: impl Into<String>) -> Self {
        self.applied_rules.push(rule.into());
        self
    }

    /// Set enforcement mode
    pub fn with_enforcement(mut self, mode: EnforcementMode) -> Self {
        self.enforcement_mode = mode;
        self
    }

    /// Create a permissive policy context (audit only)
    pub fn permissive(policy_id: impl Into<String>) -> Self {
        Self::new(policy_id).with_enforcement(EnforcementMode::Audit)
    }
}

/// Policy enforcement mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnforcementMode {
    /// Enforce policies, block violations
    Enforce,
    /// Audit mode, log violations but don't block
    Audit,
    /// Dry-run mode, report what would happen
    DryRun,
}

/// Priority class for command execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PriorityClass {
    /// Best effort, lowest priority
    BestEffort,
    /// Normal priority (default)
    Normal,
    /// High priority, expedited
    High,
    /// Critical priority, must succeed
    Critical,
}

impl Default for PriorityClass {
    fn default() -> Self {
        PriorityClass::Normal
    }
}

/// Quality of Service (QoS) hints for scheduling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QoSHints {
    /// Priority class
    pub priority: PriorityClass,
    /// Deadline (ISO 8601 timestamp)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline: Option<String>,
    /// Maximum latency category (e.g., "hot-path", "interactive", "batch")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_category: Option<String>,
    /// Importance hint (higher = more important)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub importance: Option<u8>,
    /// Whether this must succeed (vs best-effort)
    pub must_succeed: bool,
}

impl Default for QoSHints {
    fn default() -> Self {
        Self {
            priority: PriorityClass::Normal,
            deadline: None,
            latency_category: None,
            importance: None,
            must_succeed: false,
        }
    }
}

impl QoSHints {
    /// Create new QoS hints with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set priority class
    pub fn with_priority(mut self, priority: PriorityClass) -> Self {
        self.priority = priority;
        self
    }

    /// Set deadline
    pub fn with_deadline(mut self, deadline: impl Into<String>) -> Self {
        self.deadline = Some(deadline.into());
        self
    }

    /// Set latency category
    pub fn with_latency_category(mut self, category: impl Into<String>) -> Self {
        self.latency_category = Some(category.into());
        self
    }

    /// Set importance (0-255)
    pub fn with_importance(mut self, importance: u8) -> Self {
        self.importance = Some(importance);
        self
    }

    /// Mark as must-succeed
    pub fn must_succeed(mut self) -> Self {
        self.must_succeed = true;
        self
    }

    /// Create QoS hints for best-effort execution
    pub fn best_effort() -> Self {
        Self::new().with_priority(PriorityClass::BestEffort)
    }

    /// Create QoS hints for critical execution
    pub fn critical() -> Self {
        Self::new().with_priority(PriorityClass::Critical).must_succeed()
    }
}

/// Complete invocation context for swarm-native execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvocationContext {
    /// Agent identity
    pub agent: AgentIdentity,
    /// Tenant identity
    pub tenant: TenantIdentity,
    /// Policy context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy: Option<PolicyContext>,
    /// QoS hints
    #[serde(default)]
    pub qos: QoSHints,
    /// Correlation ID for tracing
    pub correlation_id: String,
    /// Parent invocation ID (for nested calls)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_invocation_id: Option<String>,
}

impl InvocationContext {
    /// Create a new invocation context
    pub fn new(agent: AgentIdentity, tenant: TenantIdentity) -> Self {
        Self {
            agent,
            tenant,
            policy: None,
            qos: QoSHints::default(),
            correlation_id: uuid::Uuid::new_v4().to_string(),
            parent_invocation_id: None,
        }
    }

    /// Set policy context
    pub fn with_policy(mut self, policy: PolicyContext) -> Self {
        self.policy = Some(policy);
        self
    }

    /// Set QoS hints
    pub fn with_qos(mut self, qos: QoSHints) -> Self {
        self.qos = qos;
        self
    }

    /// Set correlation ID
    pub fn with_correlation_id(mut self, id: impl Into<String>) -> Self {
        self.correlation_id = id.into();
        self
    }

    /// Set parent invocation ID
    pub fn with_parent(mut self, parent_id: impl Into<String>) -> Self {
        self.parent_invocation_id = Some(parent_id.into());
        self
    }

    /// Create a default context for anonymous execution
    pub fn anonymous() -> Self {
        Self::new(AgentIdentity::anonymous(), TenantIdentity::default_tenant())
    }

    /// Create a context for human user execution
    pub fn human(user_id: impl Into<String>) -> Self {
        Self::new(AgentIdentity::human(user_id), TenantIdentity::default_tenant())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_identity() {
        let agent = AgentIdentity::new("agent-123", "code-assistant")
            .with_version("1.0.0")
            .with_metadata("team", "infrastructure");

        assert_eq!(agent.agent_id, "agent-123");
        assert_eq!(agent.agent_type, "code-assistant");
        assert_eq!(agent.agent_version, Some("1.0.0".to_string()));
        assert_eq!(agent.metadata.get("team"), Some(&"infrastructure".to_string()));
    }

    #[test]
    fn test_tenant_identity() {
        let tenant = TenantIdentity::new("project-456")
            .with_name("MyProject")
            .with_organization("org-789")
            .with_environment("production");

        assert_eq!(tenant.tenant_id, "project-456");
        assert_eq!(tenant.tenant_name, Some("MyProject".to_string()));
        assert_eq!(tenant.organization_id, Some("org-789".to_string()));
        assert_eq!(tenant.environment, Some("production".to_string()));
    }

    #[test]
    fn test_qos_hints() {
        let qos = QoSHints::new()
            .with_priority(PriorityClass::High)
            .with_latency_category("interactive")
            .with_importance(200)
            .must_succeed();

        assert_eq!(qos.priority, PriorityClass::High);
        assert_eq!(qos.latency_category, Some("interactive".to_string()));
        assert_eq!(qos.importance, Some(200));
        assert!(qos.must_succeed);
    }

    #[test]
    fn test_invocation_context() {
        let agent = AgentIdentity::new("agent-123", "code-assistant");
        let tenant = TenantIdentity::new("project-456");
        let policy = PolicyContext::new("default-policy").with_rule("no-privileged");

        let ctx = InvocationContext::new(agent, tenant)
            .with_policy(policy)
            .with_qos(QoSHints::critical());

        assert_eq!(ctx.agent.agent_id, "agent-123");
        assert_eq!(ctx.tenant.tenant_id, "project-456");
        assert_eq!(ctx.qos.priority, PriorityClass::Critical);
        assert!(ctx.qos.must_succeed);
    }
}
