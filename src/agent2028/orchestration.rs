/// Orchestration Layer for Trillion-Agent Ecosystems
///
/// Central dispatcher and coordinator that bridges:
/// - Individual agent coordination (2028)
/// - Swarm intelligence (2029-2030+)
/// - Resource management and scheduling
/// - Cross-tier communication and failure recovery

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Agent execution tier (2028 or 2029+)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentTier {
    /// Individual agent coordination (2028)
    Individual,
    /// Swarm intelligence (2029-2030+)
    Swarm,
}

/// Operation result with success/failure and error details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResult {
    pub operation_id: String,
    pub success: bool,
    pub tier_executed: AgentTier,
    pub result_data: String,
    pub error: Option<String>,
    pub execution_time_ms: u64,
}

impl OperationResult {
    pub fn success(operation_id: String, tier: AgentTier, data: String, time_ms: u64) -> Self {
        Self {
            operation_id,
            success: true,
            tier_executed: tier,
            result_data: data,
            error: None,
            execution_time_ms: time_ms,
        }
    }

    pub fn failure(operation_id: String, tier: AgentTier, error: String, time_ms: u64) -> Self {
        Self {
            operation_id,
            success: false,
            tier_executed: tier,
            result_data: String::new(),
            error: Some(error),
            execution_time_ms: time_ms,
        }
    }
}

/// Agent operation request routed through orchestrator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationRequest {
    pub operation_id: String,
    pub agent_id: String,
    pub operation_type: String,        // e.g., "compute", "coordinate", "swarm"
    pub tier_preference: Option<AgentTier>, // Which tier should handle this
    pub payload: String,
    pub priority: u32,                 // 1-10, 10 is highest
    pub timeout_ms: u64,
}

impl OperationRequest {
    pub fn new(agent_id: String, op_type: String, payload: String) -> Self {
        Self {
            operation_id: Uuid::new_v4().to_string(),
            agent_id,
            operation_type: op_type,
            tier_preference: None,
            payload,
            priority: 5,
            timeout_ms: 30000,
        }
    }
}

/// Resource allocation tracking per tier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocation {
    pub tier: AgentTier,
    pub cpu_percent: f64,
    pub memory_mb: u64,
    pub agents_active: usize,
    pub agents_capacity: usize,
}

/// Central Orchestrator - routes and coordinates all agent operations
pub struct Orchestrator {
    operation_queue: Arc<RwLock<Vec<OperationRequest>>>,
    completed_operations: Arc<RwLock<Vec<OperationResult>>>,
    resource_allocations: Arc<RwLock<HashMap<String, ResourceAllocation>>>,
    agent_registry: Arc<RwLock<HashMap<String, AgentTier>>>,
    tier_status: Arc<RwLock<HashMap<AgentTier, bool>>>, // tier -> is_healthy
}

impl Orchestrator {
    pub fn new() -> Self {
        let mut tier_status = HashMap::new();
        tier_status.insert(AgentTier::Individual, true);
        tier_status.insert(AgentTier::Swarm, true);

        Self {
            operation_queue: Arc::new(RwLock::new(Vec::new())),
            completed_operations: Arc::new(RwLock::new(Vec::new())),
            resource_allocations: Arc::new(RwLock::new(HashMap::new())),
            agent_registry: Arc::new(RwLock::new(HashMap::new())),
            tier_status: Arc::new(RwLock::new(tier_status)),
        }
    }

    /// Register agent in registry with tier assignment
    pub async fn register_agent(&self, agent_id: String, tier: AgentTier) {
        let mut registry = self.agent_registry.write().await;
        registry.insert(agent_id, tier);
    }

    /// Route operation to appropriate tier
    pub async fn route_operation(&self, mut request: OperationRequest) -> OperationResult {
        let start = std::time::Instant::now();
        let operation_id = request.operation_id.clone();

        // Determine tier
        let tier = if let Some(pref) = request.tier_preference {
            pref
        } else {
            // Auto-route based on operation type
            self.determine_tier(&request.operation_type)
        };

        // Check tier health
        let tier_status = self.tier_status.read().await;
        if !tier_status.get(&tier).copied().unwrap_or(false) {
            let elapsed = start.elapsed().as_millis() as u64;
            return OperationResult::failure(
                operation_id,
                tier,
                format!("Tier {:?} is unhealthy", tier),
                elapsed,
            );
        }
        drop(tier_status);

        // Queue operation
        let mut queue = self.operation_queue.write().await;
        queue.push(request);
        drop(queue);

        // Simulate execution
        let elapsed = start.elapsed().as_millis() as u64;
        let result = OperationResult::success(
            operation_id.clone(),
            tier,
            format!("Operation executed successfully"),
            elapsed,
        );

        // Record completion
        let mut completed = self.completed_operations.write().await;
        completed.push(result.clone());

        result
    }

    /// Determine tier based on operation type
    fn determine_tier(&self, op_type: &str) -> AgentTier {
        match op_type.to_lowercase().as_str() {
            "swarm" | "collective" | "consensus" | "emergence" => AgentTier::Swarm,
            "trust" | "contract" | "capability" | "auction" => AgentTier::Swarm,
            _ => AgentTier::Individual,
        }
    }

    /// Allocate resources to tier
    pub async fn allocate_resources(
        &self,
        tier: AgentTier,
        cpu_percent: f64,
        memory_mb: u64,
        agent_capacity: usize,
    ) {
        let allocation = ResourceAllocation {
            tier,
            cpu_percent,
            memory_mb,
            agents_active: 0,
            agents_capacity: agent_capacity,
        };

        let mut allocs = self.resource_allocations.write().await;
        allocs.insert(format!("{:?}", tier), allocation);
    }

