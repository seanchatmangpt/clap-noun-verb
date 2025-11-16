//! # Distributed Identity and Delegation Protocol
//!
//! Enables agents to delegate capabilities to other agents with fine-grained control.
//! Every invocation carries a delegation chain showing who authorized what.
//!
//! ## Design Principles
//!
//! 1. **Immutable Tokens**: Delegation tokens are immutable once constructed
//! 2. **Cheap Verification**: Token checks are fast and cache-friendly
//! 3. **Complete Chains**: Full delegation history from origin to executor
//! 4. **Policy Integration**: Policies can match on delegation chains

use super::{
    capability_id::CapabilityId,
    tenancy::{AgentIdentity, TenantIdentity},
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::time::{Duration, SystemTime};

/// Principal in the delegation chain
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Principal {
    /// Agent identity
    pub agent: AgentIdentity,

    /// Tenant identity
    pub tenant: TenantIdentity,

    /// Principal type
    pub principal_type: PrincipalType,
}

impl Principal {
    /// Create a new principal
    pub fn new(agent: AgentIdentity, tenant: TenantIdentity) -> Self {
        Self {
            agent,
            tenant,
            principal_type: PrincipalType::Direct,
        }
    }

    /// Create a delegated principal
    pub fn delegated(agent: AgentIdentity, tenant: TenantIdentity) -> Self {
        Self {
            agent,
            tenant,
            principal_type: PrincipalType::Delegated,
        }
    }

    /// Create a service principal
    pub fn service(agent: AgentIdentity, tenant: TenantIdentity) -> Self {
        Self {
            agent,
            tenant,
            principal_type: PrincipalType::Service,
        }
    }
}

/// Type of principal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrincipalType {
    /// Direct execution by this agent
    Direct,

    /// Delegated execution on behalf of another
    Delegated,

    /// Service account
    Service,

    /// System-level principal
    System,
}

/// Capability constraint in a delegation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityConstraint {
    /// Allowed capabilities (if None, all capabilities from parent)
    pub allowed_capabilities: Option<HashSet<CapabilityId>>,

    /// Forbidden capabilities (takes precedence over allowed)
    pub forbidden_capabilities: HashSet<CapabilityId>,

    /// Allowed nouns (if None, all nouns)
    pub allowed_nouns: Option<HashSet<String>>,

    /// Allowed verbs (if None, all verbs)
    pub allowed_verbs: Option<HashSet<String>>,

    /// Maximum effect level
    pub max_effect_level: Option<EffectLevel>,
}

impl CapabilityConstraint {
    /// Create unrestricted constraints (inherit all from parent)
    pub fn unrestricted() -> Self {
        Self {
            allowed_capabilities: None,
            forbidden_capabilities: HashSet::new(),
            allowed_nouns: None,
            allowed_verbs: None,
            max_effect_level: None,
        }
    }

    /// Create strict constraints (no capabilities)
    pub fn strict() -> Self {
        Self {
            allowed_capabilities: Some(HashSet::new()),
            forbidden_capabilities: HashSet::new(),
            allowed_nouns: Some(HashSet::new()),
            allowed_verbs: Some(HashSet::new()),
            max_effect_level: Some(EffectLevel::ReadOnly),
        }
    }

    /// Check if a capability is allowed
    pub fn allows_capability(&self, capability_id: &CapabilityId) -> bool {
        // Check forbidden first
        if self.forbidden_capabilities.contains(capability_id) {
            return false;
        }

        // Check allowed
        match &self.allowed_capabilities {
            Some(allowed) => allowed.contains(capability_id),
            None => true, // None means all allowed
        }
    }

    /// Check if a noun/verb pair is allowed
    pub fn allows_command(&self, noun: &str, verb: &str) -> bool {
        let noun_ok = match &self.allowed_nouns {
            Some(allowed) => allowed.contains(noun),
            None => true,
        };

        let verb_ok = match &self.allowed_verbs {
            Some(allowed) => allowed.contains(verb),
            None => true,
        };

        noun_ok && verb_ok
    }

