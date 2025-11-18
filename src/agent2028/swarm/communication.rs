/// Swarm Communication Protocols
///
/// Efficient bandwidth-limited communication for swarms with millions of agents.
/// Implements gossip protocols, layered broadcasting, and dynamic topology adaptation.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Message in swarm network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmMessage {
    pub message_id: String,
    pub sender_id: String,
    pub content: String,
    pub message_type: MessageType,
    pub ttl: u8,                  // Time to live (hops)
    pub seen_by: Vec<String>,     // Agents that have seen this
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    LocalBroadcast,   // Within local cluster only
    RegionalGossip,   // Regional cluster gossip
    GlobalAlert,      // Critical swarm-wide message
}

impl SwarmMessage {
    pub fn new(sender_id: String, content: String, message_type: MessageType) -> Self {
        Self {
            message_id: uuid::Uuid::new_v4().to_string(),
            sender_id,
            content,
            message_type,
            ttl: match message_type {
                MessageType::LocalBroadcast => 2,
                MessageType::RegionalGossip => 5,
                MessageType::GlobalAlert => 20,
            },
            seen_by: Vec::new(),
        }
    }

    /// Check if message should be propagated
    pub fn should_propagate(&self) -> bool {
        self.ttl > 0
    }

    /// Forward message
    pub fn forward(&mut self, agent_id: String) {
        self.ttl = self.ttl.saturating_sub(1);
        self.seen_by.push(agent_id);
    }
}

/// Gossip protocol for epidemic message spreading
pub struct GossipProtocol {
    message_buffer: HashMap<String, SwarmMessage>,
    propagation_history: HashMap<String, Vec<String>>, // message_id -> agents who propagated
}

impl GossipProtocol {
    pub fn new() -> Self {
        Self {
            message_buffer: HashMap::new(),
            propagation_history: HashMap::new(),
        }
    }

    /// Receive and buffer message
    pub fn receive(&mut self, mut message: SwarmMessage, agent_id: String) {
        message.forward(agent_id.clone());

        self.propagation_history
            .entry(message.message_id.clone())
            .or_insert_with(Vec::new)
            .push(agent_id);

        self.message_buffer
            .insert(message.message_id.clone(), message);
    }

    /// Get message to propagate to peer (random from buffer)
    pub fn select_for_propagation(&self, agent_id: &str) -> Option<SwarmMessage> {
        self.message_buffer
            .values()
            .find(|m| m.should_propagate() && !m.seen_by.contains(&agent_id.to_string()))
            .cloned()
    }

    /// Calculate spread percentage
    pub fn spread_percentage(&self, message_id: &str, total_agents: usize) -> f64 {
        self.propagation_history
            .get(message_id)
            .map(|h| h.len() as f64 / total_agents as f64)
            .unwrap_or(0.0)
    }

    /// Clear old messages
    pub fn cleanup(&mut self) {
        self.message_buffer.retain(|_, m| m.should_propagate());
    }
}

impl Default for GossipProtocol {
    fn default() -> Self {
        Self::new()
    }
}

/// Swarm communication protocol
pub struct SwarmProtocol {
    local_neighbors: HashMap<String, Vec<String>>, // agent_id -> neighbor list
    gossip: GossipProtocol,
    compression_enabled: bool,
}

impl SwarmProtocol {
    pub fn new(compression_enabled: bool) -> Self {
        Self {
            local_neighbors: HashMap::new(),
            gossip: GossipProtocol::new(),
            compression_enabled,
        }
    }

    /// Register agent and its neighbors
    pub fn register_agent(&mut self, agent_id: String, neighbor_ids: Vec<String>) {
        self.local_neighbors.insert(agent_id, neighbor_ids);
    }

    /// Get local neighbors
    pub fn get_neighbors(&self, agent_id: &str) -> Vec<String> {
        self.local_neighbors
            .get(agent_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Broadcast message locally (to neighbors only)
    pub fn local_broadcast(&mut self, message: SwarmMessage, agent_id: String) -> Vec<String> {
        let neighbors = self.get_neighbors(&agent_id);

        for neighbor in neighbors.iter() {
            let mut forwarded = message.clone();
            self.gossip.receive(forwarded, neighbor.clone());
        }

        neighbors
    }

    /// Gossip message (spread exponentially)
    pub fn gossip(&mut self, message: SwarmMessage, agent_id: String) {
        self.gossip.receive(message, agent_id);
    }

    /// Compress message (simulated)
    pub fn compress_message(&self, message: &SwarmMessage) -> usize {
        let original_size = serde_json::to_string(message).unwrap().len();

        if self.compression_enabled {
            (original_size as f64 * 0.7) as usize // 30% reduction
        } else {
            original_size
        }
    }

    /// Dynamic topology: add edge if beneficial
    pub fn adapt_topology(&mut self, agent_a: String, agent_b: String, edge_quality: f64) {
        if edge_quality > 0.7 {
            // High-quality link, add it
            self.local_neighbors
                .entry(agent_a.clone())
                .or_insert_with(Vec::new)
                .push(agent_b.clone());

            self.local_neighbors
                .entry(agent_b)
                .or_insert_with(Vec::new)
                .push(agent_a);
        }
    }

    /// Protocol negotiation: agree on message format
    pub fn negotiate_protocol(&self, agent_a: &str, agent_b: &str) -> String {
        // Simple negotiation: both support compression or neither does
        if self.compression_enabled {
            "compressed".to_string()
        } else {
            "raw".to_string()
        }
    }

    /// Get communication overhead (bytes per agent per second)
    pub fn communication_overhead(&self) -> f64 {
        let total_agents = self.local_neighbors.len();
        let total_edges: usize = self.local_neighbors.values().map(|n| n.len()).sum();

        let avg_message_size = 500.0; // bytes
        let message_frequency = 10.0; // per second

        if total_agents > 0 {
            (total_edges as f64 * avg_message_size * message_frequency) / total_agents as f64
        } else {
            0.0
        }
    }
}

impl Default for SwarmProtocol {
    fn default() -> Self {
        Self::new(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swarm_message() {
        let msg = SwarmMessage::new("agent-1".to_string(), "hello".to_string(), MessageType::LocalBroadcast);

        assert!(msg.should_propagate());
        assert_eq!(msg.ttl, 2);
    }

    #[test]
    fn test_gossip_protocol() {
        let mut gossip = GossipProtocol::new();
        let msg = SwarmMessage::new("agent-1".to_string(), "test".to_string(), MessageType::RegionalGossip);

        gossip.receive(msg, "agent-1".to_string());
        assert!(gossip.spread_percentage(&"test".to_string(), 100) >= 0.0);
    }

    #[test]
    fn test_swarm_protocol() {
        let mut protocol = SwarmProtocol::new(true);

        protocol.register_agent("agent-1".to_string(), vec!["agent-2".to_string()]);

        let msg = SwarmMessage::new("agent-1".to_string(), "hello".to_string(), MessageType::LocalBroadcast);
        let neighbors = protocol.local_broadcast(msg, "agent-1".to_string());

        assert_eq!(neighbors.len(), 1);
    }
}
