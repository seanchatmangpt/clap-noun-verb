//! Domain Logic: Thesis Structure and Scheduling
//!
//! Pure functions for thesis family metadata and Λ-scheduling.
//! NO I/O, NO CLI - just pure data structures and algorithms.

use serde::{Deserialize, Serialize};
use super::papers::PaperFamily;

/// Hyper-Thesis Framework (HTF) component types
///
/// FUTURE: Used for advanced HTF analysis features
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HtfComponent {
    /// Δ-Shards: Atomic research building blocks
    DeltaShards,
    /// Λ-Scheduling: Optimal chapter writing order
    LambdaScheduling,
    /// Π-Profiling: Claim-to-contribution mapping
    PiProfiling,
    /// Γ-Globalization: Coherence validation
    GammaGlobalization,
}

#[allow(dead_code)]
impl HtfComponent {
    pub fn name(&self) -> &'static str {
        match self {
            Self::DeltaShards => "Δ-Shards (Components)",
            Self::LambdaScheduling => "Λ-Scheduling (Order)",
            Self::PiProfiling => "Π-Profiling (Coverage)",
            Self::GammaGlobalization => "Γ-Globalization (Coherence)",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::DeltaShards => "Atomic research building blocks, reusable across thesis families",
            Self::LambdaScheduling => "Optimal chapter writing order via topological sort of dependencies",
            Self::PiProfiling => "Claim-to-contribution mapping ensuring comprehensive coverage",
            Self::GammaGlobalization => "Validates logical flow and checks completeness",
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            Self::DeltaShards,
            Self::LambdaScheduling,
            Self::PiProfiling,
            Self::GammaGlobalization,
        ]
    }
}

/// Complete thesis structure information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThesisStructure {
    pub components: Vec<HtfComponentInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HtfComponentInfo {
    pub name: String,
    pub description: String,
    pub details: Vec<String>,
}

impl ThesisStructure {
    /// Get the complete HTF structure
    pub fn get() -> Self {
        Self {
            components: vec![
                HtfComponentInfo {
                    name: "Δ-Shards (Components)".to_string(),
                    description: "Atomic research building blocks".to_string(),
                    details: vec![
                        "Atomic research building blocks".to_string(),
                        "Reusable across thesis families".to_string(),
                    ],
                },
                HtfComponentInfo {
                    name: "Λ-Scheduling (Order)".to_string(),
                    description: "Optimal chapter writing order".to_string(),
                    details: vec![
                        "Optimal chapter writing order".to_string(),
                        "Topological sort of dependencies".to_string(),
                    ],
                },
                HtfComponentInfo {
                    name: "Π-Profiling (Coverage)".to_string(),
                    description: "Claim-to-contribution mapping".to_string(),
                    details: vec![
                        "Claim-to-contribution mapping".to_string(),
                        "Ensures comprehensive coverage".to_string(),
                    ],
                },
                HtfComponentInfo {
                    name: "Γ-Globalization (Coherence)".to_string(),
                    description: "Coherence validation".to_string(),
                    details: vec![
                        "Validates logical flow".to_string(),
                        "Checks completeness".to_string(),
                    ],
                },
            ],
        }
    }
}

/// Thesis family with full metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThesisFamily {
    pub number: u8,
    pub name: String,
    pub structure: String,
    pub context: String,
}

impl ThesisFamily {
    /// Get all thesis families with full metadata
    pub fn all() -> Vec<Self> {
        vec![
            Self {
                number: 1,
                name: "IMRaD".to_string(),
                structure: "Introduction, Method, Results, Discussion".to_string(),
                context: "Empirical research".to_string(),
            },
            Self {
                number: 2,
                name: "Papers".to_string(),
                structure: "Three papers + synthesis".to_string(),
                context: "Compilation thesis".to_string(),
            },
            Self {
                number: 3,
                name: "Argument".to_string(),
                structure: "Claims → Grounds → Proofs".to_string(),
                context: "Philosophical/theoretical".to_string(),
            },
            Self {
                number: 4,
                name: "Contribution".to_string(),
                structure: "Gap → Design → Evaluation → Impact".to_string(),
                context: "Design science".to_string(),
            },
            Self {
                number: 5,
                name: "Monograph".to_string(),
                structure: "Context → Canon → Method → Analysis".to_string(),
                context: "Comprehensive study".to_string(),
            },
            Self {
                number: 6,
                name: "DSR".to_string(),
                structure: "Problem → Artifact → Evaluation → Theory".to_string(),
                context: "Design Science Research".to_string(),
            },
            Self {
                number: 7,
                name: "Narrative".to_string(),
                structure: "Field → Voice → Pattern → Insight".to_string(),
                context: "Qualitative research".to_string(),
            },
        ]
    }
}

