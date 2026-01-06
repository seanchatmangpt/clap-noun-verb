#![allow(dead_code)]
// FUTURE: These types are part of the frontier feature set and will be integrated in future phases

//! Learning Trajectory Macros for clap-noun-verb-macros-frontier
//!
//! This module provides procedural macros for defining competency dimensions,
//! assessment criteria, and optimal learning paths with Byzantine consensus validation.
//!
//! # Features
//!
//! - Multi-dimensional competency tracking with type-safe skill levels
//! - Assessment engine with proficiency evaluation
//! - Path optimizer for generating minimal learning sequences
//! - Byzantine consensus validator (33% fault tolerance)
//! - Adaptive difficulty controller based on performance metrics
//!
//! # Competency Levels
//!
//! - Foundation: CLI basics and fundamental concepts
//! - Intermediate: Advanced patterns and best practices
//! - Advanced: Frontier features and optimization techniques
//! - Expert: Framework design and architectural mastery
//!
//! # Example
//!
//! ```rust,ignore
//! use clap_noun_verb_macros::learning_trajectories::{competency, assessment, learning_path};
//!
//! #[competency(dimension = "CLI Development")]
//! struct CliSkills {
//!     parsing: ProficiencyLevel,
//!     validation: ProficiencyLevel,
//!     composition: ProficiencyLevel,
//! }
//!
//! #[assessment(threshold = 0.8)]
//! fn evaluate_cli_proficiency(learner: &Learner) -> AssessmentResult {
//!     // Assessment logic
//!     AssessmentResult::new(0.85, "Proficient in CLI development")
//! }
//!
//! #[learning_path(target = "Expert")]
//! fn generate_path(current: CompetencyLevel) -> LearningPath {
//!     // Path generation logic
//!     LearningPath::new(vec![
//!         Step::new("Foundation", "Learn CLI basics"),
//!         Step::new("Intermediate", "Master advanced patterns"),
//!         Step::new("Advanced", "Implement frontier features"),
//!         Step::new("Expert", "Design frameworks"),
//!     ])
//! }
//! ```

use proc_macro2::TokenStream;
use quote::quote;
use std::collections::HashMap;
use syn::{parse::Parser, DeriveInput, ItemFn};

// ============================================================================
// Type Definitions - Type-First Thinking
// ============================================================================

/// Competency level enumeration
///
/// Represents the four levels of mastery in the learning trajectory.
/// Each level has specific prerequisites and measurable outcomes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CompetencyLevel {
    /// Foundation: CLI basics (0-25% mastery)
    Foundation,
    /// Intermediate: Advanced patterns (25-50% mastery)
    Intermediate,
    /// Advanced: Frontier features (50-75% mastery)
    Advanced,
    /// Expert: Framework design (75-100% mastery)
    Expert,
}

impl CompetencyLevel {
    /// Parse level from string
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "foundation" => Ok(CompetencyLevel::Foundation),
            "intermediate" => Ok(CompetencyLevel::Intermediate),
            "advanced" => Ok(CompetencyLevel::Advanced),
            "expert" => Ok(CompetencyLevel::Expert),
            other => Err(format!(
                "Invalid competency level '{}'. Expected: foundation, intermediate, advanced, expert",
                other
            )),
        }
    }

    /// Get the type name for this level
    pub fn type_name(&self) -> &'static str {
        match self {
            CompetencyLevel::Foundation => "FoundationLevel",
            CompetencyLevel::Intermediate => "IntermediateLevel",
            CompetencyLevel::Advanced => "AdvancedLevel",
            CompetencyLevel::Expert => "ExpertLevel",
        }
    }

    /// Get mastery range for this level
    #[allow(dead_code)] // Part of public API for macro-generated code
    pub fn mastery_range(&self) -> (f64, f64) {
        match self {
            CompetencyLevel::Foundation => (0.0, 0.25),
            CompetencyLevel::Intermediate => (0.25, 0.5),
            CompetencyLevel::Advanced => (0.5, 0.75),
            CompetencyLevel::Expert => (0.75, 1.0),
        }
    }

    /// Get next level in progression
    #[allow(dead_code)] // Part of public API for macro-generated code
    pub fn next(&self) -> Option<Self> {
        match self {
            CompetencyLevel::Foundation => Some(CompetencyLevel::Intermediate),
            CompetencyLevel::Intermediate => Some(CompetencyLevel::Advanced),
            CompetencyLevel::Advanced => Some(CompetencyLevel::Expert),
            CompetencyLevel::Expert => None,
        }
    }

    /// Get previous level in progression
    #[allow(dead_code)] // Part of public API for macro-generated code
    pub fn previous(&self) -> Option<Self> {
        match self {
            CompetencyLevel::Foundation => None,
            CompetencyLevel::Intermediate => Some(CompetencyLevel::Foundation),
            CompetencyLevel::Advanced => Some(CompetencyLevel::Intermediate),
            CompetencyLevel::Expert => Some(CompetencyLevel::Advanced),
        }
    }

    /// Check if learner can progress to this level
    #[allow(dead_code)] // Part of public API for macro-generated code
    pub fn can_progress_from(&self, current: &Self) -> bool {
        match self.previous() {
            Some(prev) => current >= &prev,
            None => true,
        }
    }
}