    /// Intersect with another constraint (take the more restrictive)
    pub fn intersect(&self, other: &CapabilityConstraint) -> CapabilityConstraint {
        // Intersect allowed capabilities
        let allowed_capabilities = match (&self.allowed_capabilities, &other.allowed_capabilities) {
            (Some(a), Some(b)) => Some(a.intersection(b).cloned().collect()),
            (Some(a), None) => Some(a.clone()),
            (None, Some(b)) => Some(b.clone()),
            (None, None) => None,
        };

        // Union forbidden capabilities
        let forbidden_capabilities: HashSet<_> = self
            .forbidden_capabilities
            .union(&other.forbidden_capabilities)
            .cloned()
            .collect();

        // Intersect allowed nouns
        let allowed_nouns = match (&self.allowed_nouns, &other.allowed_nouns) {
            (Some(a), Some(b)) => Some(a.intersection(b).cloned().collect()),
            (Some(a), None) => Some(a.clone()),
            (None, Some(b)) => Some(b.clone()),
            (None, None) => None,
        };

        // Intersect allowed verbs
        let allowed_verbs = match (&self.allowed_verbs, &other.allowed_verbs) {
            (Some(a), Some(b)) => Some(a.intersection(b).cloned().collect()),
            (Some(a), None) => Some(a.clone()),
            (None, Some(b)) => Some(b.clone()),
            (None, None) => None,
        };

        // Take minimum effect level
        let max_effect_level = match (self.max_effect_level, other.max_effect_level) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        };

        CapabilityConstraint {
            allowed_capabilities,
            forbidden_capabilities,
            allowed_nouns,
            allowed_verbs,
            max_effect_level,
        }
    }
}

/// Effect level for delegation constraints
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum EffectLevel {
    ReadOnly = 0,
    Mutate = 1,
    Network = 2,
    Privileged = 3,
}

/// Temporal constraint for delegation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemporalConstraint {
    /// Not valid before this time
    pub not_before: SystemTime,

    /// Not valid after this time
    pub not_after: SystemTime,

    /// Maximum number of uses (None = unlimited)
    pub max_uses: Option<u32>,
}

impl TemporalConstraint {
    /// Create a constraint valid for a duration
    pub fn valid_for(duration: Duration) -> Self {
        let now = SystemTime::now();
        Self {
            not_before: now,
            not_after: now + duration,
            max_uses: None,
        }
    }

    /// Create a constraint valid for a time window
    pub fn valid_between(start: SystemTime, end: SystemTime) -> Self {
        Self {
            not_before: start,
            not_after: end,
            max_uses: None,
        }
    }

    /// With maximum uses
    pub fn with_max_uses(mut self, max_uses: u32) -> Self {
        self.max_uses = Some(max_uses);
        self
    }

    /// Check if currently valid
    pub fn is_valid_at(&self, time: SystemTime) -> bool {
        time >= self.not_before && time <= self.not_after
    }

    /// Check if currently valid
    pub fn is_valid(&self) -> bool {
        self.is_valid_at(SystemTime::now())
    }
}

/// Delegation token - immutable proof of delegation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationToken {
    /// Token ID
    pub token_id: TokenId,

    /// Who granted this delegation (delegator)
    pub delegator: Principal,

    /// Who receives this delegation (delegate)
    pub delegate: Principal,

    /// Capability constraints
    pub constraints: CapabilityConstraint,

    /// Temporal constraints
    pub temporal: TemporalConstraint,

    /// Parent token (if this is a sub-delegation)
    pub parent_token_id: Option<TokenId>,

    /// Metadata
    pub metadata: DelegationMetadata,

    /// Number of times this token has been used
    #[serde(skip)]
    uses: std::sync::atomic::AtomicU32,
}

impl DelegationToken {
    /// Create a new delegation token
    pub fn new(
        delegator: Principal,
        delegate: Principal,
        constraints: CapabilityConstraint,
        temporal: TemporalConstraint,
    ) -> Self {
        Self {
            token_id: TokenId::generate(),
            delegator,
            delegate,
            constraints,
            temporal,
            parent_token_id: None,
            metadata: DelegationMetadata::default(),
            uses: std::sync::atomic::AtomicU32::new(0),
        }
    }

