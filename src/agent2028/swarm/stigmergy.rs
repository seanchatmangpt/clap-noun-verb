use chrono::{DateTime, Duration, Utc};
/// Stigmergic Communication System
///
/// Indirect coordination via virtual pheromones. Agents leave chemical markers
/// in the environment which other agents read, enabling coordination without
/// explicit messaging.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Single pheromone cell in the field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PheromoneCell {
    pub x: i32,
    pub y: i32,
    pub intensity: f64,         // 0.0 to 1.0
    pub pheromone_type: String, // "food", "danger", "home", etc.
    pub deposited_by: String,   // Agent ID
    pub created_at: DateTime<Utc>,
    pub last_reinforced: DateTime<Utc>,
}

impl PheromoneCell {
    pub fn new(x: i32, y: i32, pheromone_type: String, agent_id: String) -> Self {
        Self {
            x,
            y,
            intensity: 1.0,
            pheromone_type,
            deposited_by: agent_id,
            created_at: Utc::now(),
            last_reinforced: Utc::now(),
        }
    }

    /// Decay pheromone intensity over time
    pub fn decay(&mut self, decay_rate: f64) {
        let age_seconds = (Utc::now() - self.created_at).num_seconds() as f64;
        let decay_factor = (-decay_rate * age_seconds).exp();
        self.intensity *= decay_factor;
    }

    /// Reinforce this pheromone
    pub fn reinforce(&mut self, strength: f64) {
        self.intensity = (self.intensity + strength).min(1.0);
        self.last_reinforced = Utc::now();
    }

    /// Check if pheromone is still active
    pub fn is_active(&self) -> bool {
        self.intensity > 0.01
    }
}

/// Global pheromone field
pub struct PheromoneField {
    cells: Arc<RwLock<HashMap<(i32, i32), PheromoneCell>>>,
    decay_rate: f64,
    diffusion_rate: f64,
}

impl PheromoneField {
    pub fn new(decay_rate: f64, diffusion_rate: f64) -> Self {
        Self { cells: Arc::new(RwLock::new(HashMap::new())), decay_rate, diffusion_rate }
    }

    /// Deposit pheromone at a location
    pub async fn deposit(&self, x: i32, y: i32, pheromone_type: String, agent_id: String) {
        let mut cells = self.cells.write().await;
        let cell = cells
            .entry((x, y))
            .or_insert_with(|| PheromoneCell::new(x, y, pheromone_type.clone(), agent_id.clone()));

        cell.reinforce(0.5);
    }

    /// Read pheromone intensity at a location
    pub async fn read(&self, x: i32, y: i32) -> f64 {
        let cells = self.cells.read().await;
        cells.get(&(x, y)).map(|c| c.intensity).unwrap_or(0.0)
    }

    /// Get gradient at location (direction of strongest pheromone)
    pub async fn gradient(&self, x: i32, y: i32) -> (f64, f64) {
        let cells = self.cells.read().await;

        let center = cells.get(&(x, y)).map(|c| c.intensity).unwrap_or(0.0);
        let left = cells.get(&(x - 1, y)).map(|c| c.intensity).unwrap_or(0.0);
        let right = cells.get(&(x + 1, y)).map(|c| c.intensity).unwrap_or(0.0);
        let up = cells.get(&(x, y - 1)).map(|c| c.intensity).unwrap_or(0.0);
        let down = cells.get(&(x, y + 1)).map(|c| c.intensity).unwrap_or(0.0);

        let dx = (right - left) / 2.0;
        let dy = (down - up) / 2.0;

        // Normalize
        let magnitude = (dx * dx + dy * dy).sqrt();
        if magnitude > 0.01 {
            (dx / magnitude, dy / magnitude)
        } else {
            (0.0, 0.0)
        }
    }

    /// Diffuse pheromones to neighboring cells
    pub async fn diffuse(&self) {
        let mut cells = self.cells.write().await;
        let mut updates = HashMap::new();

        for (&(x, y), cell) in cells.iter() {
            if !cell.is_active() {
                continue;
            }

            let diffuse_amount = cell.intensity * self.diffusion_rate;

            // Distribute to 4 neighbors
            for (dx, dy) in &[(-1, 0), (1, 0), (0, -1), (0, 1)] {
                let neighbor_pos = (x + dx, y + dy);
                *updates.entry(neighbor_pos).or_insert(0.0) += diffuse_amount / 4.0;
            }
        }

        // Apply updates
        for ((x, y), amount) in updates {
            cells
                .entry((x, y))
                .or_insert_with(|| {
                    PheromoneCell::new(x, y, "diffused".to_string(), "environment".to_string())
                })
                .intensity += amount;
        }
    }

