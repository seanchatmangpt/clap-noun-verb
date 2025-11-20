/// False Positive Detection & Recovery
///
/// Comprehensive systems for detecting, recovering from, and correcting
/// false alarms, bad decisions, misleading signals, and incorrect information
/// in agent swarms.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// False alert severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertSeverity {
    LowRisk,  // Minor false positive
    Medium,   // Moderate impact
    HighRisk, // Significant misalignment
    Critical, // Major swarm dysfunction
}

/// False alert record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FalseAlert {
    pub alert_id: String,
    pub source_agent_id: String,
    pub alert_type: String,
    pub claimed_severity: f64,
    pub verified_severity: f64, // What it actually was
    pub is_false_positive: bool,
    pub falseness_confidence: f64, // How sure we are it's false
    pub affected_agents: Vec<String>,
}

impl FalseAlert {
    pub fn new(source_agent_id: String, alert_type: String, claimed_severity: f64) -> Self {
        Self {
            alert_id: uuid::Uuid::new_v4().to_string(),
            source_agent_id,
            alert_type,
            claimed_severity,
            verified_severity: 0.0,
            is_false_positive: false,
            falseness_confidence: 0.0,
            affected_agents: Vec::new(),
        }
    }
}

/// False Alert Detector - Identifies incorrect signals
pub struct FalseAlertDetector {
    historical_alerts: Vec<FalseAlert>,
    agent_accuracy: HashMap<String, f64>, // Agent ID -> accuracy rate
    alert_thresholds: HashMap<String, (f64, f64)>, // alert_type -> (normal_min, normal_max)
}

impl FalseAlertDetector {
    pub fn new() -> Self {
        Self {
            historical_alerts: Vec::new(),
            agent_accuracy: HashMap::new(),
            alert_thresholds: HashMap::new(),
        }
    }

    /// Register normal thresholds for alert type
    pub fn register_threshold(&mut self, alert_type: String, min: f64, max: f64) {
        self.alert_thresholds.insert(alert_type, (min, max));
    }

    /// Detect if alert is likely false
    pub fn detect_false_alert(&mut self, alert: &mut FalseAlert, verification_value: f64) -> bool {
        // Check if verified value is outside claimed range
        let thresholds = self.alert_thresholds.get(&alert.alert_type).copied();

        if let Some((min, max)) = thresholds {
            if verification_value < min || verification_value > max {
                // Outside normal range - might be false
                let deviation = if verification_value < min {
                    (min - verification_value) / min
                } else {
                    (verification_value - max) / max
                };

                alert.verified_severity = verification_value;
                alert.falseness_confidence = (deviation / 2.0).min(1.0);
                alert.is_false_positive = alert.falseness_confidence > 0.7;

                if alert.is_false_positive {
                    // Penalize agent accuracy
                    let accuracy =
                        self.agent_accuracy.entry(alert.source_agent_id.clone()).or_insert(1.0);
                    *accuracy *= 0.95; // Reduce accuracy
                }

                return alert.is_false_positive;
            }
        }

        false
    }

    /// Get agent credibility (based on historical accuracy)
    pub fn agent_credibility(&self, agent_id: &str) -> f64 {
        self.agent_accuracy.get(agent_id).copied().unwrap_or(0.8)
    }

    /// Record false alert for analysis
    pub fn record_alert(&mut self, alert: FalseAlert) {
        self.historical_alerts.push(alert);
    }

    /// Get false positive rate for agent
    pub fn agent_false_positive_rate(&self, agent_id: &str) -> f64 {
        let agent_alerts: Vec<_> =
            self.historical_alerts.iter().filter(|a| a.source_agent_id == agent_id).collect();

        if agent_alerts.is_empty() {
            return 0.0;
        }

        let false_count = agent_alerts.iter().filter(|a| a.is_false_positive).count();
        false_count as f64 / agent_alerts.len() as f64
    }

