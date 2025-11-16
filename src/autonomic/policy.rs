//! Policy hooks and governance system for swarm-native CLIs
//!
//! Provides pluggable policy engines that can:
//! - Veto commands before execution
//! - Rewrite arguments within allowed transformations
//! - Redirect to alternative noun/verb
//! - Enforce global governance rules

use super::effects::EffectMetadata;
use super::tenancy::InvocationContext;
use crate::error::{NounVerbError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Policy decision result
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "decision", rename_all = "snake_case")]
pub enum PolicyDecision {
    /// Allow the command to proceed
    Allow,
    /// Deny the command
    Deny {
        /// Reason for denial
        reason: String,
        /// Suggested alternative
        suggestion: Option<String>,
    },
    /// Rewrite the command arguments
    Rewrite {
        /// New arguments
        new_args: HashMap<String, serde_json::Value>,
    },
    /// Redirect to a different command
    Redirect {
        /// Target noun
        noun: String,
        /// Target verb
        verb: String,
        /// Modified arguments
        args: HashMap<String, serde_json::Value>,
    },
}

impl PolicyDecision {
    /// Create an allow decision
    pub fn allow() -> Self {
        PolicyDecision::Allow
    }

    /// Create a deny decision
    pub fn deny(reason: impl Into<String>) -> Self {
        PolicyDecision::Deny { reason: reason.into(), suggestion: None }
    }

    /// Create a deny decision with suggestion
    pub fn deny_with_suggestion(reason: impl Into<String>, suggestion: impl Into<String>) -> Self {
        PolicyDecision::Deny {
            reason: reason.into(),
            suggestion: Some(suggestion.into()),
        }
    }

    /// Create a rewrite decision
    pub fn rewrite(new_args: HashMap<String, serde_json::Value>) -> Self {
        PolicyDecision::Rewrite { new_args }
    }

    /// Create a redirect decision
    pub fn redirect(
        noun: impl Into<String>,
        verb: impl Into<String>,
        args: HashMap<String, serde_json::Value>,
    ) -> Self {
        PolicyDecision::Redirect {
            noun: noun.into(),
            verb: verb.into(),
            args,
        }
    }

    /// Check if this decision allows execution
    pub fn is_allowed(&self) -> bool {
        !matches!(self, PolicyDecision::Deny { .. })
    }

    /// Check if this decision requires changes
    pub fn requires_changes(&self) -> bool {
        matches!(self, PolicyDecision::Rewrite { .. } | PolicyDecision::Redirect { .. })
    }
}

/// Policy evaluation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRequest {
    /// Invocation context
    pub context: InvocationContext,
    /// Noun being invoked
    pub noun: String,
    /// Verb being invoked
    pub verb: String,
    /// Command arguments
    pub args: HashMap<String, serde_json::Value>,
    /// Effect metadata for the command
    pub effects: EffectMetadata,
    /// Whether this is a dry-run
    pub dry_run: bool,
}

impl PolicyRequest {
    /// Create a new policy request
    pub fn new(
        context: InvocationContext,
        noun: impl Into<String>,
        verb: impl Into<String>,
        args: HashMap<String, serde_json::Value>,
        effects: EffectMetadata,
    ) -> Self {
        Self {
            context,
            noun: noun.into(),
            verb: verb.into(),
            args,
            effects,
            dry_run: false,
        }
    }

    /// Mark as dry-run
    pub fn dry_run(mut self) -> Self {
        self.dry_run = true;
        self
    }
}

