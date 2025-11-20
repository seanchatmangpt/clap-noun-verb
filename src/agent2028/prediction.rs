use chrono::{DateTime, Duration, Utc};
/// Predictive Capability Planning
///
/// ML-based workload forecasting, capacity planning, and resource provisioning
/// to proactively allocate resources before bottlenecks occur.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Time series data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    pub timestamp: DateTime<Utc>,
    pub value: f64,
}

/// Workload forecast
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadForecast {
    pub forecast_id: String,
    pub capability_name: String,
    pub predictions: Vec<(DateTime<Utc>, f64)>, // (timestamp, predicted_volume)
    pub confidence_interval: f64,               // 0.0-1.0
    pub forecast_horizon_hours: u32,
    pub model_accuracy: f64, // MAPE (Mean Absolute Percentage Error)
}

impl WorkloadForecast {
    pub fn new(capability_name: String, horizon_hours: u32) -> Self {
        Self {
            forecast_id: uuid::Uuid::new_v4().to_string(),
            capability_name,
            predictions: Vec::new(),
            confidence_interval: 0.95,
            forecast_horizon_hours: horizon_hours,
            model_accuracy: 0.0,
        }
    }

    /// Get prediction for a specific time
    pub fn prediction_at(&self, time: DateTime<Utc>) -> Option<f64> {
        self.predictions.iter().find(|(t, _)| (*t - time).num_minutes().abs() < 5).map(|(_, v)| *v)
    }

    /// Get peak predicted load in forecast
    pub fn peak_load(&self) -> Option<f64> {
        self.predictions.iter().map(|(_, v)| v).copied().max_by(|a, b| a.partial_cmp(b).unwrap())
    }

    /// Get average predicted load
    pub fn average_load(&self) -> f64 {
        if self.predictions.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.predictions.iter().map(|(_, v)| v).sum();
        sum / self.predictions.len() as f64
    }
}

/// Workload Forecaster using simple time series methods
#[derive(Clone)]
pub struct WorkloadForecaster {
    historical_data: Arc<RwLock<HashMap<String, Vec<TimeSeriesPoint>>>>,
}

impl WorkloadForecaster {
    pub fn new() -> Self {
        Self { historical_data: Arc::new(RwLock::new(HashMap::new())) }
    }

    /// Record historical data point
    pub async fn record(&self, capability_name: String, value: f64) {
        let mut data = self.historical_data.write().await;
        let series = data.entry(capability_name).or_insert_with(Vec::new);
        series.push(TimeSeriesPoint { timestamp: Utc::now(), value });

        // Keep only last 30 days of data
        let cutoff = Utc::now() - Duration::days(30);
        series.retain(|p| p.timestamp > cutoff);
    }

    /// Forecast workload for a capability
    pub async fn forecast(&self, capability_name: &str, hours_ahead: u32) -> WorkloadForecast {
        let data = self.historical_data.read().await;
        let mut forecast = WorkloadForecast::new(capability_name.to_string(), hours_ahead);

        if let Some(series) = data.get(capability_name) {
            if series.len() < 2 {
                return forecast; // Not enough data
            }

            // Simple exponential smoothing + trend
            let avg = series.iter().map(|p| p.value).sum::<f64>() / series.len() as f64;
            let trend = if series.len() >= 2 {
                (series[series.len() - 1].value - series[0].value) / (series.len() as f64 - 1.0)
            } else {
                0.0
            };

            // Generate forecast points
            let now = Utc::now();
            for hour in 1..=hours_ahead {
                let predicted_time = now + Duration::hours(hour as i64);
                let predicted_value = (avg + trend * hour as f64).max(0.0);
                forecast.predictions.push((predicted_time, predicted_value));
            }

            // Simple model accuracy: based on recent variance
            let recent = series.iter().rev().take(10).collect::<Vec<_>>();
            if recent.len() >= 2 {
                let mean = recent.iter().map(|p| p.value).sum::<f64>() / recent.len() as f64;
                let variance = recent.iter().map(|p| (p.value - mean).powi(2)).sum::<f64>()
                    / recent.len() as f64;
                let stddev = variance.sqrt();
                forecast.model_accuracy = (1.0 - (stddev / (mean + 1.0))).max(0.0);
            }
        }

        forecast
    }