/// Proficiency level for individual skills (0.0 to 1.0)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[allow(dead_code)]
// FUTURE: Integrate learning trajectory optimization
pub struct ProficiencyLevel(f64);

impl ProficiencyLevel {
    /// Create new proficiency level (clamped to 0.0-1.0)
    pub fn new(value: f64) -> Self {
        ProficiencyLevel(value.clamp(0.0, 1.0))
    }

    /// Get raw value
    pub fn value(&self) -> f64 {
        self.0
    }

    /// Convert to competency level
    pub fn to_competency_level(self) -> CompetencyLevel {
        if self.0 < 0.25 {
            CompetencyLevel::Foundation
        } else if self.0 < 0.5 {
            CompetencyLevel::Intermediate
        } else if self.0 < 0.75 {
            CompetencyLevel::Advanced
        } else {
            CompetencyLevel::Expert
        }
    }
}

// ============================================================================
// Component 1: CompetencyDimension - Define Measurable Skills
// ============================================================================

/// Competency dimension trait
///
/// Defines a measurable skill dimension with proficiency tracking.
#[allow(dead_code)]
// FUTURE: Integrate learning trajectory optimization
pub trait CompetencyDimension {
    /// Get dimension name
    fn name(&self) -> &str;

    /// Get current proficiency levels for all skills
    fn proficiencies(&self) -> HashMap<String, ProficiencyLevel>;

    /// Calculate aggregate proficiency
    fn aggregate_proficiency(&self) -> ProficiencyLevel {
        let profs = self.proficiencies();
        if profs.is_empty() {
            return ProficiencyLevel::new(0.0);
        }
        let sum: f64 = profs.values().map(|p| p.value()).sum();
        ProficiencyLevel::new(sum / profs.len() as f64)
    }

    /// Get competency level based on aggregate proficiency
    fn competency_level(&self) -> CompetencyLevel {
        self.aggregate_proficiency().to_competency_level()
    }
}

/// Generate CompetencyDimension implementation
pub fn generate_competency_impl(input: &DeriveInput, dimension: &str) -> TokenStream {
    let struct_name = &input.ident;

    // Extract struct fields
    let fields = match &input.data {
        syn::Data::Struct(data) => &data.fields,
        _ => {
            return syn::Error::new_spanned(
                struct_name,
                "competency can only be applied to structs",
            )
            .to_compile_error()
        }
    };

    // Generate proficiency field accessors
    let field_names: Vec<_> = match fields {
        syn::Fields::Named(named) => named.named.iter().filter_map(|f| f.ident.as_ref()).collect(),
        _ => {
            return syn::Error::new_spanned(struct_name, "competency requires named fields")
                .to_compile_error()
        }
    };

    let proficiency_map = field_names.iter().map(|name| {
        let name_str = name.to_string();
        quote! {
            map.insert(#name_str.to_string(), self.#name);
        }
    });

    quote! {
        impl ::clap_noun_verb_macros::macros::learning_trajectories::CompetencyDimension for #struct_name {
            fn name(&self) -> &str {
                #dimension
            }

            fn proficiencies(&self) -> ::std::collections::HashMap<String, ::clap_noun_verb_macros::macros::learning_trajectories::ProficiencyLevel> {
                let mut map = ::std::collections::HashMap::new();
                #(#proficiency_map)*
                map
            }
        }
    }
}

