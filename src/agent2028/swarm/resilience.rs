/// Swarm Resilience & Adaptation
///
/// Strategies for maintaining swarm function despite agent failures
/// and dynamic environmental changes.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Agent health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Critical,
    Failed,
}

/// Resilience metrics for swarms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResilienceMetrics {
    pub total_agents: usize,
    pub healthy_agents: usize,
    pub degraded_agents: usize,
    pub failed_agents: usize,
    pub functional_capacity: f64, // Percentage of full capacity
    pub redundancy_factor: f64,   // How many agents can fail before collapse
}

impl ResilienceMetrics {
    pub fn calculate(agent_states: &[(String, HealthStatus)]) -> Self {
        let total = agent_states.len();
        let healthy = agent_states.iter().filter(|(_, s)| *s == HealthStatus::Healthy).count();
        let degraded = agent_states.iter().filter(|(_, s)| *s == HealthStatus::Degraded).count();
        let failed = agent_states.iter().filter(|(_, s)| *s == HealthStatus::Failed).count();

        let functional = (healthy + degraded / 2) as f64 / total.max(1) as f64;
        let redundancy = (healthy as f64 / total as f64).max(0.01);

        Self {
            total_agents: total,
            healthy_agents: healthy,
            degraded_agents: degraded,
            failed_agents: failed,
            functional_capacity: functional,
            redundancy_factor: redundancy,
        }
    }
}

/// Swarm Resilience System
pub struct SwarmResilience {
    agent_states: HashMap<String, HealthStatus>,
    role_redundancy: HashMap<String, Vec<String>>, // role -> agents
    recovery_strategies: HashMap<String, String>,
}

impl SwarmResilience {
    pub fn new() -> Self {
        Self {
            agent_states: HashMap::new(),
            role_redundancy: HashMap::new(),
            recovery_strategies: HashMap::new(),
        }
    }

    /// Register agent for role redundancy
    pub fn register_for_role(&mut self, agent_id: String, role: String) {
        self.agent_states.insert(agent_id.clone(), HealthStatus::Healthy);
        self.role_redundancy.entry(role).or_insert_with(Vec::new).push(agent_id);
    }

    /// Report agent health status
    pub fn set_agent_health(&mut self, agent_id: &str, status: HealthStatus) {
        self.agent_states.insert(agent_id.to_string(), status);
    }

    /// Get agents for a role (prefers healthy ones)
    pub fn get_role_agents(&self, role: &str) -> Vec<String> {
        let mut agents = self.role_redundancy.get(role).cloned().unwrap_or_default();

        // Sort by health status (healthy first)
        agents.sort_by_key(|a| match self.agent_states.get(a) {
            Some(HealthStatus::Healthy) => 0,
            Some(HealthStatus::Degraded) => 1,
            Some(HealthStatus::Critical) => 2,
            _ => 3,
        });

        agents
    }

    /// Activate backup agent for failed agent
    pub fn activate_backup(&self, failed_agent: &str, role: &str) -> Option<String> {
        self.get_role_agents(role)
            .iter()
            .find(|a| {
                self.agent_states.get(*a) != Some(&HealthStatus::Failed) && *a != failed_agent
            })
            .cloned()
    }

    /// Implement graceful degradation
    pub fn degrade_gracefully(&mut self, failed_agent: &str) {
        // Mark as failed
        self.agent_states.insert(failed_agent.to_string(), HealthStatus::Failed);

        // Find and activate backup
        for (role, agents) in self.role_redundancy.iter() {
            if agents.contains(&failed_agent.to_string()) {
                if let Some(backup) = self.activate_backup(failed_agent, role) {
                    // Mark backup as critical (taking extra load)
                    self.agent_states.insert(backup, HealthStatus::Critical);
                }
            }
        }
    }

    /// Get resilience metrics
    pub fn metrics(&self) -> ResilienceMetrics {
        let agent_states: Vec<(String, HealthStatus)> =
            self.agent_states.iter().map(|(k, v)| (k.clone(), *v)).collect();
        ResilienceMetrics::calculate(&agent_states)
    }

    /// Check if swarm is still functional
    pub fn is_functional(&self) -> bool {
        let metrics = self.metrics();
        metrics.functional_capacity > 0.5 // More than 50% capacity
    }

    /// Check if swarm can tolerate more failures
    pub fn failure_tolerance(&self) -> usize {
        let metrics = self.metrics();
        (metrics.healthy_agents as f64 * metrics.redundancy_factor).ceil() as usize
    }

    /// Implement role flexibility (agents switch roles)
    pub fn adapt_role_assignment(&mut self, _swarm_goal: &str) {
        // Simple heuristic: agents with critical status should change roles
        for agent in self.agent_states.keys().cloned().collect::<Vec<_>>() {
            if let Some(HealthStatus::Critical) = self.agent_states.get(&agent) {
                // Try to find less loaded role
                if let Some((_role, agents)) =
                    self.role_redundancy.iter_mut().min_by_key(|(_, a)| a.len())
                {
                    if !agents.contains(&agent) {
                        agents.push(agent.clone());
                    }
                }
            }
        }
    }
}

impl Default for SwarmResilience {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resilience_metrics() {
        let states = vec![
            ("agent-1".to_string(), HealthStatus::Healthy),
            ("agent-2".to_string(), HealthStatus::Healthy),
            ("agent-3".to_string(), HealthStatus::Failed),
        ];

        let metrics = ResilienceMetrics::calculate(&states);
        assert_eq!(metrics.healthy_agents, 2);
        assert_eq!(metrics.failed_agents, 1);
    }

    #[test]
    fn test_swarm_resilience() {
        let mut resilience = SwarmResilience::new();

        resilience.register_for_role("agent-1".to_string(), "scout".to_string());
        resilience.register_for_role("agent-2".to_string(), "scout".to_string());

        let agents = resilience.get_role_agents("scout");
        assert_eq!(agents.len(), 2);

        resilience.degrade_gracefully("agent-1");
        assert!(!resilience.is_functional() || resilience.metrics().healthy_agents > 0);
    }
}