    /// Get recent data for a capability
    pub async fn recent_data(&self, capability_name: &str, hours: i64) -> Vec<TimeSeriesPoint> {
        let data = self.historical_data.read().await;
        let cutoff = Utc::now() - Duration::hours(hours);

        data.get(capability_name)
            .map(|series| series.iter().filter(|p| p.timestamp > cutoff).cloned().collect())
            .unwrap_or_default()
    }
}

impl Default for WorkloadForecaster {
    fn default() -> Self {
        Self::new()
    }
}

/// Capability demand prediction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityDemand {
    pub capability: String,
    pub predicted_demand: f64, // 0.0 to 1.0 (normalized)
    pub priority: u8,          // 1-10, higher = more important
    pub predicted_at: DateTime<Utc>,
}

/// Resource provisioning recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisioningRecommendation {
    pub recommendation_id: String,
    pub capability: String,
    pub current_capacity: u32,
    pub recommended_capacity: u32,
    pub confidence: f64,
    pub urgency: u8, // 1-10, higher = more urgent
    pub estimated_cost: f64,
}

impl ProvisioningRecommendation {
    pub fn new(capability: String, current: u32, recommended: u32) -> Self {
        let urgency = if recommended > current * 2 {
            10
        } else if recommended > current {
            5
        } else {
            1
        };

        Self {
            recommendation_id: uuid::Uuid::new_v4().to_string(),
            capability,
            current_capacity: current,
            recommended_capacity: recommended,
            confidence: 0.8,
            urgency,
            estimated_cost: 0.0,
        }
    }

    /// Calculate required scale factor
    pub fn scale_factor(&self) -> f64 {
        self.recommended_capacity as f64 / (self.current_capacity as f64).max(1.0)
    }
}

/// Capacity Planner
pub struct CapacityPlanner {
    forecaster: WorkloadForecaster,
    recommendations: Arc<RwLock<Vec<ProvisioningRecommendation>>>,
    current_capacities: Arc<RwLock<HashMap<String, u32>>>,
}

impl CapacityPlanner {
    pub fn new(forecaster: WorkloadForecaster) -> Self {
        Self {
            forecaster,
            recommendations: Arc::new(RwLock::new(Vec::new())),
            current_capacities: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Set current capacity for a capability
    pub async fn set_capacity(&self, capability: String, capacity: u32) {
        let mut capacities = self.current_capacities.write().await;
        capacities.insert(capability, capacity);
    }

    /// Plan capacity based on forecast
    pub async fn plan_capacity(
        &self,
        capability: &str,
        hours_ahead: u32,
    ) -> Option<ProvisioningRecommendation> {
        let forecast = self.forecaster.forecast(capability, hours_ahead).await;
        let peak = forecast.peak_load()?;

        let capacities = self.current_capacities.read().await;
        let current_capacity = capacities.get(capability).copied().unwrap_or(100);

        // Recommend capacity to handle peak with 20% buffer
        let required_capacity = ((peak * 1.2) as u32).max(current_capacity);

        if required_capacity > current_capacity {
            let mut recommendation = ProvisioningRecommendation::new(
                capability.to_string(),
                current_capacity,
                required_capacity,
            );

            recommendation.confidence = forecast.model_accuracy;
            recommendation.estimated_cost = (required_capacity - current_capacity) as f64 * 10.0; // $10 per unit

            let mut recommendations = self.recommendations.write().await;
            recommendations.push(recommendation.clone());

            Some(recommendation)
        } else {
            None
        }
    }

    /// Get pending recommendations
    pub async fn pending_recommendations(&self) -> Vec<ProvisioningRecommendation> {
        let recommendations = self.recommendations.read().await;
        recommendations.clone()
    }

    /// Accept a recommendation and update capacity
    pub async fn accept_recommendation(&self, rec_id: &str) -> bool {
        let mut recommendations = self.recommendations.write().await;

        if let Some(rec) = recommendations.iter_mut().find(|r| r.recommendation_id == rec_id) {
            // Update capacity
            let mut capacities = self.current_capacities.write().await;
            capacities.insert(rec.capability.clone(), rec.recommended_capacity);
            true
        } else {
            false
        }
    }

    /// Get cost optimization suggestions
    pub async fn optimize_costs(&self) -> Vec<CostOptimization> {
        let capacities = self.current_capacities.read().await;
        let mut optimizations = Vec::new();

        for (capability, &current) in capacities.iter() {
            let recent = self.forecaster.recent_data(capability, 24).await;
            if !recent.is_empty() {
                let max_recent = recent.iter().map(|p| p.value).fold(0.0, f64::max);
                let recommended = ((max_recent * 1.1) as u32).max(50);

                if recommended < current {
                    optimizations.push(CostOptimization {
                        capability: capability.clone(),
                        current_capacity: current,
                        optimized_capacity: recommended,
                        estimated_savings: (current - recommended) as f64 * 10.0,
                    });
                }
            }
        }

        optimizations
    }
}

/// Cost optimization suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostOptimization {
    pub capability: String,
    pub current_capacity: u32,
    pub optimized_capacity: u32,
    pub estimated_savings: f64,
}

/// Risk assessment for capacity planning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub assessment_id: String,
    pub capability: String,
    pub bottleneck_probability: f64, // 0.0 to 1.0
    pub severity: String,            // "low", "medium", "high", "critical"
    pub recommended_actions: Vec<String>,
}

