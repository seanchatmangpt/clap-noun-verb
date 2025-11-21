//! Domain Logic: Academic Paper Generation
//!
//! Pure functions for paper structure and content generation.
//! NO I/O, NO templates, NO file writing - just data structures and logic.

use serde::{Deserialize, Serialize};

/// Supported paper/thesis families
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(clippy::upper_case_acronyms)] // DSR is a well-known acronym (Design Science Research)
pub enum PaperFamily {
    IMRaD,
    Papers,
    Argument,
    Contribution,
    Monograph,
    DSR,
    Narrative,
}

impl PaperFamily {
    /// Parse a family name from string (case-insensitive)
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "imrad" => Some(Self::IMRaD),
            "papers" => Some(Self::Papers),
            "argument" => Some(Self::Argument),
            "contribution" => Some(Self::Contribution),
            "monograph" => Some(Self::Monograph),
            "dsr" => Some(Self::DSR),
            "narrative" => Some(Self::Narrative),
            _ => None,
        }
    }

    /// Get the display name
    pub fn name(&self) -> &'static str {
        match self {
            Self::IMRaD => "IMRaD",
            Self::Papers => "Papers",
            Self::Argument => "Argument",
            Self::Contribution => "Contribution",
            Self::Monograph => "Monograph",
            Self::DSR => "DSR",
            Self::Narrative => "Narrative",
        }
    }

    /// Get the description
    pub fn description(&self) -> &'static str {
        match self {
            Self::IMRaD => "Introduction, Method, Results, Discussion",
            Self::Papers => "Three papers + synthesis",
            Self::Argument => "Claims, grounds, proofs",
            Self::Contribution => "Gap, design, evaluation, impact",
            Self::Monograph => "Context, canon, method, analysis",
            Self::DSR => "Problem, artifact, evaluation, theory",
            Self::Narrative => "Field, voice, pattern, insight",
        }
    }

    /// Get all available families
    pub fn all() -> Vec<Self> {
        vec![
            Self::IMRaD,
            Self::Papers,
            Self::Argument,
            Self::Contribution,
            Self::Monograph,
            Self::DSR,
            Self::Narrative,
        ]
    }
}

/// A section within a paper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperSection {
    pub title: String,
    pub content: String,
}

impl PaperSection {
    pub fn new(title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            content: content.into(),
        }
    }
}

/// A complete paper structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paper {
    pub title: String,
    pub author: String,
    pub family: PaperFamily,
    pub abstract_text: String,
    pub sections: Vec<PaperSection>,
}

impl Paper {
    /// Create a new paper with default sections for the given family
    pub fn new(family: PaperFamily, title: Option<String>, author: Option<String>) -> Self {
        let title = title.unwrap_or_else(|| format!("Sample {} Paper", family.name()));
        let author = author.unwrap_or_else(|| "Playground CLI".to_string());
        let abstract_text = format!(
            "This {} paper demonstrates the clap-noun-verb framework for building semantic CLIs.",
            family.name()
        );
        let sections = Self::default_sections(&family);

        Self {
            title,
            author,
            family,
            abstract_text,
            sections,
        }
    }