/// Policy evaluation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyResult {
    /// Decision
    pub decision: PolicyDecision,
    /// Policy rules that were evaluated
    pub evaluated_rules: Vec<String>,
    /// Metadata about the evaluation
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl PolicyResult {
    /// Create a new policy result
    pub fn new(decision: PolicyDecision) -> Self {
        Self {
            decision,
            evaluated_rules: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add an evaluated rule
    pub fn with_rule(mut self, rule: impl Into<String>) -> Self {
        self.evaluated_rules.push(rule.into());
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Check if the policy allows execution
    pub fn is_allowed(&self) -> bool {
        self.decision.is_allowed()
    }

    /// Convert to a Result, returning error if denied
    pub fn to_result(self) -> Result<PolicyDecision> {
        match &self.decision {
            PolicyDecision::Deny { reason, .. } => {
                Err(NounVerbError::execution_error(format!("Policy denied: {}", reason)))
            }
            _ => Ok(self.decision),
        }
    }
}

/// Trait for pluggable policy engines
pub trait PolicyEngine: Send + Sync {
    /// Evaluate a policy request
    fn evaluate(&self, request: &PolicyRequest) -> Result<PolicyResult>;

    /// Get policy engine name
    fn name(&self) -> &str;

    /// Get policy engine version
    fn version(&self) -> &str {
        "1.0.0"
    }
}

/// Policy rule for declarative policy definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    /// Rule name
    pub name: String,
    /// Rule description
    pub description: String,
    /// Conditions that must be met
    pub conditions: Vec<PolicyCondition>,
    /// Action to take when conditions are met
    pub action: PolicyAction,
    /// Rule priority (higher = evaluated first)
    #[serde(default)]
    pub priority: i32,
    /// Whether this rule is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

impl PolicyRule {
    /// Create a new policy rule
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            conditions: Vec::new(),
            action: PolicyAction::Allow,
            priority: 0,
            enabled: true,
        }
    }

    /// Add a condition
    pub fn with_condition(mut self, condition: PolicyCondition) -> Self {
        self.conditions.push(condition);
        self
    }

    /// Set action
    pub fn with_action(mut self, action: PolicyAction) -> Self {
        self.action = action;
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Disable the rule
    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }

    /// Check if all conditions are met
    pub fn matches(&self, request: &PolicyRequest) -> bool {
        if !self.enabled {
            return false;
        }

        self.conditions.iter().all(|condition| condition.matches(request))
    }
}

/// Policy condition for rule matching
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PolicyCondition {
    /// Match effect type
    EffectType { effect: String },
    /// Match sensitivity level
    Sensitivity { min_level: String },
    /// Match agent type
    AgentType { agent_type: String },
    /// Match tenant
    Tenant { tenant_id: String },
    /// Match noun/verb pattern
    Command { pattern: String },
    /// Check if command handles sensitive data
    HandlesSensitiveData,
    /// Check if command requires isolation
    RequiresIsolation,
}

impl PolicyCondition {
    /// Check if this condition matches the request
    pub fn matches(&self, request: &PolicyRequest) -> bool {
        match self {
            PolicyCondition::EffectType { effect } => {
                format!("{:?}", request.effects.effect_type).to_lowercase() == effect.to_lowercase()
            }
            PolicyCondition::Sensitivity { min_level } => {
                let min = match min_level.to_lowercase().as_str() {
                    "low" => 0,
                    "medium" => 1,
                    "high" => 2,
                    "critical" => 3,
                    _ => 0,
                };
                let current = match request.effects.sensitivity {
                    super::effects::Sensitivity::Low => 0,
                    super::effects::Sensitivity::Medium => 1,
                    super::effects::Sensitivity::High => 2,
                    super::effects::Sensitivity::Critical => 3,
                };
                current >= min
            }
            PolicyCondition::AgentType { agent_type } => {
                request.context.agent.agent_type == *agent_type
            }
            PolicyCondition::Tenant { tenant_id } => request.context.tenant.tenant_id == *tenant_id,
            PolicyCondition::Command { pattern } => {
                let command = format!("{}.{}", request.noun, request.verb);
                // Simple pattern matching (could be enhanced with regex)
                command.contains(pattern)
            }
            PolicyCondition::HandlesSensitiveData => request.effects.handles_sensitive_data(),
            PolicyCondition::RequiresIsolation => request.effects.requires_isolation(),
        }
    }
}

/// Policy action to take when conditions are met
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PolicyAction {
    /// Allow the command
    Allow,
    /// Deny the command
    Deny { reason: String },
    /// Require approval (audit mode)
    RequireApproval { approver: String },
}

/// Simple rule-based policy engine
pub struct RuleBasedPolicyEngine {
    name: String,
    rules: Vec<PolicyRule>,
}

impl RuleBasedPolicyEngine {
    /// Create a new rule-based policy engine
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            rules: Vec::new(),
        }
    }

    /// Add a rule
    pub fn add_rule(mut self, rule: PolicyRule) -> Self {
        self.rules.push(rule);
        self.sort_rules();
        self
    }

    /// Sort rules by priority (higher first)
    fn sort_rules(&mut self) {
        self.rules.sort_by(|a, b| b.priority.cmp(&a.priority));
    }
}

impl PolicyEngine for RuleBasedPolicyEngine {
    fn evaluate(&self, request: &PolicyRequest) -> Result<PolicyResult> {
        let mut result = PolicyResult::new(PolicyDecision::Allow);

        for rule in &self.rules {
            if rule.matches(request) {
                result = result.with_rule(&rule.name);

                match &rule.action {
                    PolicyAction::Allow => {
                        // Continue evaluating other rules
                    }
                    PolicyAction::Deny { reason } => {
                        return Ok(PolicyResult::new(PolicyDecision::deny(reason))
                            .with_rule(&rule.name));
                    }
                    PolicyAction::RequireApproval { approver } => {
                        result = result.with_metadata(
                            "requires_approval",
                            serde_json::json!({
                                "approver": approver,
                                "rule": rule.name,
                            }),
                        );
                    }
                }
            }
        }

        Ok(result)
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_decision() {
        let allow = PolicyDecision::allow();
        assert!(allow.is_allowed());
        assert!(!allow.requires_changes());

        let deny = PolicyDecision::deny("Not authorized");
        assert!(!deny.is_allowed());
        assert!(!deny.requires_changes());

        let mut rewrite_args = HashMap::new();
        rewrite_args.insert("timeout".to_string(), serde_json::json!(30));
        let rewrite = PolicyDecision::rewrite(rewrite_args);
        assert!(rewrite.is_allowed());
        assert!(rewrite.requires_changes());
    }

    #[test]
    fn test_rule_based_policy_engine() {
        let engine = RuleBasedPolicyEngine::new("test-policy")
            .add_rule(
                PolicyRule::new("deny-privileged", "Deny privileged operations")
                    .with_condition(PolicyCondition::EffectType {
                        effect: "privileged".to_string(),
                    })
                    .with_action(PolicyAction::Deny {
                        reason: "Privileged operations not allowed".to_string(),
                    }),
            );

        let ctx = InvocationContext::anonymous();
        let request = PolicyRequest::new(
            ctx,
            "services",
            "restart",
            HashMap::new(),
            super::effects::EffectMetadata::new(super::effects::EffectType::Privileged),
        );

        let result = engine.evaluate(&request).unwrap();
        assert!(!result.is_allowed());
    }
}