    /// Identify sources of false positives (faulty agents)
    pub fn identify_faulty_sources(&self) -> Vec<(String, f64)> {
        let mut faulty_agents = Vec::new();

        let unique_agents: Vec<_> = self
            .historical_alerts
            .iter()
            .map(|a| a.source_agent_id.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        for agent_id in unique_agents {
            let fp_rate = self.agent_false_positive_rate(&agent_id);
            if fp_rate > 0.3 {
                faulty_agents.push((agent_id, fp_rate));
            }
        }

        faulty_agents.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        faulty_agents
    }
}

impl Default for FalseAlertDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Bad Consensus Recovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusVerification {
    pub voting_id: String,
    pub original_decision: String,
    pub verified_outcome: String,
    pub was_wrong: bool,
    pub confidence_in_reversal: f64,
    pub reversal_timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

pub struct ConsensusRecoverySystem {
    verifications: Vec<ConsensusVerification>,
    decision_success_rates: HashMap<String, (usize, usize)>, // decision -> (successes, total)
}

impl ConsensusRecoverySystem {
    pub fn new() -> Self {
        Self { verifications: Vec::new(), decision_success_rates: HashMap::new() }
    }

    /// Verify if consensus decision was actually correct
    pub fn verify_decision(
        &mut self,
        voting_id: String,
        original_decision: String,
        actual_outcome: String,
    ) -> bool {
        let was_wrong = original_decision != actual_outcome;

        if was_wrong {
            let verification = ConsensusVerification {
                voting_id,
                original_decision: original_decision.clone(),
                verified_outcome: actual_outcome,
                was_wrong: true,
                confidence_in_reversal: 0.95,
                reversal_timestamp: Some(chrono::Utc::now()),
            };

            self.verifications.push(verification);

            // Update success rate
            let stats = self.decision_success_rates.entry(original_decision).or_insert((0, 0));
            stats.1 += 1; // Total
                          // stats.0 stays same (no success)

            true
        } else {
            // Update success rate
            let stats = self.decision_success_rates.entry(original_decision).or_insert((0, 0));
            stats.0 += 1; // Success
            stats.1 += 1; // Total

            false
        }
    }

    /// Get success rate for a decision type
    pub fn decision_success_rate(&self, decision: &str) -> f64 {
        if let Some((successes, total)) = self.decision_success_rates.get(decision) {
            if *total == 0 {
                return 0.5; // Unknown
            }
            *successes as f64 / *total as f64
        } else {
            0.5 // Unknown
        }
    }

    /// Identify unreliable decision types
    pub fn unreliable_decisions(&self) -> Vec<(String, f64)> {
        let mut unreliable = Vec::new();

        for (decision, (successes, total)) in &self.decision_success_rates {
            if *total >= 3 {
                // Only if we have enough data
                let success_rate = *successes as f64 / *total as f64;
                if success_rate < 0.6 {
                    unreliable.push((decision.clone(), success_rate));
                }
            }
        }

        unreliable.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        unreliable
    }
}

impl Default for ConsensusRecoverySystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Trust Score Verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustScoreAudit {
    pub agent_id: String,
    pub recorded_score: f64,
    pub verified_score: f64,
    pub deviation: f64,
    pub requires_correction: bool,
    pub audit_timestamp: chrono::DateTime<chrono::Utc>,
}

pub struct TrustScoreVerifier {
    audits: Vec<TrustScoreAudit>,
}

impl TrustScoreVerifier {
    pub fn new() -> Self {
        Self { audits: Vec::new() }
    }

    /// Verify trust score by checking agent's actual performance
    pub fn verify_trust(&mut self, agent_id: String, recorded: f64, actual_performance: f64) {
        let deviation = (recorded - actual_performance).abs();
        let requires_correction = deviation > 0.2; // More than 20% deviation

        let audit = TrustScoreAudit {
            agent_id,
            recorded_score: recorded,
            verified_score: actual_performance,
            deviation,
            requires_correction,
            audit_timestamp: chrono::Utc::now(),
        };

        self.audits.push(audit);
    }