impl RiskAssessment {
    pub fn new(capability: String, probability: f64) -> Self {
        let severity = match probability {
            p if p > 0.75 => "critical",
            p if p > 0.5 => "high",
            p if p > 0.25 => "medium",
            _ => "low",
        };

        Self {
            assessment_id: uuid::Uuid::new_v4().to_string(),
            capability,
            bottleneck_probability: probability,
            severity: severity.to_string(),
            recommended_actions: vec![],
        }
    }
}

/// Risk Assessor
pub struct RiskAssessor;

impl RiskAssessor {
    /// Assess risk of bottleneck for a capability
    pub fn assess(forecast: &WorkloadForecast, current_capacity: f64) -> RiskAssessment {
        let peak = forecast.peak_load().unwrap_or(0.0);
        let probability = if peak > current_capacity {
            ((peak - current_capacity) / current_capacity).min(1.0)
        } else {
            0.0
        };

        let mut assessment = RiskAssessment::new(forecast.capability_name.clone(), probability);

        if probability > 0.5 {
            assessment.recommended_actions.push("Scale capacity immediately".to_string());
            assessment.recommended_actions.push("Review caching strategies".to_string());
            assessment.recommended_actions.push("Implement rate limiting".to_string());
        }

        assessment
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_workload_forecaster() {
        let forecaster = WorkloadForecaster::new();

        for i in 0..24 {
            forecaster.record("database.query".to_string(), 50.0 + i as f64 * 2.0).await;
        }

        let forecast = forecaster.forecast("database.query", 24).await;
        assert!(!forecast.predictions.is_empty());
        assert!(forecast.average_load() > 0.0);
    }

    #[test]
    fn test_provisioning_recommendation() {
        let rec = ProvisioningRecommendation::new("database.query".to_string(), 100, 200);

        assert_eq!(rec.scale_factor(), 2.0);
        assert_eq!(rec.urgency, 5);
    }

    #[tokio::test]
    async fn test_capacity_planner() {
        let forecaster = WorkloadForecaster::new();
        let planner = CapacityPlanner::new(forecaster.clone());

        planner.set_capacity("database.query".to_string(), 100).await;

        // Record increasing load
        for i in 0..24 {
            forecaster.record("database.query".to_string(), 50.0 + i as f64 * 3.0).await;
        }

        let recommendation = planner.plan_capacity("database.query", 24).await;
        assert!(recommendation.is_some());
    }

    #[test]
    fn test_risk_assessment() {
        let forecast = WorkloadForecast::new("database.query".to_string(), 24);
        let assessment = RiskAssessor::assess(&forecast, 100.0);

        assert!(
            assessment.bottleneck_probability >= 0.0 && assessment.bottleneck_probability <= 1.0
        );
    }
}
