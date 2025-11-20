use chrono::{DateTime, Duration, Utc};
/// Self-Healing Autonomic Systems
///
/// MAPE-K loop enhancements for autonomous system recovery from failures
/// without human intervention. Features health monitoring, anomaly detection,
/// and automatic remediation.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Health status of a system component
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Critical,
    Failed,
}

/// System metric for health assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetric {
    pub name: String,
    pub value: f64,
    pub threshold_warning: f64,
    pub threshold_critical: f64,
    pub timestamp: DateTime<Utc>,
}

impl SystemMetric {
    pub fn new(name: String, value: f64) -> Self {
        Self {
            name,
            value,
            threshold_warning: 70.0,
            threshold_critical: 90.0,
            timestamp: Utc::now(),
        }
    }

    /// Determine health status based on metric value
    pub fn status(&self) -> HealthStatus {
        if self.value >= self.threshold_critical {
            HealthStatus::Critical
        } else if self.value >= self.threshold_warning {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        }
    }
}

/// Component health assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub component_id: String,
    pub status: HealthStatus,
    pub metrics: Vec<SystemMetric>,
    pub last_checked: DateTime<Utc>,
    pub consecutive_failures: u32,
}

impl ComponentHealth {
    pub fn new(component_id: String) -> Self {
        Self {
            component_id,
            status: HealthStatus::Healthy,
            metrics: Vec::new(),
            last_checked: Utc::now(),
            consecutive_failures: 0,
        }
    }

    /// Update health based on metrics
    pub fn update(&mut self) {
        if self.metrics.is_empty() {
            self.status = HealthStatus::Healthy;
            return;
        }

        let worst_status = self
            .metrics
            .iter()
            .map(|m| m.status())
            .max_by_key(|s| match s {
                HealthStatus::Healthy => 0,
                HealthStatus::Degraded => 1,
                HealthStatus::Critical => 2,
                HealthStatus::Failed => 3,
            })
            .unwrap_or(HealthStatus::Healthy);

        self.status = worst_status;
        self.last_checked = Utc::now();
    }

    /// Record a failure
    pub fn record_failure(&mut self) {
        self.consecutive_failures += 1;
        if self.consecutive_failures > 3 {
            self.status = HealthStatus::Failed;
        }
    }

    /// Reset failure count after recovery
    pub fn reset(&mut self) {
        self.consecutive_failures = 0;
        self.status = HealthStatus::Healthy;
    }
}

/// Anomaly detected in system behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyEvent {
    pub anomaly_id: String,
    pub component_id: String,
    pub anomaly_type: String,
    pub severity: f64, // 0.0 to 1.0
    pub detected_at: DateTime<Utc>,
    pub description: String,
}

impl AnomalyEvent {
    pub fn new(component_id: String, anomaly_type: String, severity: f64) -> Self {
        Self {
            anomaly_id: uuid::Uuid::new_v4().to_string(),
            component_id,
            anomaly_type,
            severity,
            detected_at: Utc::now(),
            description: String::new(),
        }
    }
}

/// Root cause analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootCauseAnalysis {
    pub analysis_id: String,
    pub anomaly_id: String,
    pub primary_cause: String,
    pub contributing_factors: Vec<String>,
    pub confidence: f64, // 0.0 to 1.0
    pub analysis_timestamp: DateTime<Utc>,
}

/// Healing action to remediate a failure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealingAction {
    pub action_id: String,
    pub component_id: String,
    pub action_type: String, // "restart", "scale", "isolate", "rollback", etc.
    pub parameters: HashMap<String, String>,
    pub priority: u8, // 1-10, higher = more urgent
    pub status: HealingStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealingStatus {
    Planned,
    Executing,
    Completed,
    Failed,
    Cancelled,
}

impl HealingAction {
    pub fn new(component_id: String, action_type: String) -> Self {
        Self {
            action_id: uuid::Uuid::new_v4().to_string(),
            component_id,
            action_type,
            parameters: HashMap::new(),
            priority: 5,
            status: HealingStatus::Planned,
        }
    }

    /// Add a parameter to the healing action
    pub fn with_param(mut self, key: String, value: String) -> Self {
        self.parameters.insert(key, value);
        self
    }
}

/// Health Monitor
pub struct HealthMonitor {
    components: Arc<RwLock<HashMap<String, ComponentHealth>>>,
    metrics_history: Arc<RwLock<Vec<SystemMetric>>>,
}

