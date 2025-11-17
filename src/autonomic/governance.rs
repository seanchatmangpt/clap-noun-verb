//! # Governance Observability and Replay
//!
//! Provides governance-grade telemetry and replay capabilities for proving that
//! agent swarms operated within defined rules and constraints.
//!
//! ## Design Principles
//!
//! 1. **Append-Only Ledger**: Immutable record of governance events
//! 2. **Compact Format**: Efficient storage and sequential reads
//! 3. **Queryable**: High-level API for analysis and auditing
//! 4. **Replayable**: Re-evaluate decisions with different policies

use super::{
    capability_id::CapabilityId,
    policy::PolicyDecision,
    tenancy::{AgentIdentity, TenantIdentity},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

/// Governance event - immutable record of a governance-relevant action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceEvent {
    /// Event ID
    pub event_id: EventId,

    /// Timestamp
    pub timestamp: SystemTime,

    /// Event type and data
    pub event_type: EventType,

    /// Agent that triggered this event
    pub agent: AgentIdentity,

    /// Tenant context
    pub tenant: TenantIdentity,

    /// Correlation ID linking related events
    pub correlation_id: String,

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Event ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventId(pub u64);

impl EventId {
    /// Create a new event ID
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

/// Type of governance event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum EventType {
    /// Capability was granted
    CapabilityGranted {
        capability_id: CapabilityId,
        granted_by: AgentIdentity,
    },

    /// Capability was revoked
    CapabilityRevoked {
        capability_id: CapabilityId,
        revoked_by: AgentIdentity,
    },

    /// Policy was changed
    PolicyChanged {
        policy_id: String,
        change_type: PolicyChangeType,
        changed_by: AgentIdentity,
    },

    /// Delegation was created
    DelegationCreated {
        token_id: String,
        delegator: AgentIdentity,
        delegate: AgentIdentity,
    },

    /// Delegation expired
    DelegationExpired {
        token_id: String,
    },

    /// Operating mode changed
    ModeChanged {
        from_mode: OperatingMode,
        to_mode: OperatingMode,
        changed_by: AgentIdentity,
    },

    /// Policy decision was made
    PolicyDecision {
        decision: PolicyDecision,
        capability_id: CapabilityId,
        command: String,
    },

    /// Security violation detected
    SecurityViolation {
        violation_type: String,
        severity: ViolationSeverity,
        description: String,
    },

    /// Audit checkpoint
    AuditCheckpoint {
        checkpoint_id: String,
        events_since_last: u64,
    },
}

/// Type of policy change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyChangeType {
    Created,
    Updated,
    Deleted,
    Enabled,
    Disabled,
}

/// Operating mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperatingMode {
    Normal,
    Restricted,
    Maintenance,
    Emergency,
}

/// Violation severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Governance ledger - append-only log of governance events
pub struct GovernanceLedger {
    /// Events in memory
    events: Arc<RwLock<Vec<GovernanceEvent>>>,

    /// Next event ID
    next_event_id: Arc<RwLock<u64>>,

    /// Persistent storage (optional)
    storage: Option<LedgerStorage>,
}