    /// Get agents with incorrect trust scores
    pub fn agents_needing_correction(&self) -> Vec<(String, f64)> {
        let mut corrections = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for audit in self.audits.iter().rev() {
            if audit.requires_correction && !seen.contains(&audit.agent_id) {
                seen.insert(audit.agent_id.clone());
                corrections.push((audit.agent_id.clone(), audit.verified_score));
            }
        }

        corrections
    }

    /// Get audit history for agent
    pub fn audit_history(&self, agent_id: &str) -> Vec<TrustScoreAudit> {
        self.audits.iter().filter(|a| a.agent_id == agent_id).cloned().collect()
    }
}

impl Default for TrustScoreVerifier {
    fn default() -> Self {
        Self::new()
    }
}

/// Bid Validation System
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BidValidation {
    pub bid_id: String,
    pub agent_id: String,
    pub promised_completion_time: u64,
    pub actual_completion_time: Option<u64>,
    pub bid_fulfilled: bool,
    pub default_occurred: bool,
    pub penalty_applied: bool,
}

pub struct BidValidator {
    bid_history: Vec<BidValidation>,
    agent_reliability: HashMap<String, f64>, // success rate
}

impl BidValidator {
    pub fn new() -> Self {
        Self { bid_history: Vec::new(), agent_reliability: HashMap::new() }
    }

    /// Record bid outcome
    pub fn record_outcome(
        &mut self,
        bid_id: String,
        agent_id: String,
        promised_time: u64,
        actual_time: Option<u64>,
        fulfilled: bool,
    ) {
        let default = !fulfilled;
        let validation = BidValidation {
            bid_id,
            agent_id: agent_id.clone(),
            promised_completion_time: promised_time,
            actual_completion_time: actual_time,
            bid_fulfilled: fulfilled,
            default_occurred: default,
            penalty_applied: false,
        };

        self.bid_history.push(validation);

        // Update reliability
        let reliability = self.agent_reliability.entry(agent_id).or_insert(1.0);

        if fulfilled {
            *reliability = (*reliability * 0.9) + 0.1; // Increase
        } else {
            *reliability = (*reliability * 0.8); // Decrease more
        }
    }

    /// Get agent bid fulfillment rate
    pub fn fulfillment_rate(&self, agent_id: &str) -> f64 {
        let agent_bids: Vec<_> =
            self.bid_history.iter().filter(|b| b.agent_id == agent_id).collect();

        if agent_bids.is_empty() {
            return 1.0; // Unknown - assume good
        }

        let fulfilled = agent_bids.iter().filter(|b| b.bid_fulfilled).count();
        fulfilled as f64 / agent_bids.len() as f64
    }

    /// Identify unreliable bidders
    pub fn unreliable_bidders(&self) -> Vec<(String, f64)> {
        let mut unreliable = Vec::new();

        let unique_agents: Vec<_> = self
            .bid_history
            .iter()
            .map(|b| b.agent_id.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        for agent_id in unique_agents {
            let rate = self.fulfillment_rate(&agent_id);
            if rate < 0.8 {
                unreliable.push((agent_id, rate));
            }
        }

        unreliable.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        unreliable
    }
}

impl Default for BidValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Pheromone Trail Validator
pub struct PheromoneValidator {
    trail_confidence: HashMap<String, f64>, // trail_id -> confidence
    failed_trails: Vec<String>,
}

impl PheromoneValidator {
    pub fn new() -> Self {
        Self { trail_confidence: HashMap::new(), failed_trails: Vec::new() }
    }

    /// Validate pheromone trail led to promised resource
    pub fn validate_trail(&mut self, trail_id: String, success: bool) {
        if !success {
            self.failed_trails.push(trail_id.clone());
        }

        // Update confidence
        let confidence = self.trail_confidence.entry(trail_id).or_insert(0.5);

        if success {
            *confidence = (*confidence * 0.8) + 0.2; // Boost
        } else {
            *confidence = (*confidence * 0.7); // Reduce
        }
    }