    /// Create a sub-delegation (delegation of a delegation)
    pub fn sub_delegate(
        &self,
        delegate: Principal,
        constraints: CapabilityConstraint,
        temporal: TemporalConstraint,
    ) -> Result<DelegationToken, DelegationError> {
        // Constraints must be more restrictive than parent
        let combined_constraints = self.constraints.intersect(&constraints);

        // Temporal constraint cannot extend beyond parent
        let combined_temporal = TemporalConstraint {
            not_before: temporal.not_before.max(self.temporal.not_before),
            not_after: temporal.not_after.min(self.temporal.not_after),
            max_uses: match (temporal.max_uses, self.temporal.max_uses) {
                (Some(a), Some(b)) => Some(a.min(b)),
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                (None, None) => None,
            },
        };

        Ok(DelegationToken {
            token_id: TokenId::generate(),
            delegator: self.delegate.clone(),
            delegate,
            constraints: combined_constraints,
            temporal: combined_temporal,
            parent_token_id: Some(self.token_id.clone()),
            metadata: DelegationMetadata::default(),
            uses: std::sync::atomic::AtomicU32::new(0),
        })
    }

    /// Verify this token is valid
    pub fn verify(&self) -> Result<(), DelegationError> {
        // Check temporal validity
        if !self.temporal.is_valid() {
            return Err(DelegationError::TokenExpired);
        }

        // Check usage limit
        if let Some(max_uses) = self.temporal.max_uses {
            let uses = self.uses.load(std::sync::atomic::Ordering::Relaxed);
            if uses >= max_uses {
                return Err(DelegationError::UsageLimitExceeded);
            }
        }

        Ok(())
    }

    /// Record a use of this token
    pub fn record_use(&self) -> Result<(), DelegationError> {
        self.verify()?;
        self.uses.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }

    /// Check if this token allows a capability
    pub fn allows_capability(&self, capability_id: &CapabilityId) -> bool {
        self.constraints.allows_capability(capability_id)
    }

    /// Check if this token allows a command
    pub fn allows_command(&self, noun: &str, verb: &str) -> bool {
        self.constraints.allows_command(noun, verb)
    }
}

/// Token ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TokenId(pub String);

impl TokenId {
    /// Generate a new token ID
    pub fn generate() -> Self {
        use sha2::{Digest, Sha256};
        let uuid = uuid::Uuid::new_v4();
        let hash = Sha256::digest(uuid.as_bytes());
        Self(format!("tok_{}", hex::encode(&hash[..12])))
    }
}

/// Delegation metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DelegationMetadata {
    /// Human-readable description
    pub description: Option<String>,

    /// Purpose of this delegation
    pub purpose: Option<String>,

    /// Tags
    pub tags: Vec<String>,
}

/// Complete delegation chain from origin to current executor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationChain {
    /// Origin principal (who started this)
    pub origin: Principal,

    /// Chain of delegation tokens (in order from origin to current)
    pub tokens: Vec<DelegationToken>,

    /// Current executor
    pub executor: Principal,
}

impl DelegationChain {
    /// Create a new chain with direct execution (no delegation)
    pub fn direct(principal: Principal) -> Self {
        Self {
            origin: principal.clone(),
            tokens: vec![],
            executor: principal,
        }
    }

    /// Create a chain with one delegation
    pub fn with_delegation(token: DelegationToken) -> Self {
        let origin = token.delegator.clone();
        let executor = token.delegate.clone();

        Self {
            origin,
            tokens: vec![token],
            executor,
        }
    }

    /// Add a delegation to the chain
    pub fn add_delegation(mut self, token: DelegationToken) -> Self {
        self.executor = token.delegate.clone();
        self.tokens.push(token);
        self
    }

    /// Verify the entire chain
    pub fn verify(&self) -> Result<(), DelegationError> {
        // Verify each token
        for token in &self.tokens {
            token.verify()?;
        }

        // Verify chain continuity
        for i in 0..self.tokens.len() {
            let token = &self.tokens[i];

            // First token must be delegated by origin
            if i == 0 {
                if token.delegator != self.origin {
                    return Err(DelegationError::BrokenChain);
                }
            } else {
                // Subsequent tokens must be delegated by previous delegate
                let prev_delegate = &self.tokens[i - 1].delegate;
                if &token.delegator != prev_delegate {
                    return Err(DelegationError::BrokenChain);
                }
            }
        }

        // Last delegate must be executor
        if let Some(last_token) = self.tokens.last() {
            if last_token.delegate != self.executor {
                return Err(DelegationError::BrokenChain);
            }
        }

        Ok(())
    }

