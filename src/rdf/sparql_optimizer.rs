use std::collections::HashMap;
use crate::rdf::sparql_parser::{ParsedQuery, TriplePattern, FilterExpression, Aggregation, PatternElement};

/// Query execution plan
#[derive(Debug, Clone)]
pub struct QueryPlan {
    pub steps: Vec<ExecutionStep>,
    pub cost_estimate: u64,
}

/// Execution step in query plan
#[derive(Debug, Clone)]
pub enum ExecutionStep {
    ScanTriples {
        pattern: TriplePattern,
        estimated_rows: usize,
    },
    Filter {
        expression: FilterExpression,
    },
    Join {
        left_var: String,
        right_var: String,
        method: JoinMethod,
    },
    Optional {
        patterns: Vec<TriplePattern>,
    },
    Union {
        branches: Vec<Vec<TriplePattern>>,
    },
    Aggregate {
        aggregations: Vec<Aggregation>,
        group_by: Vec<String>,
    },
}

/// Join method selection
#[derive(Debug, Clone)]
pub enum JoinMethod {
    HashJoin { hash_on: String },
    NestedLoop,
}

/// Table statistics for cost estimation
#[derive(Debug, Clone)]
pub struct TableStats {
    pub triple_count: usize,
    pub predicate_counts: HashMap<String, usize>,
    pub object_counts: HashMap<String, usize>,
    pub subject_counts: HashMap<String, usize>,
}

impl Default for TableStats {
    fn default() -> Self {
        Self {
            triple_count: 0,
            predicate_counts: HashMap::new(),
            object_counts: HashMap::new(),
            subject_counts: HashMap::new(),
        }
    }
}

/// Cost-based query optimizer
pub struct QueryOptimizer;

impl QueryOptimizer {
    /// Optimize parsed query into execution plan
    pub fn optimize(query: ParsedQuery, stats: &TableStats) -> QueryPlan {
        let mut steps = Vec::new();
        let mut cost = 0u64;

        // Reorder triple patterns by estimated cardinality (smallest first)
        let mut ordered_patterns = query.where_patterns.clone();
        Self::reorder_patterns(&mut ordered_patterns, stats);

        // Add scan steps for each pattern
        for pattern in ordered_patterns {
            let estimated_rows = Self::estimate_cardinality(&pattern, stats);
            cost += estimated_rows as u64;
            steps.push(ExecutionStep::ScanTriples {
                pattern,
                estimated_rows,
            });
        }

        // Add join steps for connecting patterns
        let join_vars = Self::find_join_variables(&query.where_patterns);
        for (left_var, right_var) in join_vars {
            let method = Self::select_join_method(&left_var, &right_var, stats);
            cost += 1000; // Base join cost
            steps.push(ExecutionStep::Join {
                left_var,
                right_var,
                method,
            });
        }

        // Add filter steps (push down filters early for selectivity)
        for filter in query.filters {
            cost += 100; // Filter cost
            steps.push(ExecutionStep::Filter { expression: filter });
        }

        // Add optional steps
        if !query.optional.is_empty() {
            cost += 500; // Optional cost
            steps.push(ExecutionStep::Optional {
                patterns: query.optional,
            });
        }

        // Add union steps
        if !query.unions.is_empty() {
            cost += 800; // Union cost
            steps.push(ExecutionStep::Union {
                branches: query.unions,
            });
        }

        // Add aggregation steps
        if !query.aggregations.is_empty() {
            cost += 200; // Aggregation cost
            steps.push(ExecutionStep::Aggregate {
                aggregations: query.aggregations,
                group_by: query.group_by,
            });
        }

        QueryPlan {
            steps,
            cost_estimate: cost,
        }
    }