    /// Get unreliable pheromone trails
    pub fn unreliable_trails(&self) -> Vec<(String, f64)> {
        let mut unreliable = Vec::new();

        for (trail_id, confidence) in &self.trail_confidence {
            if *confidence < 0.5 {
                unreliable.push((trail_id.clone(), *confidence));
            }
        }

        unreliable.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        unreliable
    }

    /// Clear misleading trails
    pub fn clear_misleading_trails(&mut self, threshold: f64) -> usize {
        let before = self.trail_confidence.len();
        self.trail_confidence.retain(|_, conf| *conf >= threshold);
        before - self.trail_confidence.len()
    }
}

impl Default for PheromoneValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Role Verification System
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleVerification {
    pub agent_id: String,
    pub assigned_role: String,
    pub can_perform: bool,
    pub performance_score: f64,
    pub assignments: usize,
    pub successes: usize,
}

pub struct RoleVerifier {
    verifications: Vec<RoleVerification>,
}

impl RoleVerifier {
    pub fn new() -> Self {
        Self { verifications: Vec::new() }
    }

    /// Record role performance
    pub fn record_performance(&mut self, agent_id: String, role: String, success: bool) {
        let mut verification = self
            .verifications
            .iter_mut()
            .find(|v| v.agent_id == agent_id && v.assigned_role == role);

        if let Some(v) = verification {
            v.assignments += 1;
            if success {
                v.successes += 1;
            }
            v.performance_score = v.successes as f64 / v.assignments as f64;
            v.can_perform = v.performance_score > 0.6;
        } else {
            let mut v = RoleVerification {
                agent_id,
                assigned_role: role,
                can_perform: success,
                performance_score: if success { 1.0 } else { 0.0 },
                assignments: 1,
                successes: if success { 1 } else { 0 },
            };
            self.verifications.push(v);
        }
    }

    /// Get agents unsuited for their roles
    pub fn unsuitable_role_assignments(&self) -> Vec<(String, String, f64)> {
        self.verifications
            .iter()
            .filter(|v| !v.can_perform && v.assignments >= 3)
            .map(|v| (v.agent_id.clone(), v.assigned_role.clone(), v.performance_score))
            .collect()
    }

    /// Get best agents for a role
    pub fn best_agents_for_role(&self, role: &str) -> Vec<(String, f64)> {
        let mut agents: Vec<_> = self
            .verifications
            .iter()
            .filter(|v| v.assigned_role == role && v.assignments >= 2)
            .map(|v| (v.agent_id.clone(), v.performance_score))
            .collect();

        agents.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        agents
    }
}

impl Default for RoleVerifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_false_alert_detector() {
        let mut detector = FalseAlertDetector::new();
        detector.register_threshold("danger".to_string(), 0.0, 0.5);

        // Test: Alert claims danger at 0.9, but verified is 2.0 (way outside range)
        // This should be detected as false positive with high confidence
        let mut alert = FalseAlert::new("agent-1".to_string(), "danger".to_string(), 0.9);
        let is_false = detector.detect_false_alert(&mut alert, 2.0);

        assert!(is_false);
        assert!(alert.is_false_positive);
        assert!(alert.falseness_confidence > 0.7);
    }

    #[test]
    fn test_consensus_recovery() {
        let mut recovery = ConsensusRecoverySystem::new();
        recovery.verify_decision("vote-1".to_string(), "yes".to_string(), "no".to_string());

        let rate = recovery.decision_success_rate("yes");
        assert!(rate < 1.0);
    }

    #[test]
    fn test_bid_validator() {
        let mut validator = BidValidator::new();
        validator.record_outcome("bid-1".to_string(), "agent-1".to_string(), 100, Some(120), false);

        let rate = validator.fulfillment_rate("agent-1");
        assert!(rate < 1.0);
    }
}
