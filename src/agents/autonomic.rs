//! MAPE-K autonomic loop for self-healing systems
//!
//! This module implements the Monitor-Analyze-Plan-Execute-Knowledge (MAPE-K) loop
//! for autonomic computing. The system continuously monitors metrics, detects anomalies,
//! plans corrections, and executes self-healing actions.
//!
//! # MAPE-K Phases
//!
//! 1. **Monitor** - Collect system metrics
//! 2. **Analyze** - Detect anomalies and patterns
//! 3. **Plan** - Generate corrective actions
//! 4. **Execute** - Apply planned actions
//! 5. **Knowledge** - Update learned patterns
//!
//! # Example
//!
//! ```rust,ignore
//! use clap_noun_verb::agents::autonomic::*;
//!
//! let mut loop_engine = AutonomicLoop::new();
//!
//! // Monitor phase
//! loop_engine.record_metric("response_time_ms", 150.0);
//!
//! // Analyze phase
//! let anomalies = loop_engine.detect_anomalies();
//!
//! // Plan phase
//! let actions = loop_engine.plan_actions(&anomalies)?;
//!
//! // Execute phase
//! loop_engine.execute_actions(&actions)?;
//! ```

use std::collections::HashMap;
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::error::{NounVerbError, Result};

// =============================================================================
// Metric - System metric with timestamp
// =============================================================================

/// System metric for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    /// Metric name
    pub name: String,

    /// Metric value
    pub value: f64,

    /// Timestamp
    pub timestamp: SystemTime,
}

impl Metric {
    /// Create new metric
    pub fn new(name: impl Into<String>, value: f64) -> Self {
        Self { name: name.into(), value, timestamp: SystemTime::now() }
    }
}

// =============================================================================
// Anomaly - Detected anomaly with severity
// =============================================================================

/// Detected anomaly in system metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    /// Metric name
    pub metric_name: String,

    /// Current value
    pub current_value: f64,

    /// Expected value
    pub expected_value: f64,

    /// Deviation magnitude
    pub deviation: f64,

    /// Severity (0.0 - 1.0)
    pub severity: f64,

    /// Detection timestamp
    pub timestamp: SystemTime,
}

impl Anomaly {
    /// Create new anomaly
    pub fn new(
        metric_name: impl Into<String>,
        current_value: f64,
        expected_value: f64,
        severity: f64,
    ) -> Self {
        let deviation = (current_value - expected_value).abs();

        Self {
            metric_name: metric_name.into(),
            current_value,
            expected_value,
            deviation,
            severity,
            timestamp: SystemTime::now(),
        }
    }
}

// =============================================================================
// Self-Healing Action - Corrective action
// =============================================================================

/// Self-healing action to correct anomaly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfHealingAction {
    /// Action type
    pub action_type: ActionType,

    /// Target parameter
    pub target: String,

    /// New value
    pub new_value: f64,

    /// Reason for action
    pub reason: String,
}

/// Action type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActionType {
    /// Adjust parameter value
    AdjustParameter,

    /// Scale resources
    ScaleResources,

    /// Restart component
    RestartComponent,

    /// Throttle requests
    ThrottleRequests,

    /// Alert operator
    AlertOperator,
}

impl SelfHealingAction {
    /// Create new action
    pub fn new(
        action_type: ActionType,
        target: impl Into<String>,
        new_value: f64,
        reason: impl Into<String>,
    ) -> Self {
        Self { action_type, target: target.into(), new_value, reason: reason.into() }
    }
}

// =============================================================================
// Adaptive Parameter - Self-tuning parameter
// =============================================================================

/// Adaptive parameter that self-tunes based on feedback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveParameter {
    /// Parameter name
    pub name: String,

    /// Current value
    pub value: f64,

    /// Minimum allowed value
    pub min: f64,

    /// Maximum allowed value
    pub max: f64,

    /// Learning rate for adjustments
    pub learning_rate: f64,

    /// Historical values
    history: Vec<f64>,
}

