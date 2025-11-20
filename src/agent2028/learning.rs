use chrono::{DateTime, Duration, Utc};
/// Agent Learning & Adaptation Framework
///
/// Enables agents to improve command execution over time through ML-driven learning,
/// pattern recognition, and intelligent strategy adaptation.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Command execution metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetrics {
    pub command_id: String,
    pub command_name: String,
    pub agent_id: String,
    pub execution_time_ms: u64,
    pub memory_used_bytes: u64,
    pub cpu_percent: f64,
    pub success: bool,
    pub error_type: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// Feature vector for ML model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Features {
    pub command_name_hash: u64,
    pub hour_of_day: u8,
    pub day_of_week: u8,
    pub historical_avg_time: f64,
    pub recent_success_rate: f64,
    pub agent_health: f64,
    pub system_load: f64,
}

impl Features {
    /// Convert features to normalized vector for ML model
    pub fn to_vector(&self) -> Vec<f64> {
        vec![
            (self.command_name_hash as f64) % 1000.0 / 1000.0,
            self.hour_of_day as f64 / 24.0,
            self.day_of_week as f64 / 7.0,
            (self.historical_avg_time / 1000.0).min(1.0),
            self.recent_success_rate,
            self.agent_health,
            self.system_load,
        ]
    }
}

/// Execution profile for a command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandProfile {
    pub name: String,
    pub total_executions: u64,
    pub successful_executions: u64,
    pub avg_execution_time_ms: f64,
    pub min_execution_time_ms: u64,
    pub max_execution_time_ms: u64,
    pub avg_memory_bytes: u64,
    pub success_rate: f64,
}

impl CommandProfile {
    pub fn new(name: String) -> Self {
        Self {
            name,
            total_executions: 0,
            successful_executions: 0,
            avg_execution_time_ms: 0.0,
            min_execution_time_ms: u64::MAX,
            max_execution_time_ms: 0,
            avg_memory_bytes: 0,
            success_rate: 1.0,
        }
    }

    /// Update profile with new execution metric
    pub fn update(&mut self, metric: &ExecutionMetrics) {
        self.total_executions += 1;

        if metric.success {
            self.successful_executions += 1;
        }

        // Update running averages
        let alpha = 1.0 / self.total_executions as f64; // Exponential moving average
        self.avg_execution_time_ms =
            self.avg_execution_time_ms * (1.0 - alpha) + metric.execution_time_ms as f64 * alpha;

        self.min_execution_time_ms = self.min_execution_time_ms.min(metric.execution_time_ms);
        self.max_execution_time_ms = self.max_execution_time_ms.max(metric.execution_time_ms);

        self.avg_memory_bytes = (self.avg_memory_bytes as f64 * (1.0 - alpha)
            + metric.memory_used_bytes as f64 * alpha) as u64;

        self.success_rate = self.successful_executions as f64 / self.total_executions as f64;
    }
}

/// Execution Profiler - Collects and analyzes execution metrics
#[derive(Clone)]
pub struct ExecutionProfiler {
    metrics: Arc<RwLock<Vec<ExecutionMetrics>>>,
    profiles: Arc<RwLock<HashMap<String, CommandProfile>>>,
}

impl ExecutionProfiler {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(Vec::new())),
            profiles: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Record a command execution
    pub async fn record(&self, metric: ExecutionMetrics) {
        let mut metrics = self.metrics.write().await;
        metrics.push(metric.clone());

        let mut profiles = self.profiles.write().await;
        profiles
            .entry(metric.command_name.clone())
            .or_insert_with(|| CommandProfile::new(metric.command_name.clone()))
            .update(&metric);
    }

    /// Get profile for a command
    pub async fn profile(&self, command_name: &str) -> Option<CommandProfile> {
        let profiles = self.profiles.read().await;
        profiles.get(command_name).cloned()
    }

    /// Get metrics from last N hours
    pub async fn recent_metrics(&self, hours: i64) -> Vec<ExecutionMetrics> {
        let metrics = self.metrics.read().await;
        let cutoff = Utc::now() - Duration::hours(hours);
        metrics.iter().filter(|m| m.timestamp > cutoff).cloned().collect()
    }

    /// Get anomalous metrics (outliers)
    pub async fn anomalies(&self, std_dev_threshold: f64) -> Vec<ExecutionMetrics> {
        let metrics = self.metrics.read().await;
        let profiles = self.profiles.read().await;

        let mut anomalies = Vec::new();

        for metric in metrics.iter() {
            if let Some(profile) = profiles.get(&metric.command_name) {
                let z_score = ((metric.execution_time_ms as f64 - profile.avg_execution_time_ms)
                    / (profile.avg_execution_time_ms * 0.1).max(1.0))
                .abs();

                if z_score > std_dev_threshold {
                    anomalies.push(metric.clone());
                }
            }
        }

        anomalies
    }
}

impl Default for ExecutionProfiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple ML model for prediction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionModel {
    pub name: String,
    pub version: String,
    /// Weights for linear regression
    pub weights: Vec<f64>,
    pub bias: f64,
}

impl PredictionModel {
    pub fn new(name: String, features_dim: usize) -> Self {
        Self { name, version: "1.0".to_string(), weights: vec![0.1; features_dim], bias: 0.0 }
    }

    /// Predict execution time (in milliseconds)
    pub fn predict(&self, features: &Features) -> f64 {
        let vector = features.to_vector();
        let mut prediction = self.bias;

        for (w, f) in self.weights.iter().zip(vector.iter()) {
            prediction += w * f;
        }

        (prediction * 1000.0).max(1.0) // Min 1ms
    }