impl GovernanceLedger {
    /// Create a new in-memory ledger
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
            next_event_id: Arc::new(RwLock::new(0)),
            storage: None,
        }
    }

    /// Create a ledger with persistent storage
    pub fn with_storage(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let storage = LedgerStorage::new(path)?;

        Ok(Self {
            events: Arc::new(RwLock::new(Vec::new())),
            next_event_id: Arc::new(RwLock::new(0)),
            storage: Some(storage),
        })
    }

    /// Append an event to the ledger
    pub fn append(&self, mut event: GovernanceEvent) -> EventId {
        let mut next_id = self.next_event_id.write().unwrap();
        let event_id = EventId::new(*next_id);
        *next_id += 1;

        event.event_id = event_id.clone();

        // Write to storage if available
        if let Some(storage) = &self.storage {
            let _ = storage.append(&event);
        }

        // Add to in-memory events
        let mut events = self.events.write().unwrap();
        events.push(event);

        event_id
    }

    /// Record a capability grant
    pub fn record_capability_granted(
        &self,
        capability_id: CapabilityId,
        granted_by: AgentIdentity,
        agent: AgentIdentity,
        tenant: TenantIdentity,
        correlation_id: impl Into<String>,
    ) -> EventId {
        let event = GovernanceEvent {
            event_id: EventId::new(0), // Will be set by append
            timestamp: SystemTime::now(),
            event_type: EventType::CapabilityGranted {
                capability_id,
                granted_by,
            },
            agent,
            tenant,
            correlation_id: correlation_id.into(),
            metadata: HashMap::new(),
        };

        self.append(event)
    }

    /// Record a policy change
    pub fn record_policy_changed(
        &self,
        policy_id: impl Into<String>,
        change_type: PolicyChangeType,
        changed_by: AgentIdentity,
        agent: AgentIdentity,
        tenant: TenantIdentity,
        correlation_id: impl Into<String>,
    ) -> EventId {
        let event = GovernanceEvent {
            event_id: EventId::new(0),
            timestamp: SystemTime::now(),
            event_type: EventType::PolicyChanged {
                policy_id: policy_id.into(),
                change_type,
                changed_by,
            },
            agent,
            tenant,
            correlation_id: correlation_id.into(),
            metadata: HashMap::new(),
        };

        self.append(event)
    }

    /// Record a policy decision
    pub fn record_policy_decision(
        &self,
        decision: PolicyDecision,
        capability_id: CapabilityId,
        command: impl Into<String>,
        agent: AgentIdentity,
        tenant: TenantIdentity,
        correlation_id: impl Into<String>,
    ) -> EventId {
        let event = GovernanceEvent {
            event_id: EventId::new(0),
            timestamp: SystemTime::now(),
            event_type: EventType::PolicyDecision {
                decision,
                capability_id,
                command: command.into(),
            },
            agent,
            tenant,
            correlation_id: correlation_id.into(),
            metadata: HashMap::new(),
        };

        self.append(event)
    }

    /// Record a security violation
    pub fn record_security_violation(
        &self,
        violation_type: impl Into<String>,
        severity: ViolationSeverity,
        description: impl Into<String>,
        agent: AgentIdentity,
        tenant: TenantIdentity,
        correlation_id: impl Into<String>,
    ) -> EventId {
        let event = GovernanceEvent {
            event_id: EventId::new(0),
            timestamp: SystemTime::now(),
            event_type: EventType::SecurityViolation {
                violation_type: violation_type.into(),
                severity,
                description: description.into(),
            },
            agent,
            tenant,
            correlation_id: correlation_id.into(),
            metadata: HashMap::new(),
        };

        self.append(event)
    }

    /// Query events
    pub fn query(&self) -> LedgerQuery {
        LedgerQuery::new(Arc::clone(&self.events))
    }

    /// Get total event count
    pub fn event_count(&self) -> usize {
        let events = self.events.read().unwrap();
        events.len()
    }

    /// Create a checkpoint
    pub fn checkpoint(&self, checkpoint_id: impl Into<String>, agent: AgentIdentity, tenant: TenantIdentity) -> EventId {
        let events_since_last = self.event_count() as u64;

        let event = GovernanceEvent {
            event_id: EventId::new(0),
            timestamp: SystemTime::now(),
            event_type: EventType::AuditCheckpoint {
                checkpoint_id: checkpoint_id.into(),
                events_since_last,
            },
            agent,
            tenant,
            correlation_id: "checkpoint".to_string(),
            metadata: HashMap::new(),
        };

        self.append(event)
    }
}

impl Default for GovernanceLedger {
    fn default() -> Self {
        Self::new()
    }
}

/// Query builder for governance ledger
pub struct LedgerQuery {
    events: Arc<RwLock<Vec<GovernanceEvent>>>,
    filters: Vec<Box<dyn Fn(&GovernanceEvent) -> bool + Send + Sync>>,
}

impl LedgerQuery {
    fn new(events: Arc<RwLock<Vec<GovernanceEvent>>>) -> Self {
        Self {
            events,
            filters: Vec::new(),
        }
    }