impl AdaptiveParameter {
    /// Create new adaptive parameter
    pub fn new(name: impl Into<String>, initial: f64, min: f64, max: f64) -> Self {
        Self {
            name: name.into(),
            value: initial,
            min,
            max,
            learning_rate: 0.1, // Default 10% adjustment
            history: vec![initial],
        }
    }

    /// Adjust parameter based on gradient
    ///
    /// # Arguments
    ///
    /// * `gradient` - Gradient direction (-1.0 to 1.0)
    pub fn adjust(&mut self, gradient: f64) {
        let delta = self.learning_rate * gradient;
        let new_value = (self.value + delta).clamp(self.min, self.max);

        self.value = new_value;
        self.history.push(new_value);

        // Keep history bounded
        if self.history.len() > 100 {
            self.history.remove(0);
        }
    }

    /// Get parameter stability (lower = more stable)
    pub fn stability(&self) -> f64 {
        if self.history.len() < 2 {
            return 0.0;
        }

        // Calculate variance
        let mean = self.history.iter().sum::<f64>() / self.history.len() as f64;
        let variance = self.history.iter().map(|v| (v - mean).powi(2)).sum::<f64>()
            / self.history.len() as f64;

        variance.sqrt()
    }
}

// =============================================================================
// Anomaly Detector - Statistical anomaly detection
// =============================================================================

/// Anomaly detector using statistical methods
#[derive(Debug, Clone)]
pub struct AnomalyDetector {
    /// Metric baselines (mean values)
    baselines: HashMap<String, f64>,

    /// Metric standard deviations
    std_devs: HashMap<String, f64>,

    /// Detection threshold (sigma multiplier)
    threshold: f64,

    /// Metric history for learning
    history: HashMap<String, Vec<f64>>,
}

impl AnomalyDetector {
    /// Create new anomaly detector
    pub fn new() -> Self {
        Self {
            baselines: HashMap::new(),
            std_devs: HashMap::new(),
            threshold: 3.0, // 3-sigma threshold
            history: HashMap::new(),
        }
    }

    /// Record metric value
    ///
    /// # Arguments
    ///
    /// * `metric` - Metric to record
    pub fn record_metric(&mut self, metric: &Metric) {
        let history = self.history.entry(metric.name.clone()).or_insert_with(Vec::new);

        history.push(metric.value);

        // Keep history bounded
        if history.len() > 100 {
            history.remove(0);
        }

        // Update baseline and std dev
        self.update_statistics(&metric.name);
    }

    /// Update statistical baseline for metric
    fn update_statistics(&mut self, metric_name: &str) {
        if let Some(history) = self.history.get(metric_name) {
            if history.is_empty() {
                return;
            }

            // Calculate mean
            let mean = history.iter().sum::<f64>() / history.len() as f64;
            self.baselines.insert(metric_name.to_string(), mean);

            // Calculate standard deviation
            if history.len() > 1 {
                let variance = history.iter().map(|v| (v - mean).powi(2)).sum::<f64>()
                    / (history.len() - 1) as f64;

                let std_dev = variance.sqrt();
                self.std_devs.insert(metric_name.to_string(), std_dev);
            }
        }
    }

    /// Detect anomalies in metric
    ///
    /// # Arguments
    ///
    /// * `metric` - Metric to check
    ///
    /// # Returns
    ///
    /// Option containing anomaly if detected
    pub fn detect_anomaly(&self, metric: &Metric) -> Option<Anomaly> {
        if let (Some(&baseline), Some(&std_dev)) =
            (self.baselines.get(&metric.name), self.std_devs.get(&metric.name))
        {
            if std_dev > 0.0 {
                let z_score = (metric.value - baseline).abs() / std_dev;

                if z_score > self.threshold {
                    let severity = (z_score / (self.threshold * 2.0)).min(1.0);

                    return Some(Anomaly::new(&metric.name, metric.value, baseline, severity));
                }
            }
        }

        None
    }
}