    /// Update model weights with gradient descent
    pub fn update_weights(&mut self, features: &Features, actual_time_ms: f64, learning_rate: f64) {
        let vector = features.to_vector();
        let prediction = self.predict(features);
        let error = actual_time_ms - prediction;

        // Gradient descent update
        self.bias += learning_rate * error;

        for (w, f) in self.weights.iter_mut().zip(vector.iter()) {
            let gradient = -2.0 * error * f / vector.len() as f64;
            *w -= learning_rate * gradient;
        }
    }
}

/// Model Inference Engine
pub struct ModelInference {
    model: Arc<RwLock<PredictionModel>>,
}

impl ModelInference {
    pub fn new(model: PredictionModel) -> Self {
        Self { model: Arc::new(RwLock::new(model)) }
    }

    /// Predict execution time for given features
    pub async fn predict(&self, features: &Features) -> f64 {
        let model = self.model.read().await;
        model.predict(features)
    }

    /// Update model with new training data
    pub async fn train(&self, features: &Features, actual_time_ms: f64, learning_rate: f64) {
        let mut model = self.model.write().await;
        model.update_weights(features, actual_time_ms, learning_rate);
    }

    /// Get current model
    pub async fn get_model(&self) -> PredictionModel {
        let model = self.model.read().await;
        model.clone()
    }
}

/// Strategy for adapting agent behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptationStrategy {
    pub agent_id: String,
    pub preferred_commands: Vec<String>,
    pub retry_strategy: String,
    pub timeout_strategy: String,
    pub confidence: f64,
}

/// Adaptation Engine for continuous learning
pub struct AdaptationEngine {
    strategies: Arc<RwLock<HashMap<String, AdaptationStrategy>>>,
    profiler: ExecutionProfiler,
    inference: ModelInference,
}

impl AdaptationEngine {
    pub fn new(profiler: ExecutionProfiler, inference: ModelInference) -> Self {
        Self { strategies: Arc::new(RwLock::new(HashMap::new())), profiler, inference }
    }

    /// Get adaptation strategy for an agent
    pub async fn get_strategy(&self, agent_id: &str) -> Option<AdaptationStrategy> {
        let strategies = self.strategies.read().await;
        strategies.get(agent_id).cloned()
    }

    /// Update strategy based on learning
    pub async fn update_strategy(&self, agent_id: &str, commands: Vec<String>, confidence: f64) {
        let mut strategies = self.strategies.write().await;
        strategies.insert(
            agent_id.to_string(),
            AdaptationStrategy {
                agent_id: agent_id.to_string(),
                preferred_commands: commands,
                retry_strategy: "exponential-backoff".to_string(),
                timeout_strategy: "adaptive".to_string(),
                confidence,
            },
        );
    }

    /// Get confidence in a particular routing decision
    pub async fn confidence(&self, agent_id: &str, command: &str) -> f64 {
        if let Some(strategy) = self.get_strategy(agent_id).await {
            if strategy.preferred_commands.contains(&command.to_string()) {
                return strategy.confidence;
            }
        }
        0.5 // Default: neutral confidence
    }

    /// Recommend retry parameters based on command profile
    pub async fn recommend_retry(&self, command_name: &str) -> (u32, u64) {
        if let Some(profile) = self.profiler.profile(command_name).await {
            let failure_rate = 1.0 - profile.success_rate;
            let max_retries = if failure_rate > 0.5 { 5 } else { 3 };
            let base_delay_ms = (profile.avg_execution_time_ms as u64).min(1000);
            (max_retries, base_delay_ms)
        } else {
            (3, 100) // Default
        }
    }

    /// Predict if command will succeed
    pub async fn predict_success(&self, command_name: &str) -> f64 {
        if let Some(profile) = self.profiler.profile(command_name).await {
            profile.success_rate
        } else {
            0.5 // No data: assume 50% success
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_profile() {
        let mut profile = CommandProfile::new("test.command".to_string());
        let metric = ExecutionMetrics {
            command_id: "cmd-1".to_string(),
            command_name: "test.command".to_string(),
            agent_id: "agent-1".to_string(),
            execution_time_ms: 100,
            memory_used_bytes: 1024,
            cpu_percent: 50.0,
            success: true,
            error_type: None,
            timestamp: Utc::now(),
        };

        profile.update(&metric);
        assert_eq!(profile.total_executions, 1);
        assert_eq!(profile.success_rate, 1.0);
    }

    #[tokio::test]
    async fn test_profiler() {
        let profiler = ExecutionProfiler::new();
        let metric = ExecutionMetrics {
            command_id: "cmd-1".to_string(),
            command_name: "test.command".to_string(),
            agent_id: "agent-1".to_string(),
            execution_time_ms: 100,
            memory_used_bytes: 1024,
            cpu_percent: 50.0,
            success: true,
            error_type: None,
            timestamp: Utc::now(),
        };

        profiler.record(metric).await;
        let profile = profiler.profile("test.command").await;
        assert!(profile.is_some());
    }

    #[tokio::test]
    async fn test_model_inference() {
        let model = PredictionModel::new("test-model".to_string(), 7);
        let inference = ModelInference::new(model);

        let features = Features {
            command_name_hash: 12345,
            hour_of_day: 14,
            day_of_week: 3,
            historical_avg_time: 100.0,
            recent_success_rate: 0.95,
            agent_health: 0.99,
            system_load: 0.5,
        };

        let prediction = inference.predict(&features).await;
        assert!(prediction > 0.0);
    }
}
