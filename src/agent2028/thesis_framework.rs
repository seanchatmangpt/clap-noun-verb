/// Hyper-Thesis Framework (HTF) Integration
///
/// Formal ontology-based thesis planning and validation using RDF
/// Supports Λ-scheduling, Π-profiling, and Γ-checking
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Seven canonical shard families
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShardFamily {
    IMRaD,
    PaperBased,
    Argument,
    Contribution,
    Monograph,
    DSR,
    Narrative,
}

impl std::fmt::Display for ShardFamily {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShardFamily::IMRaD => write!(f, "IMRaD"),
            ShardFamily::PaperBased => write!(f, "PaperBased"),
            ShardFamily::Argument => write!(f, "Argument"),
            ShardFamily::Contribution => write!(f, "Contribution"),
            ShardFamily::Monograph => write!(f, "Monograph"),
            ShardFamily::DSR => write!(f, "DSR"),
            ShardFamily::Narrative => write!(f, "Narrative"),
        }
    }
}

/// Individual Δ-shard (chapter/section)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shard {
    pub id: String,
    pub name: String,
    pub family: ShardFamily,
    pub position: u32,
    pub purpose: String,
    pub status: ShardStatus,
    pub word_count: usize,
    pub word_count_target: usize,
    pub priority: u32, // 1-5, 1 is critical
    pub evidence_sources: Vec<String>,
    pub depends_on: Vec<String>, // Shard IDs this depends on
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShardStatus {
    NotStarted,
    InProgress,
    Draft,
    Review,
    Complete,
}

impl Shard {
    pub fn new(id: String, name: String, family: ShardFamily, purpose: String) -> Self {
        Self {
            id,
            name,
            family,
            position: 0,
            purpose,
            status: ShardStatus::NotStarted,
            word_count: 0,
            word_count_target: 5000,
            priority: 5,
            evidence_sources: Vec::new(),
            depends_on: Vec::new(),
        }
    }

    pub fn progress_percent(&self) -> f64 {
        if self.word_count_target == 0 {
            0.0
        } else {
            (self.word_count as f64 / self.word_count_target as f64) * 100.0
        }
    }

    pub fn is_complete(&self) -> bool {
        self.status == ShardStatus::Complete && self.word_count >= self.word_count_target
    }
}

/// Q-Invariants (properties that must hold everywhere)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invariant {
    pub name: String,
    pub description: String,
    pub scope: InvariantScope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvariantScope {
    Local,  // Within a shard
    Global, // Across entire thesis
}

/// Λ-Total Order: Schedule for chapter/section writing
pub struct LambdaSchedule {
    pub shards: Vec<Shard>,
    pub ordering: Vec<String>, // Shard IDs in order
}

impl LambdaSchedule {
    pub fn new() -> Self {
        Self { shards: Vec::new(), ordering: Vec::new() }
    }

    pub fn add_shard(&mut self, shard: Shard) {
        self.shards.push(shard);
    }

    /// Calculate optimal writing order based on dependencies and priorities
    pub fn compute_order(&mut self) -> Result<(), String> {
        // Topological sort with priority consideration
        let mut visited = HashSet::new();
        let mut order = Vec::new();

        for shard in &self.shards {
            if !visited.contains(&shard.id) {
                self.dfs_order(&shard.id, &mut visited, &mut order)?;
            }
        }

        self.ordering = order;
        Ok(())
    }

    fn dfs_order(
        &self,
        shard_id: &str,
        visited: &mut HashSet<String>,
        order: &mut Vec<String>,
    ) -> Result<(), String> {
        if visited.contains(shard_id) {
            return Ok(());
        }

        // Find shard
        let shard = self
            .shards
            .iter()
            .find(|s| s.id == shard_id)
            .ok_or_else(|| format!("Shard {} not found", shard_id))?;

        // Visit dependencies first
        for dep in &shard.depends_on {
            self.dfs_order(dep, visited, order)?;
        }

        visited.insert(shard_id.to_string());
        order.push(shard_id.to_string());

        Ok(())
    }

    /// Get recommended next shard to write
    pub fn recommend_next_shard(&self) -> Option<String> {
        for shard_id in &self.ordering {
            if let Some(shard) = self.shards.iter().find(|s| &s.id == shard_id) {
                if shard.status == ShardStatus::NotStarted {
                    return Some(shard_id.clone());
                }
            }
        }
        None
    }

    /// Get milestone deadlines
    pub fn milestone_dates(&self) -> Vec<(String, String)> {
        let total_shards = self.shards.len() as f64;

        let mut milestones = Vec::new();
        for (week, shard_id) in self.ordering.iter().enumerate() {
            let progress = ((week as f64) / total_shards) * 100.0;
            milestones
                .push((shard_id.clone(), format!("Week {}: {:.0}% done", week + 1, progress)));
        }

        milestones
    }
}