// ============================================================================
// Component 2: AssessmentEngine - Evaluate Proficiency
// ============================================================================

/// Assessment result with score and feedback
#[derive(Debug, Clone)]
#[allow(dead_code)]
// FUTURE: Integrate learning trajectory optimization
pub struct AssessmentResult {
    pub score: f64,
    pub feedback: String,
    pub passed: bool,
}

#[allow(dead_code)]
impl AssessmentResult {
    /// Create new assessment result
    pub fn new(score: f64, feedback: impl Into<String>) -> Self {
        AssessmentResult {
            score: score.clamp(0.0, 1.0),
            feedback: feedback.into(),
            passed: score >= 0.8, // Default threshold
        }
    }

    /// Create result with custom threshold
    pub fn with_threshold(score: f64, threshold: f64, feedback: impl Into<String>) -> Self {
        AssessmentResult {
            score: score.clamp(0.0, 1.0),
            feedback: feedback.into(),
            passed: score >= threshold,
        }
    }
}

/// Assessment engine trait
#[allow(dead_code)]
// FUTURE: Integrate learning trajectory optimization
pub trait AssessmentEngine {
    /// Evaluate learner proficiency
    fn evaluate(&self) -> AssessmentResult;

    /// Get assessment threshold
    fn threshold(&self) -> f64 {
        0.8 // Default 80% threshold
    }
}

/// Generate AssessmentEngine implementation
pub fn generate_assessment_impl(input: &ItemFn, threshold: f64) -> TokenStream {
    let fn_name = &input.sig.ident;

    quote! {
        impl ::clap_noun_verb_macros::macros::learning_trajectories::AssessmentEngine for #fn_name {
            fn evaluate(&self) -> ::clap_noun_verb_macros::macros::learning_trajectories::AssessmentResult {
                self()
            }

            fn threshold(&self) -> f64 {
                #threshold
            }
        }
    }
}

// ============================================================================
// Component 3: PathOptimizer - Generate Optimal Learning Sequence
// ============================================================================

/// Learning step in a path
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
// FUTURE: Integrate learning trajectory optimization
pub struct LearningStep {
    pub level: CompetencyLevel,
    pub description: String,
    pub prerequisites: Vec<CompetencyLevel>,
}

#[allow(dead_code)]
impl LearningStep {
    /// Create new learning step
    pub fn new(level: CompetencyLevel, description: impl Into<String>) -> Self {
        let prerequisites = match level {
            CompetencyLevel::Foundation => vec![],
            CompetencyLevel::Intermediate => vec![CompetencyLevel::Foundation],
            CompetencyLevel::Advanced => vec![CompetencyLevel::Intermediate],
            CompetencyLevel::Expert => vec![CompetencyLevel::Advanced],
        };

        LearningStep { level, description: description.into(), prerequisites }
    }

    /// Check if prerequisites are satisfied
    pub fn prerequisites_satisfied(&self, completed: &[CompetencyLevel]) -> bool {
        self.prerequisites.iter().all(|prereq| completed.contains(prereq))
    }
}

/// Learning path with ordered steps
#[derive(Debug, Clone)]
#[allow(dead_code)]
// FUTURE: Integrate learning trajectory optimization
pub struct LearningPath {
    pub steps: Vec<LearningStep>,
    pub target_level: CompetencyLevel,
}

#[allow(dead_code)]
impl LearningPath {
    /// Create new learning path
    pub fn new(steps: Vec<LearningStep>, target_level: CompetencyLevel) -> Self {
        LearningPath { steps, target_level }
    }

    /// Validate path is properly ordered
    pub fn validate(&self) -> Result<(), String> {
        let mut completed = Vec::new();

        for step in &self.steps {
            if !step.prerequisites_satisfied(&completed) {
                return Err(format!(
                    "Step {:?} prerequisites not satisfied. Required: {:?}, Completed: {:?}",
                    step.level, step.prerequisites, completed
                ));
            }
            completed.push(step.level);
        }

        // Verify target level is reached
        if !completed.contains(&self.target_level) {
            return Err(format!(
                "Path does not reach target level {:?}. Reached: {:?}",
                self.target_level, completed
            ));
        }

        Ok(())
    }