    /// Estimate cardinality (number of results) for a triple pattern
    fn estimate_cardinality(pattern: &TriplePattern, stats: &TableStats) -> usize {
        if stats.triple_count == 0 {
            return 1000; // Default estimate
        }

        match pattern {
            TriplePattern::Simple { subject, predicate, object } => {
                let mut selectivity = 1.0;

                // Subject selectivity
                match subject {
                    PatternElement::Constant(s) => {
                        if let Some(&count) = stats.subject_counts.get(s) {
                            selectivity *= count as f64 / stats.triple_count as f64;
                        } else {
                            selectivity *= 0.001; // Very selective if not in stats
                        }
                    }
                    PatternElement::Variable(_) => {
                        selectivity *= 1.0; // No filtering
                    }
                }

                // Predicate selectivity
                match predicate {
                    PatternElement::Constant(p) => {
                        if let Some(&count) = stats.predicate_counts.get(p) {
                            selectivity *= count as f64 / stats.triple_count as f64;
                        } else {
                            selectivity *= 0.01;
                        }
                    }
                    PatternElement::Variable(_) => {
                        selectivity *= 1.0;
                    }
                }

                // Object selectivity
                match object {
                    PatternElement::Constant(o) => {
                        if let Some(&count) = stats.object_counts.get(o) {
                            selectivity *= count as f64 / stats.triple_count as f64;
                        } else {
                            selectivity *= 0.01;
                        }
                    }
                    PatternElement::Variable(_) => {
                        selectivity *= 1.0;
                    }
                }

                (stats.triple_count as f64 * selectivity).max(1.0) as usize
            }
            TriplePattern::PropertyPath { .. } => {
                // Property paths can expand significantly
                stats.triple_count / 10
            }
        }
    }

    /// Reorder patterns by estimated cardinality (smallest first)
    fn reorder_patterns(patterns: &mut [TriplePattern], stats: &TableStats) {
        patterns.sort_by_key(|p| Self::estimate_cardinality(p, stats));
    }

    /// Find variables that connect patterns for joins
    fn find_join_variables(patterns: &[TriplePattern]) -> Vec<(String, String)> {
        let mut joins = Vec::new();
        let mut var_patterns: HashMap<String, Vec<usize>> = HashMap::new();

        // Build variable -> pattern index mapping
        for (idx, pattern) in patterns.iter().enumerate() {
            let vars = Self::extract_variables(pattern);
            for var in vars {
                var_patterns.entry(var).or_default().push(idx);
            }
        }

        // Find variables appearing in multiple patterns
        for (var, pattern_indices) in var_patterns {
            if pattern_indices.len() > 1 {
                for i in 0..pattern_indices.len() - 1 {
                    joins.push((var.clone(), var.clone()));
                }
            }
        }

        joins
    }

    /// Extract variables from a triple pattern
    fn extract_variables(pattern: &TriplePattern) -> Vec<String> {
        let mut vars = Vec::new();

        match pattern {
            TriplePattern::Simple { subject, predicate, object } => {
                if let PatternElement::Variable(v) = subject {
                    vars.push(v.clone());
                }
                if let PatternElement::Variable(v) = predicate {
                    vars.push(v.clone());
                }
                if let PatternElement::Variable(v) = object {
                    vars.push(v.clone());
                }
            }
            TriplePattern::PropertyPath { subject, object, .. } => {
                if let PatternElement::Variable(v) = subject {
                    vars.push(v.clone());
                }
                if let PatternElement::Variable(v) = object {
                    vars.push(v.clone());
                }
            }
        }

        vars
    }

    /// Select join method based on statistics
    fn select_join_method(_left_var: &str, right_var: &str, stats: &TableStats) -> JoinMethod {
        // Use hash join for larger datasets
        if stats.triple_count > 100 {
            JoinMethod::HashJoin {
                hash_on: right_var.to_string(),
            }
        } else {
            JoinMethod::NestedLoop
        }
    }
}

