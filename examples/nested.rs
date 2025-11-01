//! Multiple nouns example demonstrating compound commands
//!
//! Note: v3 attribute macro API doesn't support nested nouns.
//! This example shows multiple top-level nouns instead.

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

// Business Logic Layer (Pure Functions - Reusable)
fn run_tests() -> TestResult {
    TestResult { tests_run: 42, passed: 40, failed: 2 }
}

fn watch_tests() -> WatchResult {
    WatchResult { watching: true, message: "Watching for test changes".to_string() }
}

fn generate_coverage() -> CoverageResult {
    CoverageResult { coverage: 85.5, message: "Coverage report generated".to_string() }
}

fn check_lint() -> LintResult {
    LintResult { issues: 3, message: "Found 3 linting issues".to_string() }
}

fn fix_lint() -> LintResult {
    LintResult { issues: 0, message: "All linting issues fixed".to_string() }
}

fn check_format() -> FormatResult {
    FormatResult { needs_formatting: false, message: "Code is properly formatted".to_string() }
}

fn apply_format() -> FormatResult {
    FormatResult { needs_formatting: false, message: "Formatting applied".to_string() }
}

#[derive(Serialize, Debug)]
struct TestResult {
    tests_run: usize,
    passed: usize,
    failed: usize,
}

#[derive(Serialize, Debug)]
struct WatchResult {
    watching: bool,
    message: String,
}

#[derive(Serialize, Debug)]
struct CoverageResult {
    coverage: f64,
    message: String,
}

#[derive(Serialize, Debug)]
struct LintResult {
    issues: usize,
    message: String,
}

#[derive(Serialize, Debug)]
struct FormatResult {
    needs_formatting: bool,
    message: String,
}

#[derive(Serialize, Debug)]
struct AIOrchestrateResult {
    tests_run: usize,
    optimized: bool,
    message: String,
}

fn run_ai_tests() -> AIOrchestrateResult {
    AIOrchestrateResult {
        tests_run: 42,
        optimized: true,
        message: "AI-orchestrated tests completed".to_string(),
    }
}

fn predict_failures() -> AIOrchestrateResult {
    AIOrchestrateResult {
        tests_run: 0,
        optimized: false,
        message: "Predicted 2 potential failures".to_string(),
    }
}

fn optimize_tests() -> AIOrchestrateResult {
    AIOrchestrateResult {
        tests_run: 0,
        optimized: true,
        message: "Test execution optimized by 15%".to_string(),
    }
}

#[derive(Serialize, Debug)]
struct AnalysisResult {
    bottlenecks: Vec<String>,
    score: f64,
}

fn analyze_performance() -> AnalysisResult {
    AnalysisResult { bottlenecks: vec!["database".to_string(), "cache".to_string()], score: 7.5 }
}

fn analyze_quality() -> AnalysisResult {
    AnalysisResult {
        bottlenecks: vec!["code coverage".to_string(), "documentation".to_string()],
        score: 8.2,
    }
}

#[derive(Serialize, Debug)]
struct MonitorResult {
    status: String,
    active: bool,
}

fn start_monitoring() -> MonitorResult {
    MonitorResult { status: "Active".to_string(), active: true }
}

fn get_monitor_status() -> MonitorResult {
    MonitorResult { status: "Active".to_string(), active: true }
}

// CLI Layer (Input Validation + Output Shaping Only)
// Using multiple top-level nouns instead of nesting

// Note: This file has multiple nouns, so we need explicit nouns in #[verb] attributes
// For single-noun files, we can remove #[noun] and auto-infer from filename

/// Run tests
#[verb("run", "test")] // Explicit noun since filename is "nested.rs"
fn run_tests_cmd() -> Result<TestResult> {
    Ok(run_tests())
}

/// Watch for changes and rerun tests
#[verb("watch", "test")] // Explicit noun since filename is "nested.rs"
fn watch_tests_cmd() -> Result<WatchResult> {
    Ok(watch_tests())
}

/// Generate test coverage report
#[verb("coverage", "test")] // Explicit noun since filename is "nested.rs"
fn generate_coverage_cmd() -> Result<CoverageResult> {
    Ok(generate_coverage())
}

/// Check code style
#[verb("check", "lint")] // Explicit noun since filename is "nested.rs"
fn check_lint_cmd() -> Result<LintResult> {
    Ok(check_lint())
}

/// Auto-fix linting issues
#[verb("fix", "lint")] // Explicit noun since filename is "nested.rs"
fn fix_lint_cmd() -> Result<LintResult> {
    Ok(fix_lint())
}

/// Check formatting
#[verb("check", "format")] // Explicit noun since filename is "nested.rs"
fn check_format_cmd() -> Result<FormatResult> {
    Ok(check_format())
}

/// Apply formatting
#[verb("apply", "format")] // Explicit noun since filename is "nested.rs"
fn apply_format_cmd() -> Result<FormatResult> {
    Ok(apply_format())
}

/// Run AI-orchestrated tests
#[verb("run", "orchestrate")] // Explicit noun since filename is "nested.rs"
fn run_ai_tests_cmd() -> Result<AIOrchestrateResult> {
    Ok(run_ai_tests())
}

/// Predict test failures
#[verb("predict", "orchestrate")] // Explicit noun since filename is "nested.rs"
fn predict_failures_cmd() -> Result<AIOrchestrateResult> {
    Ok(predict_failures())
}

/// Optimize test execution
#[verb("optimize", "orchestrate")] // Explicit noun since filename is "nested.rs"
fn optimize_tests_cmd() -> Result<AIOrchestrateResult> {
    Ok(optimize_tests())
}

/// Analyze performance bottlenecks
#[verb("performance", "analyze")] // Custom verb name, explicit noun since filename is "nested.rs"
fn analyze_performance_cmd() -> Result<AnalysisResult> {
    Ok(analyze_performance())
}

/// Analyze code quality
#[verb("quality", "analyze")] // Explicit noun since filename is "nested.rs"
fn analyze_quality_cmd() -> Result<AnalysisResult> {
    Ok(analyze_quality())
}

/// Start AI monitoring
#[verb("start", "monitor")] // Explicit noun since filename is "nested.rs"
fn start_monitoring_cmd() -> Result<MonitorResult> {
    Ok(start_monitoring())
}

/// Check monitoring status
#[verb("status", "monitor")] // Explicit noun since filename is "nested.rs"
fn get_monitor_status_cmd() -> Result<MonitorResult> {
    Ok(get_monitor_status())
}

fn main() -> Result<()> {
    // Auto-discover all registered commands and run
    clap_noun_verb::run()
}