/// Π-Profile: Shows how Δ-shards fit together
pub struct PiProfile {
    pub central_claim: String,
    pub shard_contributions: HashMap<String, Vec<String>>, // Shard ID -> Claims it supports
    pub claim_support: HashMap<String, Vec<String>>,       // Claim -> Supporting shards
}

impl PiProfile {
    pub fn new(central_claim: String) -> Self {
        Self { central_claim, shard_contributions: HashMap::new(), claim_support: HashMap::new() }
    }

    pub fn add_shard_contribution(&mut self, shard_id: String, claims: Vec<String>) {
        self.shard_contributions.insert(shard_id, claims);
    }

    /// Analyze how well shards support central claim
    pub fn analyze_coverage(&self) -> CoverageAnalysis {
        let mut all_claims = HashSet::new();

        for claims in self.shard_contributions.values() {
            for claim in claims {
                all_claims.insert(claim.clone());
            }
        }

        let coverage_percent = if all_claims.is_empty() {
            0.0
        } else {
            (all_claims.len() as f64 / 10.0) * 100.0 // Assume ~10 major claims
        };

        CoverageAnalysis {
            total_unique_claims: all_claims.len(),
            shard_count: self.shard_contributions.len(),
            coverage_percent: coverage_percent.min(100.0),
            gaps: self.identify_gaps(),
        }
    }

