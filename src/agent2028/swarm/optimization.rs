/// Swarm Optimization Algorithms
///
/// Particle Swarm Optimization (PSO), Ant Colony Optimization (ACO),
/// and other swarm-based metaheuristics for solving optimization problems.
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// Solution quality score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Solution {
    pub position: Vec<f64>,
    pub fitness: f64,
    pub agent_id: String,
}

impl Solution {
    pub fn new(position: Vec<f64>, fitness: f64, agent_id: String) -> Self {
        Self { position, fitness, agent_id }
    }
}

/// Particle in PSO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Particle {
    pub position: Vec<f64>,
    pub velocity: Vec<f64>,
    pub best_position: Vec<f64>,
    pub best_fitness: f64,
}

impl Particle {
    pub fn new(position: Vec<f64>) -> Self {
        let velocity = vec![0.0; position.len()];
        Self {
            position: position.clone(),
            velocity,
            best_position: position,
            best_fitness: f64::NEG_INFINITY,
        }
    }

    /// Update fitness and best position
    pub fn evaluate(&mut self, fitness: f64) {
        if fitness > self.best_fitness {
            self.best_fitness = fitness;
            self.best_position = self.position.clone();
        }
    }
}

/// Particle Swarm Optimizer
pub struct ParticleSwarmOptimizer {
    particles: Vec<Particle>,
    global_best_position: Vec<f64>,
    global_best_fitness: f64,
    w: f64,  // Inertia weight
    c1: f64, // Cognitive parameter
    c2: f64, // Social parameter
}

impl ParticleSwarmOptimizer {
    pub fn new(swarm_size: usize, dimension: usize) -> Self {
        let mut particles = Vec::new();

        for _ in 0..swarm_size {
            let position = (0..dimension).map(|_| rand::random::<f64>() * 100.0).collect();
            particles.push(Particle::new(position));
        }

        let global_best_position = vec![0.0; dimension];

        Self {
            particles,
            global_best_position,
            global_best_fitness: f64::NEG_INFINITY,
            w: 0.7,  // Inertia
            c1: 1.5, // Cognitive
            c2: 1.5, // Social
        }
    }

    /// Update velocities and positions
    pub fn update(&mut self) {
        for particle in &mut self.particles {
            for i in 0..particle.position.len() {
                let r1 = rand::random::<f64>();
                let r2 = rand::random::<f64>();

                // Velocity update
                particle.velocity[i] = self.w * particle.velocity[i]
                    + self.c1 * r1 * (particle.best_position[i] - particle.position[i])
                    + self.c2 * r2 * (self.global_best_position[i] - particle.position[i]);

                // Position update
                particle.position[i] += particle.velocity[i];

                // Boundary conditions
                particle.position[i] = particle.position[i].max(0.0).min(100.0);
            }
        }
    }

    /// Evaluate fitness for all particles
    pub fn evaluate(&mut self, fitness_fn: &dyn Fn(&[f64]) -> f64) {
        for particle in &mut self.particles {
            let fitness = fitness_fn(&particle.position);
            particle.evaluate(fitness);

            if particle.best_fitness > self.global_best_fitness {
                self.global_best_fitness = particle.best_fitness;
                self.global_best_position = particle.best_position.clone();
            }
        }
    }

    /// Run optimization for N iterations
    pub fn optimize(&mut self, iterations: usize, fitness_fn: &dyn Fn(&[f64]) -> f64) {
        for _ in 0..iterations {
            self.evaluate(fitness_fn);
            self.update();
        }
        self.evaluate(fitness_fn);
    }

    /// Get best solution
    pub fn best_solution(&self) -> Solution {
        Solution::new(
            self.global_best_position.clone(),
            self.global_best_fitness,
            "pso".to_string(),
        )
    }
}

/// Ant for ACO
#[derive(Debug, Clone)]
pub struct Ant {
    pub path: Vec<usize>,
    pub fitness: f64,
}

impl Ant {
    pub fn new() -> Self {
        Self { path: Vec::new(), fitness: 0.0 }
    }
}

/// Ant Colony Optimizer
pub struct AntColonyOptimizer {
    ants: Vec<Ant>,
    pheromone: Vec<Vec<f64>>, // Pheromone matrix
    cities: usize,
}

impl AntColonyOptimizer {
    pub fn new(num_ants: usize, num_cities: usize) -> Self {
        let ants = (0..num_ants).map(|_| Ant::new()).collect();
        let pheromone = vec![vec![1.0; num_cities]; num_cities];

        Self { ants, pheromone, cities: num_cities }
    }

    /// Calculate distance (simplified Euclidean)
    fn calculate_distance(&self, from: usize, to: usize) -> f64 {
        ((from as f64 - to as f64).powi(2)).sqrt()
    }