    /// Get next step based on current level
    pub fn next_step(&self, current: CompetencyLevel) -> Option<&LearningStep> {
        self.steps.iter().find(|step| step.level > current)
    }
}

/// Path optimizer trait
#[allow(dead_code)]
// FUTURE: Integrate learning trajectory optimization
pub trait PathOptimizer {
    /// Generate optimal learning path
    fn generate_path(&self, current: CompetencyLevel, target: CompetencyLevel) -> LearningPath;

    /// Calculate minimum steps required
    fn min_steps(&self, current: CompetencyLevel, target: CompetencyLevel) -> usize {
        if current >= target {
            return 0;
        }

        let mut steps = 0;
        let mut level = current;

        while level < target {
            if let Some(next) = level.next() {
                level = next;
                steps += 1;
            } else {
                break;
            }
        }

        steps
    }
}

/// Generate PathOptimizer implementation
pub fn generate_path_impl(input: &ItemFn, target: CompetencyLevel) -> TokenStream {
    let fn_name = &input.sig.ident;
    let target_name = target.type_name();

    quote! {
        impl ::clap_noun_verb_macros::macros::learning_trajectories::PathOptimizer for #fn_name {
            fn generate_path(
                &self,
                current: ::clap_noun_verb_macros::macros::learning_trajectories::CompetencyLevel,
                target: ::clap_noun_verb_macros::macros::learning_trajectories::CompetencyLevel,
            ) -> ::clap_noun_verb_macros::macros::learning_trajectories::LearningPath {
                self(current, target)
            }
        }

        const _TARGET_LEVEL: &str = #target_name;
    }
}

// ============================================================================
// Component 4: ConsensusValidator - Byzantine Fault Tolerance
// ============================================================================

/// Assessment vote from a validator
#[derive(Debug, Clone)]
#[allow(dead_code)]
// FUTURE: Integrate learning trajectory optimization
pub struct AssessmentVote {
    pub validator_id: String,
    pub score: f64,
    pub timestamp: u64,
}

#[allow(dead_code)]
impl AssessmentVote {
    /// Create new vote
    pub fn new(validator_id: impl Into<String>, score: f64, timestamp: u64) -> Self {
        AssessmentVote {
            validator_id: validator_id.into(),
            score: score.clamp(0.0, 1.0),
            timestamp,
        }
    }
}

/// Consensus result with Byzantine fault tolerance
#[derive(Debug, Clone)]
#[allow(dead_code)]
// FUTURE: Integrate learning trajectory optimization
pub struct ConsensusResult {
    pub consensus_score: f64,
    pub valid_votes: usize,
    pub total_votes: usize,
    pub passed: bool,
}

#[allow(dead_code)]
impl ConsensusResult {
    /// Create new consensus result
    pub fn new(
        consensus_score: f64,
        valid_votes: usize,
        total_votes: usize,
        threshold: f64,
    ) -> Self {
        ConsensusResult {
            consensus_score: consensus_score.clamp(0.0, 1.0),
            valid_votes,
            total_votes,
            passed: consensus_score >= threshold && valid_votes >= (total_votes * 2 / 3),
        }
    }
}

/// Consensus validator with Byzantine fault tolerance
///
/// Tolerates up to 33% (f = n/3) Byzantine (malicious/faulty) validators.
/// Requires at least 2f+1 honest validators for correct consensus.
#[allow(dead_code)]
// FUTURE: Integrate learning trajectory optimization
pub struct ConsensusValidator {
    fault_tolerance: f64,
}

#[allow(dead_code)]
impl ConsensusValidator {
    /// Create new consensus validator (default 33% fault tolerance)
    pub fn new() -> Self {
        ConsensusValidator { fault_tolerance: 0.33 }
    }

    /// Create validator with custom fault tolerance
    pub fn with_fault_tolerance(tolerance: f64) -> Self {
        ConsensusValidator { fault_tolerance: tolerance.clamp(0.0, 0.5) }
    }

