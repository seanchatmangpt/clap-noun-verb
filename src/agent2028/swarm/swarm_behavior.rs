/// Swarm Behavior Patterns
///
/// Bio-inspired algorithms for flocking, herding, swarming, and formation control.
/// Based on simple local rules (separation, alignment, cohesion).

use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

/// 2D vector for spatial calculations
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Vec2 {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn magnitude(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn normalize(&self) -> Vec2 {
        let mag = self.magnitude();
        if mag > 0.0 {
            Vec2::new(self.x / mag, self.y / mag)
        } else {
            Vec2::new(0.0, 0.0)
        }
    }

    pub fn scale(&self, factor: f64) -> Vec2 {
        Vec2::new(self.x * factor, self.y * factor)
    }

    pub fn add(&self, other: Vec2) -> Vec2 {
        Vec2::new(self.x + other.x, self.y + other.y)
    }

    pub fn distance_to(&self, other: Vec2) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

/// Single boid (bird-oid) agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoidAgent {
    pub id: String,
    pub position: Vec2,
    pub velocity: Vec2,
    pub acceleration: Vec2,
    pub max_speed: f64,
    pub max_force: f64,
}

impl BoidAgent {
    pub fn new(id: String, x: f64, y: f64) -> Self {
        Self {
            id,
            position: Vec2::new(x, y),
            velocity: Vec2::new((rand::random::<f64>() - 0.5) * 2.0, (rand::random::<f64>() - 0.5) * 2.0),
            acceleration: Vec2::new(0.0, 0.0),
            max_speed: 4.0,
            max_force: 0.1,
        }
    }

    /// Apply force to boid
    pub fn apply_force(&mut self, force: Vec2) {
        self.acceleration = self.acceleration.add(force);
    }

    /// Update boid position and velocity
    pub fn update(&mut self) {
        // Update velocity
        self.velocity = self.velocity.add(self.acceleration);

        // Limit speed
        if self.velocity.magnitude() > self.max_speed {
            self.velocity = self.velocity.normalize().scale(self.max_speed);
        }

        // Update position
        self.position = self.position.add(self.velocity);

        // Reset acceleration
        self.acceleration = Vec2::new(0.0, 0.0);
    }

    /// Wrap around screen edges
    pub fn wrap_edges(&mut self, width: f64, height: f64) {
        if self.position.x < 0.0 {
            self.position.x = width;
        }
        if self.position.x > width {
            self.position.x = 0.0;
        }
        if self.position.y < 0.0 {
            self.position.y = height;
        }
        if self.position.y > height {
            self.position.y = 0.0;
        }
    }
}

/// Flocking behavior rules
pub struct FlockingBehavior {
    pub separation_distance: f64,
    pub alignment_distance: f64,
    pub cohesion_distance: f64,
    pub separation_weight: f64,
    pub alignment_weight: f64,
    pub cohesion_weight: f64,
}

impl FlockingBehavior {
    pub fn new() -> Self {
        Self {
            separation_distance: 25.0,
            alignment_distance: 50.0,
            cohesion_distance: 50.0,
            separation_weight: 1.5,
            alignment_weight: 1.0,
            cohesion_weight: 1.0,
        }
    }

    /// Separation: steer to avoid crowding local flockmates
    pub fn separation(&self, boid: &BoidAgent, neighbors: &[BoidAgent]) -> Vec2 {
        let mut steer = Vec2::new(0.0, 0.0);
        let mut count = 0;

        for neighbor in neighbors {
            let distance = boid.position.distance_to(neighbor.position);

            if distance > 0.0 && distance < self.separation_distance {
                let diff = boid.position.add(neighbor.position.scale(-1.0));
                let normalized = diff.normalize().scale(1.0 / distance);
                steer = steer.add(normalized);
                count += 1;
            }
        }

        if count > 0 {
            steer = steer.scale(1.0 / count as f64);
        }

        steer.normalize().scale(boid.max_force).scale(self.separation_weight)
    }

    /// Alignment: steer towards average heading of local flockmates
    pub fn alignment(&self, boid: &BoidAgent, neighbors: &[BoidAgent]) -> Vec2 {
        let mut avg_velocity = Vec2::new(0.0, 0.0);
        let mut count = 0;

        for neighbor in neighbors {
            let distance = boid.position.distance_to(neighbor.position);

            if distance > 0.0 && distance < self.alignment_distance {
                avg_velocity = avg_velocity.add(neighbor.velocity);
                count += 1;
            }
        }

        if count > 0 {
            avg_velocity = avg_velocity.scale(1.0 / count as f64);
            avg_velocity.normalize().scale(boid.max_force).scale(self.alignment_weight)
        } else {
            Vec2::new(0.0, 0.0)
        }
    }

    /// Cohesion: steer to move toward average location of local flockmates
    pub fn cohesion(&self, boid: &BoidAgent, neighbors: &[BoidAgent]) -> Vec2 {
        let mut avg_position = Vec2::new(0.0, 0.0);
        let mut count = 0;

        for neighbor in neighbors {
            let distance = boid.position.distance_to(neighbor.position);

            if distance > 0.0 && distance < self.cohesion_distance {
                avg_position = avg_position.add(neighbor.position);
                count += 1;
            }
        }

        if count > 0 {
            avg_position = avg_position.scale(1.0 / count as f64);
            let direction = avg_position.add(boid.position.scale(-1.0));
            direction.normalize().scale(boid.max_force).scale(self.cohesion_weight)
        } else {
            Vec2::new(0.0, 0.0)
        }
    }

    /// Apply all three rules
    pub fn apply(&self, boid: &mut BoidAgent, neighbors: &[BoidAgent]) {
        let sep = self.separation(boid, neighbors);
        let align = self.alignment(boid, neighbors);
        let cohesion = self.cohesion(boid, neighbors);

        boid.apply_force(sep);
        boid.apply_force(align);
        boid.apply_force(cohesion);
    }
}

impl Default for FlockingBehavior {
    fn default() -> Self {
        Self::new()
    }
}

/// Herding behavior (leader-follower dynamics)
pub struct HerdingBehavior {
    pub leader_id: String,
    pub following_distance: f64,
    pub following_weight: f64,
}

impl HerdingBehavior {
    pub fn new(leader_id: String) -> Self {
        Self {
            leader_id,
            following_distance: 100.0,
            following_weight: 2.0,
        }
    }

    /// Follow the leader
    pub fn follow(&self, boid: &BoidAgent, leader: &BoidAgent) -> Vec2 {
        let distance = boid.position.distance_to(leader.position);

        if distance < self.following_distance {
            let direction = leader.position.add(boid.position.scale(-1.0));
            direction.normalize().scale(boid.max_force).scale(self.following_weight)
        } else {
            Vec2::new(0.0, 0.0)
        }
    }
}

/// Swarming behavior (aggressive clustering)
pub struct SwarmingBehavior {
    pub target: Vec2,
    pub attraction_weight: f64,
    pub max_cluster_distance: f64,
}

impl SwarmingBehavior {
    pub fn new(target: Vec2) -> Self {
        Self {
            target,
            attraction_weight: 3.0,
            max_cluster_distance: 200.0,
        }
    }

    /// Swarm toward target
    pub fn swarm(&self, boid: &BoidAgent) -> Vec2 {
        let distance = boid.position.distance_to(self.target);

        if distance < self.max_cluster_distance {
            let direction = self.target.add(boid.position.scale(-1.0));
            direction.normalize().scale(boid.max_force).scale(self.attraction_weight)
        } else {
            Vec2::new(0.0, 0.0)
        }
    }
}

/// Formation control
pub enum Formation {
    Line,       // V-formation or line
    Circle,     // Circular formation
    Grid,       // Grid pattern
    Wedge,      // Wedge/arrow formation
}

pub struct FormationController {
    pub formation: Formation,
    pub spacing: f64,
}

impl FormationController {
    pub fn new(formation: Formation) -> Self {
        Self {
            formation,
            spacing: 50.0,
        }
    }

    /// Get desired position for agent index in formation
    pub fn get_formation_position(&self, index: usize, anchor: Vec2) -> Vec2 {
        match self.formation {
            Formation::Line => {
                // Horizontal line
                Vec2::new(anchor.x + (index as f64 * self.spacing), anchor.y)
            }
            Formation::Circle => {
                // Circle around anchor
                let angle = (index as f64) * 2.0 * PI / 8.0; // Assume 8 agents
                let radius = self.spacing;
                Vec2::new(
                    anchor.x + radius * angle.cos(),
                    anchor.y + radius * angle.sin(),
                )
            }
            Formation::Grid => {
                // Grid pattern
                let cols = 4;
                let row = index / cols;
                let col = index % cols;
                Vec2::new(
                    anchor.x + (col as f64 * self.spacing),
                    anchor.y + (row as f64 * self.spacing),
                )
            }
            Formation::Wedge => {
                // Arrow/wedge formation
                let row = index / 2;
                let col = index % 2;
                Vec2::new(
                    anchor.x + (col as f64 - 0.5) * self.spacing,
                    anchor.y + row as f64 * self.spacing,
                )
            }
        }
    }

    /// Generate force to maintain formation
    pub fn maintain_formation(&self, boid: &BoidAgent, desired_position: Vec2) -> Vec2 {
        let direction = desired_position.add(boid.position.scale(-1.0));
        direction.normalize().scale(boid.max_force)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boid_creation() {
        let boid = BoidAgent::new("boid-1".to_string(), 10.0, 20.0);
        assert_eq!(boid.position.x, 10.0);
        assert_eq!(boid.position.y, 20.0);
    }

    #[test]
    fn test_flocking_behavior() {
        let behavior = FlockingBehavior::new();
        assert!(behavior.separation_distance > 0.0);
    }

    #[test]
    fn test_formation_controller() {
        let controller = FormationController::new(Formation::Line);
        let pos = controller.get_formation_position(0, Vec2::new(0.0, 0.0));
        assert_eq!(pos.x, 0.0);
    }
}
