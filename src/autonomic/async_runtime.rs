//! Async Runtime Support for Distributed Agent Swarms
//!
//! This module provides async/await integration for trillion-scale distributed
//! agent swarms operating across networks.

use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

use super::*;

/// Async policy engine trait for distributed policy evaluation
#[async_trait::async_trait]
pub trait AsyncPolicyEngine: Send + Sync {
    /// Evaluate policy asynchronously, potentially across network
    async fn evaluate_async(&self, request: &PolicyRequest) -> Result<PolicyResult, String>;

    /// Batch evaluate multiple requests for efficiency
    async fn evaluate_batch_async(
        &self,
        requests: &[PolicyRequest],
    ) -> Vec<Result<PolicyResult, String>>;
}

/// Async delegation registry with distributed consensus
pub struct AsyncDelegationRegistry {
    /// Local cache of delegations
    local: Arc<RwLock<DelegationRegistry>>,

    /// Channel for distributed sync
    sync_tx: mpsc::UnboundedSender<DelegationSyncMessage>,
}

impl AsyncDelegationRegistry {
    /// Create a new async delegation registry
    pub fn new() -> (Self, mpsc::UnboundedReceiver<DelegationSyncMessage>) {
        let (sync_tx, sync_rx) = mpsc::unbounded_channel();

        let registry = Self {
            local: Arc::new(RwLock::new(DelegationRegistry::new())),
            sync_tx,
        };

        (registry, sync_rx)
    }

    /// Register a delegation token asynchronously
    pub async fn register_async(&self, token: DelegationToken) -> Result<(), String> {
        // Acquire write lock
        let mut local = self.local.write().await;

        // Register locally
        local.register(token.clone());

        // Broadcast to distributed nodes
        let _ = self.sync_tx.send(DelegationSyncMessage::Register(token));

        Ok(())
    }

    /// Get token asynchronously
    pub async fn get_token_async(&self, token_id: &str) -> Option<DelegationToken> {
        let local = self.local.read().await;
        local.get(&TokenId(token_id.to_string()))
    }

    /// Revoke token asynchronously
    pub async fn revoke_async(&self, token_id: &str) {
        let mut local = self.local.write().await;
        local.revoke(&TokenId(token_id.to_string()));
    }
}

impl Default for AsyncDelegationRegistry {
    fn default() -> Self {
        Self::new().0
    }
}

/// Messages for distributed delegation synchronization
#[derive(Clone)]
pub enum DelegationSyncMessage {
    /// Register a new delegation
    Register(DelegationToken),
    /// Revoke a delegation
    Revoke(String),
}

/// Async graph query executor for parallel composition
pub struct AsyncGraphQueryExecutor {
    graph: Arc<RwLock<CapabilityGraph>>,
}

impl AsyncGraphQueryExecutor {
    /// Create a new async graph executor
    pub fn new(graph: CapabilityGraph) -> Self {
        Self {
            graph: Arc::new(RwLock::new(graph)),
        }
    }

    /// Compute reachability asynchronously
    pub async fn is_reachable_async(&self, from: NodeId, to: NodeId) -> bool {
        let graph = self.graph.read().await;
        graph.is_reachable(from, to)
    }

    /// Compute shortest path asynchronously
    pub async fn shortest_path_async(&self, from: NodeId, to: NodeId) -> Option<Vec<NodeId>> {
        let graph = self.graph.read().await;
        graph.shortest_path(from, to)
    }

    /// Get graph statistics asynchronously
    pub async fn stats_async(&self) -> GraphStats {
        let graph = self.graph.read().await;
        graph.stats()
    }

    /// Batch reachability queries in parallel
    pub async fn batch_reachability_async(
        &self,
        queries: Vec<(NodeId, NodeId)>,
    ) -> Vec<bool> {
        let graph = self.graph.read().await;

        queries
            .into_iter()
            .map(|(from, to)| graph.is_reachable(from, to))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_async_delegation_registry() {
        let (registry, mut _sync_rx) = AsyncDelegationRegistry::new();

        let delegator = Principal::new(
            AgentIdentity::anonymous(),
            TenantIdentity::default_tenant(),
        );

        let delegate = Principal::new(
            AgentIdentity::anonymous(),
            TenantIdentity::default_tenant(),
        );

        let constraint = CapabilityConstraint::unrestricted();

        let token = DelegationToken::new(
            delegator,
            delegate,
            constraint,
            TemporalConstraint::valid_for(std::time::Duration::from_secs(3600)),
        );

        // Register async
        let result = registry.register_async(token.clone()).await;
        assert!(result.is_ok());

        // Retrieve async
        let retrieved = registry.get_token_async(&token.token_id.0).await;
        assert!(retrieved.is_some());
    }

    #[tokio::test]
    async fn test_async_graph_queries() {
        let mut graph = CapabilityGraph::new();

        let n1 = graph.add_node(
            CapabilityId::from_path("a"),
            "A",
            InputSchema::default(),
            OutputSchema::new(TypeSchema::primitive(PrimitiveType::String)),
            vec![],
        );

        let n2 = graph.add_node(
            CapabilityId::from_path("b"),
            "B",
            InputSchema::default(),
            OutputSchema::new(TypeSchema::primitive(PrimitiveType::String)),
            vec![],
        );

        graph.add_edge(n1, n2, EdgeType::Produces).unwrap();

        let executor = AsyncGraphQueryExecutor::new(graph);

        // Async reachability query
        let reachable = executor.is_reachable_async(n1, n2).await;
        assert!(reachable);

        // Async shortest path
        let path = executor.shortest_path_async(n1, n2).await;
        assert!(path.is_some());
    }
}