    /// Validate consensus from votes
    ///
    /// Algorithm:
    /// 1. Remove outliers (scores > 2 std deviations from median)
    /// 2. Calculate median of remaining votes
    /// 3. Require at least 2f+1 votes for consensus
    pub fn validate(&self, votes: &[AssessmentVote], threshold: f64) -> ConsensusResult {
        if votes.is_empty() {
            return ConsensusResult::new(0.0, 0, 0, threshold);
        }

        let total_votes = votes.len();
        let min_honest =
            ((total_votes as f64 * (1.0 - self.fault_tolerance)).ceil() as usize).max(1);

        // Sort scores for median calculation
        let mut scores: Vec<f64> = votes.iter().map(|v| v.score).collect();
        scores.sort_by(|a, b| a.partial_cmp(b).unwrap());

        // Calculate median
        let median = if scores.len() % 2 == 0 {
            (scores[scores.len() / 2 - 1] + scores[scores.len() / 2]) / 2.0
        } else {
            scores[scores.len() / 2]
        };

        // Calculate standard deviation
        let mean: f64 = scores.iter().sum::<f64>() / scores.len() as f64;
        let variance: f64 =
            scores.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / scores.len() as f64;
        let std_dev = variance.sqrt();

        // Filter outliers (more than 2 std deviations from median)
        let valid_scores: Vec<f64> =
            scores.iter().copied().filter(|s| (s - median).abs() <= 2.0 * std_dev).collect();

        let valid_votes = valid_scores.len();

        // Use median of valid scores for consensus
        let consensus_score = if !valid_scores.is_empty() {
            let mid = valid_scores.len() / 2;
            if valid_scores.len() % 2 == 0 {
                (valid_scores[mid - 1] + valid_scores[mid]) / 2.0
            } else {
                valid_scores[mid]
            }
        } else {
            median
        };

        // Check if we have enough valid votes
        let has_consensus = valid_votes >= min_honest;

        ConsensusResult::new(
            if has_consensus { consensus_score } else { 0.0 },
            valid_votes,
            total_votes,
            threshold,
        )
    }
}

impl Default for ConsensusValidator {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Component 5: AdaptivityController - Adjust Difficulty Dynamically
// ============================================================================

/// Performance metrics for adaptivity
#[derive(Debug, Clone)]
#[allow(dead_code)]
// FUTURE: Integrate learning trajectory optimization
pub struct PerformanceMetrics {
    pub success_rate: f64,
    pub avg_completion_time: f64,
    pub attempts: usize,
}

#[allow(dead_code)]
impl PerformanceMetrics {
    /// Create new performance metrics
    pub fn new(success_rate: f64, avg_completion_time: f64, attempts: usize) -> Self {
        PerformanceMetrics {
            success_rate: success_rate.clamp(0.0, 1.0),
            avg_completion_time,
            attempts,
        }
    }
}

/// Difficulty adjustment
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
// FUTURE: Integrate learning trajectory optimization
pub enum DifficultyAdjustment {
    Decrease(f64), // Decrease by percentage
    Maintain,
    Increase(f64), // Increase by percentage
}

/// Adaptivity controller for performance-based difficulty scaling
#[allow(dead_code)]
// FUTURE: Integrate learning trajectory optimization
pub struct AdaptivityController {
    target_success_rate: f64,
    adjustment_factor: f64,
}

#[allow(dead_code)]
impl AdaptivityController {
    /// Create new adaptivity controller (target 70% success rate)
    pub fn new() -> Self {
        AdaptivityController { target_success_rate: 0.7, adjustment_factor: 0.1 }
    }

    /// Create controller with custom parameters
    pub fn with_params(target_success_rate: f64, adjustment_factor: f64) -> Self {
        AdaptivityController {
            target_success_rate: target_success_rate.clamp(0.5, 0.9),
            adjustment_factor: adjustment_factor.clamp(0.05, 0.3),
        }
    }

    /// Calculate difficulty adjustment based on performance
    pub fn adjust_difficulty(&self, metrics: &PerformanceMetrics) -> DifficultyAdjustment {
        // Need minimum attempts for reliable adjustment
        if metrics.attempts < 3 {
            return DifficultyAdjustment::Maintain;
        }

        let deviation = metrics.success_rate - self.target_success_rate;

        // If success rate is too high, increase difficulty
        if deviation > 0.1 {
            DifficultyAdjustment::Increase(self.adjustment_factor)
        }
        // If success rate is too low, decrease difficulty
        else if deviation < -0.1 {
            DifficultyAdjustment::Decrease(self.adjustment_factor)
        }
        // Within acceptable range, maintain difficulty
        else {
            DifficultyAdjustment::Maintain
        }
    }
}

impl Default for AdaptivityController {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Attribute Parsing Utilities
// ============================================================================

/// Parse competency attribute arguments
pub fn parse_competency_args(args: TokenStream) -> Result<String, syn::Error> {
    let parser = syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated;
    let args = parser.parse2(args)?;

    for meta in args {
        if let syn::Meta::NameValue(nv) = meta {
            if nv.path.is_ident("dimension") {
                if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) = &nv.value {
                    return Ok(s.value());
                }
            }
        }
    }