    /// Generate default sections based on family type
    fn default_sections(family: &PaperFamily) -> Vec<PaperSection> {
        match family {
            PaperFamily::IMRaD => vec![
                PaperSection::new(
                    "Introduction",
                    "Background on semantic CLI frameworks for AI agents.\n\nThis research explores the integration of semantic ontologies with command-line interfaces.",
                ),
                PaperSection::new(
                    "Method",
                    "Implementation using clap-noun-verb builder API with RDF ontology.\n\nWe employ the Hyper-Thesis Framework (HTF) with Λ-scheduling.",
                ),
                PaperSection::new(
                    "Results",
                    "Successful integration of tera templates and oxigraph SPARQL queries.\n\nThe framework enables machine-grade CLI introspection.",
                ),
                PaperSection::new(
                    "Discussion",
                    "The framework enables machine-grade introspection for autonomous systems.\n\nFuture work includes enhanced semantic discovery.",
                ),
            ],
            PaperFamily::DSR => vec![
                PaperSection::new("Problem", "Identification of the research gap and motivation."),
                PaperSection::new("Artifact", "Design and implementation of the solution artifact."),
                PaperSection::new("Evaluation", "Validation of artifact effectiveness."),
                PaperSection::new("Theory", "Contribution to the knowledge base."),
            ],
            PaperFamily::Papers => vec![
                PaperSection::new("Paper 1", "First standalone paper contribution."),
                PaperSection::new("Paper 2", "Second standalone paper contribution."),
                PaperSection::new("Paper 3", "Third standalone paper contribution."),
                PaperSection::new("Synthesis", "Integration and synthesis of the three papers."),
            ],
            PaperFamily::Argument => vec![
                PaperSection::new("Claims", "Statement of the central argument and claims."),
                PaperSection::new("Grounds", "Evidence and grounds supporting the claims."),
                PaperSection::new("Proofs", "Logical proofs and validation of the argument."),
            ],
            PaperFamily::Contribution => vec![
                PaperSection::new("Gap", "Identification of the knowledge gap."),
                PaperSection::new("Design", "Design of the contribution."),
                PaperSection::new("Evaluation", "Evaluation of the contribution."),
                PaperSection::new("Impact", "Impact and implications of the contribution."),
            ],
            PaperFamily::Monograph => vec![
                PaperSection::new("Context", "Contextual background and setting."),
                PaperSection::new("Canon", "Review of canonical literature."),
                PaperSection::new("Method", "Methodological approach."),
                PaperSection::new("Analysis", "Comprehensive analysis and findings."),
            ],
            PaperFamily::Narrative => vec![
                PaperSection::new("Field", "Description of the research field."),
                PaperSection::new("Voice", "Researcher's voice and perspective."),
                PaperSection::new("Pattern", "Patterns identified in the data."),
                PaperSection::new("Insight", "Insights and interpretations."),
            ],
        }
    }
}

/// Validation result for a paper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub structure_valid: bool,
    pub citations_valid: bool,
    pub formatting_valid: bool,
    pub errors: Vec<String>,
}

impl ValidationResult {
    /// Validate a paper file path (domain logic only - checks extension and name)
    pub fn validate_path(path: &str) -> Self {
        let mut errors = Vec::new();
        let structure_valid = path.ends_with(".tex") || path.ends_with(".md");
        let citations_valid = true; // Would check BibTeX references
        let formatting_valid = !path.contains(' '); // No spaces in path

        if !structure_valid {
            errors.push("File must be .tex or .md format".to_string());
        }
        if !formatting_valid {
            errors.push("Path should not contain spaces".to_string());
        }

        Self {
            is_valid: structure_valid && citations_valid && formatting_valid,
            structure_valid,
            citations_valid,
            formatting_valid,
            errors,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paper_family_from_str() {
        assert_eq!(PaperFamily::from_str("imrad"), Some(PaperFamily::IMRaD));
        assert_eq!(PaperFamily::from_str("IMRAD"), Some(PaperFamily::IMRaD));
        assert_eq!(PaperFamily::from_str("dsr"), Some(PaperFamily::DSR));
        assert_eq!(PaperFamily::from_str("unknown"), None);
    }

    #[test]
    fn test_paper_family_all() {
        let families = PaperFamily::all();
        assert_eq!(families.len(), 7);
    }

    #[test]
    fn test_paper_new_creates_correct_sections() {
        let paper = Paper::new(PaperFamily::IMRaD, None, None);
        assert_eq!(paper.sections.len(), 4);
        assert_eq!(paper.sections[0].title, "Introduction");
        assert_eq!(paper.sections[1].title, "Method");
        assert_eq!(paper.sections[2].title, "Results");
        assert_eq!(paper.sections[3].title, "Discussion");
    }

    #[test]
    fn test_paper_new_with_custom_title() {
        let paper = Paper::new(
            PaperFamily::DSR,
            Some("My Custom Title".to_string()),
            Some("John Doe".to_string()),
        );
        assert_eq!(paper.title, "My Custom Title");
        assert_eq!(paper.author, "John Doe");
        assert_eq!(paper.sections.len(), 4);
    }

    #[test]
    fn test_validation_result_valid_path() {
        let result = ValidationResult::validate_path("thesis.tex");
        assert!(result.is_valid);
        assert!(result.structure_valid);
    }

    #[test]
    fn test_validation_result_invalid_extension() {
        let result = ValidationResult::validate_path("thesis.doc");
        assert!(!result.is_valid);
        assert!(!result.structure_valid);
    }
}