    fn identify_gaps(&self) -> Vec<String> {
        // Simple heuristic: missing shards for key areas
        let expected_families = vec!["Contribution", "IMRaD", "Results", "Discussion"];
        let found_families: HashSet<_> = self
            .shard_contributions
            .keys()
            .map(|k| k.split('-').next().unwrap_or("").to_string())
            .collect();

        expected_families
            .iter()
            .filter(|f| !found_families.contains(&f.to_string()))
            .map(|s| format!("Missing {}", s))
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageAnalysis {
    pub total_unique_claims: usize,
    pub shard_count: usize,
    pub coverage_percent: f64,
    pub gaps: Vec<String>,
}

/// Γ-Checker: Ensures all Δ obey Q and avoid drift
pub struct GammaChecker {
    pub shards: Vec<Shard>,
    pub invariants: Vec<Invariant>,
    pub check_results: Vec<CheckResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub check_name: String,
    pub shard_id: Option<String>,
    pub passed: bool,
    pub message: String,
    pub severity: Severity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

impl GammaChecker {
    pub fn new(shards: Vec<Shard>) -> Self {
        Self {
            shards,
            invariants: vec![
                Invariant {
                    name: "Coherence".to_string(),
                    description: "All shards must align with central claim".to_string(),
                    scope: InvariantScope::Global,
                },
                Invariant {
                    name: "Completeness".to_string(),
                    description: "All required shards must be present".to_string(),
                    scope: InvariantScope::Global,
                },
                Invariant {
                    name: "Evidence".to_string(),
                    description: "All claims must have supporting evidence".to_string(),
                    scope: InvariantScope::Local,
                },
                Invariant {
                    name: "Logicality".to_string(),
                    description: "Argument chain must be logically sound".to_string(),
                    scope: InvariantScope::Global,
                },
                Invariant {
                    name: "Clarity".to_string(),
                    description: "All sections must be understandable".to_string(),
                    scope: InvariantScope::Local,
                },
            ],
            check_results: Vec::new(),
        }
    }

    pub fn run_all_checks(&mut self) -> CheckReport {
        self.check_results.clear();

        // Run each invariant check
        self.check_coherence();
        self.check_completeness();
        self.check_evidence();
        self.check_logicality();
        self.check_clarity();
        self.check_dependencies();

        self.generate_report()
    }

    fn check_coherence(&mut self) {
        for shard in &self.shards {
            if shard.evidence_sources.is_empty() && shard.priority < 3 {
                self.check_results.push(CheckResult {
                    check_name: "Coherence".to_string(),
                    shard_id: Some(shard.id.clone()),
                    passed: false,
                    message: format!("Shard {} has no evidence sources", shard.name),
                    severity: Severity::Warning,
                });
            }
        }
    }

    fn check_completeness(&mut self) {
        let required_families =
            vec![ShardFamily::Contribution, ShardFamily::Monograph, ShardFamily::IMRaD];

        for family in required_families {
            if !self.shards.iter().any(|s| s.family == family) {
                self.check_results.push(CheckResult {
                    check_name: "Completeness".to_string(),
                    shard_id: None,
                    passed: false,
                    message: format!("Missing required family: {:?}", family),
                    severity: Severity::Error,
                });
            }
        }
    }

    fn check_evidence(&mut self) {
        for shard in &self.shards {
            if shard.priority < 3 && shard.evidence_sources.len() < 3 {
                self.check_results.push(CheckResult {
                    check_name: "Evidence".to_string(),
                    shard_id: Some(shard.id.clone()),
                    passed: false,
                    message: format!(
                        "Shard {} needs more evidence (has {}, need 3+)",
                        shard.name,
                        shard.evidence_sources.len()
                    ),
                    severity: Severity::Warning,
                });
            }
        }
    }

    fn check_logicality(&mut self) {
        // Check if dependency chain is acyclic
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for shard in &self.shards {
            if !visited.contains(&shard.id) {
                if self.has_cycle(&shard.id, &mut visited, &mut rec_stack) {
                    self.check_results.push(CheckResult {
                        check_name: "Logicality".to_string(),
                        shard_id: Some(shard.id.clone()),
                        passed: false,
                        message: "Circular dependency detected".to_string(),
                        severity: Severity::Critical,
                    });
                }
            }
        }
    }

    fn has_cycle(
        &self,
        node: &str,
        visited: &mut HashSet<String>,
        rec: &mut HashSet<String>,
    ) -> bool {
        visited.insert(node.to_string());
        rec.insert(node.to_string());

        if let Some(shard) = self.shards.iter().find(|s| s.id == node) {
            for dep in &shard.depends_on {
                if !visited.contains(dep) {
                    if self.has_cycle(dep, visited, rec) {
                        return true;
                    }
                } else if rec.contains(dep) {
                    return true;
                }
            }
        }

        rec.remove(node);
        false
    }

    fn check_clarity(&mut self) {
        for shard in &self.shards {
            if shard.purpose.is_empty() {
                self.check_results.push(CheckResult {
                    check_name: "Clarity".to_string(),
                    shard_id: Some(shard.id.clone()),
                    passed: false,
                    message: "Shard purpose is undefined".to_string(),
                    severity: Severity::Warning,
                });
            }
        }
    }

    fn check_dependencies(&mut self) {
        for shard in &self.shards {
            for dep in &shard.depends_on {
                if !self.shards.iter().any(|s| s.id == *dep) {
                    self.check_results.push(CheckResult {
                        check_name: "Dependencies".to_string(),
                        shard_id: Some(shard.id.clone()),
                        passed: false,
                        message: format!("Depends on non-existent shard: {}", dep),
                        severity: Severity::Error,
                    });
                }
            }
        }
    }

    pub fn generate_report(&self) -> CheckReport {
        let total_checks = self.check_results.len();
        let passed = self.check_results.iter().filter(|r| r.passed).count();
        let critical =
            self.check_results.iter().filter(|r| r.severity == Severity::Critical).count();
        let errors = self.check_results.iter().filter(|r| r.severity == Severity::Error).count();
        let warnings =
            self.check_results.iter().filter(|r| r.severity == Severity::Warning).count();

        let health = if critical > 0 {
            "Critical"
        } else if errors > 0 {
            "Poor"
        } else if warnings > 3 {
            "Fair"
        } else {
            "Good"
        };

        CheckReport {
            total_checks,
            passed,
            critical,
            errors,
            warnings,
            health: health.to_string(),
            results: self.check_results.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckReport {
    pub total_checks: usize,
    pub passed: usize,
    pub critical: usize,
    pub errors: usize,
    pub warnings: usize,
    pub health: String,
    pub results: Vec<CheckResult>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shard_creation() {
        let shard = Shard::new(
            "intro-1".to_string(),
            "Introduction".to_string(),
            ShardFamily::IMRaD,
            "Establish context".to_string(),
        );

        assert_eq!(shard.name, "Introduction");
        assert_eq!(shard.status, ShardStatus::NotStarted);
    }

    #[test]
    fn test_lambda_schedule() {
        let mut schedule = LambdaSchedule::new();

        let mut shard1 = Shard::new(
            "problem".to_string(),
            "Problem".to_string(),
            ShardFamily::Contribution,
            "Define problem".to_string(),
        );

        let mut shard2 = Shard::new(
            "gap".to_string(),
            "Gap".to_string(),
            ShardFamily::Contribution,
            "Identify gap".to_string(),
        );
        shard2.depends_on = vec!["problem".to_string()];

        schedule.add_shard(shard1);
        schedule.add_shard(shard2);

        assert!(schedule.compute_order().is_ok());
        assert_eq!(schedule.ordering[0], "problem");
        assert_eq!(schedule.ordering[1], "gap");
    }

    #[test]
    fn test_gamma_checker() {
        let shards = vec![
            Shard::new(
                "contrib".to_string(),
                "Contribution".to_string(),
                ShardFamily::Contribution,
                "Main contribution".to_string(),
            ),
            Shard::new(
                "method".to_string(),
                "Method".to_string(),
                ShardFamily::IMRaD,
                "Methodology".to_string(),
            ),
        ];

        let mut checker = GammaChecker::new(shards);
        let report = checker.run_all_checks();

        assert!(report.health == "Good" || report.health == "Fair");
    }
}