    /// Filter by time range
    pub fn time_range(mut self, start: SystemTime, end: SystemTime) -> Self {
        self.filters.push(Box::new(move |event| {
            event.timestamp >= start && event.timestamp <= end
        }));
        self
    }

    /// Filter by agent
    pub fn agent(mut self, agent_id: String) -> Self {
        self.filters.push(Box::new(move |event| {
            event.agent.agent_id == agent_id
        }));
        self
    }

    /// Filter by tenant
    pub fn tenant(mut self, tenant_id: String) -> Self {
        self.filters.push(Box::new(move |event| {
            event.tenant.tenant_id == tenant_id
        }));
        self
    }

    /// Filter by correlation ID
    pub fn correlation_id(mut self, correlation_id: String) -> Self {
        self.filters.push(Box::new(move |event| {
            event.correlation_id == correlation_id
        }));
        self
    }

    /// Execute query and return matching events
    pub fn execute(self) -> Vec<GovernanceEvent> {
        let events = self.events.read().unwrap();

        events
            .iter()
            .filter(|event| self.filters.iter().all(|f| f(event)))
            .cloned()
            .collect()
    }

    /// Count matching events
    pub fn count(self) -> usize {
        let events = self.events.read().unwrap();

        events
            .iter()
            .filter(|event| self.filters.iter().all(|f| f(event)))
            .count()
    }
}

/// Persistent storage for governance ledger
struct LedgerStorage {
    /// File path
    #[allow(dead_code)]
    path: std::path::PathBuf,

    /// File handle for appending
    file: Arc<RwLock<std::fs::File>>,
}

impl LedgerStorage {
    /// Create new storage
    fn new(path: impl AsRef<Path>) -> std::io::Result<Self> {
        use std::fs::OpenOptions;

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;

        Ok(Self {
            path: path.as_ref().to_path_buf(),
            file: Arc::new(RwLock::new(file)),
        })
    }

    /// Append an event
    fn append(&self, event: &GovernanceEvent) -> std::io::Result<()> {
        let mut file = self.file.write().unwrap();

        // Serialize as JSON line
        let json = serde_json::to_string(event)?;
        writeln!(file, "{}", json)?;
        file.flush()?;

        Ok(())
    }

    /// Load all events
    fn load_all(&self) -> std::io::Result<Vec<GovernanceEvent>> {
        let file = std::fs::File::open(&self.path)?;
        let reader = BufReader::new(file);

        let mut events = Vec::new();
        for line in reader.lines() {
            let line = line?;
            if let Ok(event) = serde_json::from_str::<GovernanceEvent>(&line) {
                events.push(event);
            }
        }

        Ok(events)
    }
}

/// Replay engine for re-evaluating decisions
pub struct ReplayEngine {
    /// Original ledger
    ledger: Arc<GovernanceLedger>,
}

impl ReplayEngine {
    /// Create a new replay engine
    pub fn new(ledger: Arc<GovernanceLedger>) -> Self {
        Self { ledger }
    }

    /// Replay a time slice with original policies
    pub fn replay_timeslice(
        &self,
        start: SystemTime,
        end: SystemTime,
    ) -> ReplayResult {
        let events = self.ledger.query().time_range(start, end).execute();

        // Extract policy decisions
        let decisions: Vec<_> = events
            .iter()
            .filter_map(|event| {
                if let EventType::PolicyDecision { decision, capability_id, command } = &event.event_type {
                    Some((decision.clone(), capability_id.clone(), command.clone()))
                } else {
                    None
                }
            })
            .collect();

        ReplayResult {
            total_events: events.len(),
            policy_decisions: decisions.len(),
            decisions,
            differences: vec![],
        }
    }