    /// Decay all pheromones
    pub async fn decay_all(&self) {
        let mut cells = self.cells.write().await;
        for cell in cells.values_mut() {
            cell.decay(self.decay_rate);
        }

        // Remove inactive cells
        cells.retain(|_, cell| cell.is_active());
    }

    /// Get all active pheromones
    pub async fn active_pheromones(&self) -> Vec<PheromoneCell> {
        let cells = self.cells.read().await;
        cells.values().filter(|c| c.is_active()).cloned().collect()
    }

    /// Clear all pheromones
    pub async fn clear(&self) {
        let mut cells = self.cells.write().await;
        cells.clear();
    }
}

/// Stigmergic Protocol for coordinating agents
pub struct StigmergicProtocol {
    field: Arc<PheromoneField>,
}

impl StigmergicProtocol {
    pub fn new(field: Arc<PheromoneField>) -> Self {
        Self { field }
    }

    /// Agent deposits pheromone to signal success
    pub async fn signal_success(&self, x: i32, y: i32, agent_id: String) {
        self.field.deposit(x, y, "success".to_string(), agent_id).await;
    }

    /// Agent deposits pheromone to signal danger
    pub async fn signal_danger(&self, x: i32, y: i32, agent_id: String) {
        self.field.deposit(x, y, "danger".to_string(), agent_id).await;
    }

    /// Agent deposits pheromone to signal food/resource
    pub async fn signal_resource(&self, x: i32, y: i32, resource_type: String, agent_id: String) {
        self.field.deposit(x, y, format!("resource:{}", resource_type), agent_id).await;
    }

    /// Agent reads pheromone gradient to navigate
    pub async fn follow_gradient(&self, x: i32, y: i32) -> (f64, f64) {
        self.field.gradient(x, y).await
    }

    /// Agent finds nearest pheromone of type
    pub async fn find_nearest(
        &self,
        x: i32,
        y: i32,
        pheromone_type: &str,
        search_radius: i32,
    ) -> Option<(i32, i32)> {
        let mut best_pos = None;
        let mut best_dist = f64::MAX;

        let pheromones = self.field.active_pheromones().await;

        for cell in pheromones {
            if cell.pheromone_type.starts_with(pheromone_type) {
                let dist = ((cell.x - x).pow(2) + (cell.y - y).pow(2)) as f64;
                if dist <= (search_radius as f64).powi(2) && dist < best_dist {
                    best_dist = dist;
                    best_pos = Some((cell.x, cell.y));
                }
            }
        }

        best_pos
    }

    /// Simulate environmental diffusion
    pub async fn diffuse_pheromones(&self) {
        self.field.diffuse().await;
    }

    /// Decay pheromones over time
    pub async fn decay_pheromones(&self) {
        self.field.decay_all().await;
    }
}

/// Trail reinforcement for path learning
pub struct TrailReinforcer {
    protocol: Arc<StigmergicProtocol>,
}

impl TrailReinforcer {
    pub fn new(protocol: Arc<StigmergicProtocol>) -> Self {
        Self { protocol }
    }

    /// Mark a complete successful path
    pub async fn mark_successful_path(&self, path: Vec<(i32, i32)>, agent_id: String) {
        for (x, y) in path {
            self.protocol.signal_success(x, y, agent_id.clone()).await;
        }
    }

    /// Reinforce path segments
    pub async fn reinforce_path(&self, path: Vec<(i32, i32)>, strength: f64) {
        let field = &self.protocol.field;
        let mut cells = field.cells.write().await;

        for (x, y) in path {
            if let Some(cell) = cells.get_mut(&(x, y)) {
                cell.reinforce(strength);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pheromone_deposit() {
        let field = Arc::new(PheromoneField::new(0.1, 0.05));
        field.deposit(0, 0, "test".to_string(), "agent-1".to_string()).await;

        let intensity = field.read(0, 0).await;
        assert!(intensity > 0.0);
    }

    #[tokio::test]
    async fn test_pheromone_decay() {
        let field = Arc::new(PheromoneField::new(0.5, 0.05));
        field.deposit(0, 0, "test".to_string(), "agent-1".to_string()).await;

        let intensity_before = field.read(0, 0).await;
        field.decay_all().await;
        let intensity_after = field.read(0, 0).await;

        assert!(intensity_after < intensity_before);
    }

    #[tokio::test]
    async fn test_stigmergic_protocol() {
        let field = Arc::new(PheromoneField::new(0.1, 0.05));
        let protocol = StigmergicProtocol::new(field);

        protocol.signal_success(5, 5, "agent-1".to_string()).await;
        let nearest = protocol.find_nearest(5, 5, "success", 10).await;

        assert!(nearest.is_some());
    }
}
