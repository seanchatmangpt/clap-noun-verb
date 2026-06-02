// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Test plan derivation from changed files
//!
//! Determines which tests are likely affected by a set of changed files.

use crate::adapters::git_diff::FileClassification;
use anyhow::Result;
use serde::Serialize;

/// Test plan generation
pub struct TestPlan;

#[derive(Debug, Clone, Serialize)]
pub struct TestPlanInfo {
    pub selected_tests: Vec<TestSelection>,
    pub is_conservative: bool,
    pub reason: String,
    pub estimated_runtime_seconds: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct TestSelection {
    pub test_name: String,
    pub test_type: String,
    pub affected_modules: Vec<String>,
}

impl TestPlan {
    /// Derive test plan from classified files
    pub fn derive(classifications: &[FileClassification]) -> Result<TestPlanInfo> {
        let mut selected_tests = vec![];
        let mut affected_modules = std::collections::HashSet::new();

        // Collect affected modules
        for classification in classifications {
            if !classification.module_path.is_empty() {
                affected_modules.insert(classification.module_path.clone());
            }
        }

        // Determine test selection strategy
        let (is_conservative, reason) = if classifications.iter().any(|c| c.is_lib) {
            (true, "Library-level changes detected; running comprehensive tests".to_string())
        } else if classifications.iter().any(|c| c.is_macro) {
            (
                true,
                "Macro changes detected; running all tests (macros affect compilation)".to_string(),
            )
        } else {
            (false, format!("Selected tests for {} affected modules", affected_modules.len()))
        };

        // Generate test selections
        if is_conservative {
            // Run full test suite for lib or macro changes
            selected_tests.push(TestSelection {
                test_name: "all".to_string(),
                test_type: "full".to_string(),
                affected_modules: affected_modules.iter().cloned().collect(),
            });
        } else {
            // Generate module-specific tests
            for module in affected_modules {
                selected_tests.push(TestSelection {
                    test_name: format!("test_{}", module.replace("::", "_")),
                    test_type: "module".to_string(),
                    affected_modules: vec![module],
                });
            }
        }

        let estimated_runtime = if is_conservative { 120 } else { 30 };

        Ok(TestPlanInfo {
            selected_tests,
            is_conservative,
            reason,
            estimated_runtime_seconds: estimated_runtime,
        })
    }
}