    /// Get effective constraints (intersection of all tokens)
    pub fn effective_constraints(&self) -> CapabilityConstraint {
        self.tokens
            .iter()
            .fold(CapabilityConstraint::unrestricted(), |acc, token| {
                acc.intersect(&token.constraints)
            })
    }

    /// Check if chain allows a capability
    pub fn allows_capability(&self, capability_id: &CapabilityId) -> bool {
        self.effective_constraints().allows_capability(capability_id)
    }

    /// Get chain depth (number of delegations)
    pub fn depth(&self) -> usize {
        self.tokens.len()
    }

    /// Check if this is direct execution
    pub fn is_direct(&self) -> bool {
        self.tokens.is_empty()
    }
}

/// Delegation errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum DelegationError {
    #[error("Token expired")]
    TokenExpired,

    #[error("Usage limit exceeded")]
    UsageLimitExceeded,

    #[error("Broken delegation chain")]
    BrokenChain,

    #[error("Capability not allowed")]
    CapabilityNotAllowed,

    #[error("Invalid delegation")]
    InvalidDelegation,
}

/// Delegation registry for managing active delegations
pub struct DelegationRegistry {
    /// Active tokens indexed by token ID
    tokens: std::sync::RwLock<std::collections::HashMap<TokenId, DelegationToken>>,
}

impl DelegationRegistry {
    /// Create a new delegation registry
    pub fn new() -> Self {
        Self {
            tokens: std::sync::RwLock::new(std::collections::HashMap::new()),
        }
    }

    /// Register a delegation token
    pub fn register(&self, token: DelegationToken) {
        let mut tokens = self.tokens.write().unwrap();
        tokens.insert(token.token_id.clone(), token);
    }

    /// Revoke a delegation token
    pub fn revoke(&self, token_id: &TokenId) {
        let mut tokens = self.tokens.write().unwrap();
        tokens.remove(token_id);
    }

    /// Get a token by ID
    pub fn get(&self, token_id: &TokenId) -> Option<DelegationToken> {
        let tokens = self.tokens.read().unwrap();
        tokens.get(token_id).cloned()
    }

    /// Cleanup expired tokens
    pub fn cleanup_expired(&self) {
        let mut tokens = self.tokens.write().unwrap();
        tokens.retain(|_, token| token.temporal.is_valid());
    }

    /// Get all active tokens
    pub fn active_tokens(&self) -> Vec<DelegationToken> {
        let tokens = self.tokens.read().unwrap();
        tokens.values().filter(|t| t.temporal.is_valid()).cloned().collect()
    }
}

impl Default for DelegationRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delegation_token_creation() {
        let delegator = Principal::new(AgentIdentity::anonymous(), TenantIdentity::default());
        let delegate = Principal::delegated(AgentIdentity::anonymous(), TenantIdentity::default());

        let token = DelegationToken::new(
            delegator,
            delegate,
            CapabilityConstraint::unrestricted(),
            TemporalConstraint::valid_for(Duration::from_secs(3600)),
        );

        assert!(token.verify().is_ok());
    }

    #[test]
    fn test_delegation_chain() {
        let origin = Principal::new(AgentIdentity::anonymous(), TenantIdentity::default());
        let delegate1 = Principal::delegated(AgentIdentity::anonymous(), TenantIdentity::default());

        let token = DelegationToken::new(
            origin.clone(),
            delegate1.clone(),
            CapabilityConstraint::unrestricted(),
            TemporalConstraint::valid_for(Duration::from_secs(3600)),
        );

        let chain = DelegationChain::with_delegation(token);

        assert!(chain.verify().is_ok());
        assert_eq!(chain.depth(), 1);
    }

    #[test]
    fn test_capability_constraints() {
        let mut allowed = HashSet::new();
        allowed.insert(CapabilityId::from_path("user.read"));

        let constraint = CapabilityConstraint {
            allowed_capabilities: Some(allowed),
            forbidden_capabilities: HashSet::new(),
            allowed_nouns: None,
            allowed_verbs: None,
            max_effect_level: None,
        };

        assert!(constraint.allows_capability(&CapabilityId::from_path("user.read")));
        assert!(!constraint.allows_capability(&CapabilityId::from_path("user.delete")));
    }
}
