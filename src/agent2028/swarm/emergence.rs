/// Emergent Behavior & Self-Organization
///
/// Simple rules that lead to complex emergent behaviors without central control.
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;

/// State transition rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub name: String,
    pub condition: String,
    pub action: String,
    pub success_rate: f64, // Empirical success rate
}

impl Rule {
    pub fn new(name: String, condition: String, action: String) -> Self {
        Self { name, condition, action, success_rate: 0.5 }
    }
}

/// Rule Engine for simple behavioral rules
pub struct RuleEngine {
    rules: Vec<Rule>,
    match_count: HashMap<String, usize>,
    success_count: HashMap<String, usize>,
}

impl RuleEngine {
    pub fn new() -> Self {
        Self { rules: Vec::new(), match_count: HashMap::new(), success_count: HashMap::new() }
    }

    /// Add a rule
    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
    }

    /// Evaluate rules for an agent state
    pub fn evaluate(&mut self, agent_state: &str) -> Vec<String> {
        let mut actions = Vec::new();

        for rule in &self.rules {
            // Simple string matching for conditions
            if agent_state.contains(&rule.condition) {
                actions.push(rule.action.clone());

                // Track rule usage
                *self.match_count.entry(rule.name.clone()).or_insert(0) += 1;
            }
        }

        actions
    }

    /// Record successful action
    pub fn record_success(&mut self, rule_name: &str) {
        *self.success_count.entry(rule_name.to_string()).or_insert(0) += 1;

        // Update success rate
        for rule in &mut self.rules {
            if rule.name == rule_name {
                let matches = self.match_count.get(rule_name).copied().unwrap_or(1);
                let successes = self.success_count.get(rule_name).copied().unwrap_or(0);
                rule.success_rate = successes as f64 / matches as f64;
            }
        }
    }

    /// Get most successful rules
    pub fn get_best_rules(&self, top_n: usize) -> Vec<&Rule> {
        let mut sorted_rules: Vec<&Rule> = self.rules.iter().collect();
        sorted_rules
            .sort_by(|a, b| b.success_rate.partial_cmp(&a.success_rate).unwrap_or(Ordering::Equal));
        sorted_rules.into_iter().take(top_n).collect()
    }
}

impl Default for RuleEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Detects phase transitions in system behavior
pub struct CriticalityDetector {
    history: Vec<f64>,
    threshold: f64,
    window_size: usize,
}

impl CriticalityDetector {
    pub fn new(threshold: f64) -> Self {
        Self { history: Vec::new(), threshold, window_size: 50 }
    }

    /// Record a measurement
    pub fn record(&mut self, value: f64) {
        self.history.push(value);
        if self.history.len() > 1000 {
            self.history.remove(0);
        }
    }

    /// Detect if system is at critical point (maximum sensitivity)
    pub fn is_critical(&self) -> bool {
        if self.history.len() < self.window_size {
            return false;
        }

        let recent = &self.history[self.history.len() - self.window_size..];
        let mean = recent.iter().sum::<f64>() / recent.len() as f64;
        let variance: f64 =
            recent.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / recent.len() as f64;

        variance > self.threshold
    }

    /// Detect phase transition (organized -> chaotic or vice versa)
    pub fn phase_transition_detected(&self) -> bool {
        if self.history.len() < 2 * self.window_size {
            return false;
        }

        let window1 = &self.history
            [self.history.len() - 2 * self.window_size..self.history.len() - self.window_size];
        let window2 = &self.history[self.history.len() - self.window_size..];

        let var1: f64 = window1.iter().map(|x| x * x).sum::<f64>() / window1.len() as f64;
        let var2: f64 = window2.iter().map(|x| x * x).sum::<f64>() / window2.len() as f64;

        (var1 - var2).abs() > self.threshold
    }
}

/// Self-organizing system
pub struct SelfOrganizer {
    agent_roles: HashMap<String, String>,
    role_stability: HashMap<String, f64>, // How stable is each role assignment
}

impl SelfOrganizer {
    pub fn new() -> Self {
        Self { agent_roles: HashMap::new(), role_stability: HashMap::new() }
    }

    /// Assign role to agent (can change dynamically)
    pub fn assign_role(&mut self, agent_id: String, role: String) {
        self.agent_roles.insert(agent_id.clone(), role);
        self.role_stability.insert(agent_id, 1.0); // High stability on new assignment
    }

    /// Degrade role stability (agents can switch roles)
    pub fn degrade_stability(&mut self) {
        for stability in self.role_stability.values_mut() {
            *stability *= 0.95; // Slowly degrade
        }
    }

    /// Automatically change role if stability too low
    pub fn auto_role_change(&mut self, agent_id: &str, new_role: String) {
        if let Some(stability) = self.role_stability.get(agent_id) {
            if *stability < 0.3 {
                self.assign_role(agent_id.to_string(), new_role);
            }
        }
    }

    /// Get current roles
    pub fn get_roles(&self) -> Vec<(String, String)> {
        self.agent_roles.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }
}

impl Default for SelfOrganizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_engine() {
        let mut engine = RuleEngine::new();
        engine.add_rule(Rule::new(
            "avoid_crowd".to_string(),
            "crowded".to_string(),
            "disperse".to_string(),
        ));

        let actions = engine.evaluate("crowded nearby");
        assert!(!actions.is_empty());
    }

    #[test]
    fn test_criticality_detector() {
        let mut detector = CriticalityDetector::new(0.5);
        for i in 0..60 {
            detector.record((i as f64) % 10.0);
        }

        assert!(!detector.history.is_empty());
    }

    #[test]
    fn test_self_organizer() {
        let mut organizer = SelfOrganizer::new();
        organizer.assign_role("agent-1".to_string(), "scout".to_string());

        let roles = organizer.get_roles();
        assert_eq!(roles.len(), 1);
    }
}