    /// Get resource usage
    pub async fn resource_usage(&self) -> HashMap<String, ResourceAllocation> {
        self.resource_allocations.read().await.clone()
    }

    /// Get operations queue length
    pub async fn queue_length(&self) -> usize {
        self.operation_queue.read().await.len()
    }

    /// Get completed operations count
    pub async fn completed_count(&self) -> usize {
        self.completed_operations.read().await.len()
    }

    /// Mark tier as unhealthy
    pub async fn mark_tier_unhealthy(&self, tier: AgentTier) {
        let mut status = self.tier_status.write().await;
        status.insert(tier, false);
    }

    /// Mark tier as healthy
    pub async fn mark_tier_healthy(&self, tier: AgentTier) {
        let mut status = self.tier_status.write().await;
        status.insert(tier, true);
    }

    /// Get orchestrator stats
    pub async fn stats(&self) -> OrchestrationStats {
        let queue_len = self.operation_queue.read().await.len();
        let completed_len = self.completed_operations.read().await.len();
        let registry_len = self.agent_registry.read().await.len();
        let tier_status = self.tier_status.read().await.clone();

        OrchestrationStats {
            queued_operations: queue_len,
            completed_operations: completed_len,
            total_agents: registry_len,
            tiers_healthy: tier_status
                .iter()
                .filter(|(_, &healthy)| healthy)
                .count(),
            tiers_total: tier_status.len(),
        }
    }
}

impl Default for Orchestrator {
    fn default() -> Self {
        Self::new()
    }
}

/// Orchestration statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationStats {
    pub queued_operations: usize,
    pub completed_operations: usize,
    pub total_agents: usize,
    pub tiers_healthy: usize,
    pub tiers_total: usize,
}

/// Integration Bridge - translates between 2028 and swarm systems
pub struct IntegrationBridge {
    individual_to_swarm_mapping: Arc<RwLock<HashMap<String, Vec<String>>>>, // individual -> swarm agents
    swarm_to_individual_mapping: Arc<RwLock<HashMap<String, String>>>,      // swarm -> individual coordinator
}

impl IntegrationBridge {
    pub fn new() -> Self {
        Self {
            individual_to_swarm_mapping: Arc::new(RwLock::new(HashMap::new())),
            swarm_to_individual_mapping: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Map individual agent to swarm agents
    pub async fn add_agent_to_swarm(
        &self,
        individual_agent: String,
        swarm_agents: Vec<String>,
    ) {
        let mut mapping = self.individual_to_swarm_mapping.write().await;
        mapping.insert(individual_agent, swarm_agents);
    }

    /// Get swarm agents for individual agent
    pub async fn get_swarm_agents(&self, individual_agent: &str) -> Option<Vec<String>> {
        self.individual_to_swarm_mapping
            .read()
            .await
            .get(individual_agent)
            .cloned()
    }

    /// Map swarm agents back to coordinating individual
    pub async fn set_swarm_coordinator(&self, swarm_id: String, individual_agent: String) {
        let mut mapping = self.swarm_to_individual_mapping.write().await;
        mapping.insert(swarm_id, individual_agent);
    }

    /// Translate individual agent request to swarm operation
    pub fn translate_to_swarm_operation(&self, request: &OperationRequest) -> OperationRequest {
        let mut swarm_request = request.clone();
        swarm_request.tier_preference = Some(AgentTier::Swarm);
        swarm_request.operation_type = format!("swarm_{}", request.operation_type);
        swarm_request
    }

    /// Translate swarm result back to individual agent response
    pub fn translate_from_swarm_result(&self, result: &OperationResult) -> OperationResult {
        let mut individual_result = result.clone();
        individual_result.tier_executed = AgentTier::Individual;
        individual_result
    }
}

impl Default for IntegrationBridge {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_orchestrator_creation() {
        let orchestrator = Orchestrator::new();
        let stats = orchestrator.stats().await;

        assert_eq!(stats.queued_operations, 0);
        assert_eq!(stats.completed_operations, 0);
        assert_eq!(stats.tiers_total, 2);
        assert_eq!(stats.tiers_healthy, 2);
    }

    #[tokio::test]
    async fn test_agent_registration() {
        let orchestrator = Orchestrator::new();
        orchestrator
            .register_agent("agent-1".to_string(), AgentTier::Individual)
            .await;

        let stats = orchestrator.stats().await;
        assert_eq!(stats.total_agents, 1);
    }

    #[tokio::test]
    async fn test_operation_routing() {
        let orchestrator = Orchestrator::new();
        orchestrator
            .allocate_resources(AgentTier::Individual, 50.0, 1024, 100)
            .await;

        let request = OperationRequest::new(
            "agent-1".to_string(),
            "compute".to_string(),
            "test payload".to_string(),
        );

        let result = orchestrator.route_operation(request).await;
        assert!(result.success);
        assert_eq!(result.tier_executed, AgentTier::Individual);

        let stats = orchestrator.stats().await;
        assert_eq!(stats.completed_operations, 1);
    }

    #[tokio::test]
    async fn test_integration_bridge() {
        let bridge = IntegrationBridge::new();

        bridge
            .add_agent_to_swarm(
                "agent-1".to_string(),
                vec!["swarm-1".to_string(), "swarm-2".to_string()],
            )
            .await;

        let swarm_agents = bridge.get_swarm_agents("agent-1").await;
        assert_eq!(swarm_agents.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_tier_health_check() {
        let orchestrator = Orchestrator::new();

        orchestrator.mark_tier_unhealthy(AgentTier::Individual).await;

        let request = OperationRequest::new(
            "agent-1".to_string(),
            "compute".to_string(),
            "test".to_string(),
        );

        let result = orchestrator.route_operation(request).await;
        assert!(!result.success);
    }
}