impl HealthMonitor {
    pub fn new() -> Self {
        Self {
            components: Arc::new(RwLock::new(HashMap::new())),
            metrics_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Register a component for monitoring
    pub async fn register(&self, component_id: String) {
        let mut components = self.components.write().await;
        components.insert(component_id.clone(), ComponentHealth::new(component_id));
    }

    /// Update metrics for a component
    pub async fn update_metric(&self, component_id: &str, metric: SystemMetric) {
        let mut components = self.components.write().await;
        if let Some(component) = components.get_mut(component_id) {
            component.metrics.push(metric.clone());
            component.update();
        }

        let mut history = self.metrics_history.write().await;
        history.push(metric);
    }

    /// Get health status of a component
    pub async fn status(&self, component_id: &str) -> Option<HealthStatus> {
        let components = self.components.read().await;
        components.get(component_id).map(|c| c.status)
    }

    /// Get all unhealthy components
    pub async fn unhealthy_components(&self) -> Vec<ComponentHealth> {
        let components = self.components.read().await;
        components.values().filter(|c| c.status != HealthStatus::Healthy).cloned().collect()
    }

    /// Get metrics from last N hours
    pub async fn recent_metrics(&self, hours: i64) -> Vec<SystemMetric> {
        let history = self.metrics_history.read().await;
        let cutoff = Utc::now() - Duration::hours(hours);
        history.iter().filter(|m| m.timestamp > cutoff).cloned().collect()
    }
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Anomaly Detector using statistical methods
pub struct AnomalyDetector {
    anomalies: Arc<RwLock<Vec<AnomalyEvent>>>,
    baseline: Arc<RwLock<HashMap<String, f64>>>,
}

impl AnomalyDetector {
    pub fn new() -> Self {
        Self {
            anomalies: Arc::new(RwLock::new(Vec::new())),
            baseline: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Train anomaly detector with baseline metrics
    pub async fn train(&self, component_id: String, avg_value: f64) {
        let mut baseline = self.baseline.write().await;
        baseline.insert(component_id, avg_value);
    }

    /// Detect anomalies in metric value
    pub async fn detect(&self, component_id: &str, value: f64) -> Option<AnomalyEvent> {
        let baseline = self.baseline.read().await;
        let baseline_value = baseline.get(component_id).copied().unwrap_or(50.0);

        let deviation = ((value - baseline_value).abs() / baseline_value).min(2.0);
        let threshold = 0.5; // 50% deviation is anomalous

        if deviation > threshold {
            let severity = (deviation - threshold) / (2.0 - threshold);
            let anomaly = AnomalyEvent::new(
                component_id.to_string(),
                "Statistical Deviation".to_string(),
                severity,
            );

            let mut anomalies = self.anomalies.write().await;
            anomalies.push(anomaly.clone());

            return Some(anomaly);
        }

        None
    }

    /// Get recent anomalies
    pub async fn recent_anomalies(&self, hours: i64) -> Vec<AnomalyEvent> {
        let anomalies = self.anomalies.read().await;
        let cutoff = Utc::now() - Duration::hours(hours);
        anomalies.iter().filter(|a| a.detected_at > cutoff).cloned().collect()
    }
}

impl Default for AnomalyDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Root Cause Analyzer
pub struct RootCauseAnalyzer {
    analyses: Arc<RwLock<Vec<RootCauseAnalysis>>>,
}

impl RootCauseAnalyzer {
    pub fn new() -> Self {
        Self { analyses: Arc::new(RwLock::new(Vec::new())) }
    }

    /// Analyze root cause of an anomaly
    pub async fn analyze(&self, anomaly: &AnomalyEvent) -> RootCauseAnalysis {
        // Simplified RCA: map anomaly types to probable causes
        let (primary_cause, factors) = match anomaly.anomaly_type.as_str() {
            "Statistical Deviation" => (
                "Resource contention or load spike".to_string(),
                vec![
                    "Increased request volume".to_string(),
                    "Database lock contention".to_string(),
                ],
            ),
            "Timeout" => (
                "Component overload or network latency".to_string(),
                vec!["High CPU usage".to_string(), "Network congestion".to_string()],
            ),
            _ => ("Unknown cause".to_string(), vec!["Requires manual investigation".to_string()]),
        };

        let analysis = RootCauseAnalysis {
            analysis_id: uuid::Uuid::new_v4().to_string(),
            anomaly_id: anomaly.anomaly_id.clone(),
            primary_cause,
            contributing_factors: factors,
            confidence: anomaly.severity,
            analysis_timestamp: Utc::now(),
        };

        let mut analyses = self.analyses.write().await;
        analyses.push(analysis.clone());

        analysis
    }

    /// Get analysis for an anomaly
    pub async fn get_analysis(&self, anomaly_id: &str) -> Option<RootCauseAnalysis> {
        let analyses = self.analyses.read().await;
        analyses.iter().find(|a| a.anomaly_id == anomaly_id).cloned()
    }
}

impl Default for RootCauseAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Auto-Recovery Engine
pub struct AutoRecovery {
    actions: Arc<RwLock<Vec<HealingAction>>>,
    completed_actions: Arc<RwLock<Vec<String>>>,
}

impl AutoRecovery {
    pub fn new() -> Self {
        Self {
            actions: Arc::new(RwLock::new(Vec::new())),
            completed_actions: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Plan recovery action for a component
    pub async fn plan_recovery(&self, component_id: &str, cause: &str) -> HealingAction {
        // Map cause to appropriate recovery action
        let action_type = match cause {
            s if s.contains("load") || s.contains("overload") => "scale",
            s if s.contains("timeout") || s.contains("unresponsive") => "restart",
            s if s.contains("critical") => "isolate",
            _ => "monitor",
        };

        let action = HealingAction::new(component_id.to_string(), action_type.to_string())
            .with_param("max_retries".to_string(), "3".to_string())
            .with_param("backoff_ms".to_string(), "1000".to_string());

        let mut actions = self.actions.write().await;
        actions.push(action.clone());

        action
    }

    /// Execute recovery action
    pub async fn execute(&self, action_id: &str) -> bool {
        let mut actions = self.actions.write().await;
        if let Some(action) = actions.iter_mut().find(|a| a.action_id == action_id) {
            action.status = HealingStatus::Executing;

            // Simulate execution
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            action.status = HealingStatus::Completed;

            let mut completed = self.completed_actions.write().await;
            completed.push(action_id.to_string());

            true
        } else {
            false
        }
    }

    /// Get pending recovery actions
    pub async fn pending_actions(&self) -> Vec<HealingAction> {
        let actions = self.actions.read().await;
        actions.iter().filter(|a| a.status == HealingStatus::Planned).cloned().collect()
    }
}

impl Default for AutoRecovery {
    fn default() -> Self {
        Self::new()
    }
}

/// Full Autonomic system (MAPE-K loop)
pub struct Autonomic {
    pub monitor: HealthMonitor,
    pub anomaly_detector: AnomalyDetector,
    pub root_cause_analyzer: RootCauseAnalyzer,
    pub auto_recovery: AutoRecovery,
}

impl Autonomic {
    pub fn new() -> Self {
        Self {
            monitor: HealthMonitor::new(),
            anomaly_detector: AnomalyDetector::new(),
            root_cause_analyzer: RootCauseAnalyzer::new(),
            auto_recovery: AutoRecovery::new(),
        }
    }

    /// Execute MAPE-K loop
    pub async fn run_cycle(&self) {
        // Monitor: Get system metrics
        let unhealthy = self.monitor.unhealthy_components().await;

        for component in unhealthy {
            if component.status != HealthStatus::Healthy {
                // Analyze: Detect anomalies
                if let Some(anomaly) =
                    self.anomaly_detector.detect(&component.component_id, 75.0).await
                {
                    // Plan: Analyze root cause
                    let analysis = self.root_cause_analyzer.analyze(&anomaly).await;

                    // Execute: Create and execute recovery action
                    let action = self
                        .auto_recovery
                        .plan_recovery(&component.component_id, &analysis.primary_cause)
                        .await;

                    self.auto_recovery.execute(&action.action_id).await;
                }
            }
        }
    }
}

impl Default for Autonomic {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status() {
        let mut metric = SystemMetric::new("cpu_usage".to_string(), 45.0);
        assert_eq!(metric.status(), HealthStatus::Healthy);

        metric.value = 80.0;
        assert_eq!(metric.status(), HealthStatus::Degraded);

        metric.value = 95.0;
        assert_eq!(metric.status(), HealthStatus::Critical);
    }

    #[tokio::test]
    async fn test_health_monitor() {
        let monitor = HealthMonitor::new();
        monitor.register("component-1".to_string()).await;

        let metric = SystemMetric::new("cpu".to_string(), 50.0);
        monitor.update_metric("component-1", metric).await;

        let status = monitor.status("component-1").await;
        assert_eq!(status, Some(HealthStatus::Healthy));
    }

    #[tokio::test]
    async fn test_anomaly_detector() {
        let detector = AnomalyDetector::new();
        detector.train("component-1".to_string(), 50.0).await;

        let anomaly = detector.detect("component-1", 150.0).await;
        assert!(anomaly.is_some());
    }

    #[tokio::test]
    async fn test_auto_recovery() {
        let recovery = AutoRecovery::new();
        let action = recovery.plan_recovery("component-1", "load spike").await;

        assert_eq!(action.status, HealingStatus::Planned);

        let result = recovery.execute(&action.action_id).await;
        assert!(result);
    }
}