impl TableStats {
    /// Create statistics from triple data
    pub fn from_triples<'a>(triples: impl Iterator<Item = (&'a str, &'a str, &'a str)>) -> Self {
        let mut stats = TableStats::default();

        for (subject, predicate, object) in triples {
            stats.triple_count += 1;
            *stats.subject_counts.entry(subject.to_string()).or_insert(0) += 1;
            *stats.predicate_counts.entry(predicate.to_string()).or_insert(0) += 1;
            *stats.object_counts.entry(object.to_string()).or_insert(0) += 1;
        }

        stats
    }

    /// Update statistics incrementally
    pub fn update(&mut self, subject: &str, predicate: &str, object: &str) {
        self.triple_count += 1;
        *self.subject_counts.entry(subject.to_string()).or_insert(0) += 1;
        *self.predicate_counts.entry(predicate.to_string()).or_insert(0) += 1;
        *self.object_counts.entry(object.to_string()).or_insert(0) += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rdf::sparql_parser::SparqlParser;

    #[test]
    fn test_estimate_cardinality_constant_predicate() {
        let mut stats = TableStats::default();
        stats.triple_count = 1000;
        stats.predicate_counts.insert("a".to_string(), 100);

        let pattern = TriplePattern::Simple {
            subject: PatternElement::Variable("?x".to_string()),
            predicate: PatternElement::Constant("a".to_string()),
            object: PatternElement::Variable("?y".to_string()),
        };

        let cardinality = QueryOptimizer::estimate_cardinality(&pattern, &stats);
        assert_eq!(cardinality, 100);
    }

    #[test]
    fn test_reorder_patterns_by_selectivity() {
        let mut stats = TableStats::default();
        stats.triple_count = 1000;
        stats.predicate_counts.insert("a".to_string(), 10);
        stats.predicate_counts.insert("b".to_string(), 500);

        let mut patterns = vec![
            TriplePattern::Simple {
                subject: PatternElement::Variable("?x".to_string()),
                predicate: PatternElement::Constant("b".to_string()),
                object: PatternElement::Variable("?y".to_string()),
            },
            TriplePattern::Simple {
                subject: PatternElement::Variable("?x".to_string()),
                predicate: PatternElement::Constant("a".to_string()),
                object: PatternElement::Variable("?z".to_string()),
            },
        ];

        QueryOptimizer::reorder_patterns(&mut patterns, &stats);

        // Most selective pattern (a) should be first
        match &patterns[0] {
            TriplePattern::Simple { predicate, .. } => {
                match predicate {
                    PatternElement::Constant(p) => assert_eq!(p, "a"),
                    _ => panic!("Expected constant predicate"),
                }
            }
            _ => panic!("Expected Simple pattern"),
        }
    }

    #[test]
    fn test_optimize_simple_query() {
        let query = SparqlParser::parse("SELECT ?verb WHERE { ?verb a cnv:Verb }").unwrap();
        let stats = TableStats::default();

        let plan = QueryOptimizer::optimize(query, &stats);

        assert!(!plan.steps.is_empty());
        assert!(plan.cost_estimate > 0);
    }

    #[test]
    fn test_optimize_with_filter() {
        let query = SparqlParser::parse(
            r#"SELECT ?verb WHERE { ?verb a cnv:Verb . ?verb rdfs:comment ?comment . FILTER(CONTAINS(?comment, "test")) }"#
        ).unwrap();
        let stats = TableStats::default();

        let plan = QueryOptimizer::optimize(query, &stats);

        // Should have scan steps and filter step
        let has_filter = plan.steps.iter().any(|step| matches!(step, ExecutionStep::Filter { .. }));
        assert!(has_filter);
    }

    #[test]
    fn test_join_method_selection() {
        let mut stats = TableStats::default();
        stats.triple_count = 1000;

        let method = QueryOptimizer::select_join_method("?x", "?y", &stats);
        match method {
            JoinMethod::HashJoin { .. } => {}, // Expected for large datasets
            JoinMethod::NestedLoop => panic!("Expected hash join for large dataset"),
        }
    }
}
