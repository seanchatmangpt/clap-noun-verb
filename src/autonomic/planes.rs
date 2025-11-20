//! O/Σ/Q/ΔΣ plane interaction metadata
//!
//! This module defines how commands interact with different conceptual planes:
//! - O (Observations): Runtime observations and monitoring
//! - Σ (Ontology): Schema, types, and structural definitions
//! - Q (Invariants): Guards, constraints, and quality requirements
//! - ΔΣ (Overlays): Proposed changes to ontology

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Conceptual planes in the autonomic system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Plane {
    /// Observations plane - runtime monitoring and telemetry
    #[serde(rename = "O")]
    Observations,
    /// Ontology plane - schema and type definitions
    #[serde(rename = "Σ")]
    Ontology,
    /// Invariants plane - constraints and guards
    #[serde(rename = "Q")]
    Invariants,
    /// Overlays plane - proposed ontology changes
    #[serde(rename = "ΔΣ")]
    Overlays,
}

/// Type of interaction with a plane
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InteractionType {
    /// Read from the plane
    Read,
    /// Write to the plane
    Write,
    /// Check against the plane
    Check,
    /// Emit or propose to the plane
    Emit,
    /// Propose changes to the plane
    Propose,
}

/// Plane interaction metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaneInteraction {
    /// Map of plane to interaction types
    pub interactions: HashMap<Plane, Vec<InteractionType>>,
}

impl Default for PlaneInteraction {
    fn default() -> Self {
        Self { interactions: HashMap::new() }
    }
}

impl PlaneInteraction {
    /// Create a new empty plane interaction
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an interaction with a plane
    pub fn add(mut self, plane: Plane, interaction: InteractionType) -> Self {
        self.interactions.entry(plane).or_default().push(interaction);
        self
    }

    /// Add O:read interaction (observation)
    pub fn observe_read(self) -> Self {
        self.add(Plane::Observations, InteractionType::Read)
    }

    /// Add O:write interaction (emit observations)
    pub fn observe_write(self) -> Self {
        self.add(Plane::Observations, InteractionType::Write)
    }

    /// Add Σ:read interaction (read ontology)
    pub fn ontology_read(self) -> Self {
        self.add(Plane::Ontology, InteractionType::Read)
    }

    /// Add Σ:propose interaction (propose ontology changes)
    pub fn ontology_propose(self) -> Self {
        self.add(Plane::Ontology, InteractionType::Propose)
    }

    /// Add Q:check interaction (check invariants)
    pub fn invariants_check(self) -> Self {
        self.add(Plane::Invariants, InteractionType::Check)
    }

    /// Add ΔΣ:emit interaction (emit overlay proposals)
    pub fn overlays_emit(self) -> Self {
        self.add(Plane::Overlays, InteractionType::Emit)
    }

    /// Get all planes this command interacts with
    pub fn planes(&self) -> Vec<Plane> {
        self.interactions.keys().copied().collect()
    }

    /// Get interactions for a specific plane
    pub fn get_plane_interactions(&self, plane: Plane) -> Vec<InteractionType> {
        self.interactions.get(&plane).cloned().unwrap_or_default()
    }

    /// Check if the command interacts with a specific plane
    pub fn interacts_with(&self, plane: Plane) -> bool {
        self.interactions.contains_key(&plane)
    }

    /// Parse from a string like "O_read,Σ_read,Q_check"
    pub fn from_str(s: &str) -> Self {
        let mut interaction = Self::new();

        for part in s.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            if let Some((plane_str, interaction_str)) = part.split_once('_') {
                let plane = match plane_str {
                    "O" => Plane::Observations,
                    "Σ" => Plane::Ontology,
                    "Q" => Plane::Invariants,
                    "ΔΣ" => Plane::Overlays,
                    _ => continue,
                };

                let interaction_type = match interaction_str {
                    "read" => InteractionType::Read,
                    "write" => InteractionType::Write,
                    "check" => InteractionType::Check,
                    "emit" => InteractionType::Emit,
                    "propose" => InteractionType::Propose,
                    _ => continue,
                };

                interaction = interaction.add(plane, interaction_type);
            }
        }

        interaction
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plane_interaction_builder() {
        let interaction = PlaneInteraction::new().observe_read().ontology_read().invariants_check();

        assert!(interaction.interacts_with(Plane::Observations));
        assert!(interaction.interacts_with(Plane::Ontology));
        assert!(interaction.interacts_with(Plane::Invariants));
        assert!(!interaction.interacts_with(Plane::Overlays));
    }

    #[test]
    fn test_from_str() {
        let interaction = PlaneInteraction::from_str("O_read,Σ_read,Q_check");

        assert!(interaction.interacts_with(Plane::Observations));
        assert!(interaction.interacts_with(Plane::Ontology));
        assert!(interaction.interacts_with(Plane::Invariants));
    }
}