impl Default for AnomalyDetector {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// MAPE-K Phase - Current phase of autonomic loop
// =============================================================================

/// MAPE-K loop phase
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MapekPhase {
    /// Monitor - Collect metrics
    Monitor,

    /// Analyze - Detect anomalies
    Analyze,

    /// Plan - Generate actions
    Plan,

    /// Execute - Apply actions
    Execute,

    /// Knowledge - Update learned patterns
    Knowledge,
}

// =============================================================================
// Autonomic Loop - Main MAPE-K engine
// =============================================================================

/// Autonomic loop implementing MAPE-K
#[derive(Debug, Clone)]
pub struct AutonomicLoop {
    /// Current phase
    phase: MapekPhase,

    /// Anomaly detector
    detector: AnomalyDetector,

    /// Adaptive parameters
    parameters: HashMap<String, AdaptiveParameter>,

    /// Recent metrics
    recent_metrics: Vec<Metric>,

    /// Detected anomalies
    anomalies: Vec<Anomaly>,

    /// Planned actions
    planned_actions: Vec<SelfHealingAction>,
}

impl AutonomicLoop {
    /// Create new autonomic loop
    pub fn new() -> Self {
        Self {
            phase: MapekPhase::Monitor,
            detector: AnomalyDetector::new(),
            parameters: HashMap::new(),
            recent_metrics: Vec::new(),
            anomalies: Vec::new(),
            planned_actions: Vec::new(),
        }
    }

    /// Get current phase
    pub fn current_phase(&self) -> MapekPhase {
        self.phase
    }

    /// Record metric (Monitor phase)
    ///
    /// # Arguments
    ///
    /// * `name` - Metric name
    /// * `value` - Metric value
    pub fn record_metric(&mut self, name: impl Into<String>, value: f64) {
        let metric = Metric::new(name, value);
        self.detector.record_metric(&metric);
        self.recent_metrics.push(metric);

        // Keep recent metrics bounded
        if self.recent_metrics.len() > 50 {
            self.recent_metrics.remove(0);
        }
    }

    /// Detect anomalies (Analyze phase)
    ///
    /// # Returns
    ///
    /// List of detected anomalies
    pub fn detect_anomalies(&mut self) -> Vec<Anomaly> {
        self.phase = MapekPhase::Analyze;
        self.anomalies.clear();

        for metric in &self.recent_metrics {
            if let Some(anomaly) = self.detector.detect_anomaly(metric) {
                self.anomalies.push(anomaly);
            }
        }

        self.anomalies.clone()
    }

    /// Plan corrective actions (Plan phase)
    ///
    /// # Arguments
    ///
    /// * `anomalies` - Detected anomalies
    ///
    /// # Returns
    ///
    /// List of planned actions
    pub fn plan_actions(&mut self, anomalies: &[Anomaly]) -> Result<Vec<SelfHealingAction>> {
        self.phase = MapekPhase::Plan;
        self.planned_actions.clear();

        for anomaly in anomalies {
            // Generate action based on anomaly type
            let action = if anomaly.severity > 0.8 {
                // Critical severity - restart component
                SelfHealingAction::new(
                    ActionType::RestartComponent,
                    &anomaly.metric_name,
                    0.0,
                    format!("Critical anomaly: {:.2} deviation", anomaly.deviation),
                )
            } else if anomaly.severity > 0.5 {
                // High severity - scale resources
                SelfHealingAction::new(
                    ActionType::ScaleResources,
                    &anomaly.metric_name,
                    anomaly.expected_value,
                    format!("High anomaly: {:.2} deviation", anomaly.deviation),
                )
            } else {
                // Medium severity - adjust parameter
                SelfHealingAction::new(
                    ActionType::AdjustParameter,
                    &anomaly.metric_name,
                    anomaly.expected_value,
                    format!("Parameter drift: {:.2} deviation", anomaly.deviation),
                )
            };

            self.planned_actions.push(action);
        }

        Ok(self.planned_actions.clone())
    }