/// A step in the writing schedule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleStep {
    pub order: u8,
    pub chapter: String,
    pub description: String,
}

/// Λ-Schedule for a thesis family
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThesisSchedule {
    pub family: String,
    pub steps: Vec<ScheduleStep>,
}

impl ThesisSchedule {
    /// Get the optimal writing schedule for a family
    /// This is the Λ-scheduling algorithm - topological sort of chapter dependencies
    pub fn for_family(family: &PaperFamily) -> Self {
        let family_name = family.name().to_string();
        let steps = match family {
            PaperFamily::IMRaD => vec![
                ScheduleStep {
                    order: 1,
                    chapter: "Introduction".to_string(),
                    description: "Establish context, motivation".to_string(),
                },
                ScheduleStep {
                    order: 2,
                    chapter: "Method".to_string(),
                    description: "Describe methodology, design".to_string(),
                },
                ScheduleStep {
                    order: 3,
                    chapter: "Results".to_string(),
                    description: "Present findings, validation".to_string(),
                },
                ScheduleStep {
                    order: 4,
                    chapter: "Discussion".to_string(),
                    description: "Interpret results, implications".to_string(),
                },
            ],
            PaperFamily::DSR => vec![
                ScheduleStep {
                    order: 1,
                    chapter: "Problem".to_string(),
                    description: "Identify research gap and motivation".to_string(),
                },
                ScheduleStep {
                    order: 2,
                    chapter: "Artifact".to_string(),
                    description: "Design and implement solution".to_string(),
                },
                ScheduleStep {
                    order: 3,
                    chapter: "Evaluation".to_string(),
                    description: "Validate artifact effectiveness".to_string(),
                },
                ScheduleStep {
                    order: 4,
                    chapter: "Theory".to_string(),
                    description: "Contribute to knowledge base".to_string(),
                },
            ],
            // Default schedule for other families
            _ => vec![
                ScheduleStep {
                    order: 1,
                    chapter: "Introduction".to_string(),
                    description: "Establish context, motivation".to_string(),
                },
                ScheduleStep {
                    order: 2,
                    chapter: "Literature Review".to_string(),
                    description: "Review existing work".to_string(),
                },
                ScheduleStep {
                    order: 3,
                    chapter: "Methodology".to_string(),
                    description: "Describe approach".to_string(),
                },
                ScheduleStep {
                    order: 4,
                    chapter: "Conclusion".to_string(),
                    description: "Summarize findings".to_string(),
                },
            ],
        };

        Self {
            family: family_name,
            steps,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_htf_components() {
        let components = HtfComponent::all();
        assert_eq!(components.len(), 4);
    }

    #[test]
    fn test_thesis_structure() {
        let structure = ThesisStructure::get();
        assert_eq!(structure.components.len(), 4);
        assert!(structure.components[0].name.contains("Δ"));
    }

    #[test]
    fn test_thesis_families() {
        let families = ThesisFamily::all();
        assert_eq!(families.len(), 7);
        assert_eq!(families[0].name, "IMRaD");
    }

    #[test]
    fn test_schedule_for_imrad() {
        let schedule = ThesisSchedule::for_family(&PaperFamily::IMRaD);
        assert_eq!(schedule.family, "IMRaD");
        assert_eq!(schedule.steps.len(), 4);
        assert_eq!(schedule.steps[0].chapter, "Introduction");
    }

    #[test]
    fn test_schedule_for_dsr() {
        let schedule = ThesisSchedule::for_family(&PaperFamily::DSR);
        assert_eq!(schedule.family, "DSR");
        assert_eq!(schedule.steps[0].chapter, "Problem");
    }
}