    Err(syn::Error::new(proc_macro2::Span::call_site(), "Expected dimension = \"name\" argument"))
}

/// Parse assessment attribute arguments
pub fn parse_assessment_args(args: TokenStream) -> Result<f64, syn::Error> {
    let parser = syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated;
    let args = parser.parse2(args)?;

    for meta in args {
        if let syn::Meta::NameValue(nv) = meta {
            if nv.path.is_ident("threshold") {
                if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Float(f), .. }) = &nv.value {
                    return f.base10_parse();
                }
            }
        }
    }

    // Default threshold
    Ok(0.8)
}

/// Parse learning_path attribute arguments
pub fn parse_learning_path_args(args: TokenStream) -> Result<CompetencyLevel, syn::Error> {
    let parser = syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated;
    let args = parser.parse2(args)?;

    for meta in args {
        if let syn::Meta::NameValue(nv) = meta {
            if nv.path.is_ident("target") {
                if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) = &nv.value {
                    return CompetencyLevel::from_str(&s.value())
                        .map_err(|e| syn::Error::new_spanned(&nv.value, e));
                }
            }
        }
    }

    Err(syn::Error::new(
        proc_macro2::Span::call_site(),
        "Expected target = \"level\" argument (foundation, intermediate, advanced, or expert)",
    ))
}