    /// Build solutions
    pub fn build_solutions(&mut self) {
        let cities = self.cities;
        let pheromone = self.pheromone.clone();

        for ant in &mut self.ants {
            ant.path.clear();
            let mut current = rand::random::<usize>() % cities;
            ant.path.push(current);

            for _ in 1..cities {
                let mut next = 0;
                let mut max_prob = 0.0;

                for city in 0..cities {
                    if ant.path.contains(&city) {
                        continue;
                    }

                    let phe = pheromone[current][city];
                    let distance = ((current as f64 - city as f64).powi(2)).sqrt();
                    let prob = phe / (distance + 0.01);

                    if prob > max_prob {
                        max_prob = prob;
                        next = city;
                    }
                }

                ant.path.push(next);
                current = next;
            }

            // Evaluate path length
            let fitness_sum = ant
                .path
                .windows(2)
                .map(|w| ((w[0] as f64 - w[1] as f64).powi(2)).sqrt())
                .sum::<f64>();
            ant.fitness = 1.0 / (fitness_sum + 0.001);
        }
    }

    /// Update pheromone
    pub fn update_pheromone(&mut self, evaporation: f64) {
        // Evaporate
        for row in &mut self.pheromone {
            for cell in row {
                *cell *= 1.0 - evaporation;
            }
        }

        // Deposit
        for ant in &self.ants {
            for window in ant.path.windows(2) {
                self.pheromone[window[0]][window[1]] += ant.fitness;
                self.pheromone[window[1]][window[0]] += ant.fitness;
            }
        }
    }

    /// Run ACO for iterations
    pub fn optimize(&mut self, iterations: usize) {
        for _ in 0..iterations {
            self.build_solutions();
            self.update_pheromone(0.1);
        }
    }

    /// Get best solution
    pub fn best_solution(&self) -> Solution {
        let best_ant = self
            .ants
            .iter()
            .max_by(|a, b| a.fitness.partial_cmp(&b.fitness).unwrap_or(Ordering::Equal))
            .unwrap_or(&self.ants[0]);

        Solution::new(
            best_ant.path.iter().map(|&i| i as f64).collect(),
            best_ant.fitness,
            "aco".to_string(),
        )
    }
}

impl Default for Ant {
    fn default() -> Self {
        Self::new()
    }
}

/// Firefly algorithm for multimodal optimization
pub struct FireflyAlgorithm {
    fireflies: Vec<Solution>,
    iteration: usize,
}

impl FireflyAlgorithm {
    pub fn new(population_size: usize, dimension: usize) -> Self {
        let fireflies = (0..population_size)
            .map(|i| {
                let position = (0..dimension).map(|_| rand::random::<f64>() * 100.0).collect();
                Solution::new(position, f64::NEG_INFINITY, format!("firefly-{}", i))
            })
            .collect();

        Self { fireflies, iteration: 0 }
    }

    /// Attraction-based movement
    pub fn update(&mut self, fitness_fn: &dyn Fn(&[f64]) -> f64) {
        self.iteration += 1;
        let alpha = 0.01 * (1.0 - (self.iteration as f64 / 100.0)).max(0.0); // Cooling schedule

        for i in 0..self.fireflies.len() {
            let current_fitness = fitness_fn(&self.fireflies[i].position);
            self.fireflies[i].fitness = current_fitness;

            for j in 0..self.fireflies.len() {
                if i != j && self.fireflies[j].fitness > self.fireflies[i].fitness {
                    // Move towards brighter firefly
                    let beta = 1.0
                        / (1.0
                            + self.distance(
                                &self.fireflies[i].position,
                                &self.fireflies[j].position,
                            ));

                    for d in 0..self.fireflies[i].position.len() {
                        self.fireflies[i].position[d] += beta
                            * (self.fireflies[j].position[d] - self.fireflies[i].position[d])
                            + alpha * (rand::random::<f64>() - 0.5);
                        self.fireflies[i].position[d] =
                            self.fireflies[i].position[d].max(0.0).min(100.0);
                    }
                }
            }
        }
    }

    fn distance(&self, a: &[f64], b: &[f64]) -> f64 {
        a.iter().zip(b.iter()).map(|(x, y)| (x - y).powi(2)).sum::<f64>().sqrt()
    }

    /// Get best solution
    pub fn best_solution(&self) -> Solution {
        self.fireflies
            .iter()
            .max_by(|a, b| a.fitness.partial_cmp(&b.fitness).unwrap_or(Ordering::Equal))
            .cloned()
            .unwrap_or_else(|| self.fireflies[0].clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_particle_swarm_optimizer() {
        let mut optimizer = ParticleSwarmOptimizer::new(10, 2);

        // Simple sphere function
        let fitness = |x: &[f64]| -x.iter().map(|v| v * v).sum::<f64>();

        optimizer.optimize(50, &fitness);
        let solution = optimizer.best_solution();

        assert!(solution.fitness > f64::NEG_INFINITY);
    }

    #[test]
    fn test_ant_colony_optimizer() {
        let mut optimizer = AntColonyOptimizer::new(10, 5);
        optimizer.optimize(20);

        let solution = optimizer.best_solution();
        assert!(solution.fitness > 0.0);
    }

    #[test]
    fn test_firefly_algorithm() {
        let mut algo = FireflyAlgorithm::new(20, 2);
        let fitness = |x: &[f64]| -x.iter().map(|v| v * v).sum::<f64>();

        for _ in 0..50 {
            algo.update(&fitness);
        }

        let solution = algo.best_solution();
        assert!(solution.fitness > f64::NEG_INFINITY);
    }
}