    /// Execute planned actions (Execute phase)
    ///
    /// # Arguments
    ///
    /// * `actions` - Actions to execute
    pub fn execute_actions(&mut self, actions: &[SelfHealingAction]) -> Result<()> {
        self.phase = MapekPhase::Execute;

        for action in actions {
            match action.action_type {
                ActionType::AdjustParameter => {
                    if let Some(param) = self.parameters.get_mut(&action.target) {
                        let gradient = action.new_value - param.value;
                        param.adjust(gradient);
                    }
                }
                ActionType::ScaleResources => {
                    // In production, this would trigger actual resource scaling
                    // For now, just log the action
                }
                ActionType::RestartComponent => {
                    // In production, this would trigger component restart
                    // For now, just log the action
                }
                ActionType::ThrottleRequests => {
                    // In production, this would adjust rate limiting
                    // For now, just log the action
                }
                ActionType::AlertOperator => {
                    // In production, this would send alerts
                    // For now, just log the action
                }
            }
        }

        Ok(())
    }

    /// Update knowledge base (Knowledge phase)
    pub fn update_knowledge(&mut self) {
        self.phase = MapekPhase::Knowledge;

        // Update parameter stability metrics
        for param in self.parameters.values_mut() {
            let _ = param.stability();
        }

        // Cycle back to Monitor
        self.phase = MapekPhase::Monitor;
    }

    /// Add adaptive parameter
    ///
    /// # Arguments
    ///
    /// * `name` - Parameter name
    /// * `initial` - Initial value
    /// * `min` - Minimum value
    /// * `max` - Maximum value
    pub fn add_parameter(&mut self, name: impl Into<String>, initial: f64, min: f64, max: f64) {
        let name = name.into();
        self.parameters.insert(name.clone(), AdaptiveParameter::new(name, initial, min, max));
    }

    /// Get parameter value
    ///
    /// # Arguments
    ///
    /// * `name` - Parameter name
    pub fn get_parameter(&self, name: &str) -> Option<f64> {
        self.parameters.get(name).map(|p| p.value)
    }

    /// Run full MAPE-K cycle
    ///
    /// # Returns
    ///
    /// Number of actions executed
    pub fn run_cycle(&mut self) -> Result<usize> {
        // Analyze
        let anomalies = self.detect_anomalies();

        // Plan
        let actions = self.plan_actions(&anomalies)?;
        let action_count = actions.len();

        // Execute
        self.execute_actions(&actions)?;

        // Knowledge
        self.update_knowledge();

        Ok(action_count)
    }
}

impl Default for AutonomicLoop {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_creation() {
        // Arrange & Act
        let metric = Metric::new("response_time_ms", 150.0);

        // Assert
        assert_eq!(metric.name, "response_time_ms");
        assert_eq!(metric.value, 150.0);
    }

    #[test]
    fn test_anomaly_creation() {
        // Arrange & Act
        let anomaly = Anomaly::new("response_time_ms", 500.0, 100.0, 0.8);

        // Assert
        assert_eq!(anomaly.metric_name, "response_time_ms");
        assert_eq!(anomaly.current_value, 500.0);
        assert_eq!(anomaly.expected_value, 100.0);
        assert_eq!(anomaly.deviation, 400.0);
        assert_eq!(anomaly.severity, 0.8);
    }

    #[test]
    fn test_self_healing_action() {
        // Arrange & Act
        let action = SelfHealingAction::new(
            ActionType::AdjustParameter,
            "max_connections",
            100.0,
            "Load spike detected",
        );

        // Assert
        assert_eq!(action.action_type, ActionType::AdjustParameter);
        assert_eq!(action.target, "max_connections");
        assert_eq!(action.new_value, 100.0);
        assert_eq!(action.reason, "Load spike detected");
    }