// ============================================================================
// Chicago TDD Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // CompetencyLevel Tests
    // ========================================================================

    #[test]
    fn test_competency_level_ordering() {
        // Arrange
        let foundation = CompetencyLevel::Foundation;
        let intermediate = CompetencyLevel::Intermediate;
        let advanced = CompetencyLevel::Advanced;
        let expert = CompetencyLevel::Expert;

        // Act & Assert - verify ordering
        assert!(foundation < intermediate);
        assert!(intermediate < advanced);
        assert!(advanced < expert);
    }

    #[test]
    fn test_competency_level_from_str() {
        // Arrange & Act
        let foundation = CompetencyLevel::from_str("foundation");
        let intermediate = CompetencyLevel::from_str("Intermediate");
        let advanced = CompetencyLevel::from_str("ADVANCED");
        let expert = CompetencyLevel::from_str("Expert");

        // Assert
        assert_eq!(foundation, Ok(CompetencyLevel::Foundation));
        assert_eq!(intermediate, Ok(CompetencyLevel::Intermediate));
        assert_eq!(advanced, Ok(CompetencyLevel::Advanced));
        assert_eq!(expert, Ok(CompetencyLevel::Expert));
    }

    #[test]
    fn test_competency_level_from_str_invalid() {
        // Arrange & Act
        let result = CompetencyLevel::from_str("invalid");

        // Assert
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid competency level"));
    }

    #[test]
    fn test_competency_level_next() {
        // Arrange
        let foundation = CompetencyLevel::Foundation;
        let expert = CompetencyLevel::Expert;

        // Act
        let next_foundation = foundation.next();
        let next_expert = expert.next();

        // Assert
        assert_eq!(next_foundation, Some(CompetencyLevel::Intermediate));
        assert_eq!(next_expert, None);
    }

    #[test]
    fn test_competency_level_previous() {
        // Arrange
        let foundation = CompetencyLevel::Foundation;
        let expert = CompetencyLevel::Expert;

        // Act
        let prev_foundation = foundation.previous();
        let prev_expert = expert.previous();

        // Assert
        assert_eq!(prev_foundation, None);
        assert_eq!(prev_expert, Some(CompetencyLevel::Advanced));
    }

    #[test]
    fn test_competency_level_can_progress() {
        // Arrange
        let foundation = CompetencyLevel::Foundation;
        let intermediate = CompetencyLevel::Intermediate;
        let advanced = CompetencyLevel::Advanced;

        // Act & Assert
        assert!(intermediate.can_progress_from(&foundation));
        assert!(advanced.can_progress_from(&intermediate));
        assert!(!advanced.can_progress_from(&foundation));
    }

    // ========================================================================
    // ProficiencyLevel Tests
    // ========================================================================

    #[test]
    fn test_proficiency_level_clamping() {
        // Arrange & Act
        let below_zero = ProficiencyLevel::new(-0.5);
        let above_one = ProficiencyLevel::new(1.5);
        let valid = ProficiencyLevel::new(0.7);

        // Assert
        assert_eq!(below_zero.value(), 0.0);
        assert_eq!(above_one.value(), 1.0);
        assert_eq!(valid.value(), 0.7);
    }

    #[test]
    fn test_proficiency_to_competency_level() {
        // Arrange
        let foundation_prof = ProficiencyLevel::new(0.2);
        let intermediate_prof = ProficiencyLevel::new(0.4);
        let advanced_prof = ProficiencyLevel::new(0.6);
        let expert_prof = ProficiencyLevel::new(0.9);

        // Act
        let foundation = foundation_prof.to_competency_level();
        let intermediate = intermediate_prof.to_competency_level();
        let advanced = advanced_prof.to_competency_level();
        let expert = expert_prof.to_competency_level();

        // Assert
        assert_eq!(foundation, CompetencyLevel::Foundation);
        assert_eq!(intermediate, CompetencyLevel::Intermediate);
        assert_eq!(advanced, CompetencyLevel::Advanced);
        assert_eq!(expert, CompetencyLevel::Expert);
    }

    // ========================================================================
    // LearningPath Tests
    // ========================================================================

    #[test]
    fn test_learning_path_validation_success() {
        // Arrange
        let steps = vec![
            LearningStep::new(CompetencyLevel::Foundation, "Learn basics"),
            LearningStep::new(CompetencyLevel::Intermediate, "Master patterns"),
            LearningStep::new(CompetencyLevel::Advanced, "Implement features"),
        ];
        let path = LearningPath::new(steps, CompetencyLevel::Advanced);

        // Act
        let result = path.validate();

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_learning_path_validation_missing_prerequisite() {
        // Arrange - skip Foundation level
        let steps = vec![
            LearningStep::new(CompetencyLevel::Intermediate, "Master patterns"),
            LearningStep::new(CompetencyLevel::Advanced, "Implement features"),
        ];
        let path = LearningPath::new(steps, CompetencyLevel::Advanced);

        // Act
        let result = path.validate();

        // Assert
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("prerequisites not satisfied"));
    }

    #[test]
    fn test_learning_path_next_step() {
        // Arrange
        let steps = vec![
            LearningStep::new(CompetencyLevel::Foundation, "Learn basics"),
            LearningStep::new(CompetencyLevel::Intermediate, "Master patterns"),
            LearningStep::new(CompetencyLevel::Advanced, "Implement features"),
        ];
        let path = LearningPath::new(steps, CompetencyLevel::Advanced);

        // Act
        let next = path.next_step(CompetencyLevel::Foundation);

        // Assert
        assert!(next.is_some());
        assert_eq!(next.unwrap().level, CompetencyLevel::Intermediate);
    }

    // ========================================================================
    // ConsensusValidator Tests - Byzantine Fault Tolerance
    // ========================================================================

    #[test]
    fn test_consensus_validator_honest_majority() {
        // Arrange - 5 validators, all honest, scores around 0.8
        let validator = ConsensusValidator::new();
        let votes = vec![
            AssessmentVote::new("v1", 0.85, 1000),
            AssessmentVote::new("v2", 0.82, 1001),
            AssessmentVote::new("v3", 0.78, 1002),
            AssessmentVote::new("v4", 0.80, 1003),
            AssessmentVote::new("v5", 0.83, 1004),
        ];

        // Act
        let result = validator.validate(&votes, 0.8);

        // Assert
        assert!(result.passed);
        assert!(result.consensus_score >= 0.78);
        assert!(result.consensus_score <= 0.85);
        assert!(result.valid_votes >= 4); // At least 2f+1
    }

    #[test]
    fn test_consensus_validator_with_byzantine_outlier() {
        // Arrange - 7 validators, 1 Byzantine (outlier score)
        let validator = ConsensusValidator::new();
        let votes = vec![
            AssessmentVote::new("v1", 0.85, 1000),
            AssessmentVote::new("v2", 0.82, 1001),
            AssessmentVote::new("v3", 0.78, 1002),
            AssessmentVote::new("v4", 0.80, 1003),
            AssessmentVote::new("v5", 0.83, 1004),
            AssessmentVote::new("v6", 0.81, 1005),
            AssessmentVote::new("byzantine", 0.1, 1006), // Malicious low score
        ];

        // Act
        let result = validator.validate(&votes, 0.8);

        // Assert - Byzantine vote should be filtered out
        assert!(result.passed);
        assert!(result.consensus_score >= 0.78);
        assert_eq!(result.valid_votes, 6); // Byzantine vote filtered
    }

    #[test]
    fn test_consensus_validator_insufficient_votes() {
        // Arrange - Only 2 votes, not enough for 33% fault tolerance
        let validator = ConsensusValidator::new();
        let votes =
            vec![AssessmentVote::new("v1", 0.85, 1000), AssessmentVote::new("v2", 0.82, 1001)];

        // Act
        let result = validator.validate(&votes, 0.8);

        // Assert - Should still pass with 2 honest votes
        assert!(result.passed);
        assert_eq!(result.valid_votes, 2);
    }

    // ========================================================================
    // AdaptivityController Tests
    // ========================================================================

    #[test]
    fn test_adaptivity_increase_difficulty() {
        // Arrange - High success rate (90%), should increase difficulty
        let controller = AdaptivityController::new();
        let metrics = PerformanceMetrics::new(0.9, 50.0, 10);

        // Act
        let adjustment = controller.adjust_difficulty(&metrics);

        // Assert
        assert!(matches!(adjustment, DifficultyAdjustment::Increase(_)));
    }

    #[test]
    fn test_adaptivity_decrease_difficulty() {
        // Arrange - Low success rate (50%), should decrease difficulty
        let controller = AdaptivityController::new();
        let metrics = PerformanceMetrics::new(0.5, 100.0, 10);

        // Act
        let adjustment = controller.adjust_difficulty(&metrics);

        // Assert
        assert!(matches!(adjustment, DifficultyAdjustment::Decrease(_)));
    }

    #[test]
    fn test_adaptivity_maintain_difficulty() {
        // Arrange - Success rate near target (70%), should maintain
        let controller = AdaptivityController::new();
        let metrics = PerformanceMetrics::new(0.72, 75.0, 10);

        // Act
        let adjustment = controller.adjust_difficulty(&metrics);

        // Assert
        assert_eq!(adjustment, DifficultyAdjustment::Maintain);
    }

    #[test]
    fn test_adaptivity_insufficient_attempts() {
        // Arrange - Only 2 attempts, not enough data
        let controller = AdaptivityController::new();
        let metrics = PerformanceMetrics::new(0.5, 50.0, 2);

        // Act
        let adjustment = controller.adjust_difficulty(&metrics);

        // Assert - Should maintain with insufficient data
        assert_eq!(adjustment, DifficultyAdjustment::Maintain);
    }

    // ========================================================================
    // AssessmentResult Tests
    // ========================================================================

    #[test]
    fn test_assessment_result_default_threshold() {
        // Arrange & Act
        let passing = AssessmentResult::new(0.85, "Great work");
        let failing = AssessmentResult::new(0.75, "Needs improvement");

        // Assert
        assert!(passing.passed);
        assert!(!failing.passed);
        assert_eq!(passing.score, 0.85);
        assert_eq!(failing.score, 0.75);
    }

    #[test]
    fn test_assessment_result_custom_threshold() {
        // Arrange & Act
        let result = AssessmentResult::with_threshold(0.75, 0.7, "Passed");

        // Assert
        assert!(result.passed);
        assert_eq!(result.score, 0.75);
    }

    #[test]
    fn test_assessment_result_score_clamping() {
        // Arrange & Act
        let below = AssessmentResult::new(-0.5, "Invalid");
        let above = AssessmentResult::new(1.5, "Invalid");

        // Assert
        assert_eq!(below.score, 0.0);
        assert_eq!(above.score, 1.0);
    }
}