    /// Replay with "what if" policy changes
    pub fn replay_with_policy<F>(
        &self,
        start: SystemTime,
        end: SystemTime,
        policy_evaluator: F,
    ) -> ReplayResult
    where
        F: Fn(&CapabilityId, &str) -> PolicyDecision,
    {
        let events = self.ledger.query().time_range(start, end).execute();

        let mut original_decisions = Vec::new();
        let mut new_decisions = Vec::new();
        let mut differences = Vec::new();

        for event in &events {
            if let EventType::PolicyDecision { decision, capability_id, command } = &event.event_type {
                original_decisions.push((decision.clone(), capability_id.clone(), command.clone()));

                // Re-evaluate with new policy
                let new_decision = policy_evaluator(capability_id, command);

                new_decisions.push((new_decision.clone(), capability_id.clone(), command.clone()));

                // Check if different
                if !self.decisions_match(decision, &new_decision) {
                    differences.push(DecisionDifference {
                        event_id: event.event_id.clone(),
                        original: decision.clone(),
                        new: new_decision,
                        capability_id: capability_id.clone(),
                        command: command.clone(),
                    });
                }
            }
        }

        ReplayResult {
            total_events: events.len(),
            policy_decisions: original_decisions.len(),
            decisions: new_decisions,
            differences,
        }
    }

    fn decisions_match(&self, a: &PolicyDecision, b: &PolicyDecision) -> bool {
        match (a, b) {
            (PolicyDecision::Allow, PolicyDecision::Allow) => true,
            (PolicyDecision::Deny { .. }, PolicyDecision::Deny { .. }) => true,
            _ => false,
        }
    }
}

/// Result of replay operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayResult {
    /// Total events replayed
    pub total_events: usize,

    /// Number of policy decisions
    pub policy_decisions: usize,

    /// Decisions made
    pub decisions: Vec<(PolicyDecision, CapabilityId, String)>,

    /// Differences from original (if "what if" replay)
    pub differences: Vec<DecisionDifference>,
}

impl ReplayResult {
    /// Get statistics
    pub fn stats(&self) -> ReplayStats {
        let allow_count = self.decisions.iter().filter(|(d, _, _)| matches!(d, PolicyDecision::Allow)).count();
        let deny_count = self.decisions.iter().filter(|(d, _, _)| matches!(d, PolicyDecision::Deny { .. })).count();

        ReplayStats {
            total_events: self.total_events,
            policy_decisions: self.policy_decisions,
            allow_count,
            deny_count,
            differences_count: self.differences.len(),
        }
    }
}

/// Replay statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayStats {
    pub total_events: usize,
    pub policy_decisions: usize,
    pub allow_count: usize,
    pub deny_count: usize,
    pub differences_count: usize,
}

/// Difference in policy decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionDifference {
    pub event_id: EventId,
    pub original: PolicyDecision,
    pub new: PolicyDecision,
    pub capability_id: CapabilityId,
    pub command: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_governance_ledger() {
        let ledger = GovernanceLedger::new();

        ledger.record_capability_granted(
            CapabilityId::from_path("user.create"),
            AgentIdentity::anonymous(),
            AgentIdentity::anonymous(),
            TenantIdentity::default_tenant(),
            "test",
        );

        assert_eq!(ledger.event_count(), 1);
    }

    #[test]
    fn test_ledger_query() {
        let ledger = GovernanceLedger::new();

        let agent1 = AgentIdentity::human("user1");
        let agent2 = AgentIdentity::human("user2");

        ledger.record_capability_granted(
            CapabilityId::from_path("cap1"),
            agent1.clone(),
            agent1.clone(),
            TenantIdentity::default_tenant(),
            "test1",
        );

        ledger.record_capability_granted(
            CapabilityId::from_path("cap2"),
            agent2.clone(),
            agent2.clone(),
            TenantIdentity::default_tenant(),
            "test2",
        );

        let events = ledger.query().agent("user1".to_string()).execute();
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_replay_engine() {
        let ledger = Arc::new(GovernanceLedger::new());

        ledger.record_policy_decision(
            PolicyDecision::Allow,
            CapabilityId::from_path("test.cmd"),
            "test command",
            AgentIdentity::anonymous(),
            TenantIdentity::default_tenant(),
            "test",
        );

        let engine = ReplayEngine::new(Arc::clone(&ledger));

        let result = engine.replay_timeslice(
            SystemTime::UNIX_EPOCH,
            SystemTime::now(),
        );

        assert_eq!(result.policy_decisions, 1);
    }
}