    #[test]
    fn test_adaptive_parameter_adjustment() {
        // Arrange
        let mut param = AdaptiveParameter::new("max_connections", 50.0, 10.0, 100.0);

        // Act: Increase parameter
        param.adjust(10.0);

        // Assert
        assert!(param.value > 50.0);
        assert!(param.value <= 100.0);
    }

    #[test]
    fn test_adaptive_parameter_bounds() {
        // Arrange
        let mut param = AdaptiveParameter::new("max_connections", 95.0, 10.0, 100.0);

        // Act: Try to exceed max
        param.adjust(100.0);

        // Assert: Should be clamped to max
        assert_eq!(param.value, 100.0);
    }

    #[test]
    fn test_anomaly_detection() {
        // Arrange
        let mut detector = AnomalyDetector::new();

        // Establish baseline (mean=100, small variance)
        for i in 0..20 {
            detector.record_metric(&Metric::new("response_time_ms", 100.0 + (i % 3) as f64));
        }

        // Act: Record anomalous value
        let anomaly_metric = Metric::new("response_time_ms", 500.0);
        let anomaly = detector.detect_anomaly(&anomaly_metric);

        // Assert: Anomaly should be detected
        assert!(anomaly.is_some());
        let anomaly = anomaly.unwrap();
        assert!(anomaly.severity > 0.0);
    }

    #[test]
    fn test_autonomic_loop_phases() {
        // Arrange
        let mut loop_engine = AutonomicLoop::new();

        // Assert: Starts in Monitor phase
        assert_eq!(loop_engine.current_phase(), MapekPhase::Monitor);

        // Act: Detect anomalies
        loop_engine.detect_anomalies();

        // Assert: Moved to Analyze phase
        assert_eq!(loop_engine.current_phase(), MapekPhase::Analyze);
    }

    #[test]
    fn test_full_mapek_cycle() {
        // Arrange
        let mut loop_engine = AutonomicLoop::new();

        // Establish baseline
        for i in 0..20 {
            loop_engine.record_metric("response_time_ms", 100.0 + (i % 5) as f64);
        }

        // Record anomalous metric
        loop_engine.record_metric("response_time_ms", 500.0);

        // Act: Run full cycle
        let result = loop_engine.run_cycle();

        // Assert: Cycle completed successfully
        assert!(result.is_ok());
        let action_count = result.unwrap();
        assert!(action_count > 0); // Should have planned actions
    }

    #[test]
    fn test_parameter_management() {
        // Arrange
        let mut loop_engine = AutonomicLoop::new();
        loop_engine.add_parameter("max_connections", 50.0, 10.0, 100.0);

        // Act
        let value = loop_engine.get_parameter("max_connections");

        // Assert
        assert!(value.is_some());
        assert_eq!(value.unwrap(), 50.0);
    }

    #[test]
    fn test_action_planning_severity_based() {
        // Arrange
        let mut loop_engine = AutonomicLoop::new();
        let anomalies = vec![
            Anomaly::new("metric1", 500.0, 100.0, 0.9), // Critical
            Anomaly::new("metric2", 300.0, 100.0, 0.6), // High
            Anomaly::new("metric3", 150.0, 100.0, 0.3), // Medium
        ];

        // Act
        let actions = loop_engine.plan_actions(&anomalies);

        // Assert
        assert!(actions.is_ok());
        let actions = actions.unwrap();
        assert_eq!(actions.len(), 3);

        // Critical should trigger restart
        assert_eq!(actions[0].action_type, ActionType::RestartComponent);

        // High should trigger scaling
        assert_eq!(actions[1].action_type, ActionType::ScaleResources);

        // Medium should trigger parameter adjustment
        assert_eq!(actions[2].action_type, ActionType::AdjustParameter);
    }
}
