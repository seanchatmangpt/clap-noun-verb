use chrono::{DateTime, Utc};
/// Distributed Task Allocation
///
/// Self-organizing task markets where agents bid on tasks based on
/// capability, load, and availability.
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Task in the market
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmTask {
    pub task_id: String,
    pub description: String,
    pub required_skills: Vec<String>,
    pub priority: u8, // 1-10
    pub reward: f64,
    pub deadline: Option<DateTime<Utc>>,
    pub assigned_agent: Option<String>,
    pub status: TaskStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Open,
    Assigned,
    InProgress,
    Completed,
    Failed,
}

impl SwarmTask {
    pub fn new(
        description: String,
        required_skills: Vec<String>,
        priority: u8,
        reward: f64,
    ) -> Self {
        Self {
            task_id: uuid::Uuid::new_v4().to_string(),
            description,
            required_skills,
            priority,
            reward,
            deadline: None,
            assigned_agent: None,
            status: TaskStatus::Open,
            created_at: Utc::now(),
        }
    }

    /// Check if agent has required skills
    pub fn agent_qualifies(&self, agent_skills: &[String]) -> bool {
        self.required_skills.iter().all(|req| agent_skills.contains(req))
    }
}

/// Bid on a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskBid {
    pub bid_id: String,
    pub task_id: String,
    pub agent_id: String,
    pub bid_price: f64,                 // Lower is better for agent
    pub estimated_completion_time: u64, // Seconds
    pub confidence: f64,                // 0.0 to 1.0
    pub current_load: usize,            // Tasks agent already has
    pub timestamp: DateTime<Utc>,
}

impl TaskBid {
    pub fn new(
        task_id: String,
        agent_id: String,
        bid_price: f64,
        estimated_time: u64,
        confidence: f64,
        current_load: usize,
    ) -> Self {
        Self {
            bid_id: uuid::Uuid::new_v4().to_string(),
            task_id,
            agent_id,
            bid_price,
            estimated_completion_time: estimated_time,
            confidence,
            current_load,
            timestamp: Utc::now(),
        }
    }

    /// Score the bid (lower score = better bid)
    pub fn score(&self) -> f64 {
        let load_factor = 1.0 + (self.current_load as f64) * 0.1; // Higher load = worse
        let time_factor = (self.estimated_completion_time as f64) / 1000.0; // Longer time = worse
        let confidence_factor = 1.0 / (self.confidence + 0.1); // Lower confidence = worse

        (self.bid_price * load_factor * time_factor * confidence_factor).max(0.01)
    }
}

/// Task Market
pub struct TaskMarket {
    open_tasks: Arc<RwLock<HashMap<String, SwarmTask>>>,
    bids: Arc<RwLock<Vec<TaskBid>>>,
    agent_loads: Arc<RwLock<HashMap<String, usize>>>,
}

impl TaskMarket {
    pub fn new() -> Self {
        Self {
            open_tasks: Arc::new(RwLock::new(HashMap::new())),
            bids: Arc::new(RwLock::new(Vec::new())),
            agent_loads: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// List a new task
    pub async fn list_task(&self, task: SwarmTask) {
        let mut tasks = self.open_tasks.write().await;
        tasks.insert(task.task_id.clone(), task);
    }

    /// Get open tasks
    pub async fn get_open_tasks(&self) -> Vec<SwarmTask> {
        let tasks = self.open_tasks.read().await;
        tasks.values().filter(|t| t.status == TaskStatus::Open).cloned().collect()
    }

    /// Agent bids on a task
    pub async fn place_bid(&self, bid: TaskBid) -> bool {
        let tasks = self.open_tasks.read().await;

        if let Some(task) = tasks.get(&bid.task_id) {
            if task.status != TaskStatus::Open {
                return false;
            }

            let mut bids = self.bids.write().await;
            bids.push(bid);
            true
        } else {
            false
        }
    }

    /// Run auction for a task (Dutch auction: price descends)
    pub async fn run_auction(&self, task_id: &str, _rounds: u32) -> Option<String> {
        let tasks = self.open_tasks.read().await;

        if !tasks.contains_key(task_id) {
            return None;
        }

        drop(tasks);

        let bids = self.bids.read().await;
        let task_bids: Vec<&TaskBid> = bids.iter().filter(|b| b.task_id == task_id).collect();

        if task_bids.is_empty() {
            return None;
        }

        // Find best bid (lowest score)
        let winner = task_bids
            .iter()
            .min_by(|a, b| a.score().partial_cmp(&b.score()).unwrap_or(Ordering::Equal))?;

        let winning_agent = winner.agent_id.clone();
        drop(bids);

        // Assign task to winning agent
        let mut tasks = self.open_tasks.write().await;
        if let Some(task) = tasks.get_mut(task_id) {
            task.assigned_agent = Some(winning_agent.clone());
            task.status = TaskStatus::Assigned;

            // Update agent load
            let mut loads = self.agent_loads.write().await;
            *loads.entry(winning_agent.clone()).or_insert(0) += 1;
        }

        Some(winning_agent)
    }

    /// Update task status
    pub async fn update_task_status(&self, task_id: &str, status: TaskStatus) {
        let mut tasks = self.open_tasks.write().await;
        if let Some(task) = tasks.get_mut(task_id) {
            task.status = status;

            // Update load if task completed
            if status == TaskStatus::Completed || status == TaskStatus::Failed {
                if let Some(agent_id) = &task.assigned_agent {
                    let mut loads = self.agent_loads.write().await;
                    if let Some(load) = loads.get_mut(agent_id) {
                        if *load > 0 {
                            *load -= 1;
                        }
                    }
                }
            }
        }
    }

    /// Get agent load
    pub async fn get_agent_load(&self, agent_id: &str) -> usize {
        let loads = self.agent_loads.read().await;
        loads.get(agent_id).copied().unwrap_or(0)
    }

    /// Get auction metrics
    pub async fn auction_metrics(&self) -> (usize, usize, usize) {
        let tasks = self.open_tasks.read().await;
        let open_count = tasks.values().filter(|t| t.status == TaskStatus::Open).count();
        let assigned_count = tasks.values().filter(|t| t.status == TaskStatus::Assigned).count();
        let completed_count = tasks.values().filter(|t| t.status == TaskStatus::Completed).count();

        (open_count, assigned_count, completed_count)
    }
}

impl Default for TaskMarket {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_task_creation() {
        let task =
            SwarmTask::new("Process data".to_string(), vec!["ml.inference".to_string()], 5, 100.0);

        assert_eq!(task.status, TaskStatus::Open);
    }

    #[tokio::test]
    async fn test_task_market() {
        let market = TaskMarket::new();
        let task =
            SwarmTask::new("Process data".to_string(), vec!["ml.inference".to_string()], 5, 100.0);

        let _task_id = task.task_id.clone();
        market.list_task(task).await;

        let open_tasks = market.get_open_tasks().await;
        assert_eq!(open_tasks.len(), 1);
    }

    #[tokio::test]
    async fn test_bidding() {
        let market = TaskMarket::new();
        let task = SwarmTask::new("Compute".to_string(), vec!["compute".to_string()], 5, 100.0);
        let task_id = task.task_id.clone();
        market.list_task(task).await;

        let bid = TaskBid::new(task_id.clone(), "agent-1".to_string(), 50.0, 100, 0.95, 2);
        assert!(market.place_bid(bid).await);
    }
}
