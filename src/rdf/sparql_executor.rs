use std::collections::HashMap;
use std::sync::Arc;
use crate::rdf::ontology::Ontology;
use crate::rdf::sparql_parser::{ParsedQuery, TriplePattern, FilterExpression, Aggregation, PatternElement, PropertyPath};
use crate::rdf::sparql_optimizer::{QueryOptimizer, QueryPlan, ExecutionStep, JoinMethod, TableStats};

/// Variable binding (query result row)
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Binding {
    pub variables: HashMap<String, RdfValue>,
}

/// RDF value types
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum RdfValue {
    Uri(String),
    Literal(String),
    BlankNode(String),
}

impl RdfValue {
    pub fn as_str(&self) -> &str {
        match self {
            RdfValue::Uri(s) | RdfValue::Literal(s) | RdfValue::BlankNode(s) => s,
        }
    }
}

/// RDF triple
#[derive(Debug, Clone)]
pub struct RdfTriple {
    pub subject: String,
    pub predicate: String,
    pub object: String,
}

/// Query execution errors
#[derive(Debug)]
pub enum ExecutionError {
    VariableNotFound { var: String },
    InvalidAggregation { message: String },
    InvalidFilter { message: String },
    OntologyError { message: String },
}

impl std::fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionError::VariableNotFound { var } => write!(f, "Variable not found: {}", var),
            ExecutionError::InvalidAggregation { message } => write!(f, "Invalid aggregation: {}", message),
            ExecutionError::InvalidFilter { message } => write!(f, "Invalid filter: {}", message),
            ExecutionError::OntologyError { message } => write!(f, "Ontology error: {}", message),
        }
    }
}

impl std::error::Error for ExecutionError {}

/// Query executor
pub struct QueryExecutor {
    ontology: Arc<Ontology>,
    stats: TableStats,
}

impl QueryExecutor {
    /// Create new executor
    pub fn new(ontology: Arc<Ontology>) -> Self {
        let stats = Self::build_stats(&ontology);
        Self { ontology, stats }
    }

    /// Build statistics from ontology
    fn build_stats(ontology: &Ontology) -> TableStats {
        let triples = ontology.iter_triples();
        let triple_refs: Vec<_> = triples.iter().map(|(s, p, o)| (s.as_str(), p.as_str(), o.as_str())).collect();
        TableStats::from_triples(triple_refs.into_iter())
    }

    /// Execute parsed query
    pub fn execute(&self, query: ParsedQuery) -> Result<Vec<Binding>, ExecutionError> {
        let plan = QueryOptimizer::optimize(query, &self.stats);
        self.execute_plan(&plan)
    }

    /// Execute query plan
    fn execute_plan(&self, plan: &QueryPlan) -> Result<Vec<Binding>, ExecutionError> {
        let mut results = vec![Binding { variables: HashMap::new() }];

        for step in &plan.steps {
            results = self.execute_step(step, results)?;
        }

        Ok(results)
    }

    /// Execute single step
    fn execute_step(&self, step: &ExecutionStep, mut bindings: Vec<Binding>) -> Result<Vec<Binding>, ExecutionError> {
        match step {
            ExecutionStep::ScanTriples { pattern, .. } => {
                self.scan_and_bind(pattern, bindings)
            }
            ExecutionStep::Filter { expression } => {
                Ok(self.apply_filter(&bindings, expression))
            }
            ExecutionStep::Join { left_var, right_var, method } => {
                self.apply_join(&bindings, left_var, right_var, method)
            }
            ExecutionStep::Optional { patterns } => {
                self.apply_optional(&bindings, patterns)
            }
            ExecutionStep::Union { branches } => {
                self.apply_union(&bindings, branches)
            }
            ExecutionStep::Aggregate { aggregations, group_by } => {
                self.apply_aggregate(&bindings, aggregations, group_by)
            }
        }
    }

    /// Scan triples and create bindings
    fn scan_and_bind(&self, pattern: &TriplePattern, initial_bindings: Vec<Binding>) -> Result<Vec<Binding>, ExecutionError> {
        let mut results = Vec::new();

        for binding in &initial_bindings {
            match pattern {
                TriplePattern::Simple { subject, predicate, object } => {
                    let triples = self.scan_simple_pattern(subject, predicate, object, &binding)?;

                    for triple in triples {
                        let mut new_binding = binding.clone();

                        if let PatternElement::Variable(var) = subject {
                            new_binding.variables.insert(var.clone(), RdfValue::Uri(triple.subject.clone()));
                        }
                        if let PatternElement::Variable(var) = predicate {
                            new_binding.variables.insert(var.clone(), RdfValue::Uri(triple.predicate.clone()));
                        }
                        if let PatternElement::Variable(var) = object {
                            new_binding.variables.insert(var.clone(), RdfValue::Literal(triple.object.clone()));
                        }

                        results.push(new_binding);
                    }
                }
                TriplePattern::PropertyPath { subject, path, object } => {
                    let paths = self.evaluate_property_path(subject, path, object, &binding)?;

                    for (subj, obj) in paths {
                        let mut new_binding = binding.clone();

                        if let PatternElement::Variable(var) = subject {
                            new_binding.variables.insert(var.clone(), RdfValue::Uri(subj));
                        }
                        if let PatternElement::Variable(var) = object {
                            new_binding.variables.insert(var.clone(), RdfValue::Uri(obj));
                        }

                        results.push(new_binding);
                    }
                }
            }
        }

        if results.is_empty() && !initial_bindings.is_empty() {
            Ok(initial_bindings)
        } else {
            Ok(results)
        }
    }

    /// Scan simple triple pattern
    fn scan_simple_pattern(
        &self,
        subject: &PatternElement,
        predicate: &PatternElement,
        object: &PatternElement,
        binding: &Binding,
    ) -> Result<Vec<RdfTriple>, ExecutionError> {
        let subj_filter = self.resolve_pattern_element(subject, binding);
        let pred_filter = self.resolve_pattern_element(predicate, binding);
        let obj_filter = self.resolve_pattern_element(object, binding);

        let triples = self.ontology.iter_triples()
            .filter(|(s, p, o)| {
                subj_filter.as_ref().map_or(true, |f| s == f)
                    && pred_filter.as_ref().map_or(true, |f| p == f)
                    && obj_filter.as_ref().map_or(true, |f| o == f)
            })
            .map(|(s, p, o)| RdfTriple {
                subject: s.clone(),
                predicate: p.clone(),
                object: o.clone(),
            })
            .collect();

        Ok(triples)
    }

    /// Resolve pattern element to value
    fn resolve_pattern_element(&self, elem: &PatternElement, binding: &Binding) -> Option<String> {
        match elem {
            PatternElement::Variable(var) => {
                binding.variables.get(var).map(|v| v.as_str().to_string())
            }
            PatternElement::Constant(val) => Some(val.clone()),
        }
    }

    /// Evaluate property path
    fn evaluate_property_path(
        &self,
        subject: &PatternElement,
        path: &PropertyPath,
        object: &PatternElement,
        binding: &Binding,
    ) -> Result<Vec<(String, String)>, ExecutionError> {
        let start = self.resolve_pattern_element(subject, binding);
        let end = self.resolve_pattern_element(object, binding);

        match path {
            PropertyPath::Direct(pred) => {
                self.evaluate_direct_path(start.as_deref(), pred, end.as_deref())
            }
            PropertyPath::Inverse(inner) => {
                self.evaluate_inverse_path(start.as_deref(), inner, end.as_deref())
            }
            PropertyPath::ZeroOrMore(inner) => {
                self.evaluate_kleene_star(start.as_deref(), inner, end.as_deref())
            }
            PropertyPath::OneOrMore(inner) => {
                self.evaluate_kleene_plus(start.as_deref(), inner, end.as_deref())
            }
            PropertyPath::ZeroOrOne(inner) => {
                self.evaluate_optional_path(start.as_deref(), inner, end.as_deref())
            }
            PropertyPath::Sequence(p1, p2) => {
                self.evaluate_sequence(start.as_deref(), p1, p2, end.as_deref())
            }
            PropertyPath::Alternative(p1, p2) => {
                self.evaluate_alternative(start.as_deref(), p1, p2, end.as_deref())
            }
        }
    }

    fn evaluate_direct_path(&self, start: Option<&str>, pred: &str, end: Option<&str>) -> Result<Vec<(String, String)>, ExecutionError> {
        let mut results = Vec::new();

        for (s, p, o) in self.ontology.iter_triples() {
            if p != pred {
                continue;
            }
            if start.is_some() && start != Some(s.as_str()) {
                continue;
            }
            if end.is_some() && end != Some(o.as_str()) {
                continue;
            }
            results.push((s.clone(), o.clone()));
        }

        Ok(results)
    }

    fn evaluate_inverse_path(&self, start: Option<&str>, path: &PropertyPath, end: Option<&str>) -> Result<Vec<(String, String)>, ExecutionError> {
        // Swap start and end for inverse
        let forward = self.evaluate_property_path_internal(end, path, start)?;
        Ok(forward.into_iter().map(|(o, s)| (s, o)).collect())
    }

    fn evaluate_kleene_star(&self, start: Option<&str>, path: &PropertyPath, end: Option<&str>) -> Result<Vec<(String, String)>, ExecutionError> {
        let mut results = Vec::new();
        let mut visited = std::collections::HashSet::new();

        // Zero steps - reflexive
        if let Some(s) = start {
            results.push((s.to_string(), s.to_string()));
            visited.insert((s.to_string(), s.to_string()));
        }

        // One or more steps
        let one_step = self.evaluate_property_path_internal(start, path, None)?;
        for pair in one_step {
            if visited.insert(pair.clone()) {
                results.push(pair);
            }
        }

        // Transitive closure (simplified - max 10 hops)
        for _ in 0..10 {
            let mut new_pairs = Vec::new();
            for (s, o) in &results {
                let next = self.evaluate_property_path_internal(Some(o), path, None)?;
                for (_, o2) in next {
                    let pair = (s.clone(), o2);
                    if visited.insert(pair.clone()) {
                        new_pairs.push(pair);
                    }
                }
            }
            if new_pairs.is_empty() {
                break;
            }
            results.extend(new_pairs);
        }

        if let Some(e) = end {
            results.retain(|(_, o)| o == e);
        }

        Ok(results)
    }

    fn evaluate_kleene_plus(&self, start: Option<&str>, path: &PropertyPath, end: Option<&str>) -> Result<Vec<(String, String)>, ExecutionError> {
        let mut results = self.evaluate_kleene_star(start, path, end)?;
        // Remove zero-length paths
        results.retain(|(s, o)| s != o);
        Ok(results)
    }

    fn evaluate_optional_path(&self, start: Option<&str>, path: &PropertyPath, end: Option<&str>) -> Result<Vec<(String, String)>, ExecutionError> {
        let mut results = Vec::new();

        // Zero steps
        if let Some(s) = start {
            results.push((s.to_string(), s.to_string()));
        }

        // One step
        results.extend(self.evaluate_property_path_internal(start, path, end)?);

        Ok(results)
    }

    fn evaluate_sequence(&self, start: Option<&str>, p1: &PropertyPath, p2: &PropertyPath, end: Option<&str>) -> Result<Vec<(String, String)>, ExecutionError> {
        let first = self.evaluate_property_path_internal(start, p1, None)?;
        let mut results = Vec::new();

        for (s, intermediate) in first {
            let second = self.evaluate_property_path_internal(Some(&intermediate), p2, end)?;
            for (_, o) in second {
                results.push((s.clone(), o));
            }
        }

        Ok(results)
    }

    fn evaluate_alternative(&self, start: Option<&str>, p1: &PropertyPath, p2: &PropertyPath, end: Option<&str>) -> Result<Vec<(String, String)>, ExecutionError> {
        let mut results = self.evaluate_property_path_internal(start, p1, end)?;
        results.extend(self.evaluate_property_path_internal(start, p2, end)?);
        Ok(results)
    }

    fn evaluate_property_path_internal(&self, start: Option<&str>, path: &PropertyPath, end: Option<&str>) -> Result<Vec<(String, String)>, ExecutionError> {
        match path {
            PropertyPath::Direct(pred) => self.evaluate_direct_path(start, pred, end),
            PropertyPath::Inverse(inner) => self.evaluate_inverse_path(start, inner, end),
            PropertyPath::ZeroOrMore(inner) => self.evaluate_kleene_star(start, inner, end),
            PropertyPath::OneOrMore(inner) => self.evaluate_kleene_plus(start, inner, end),
            PropertyPath::ZeroOrOne(inner) => self.evaluate_optional_path(start, inner, end),
            PropertyPath::Sequence(p1, p2) => self.evaluate_sequence(start, p1, p2, end),
            PropertyPath::Alternative(p1, p2) => self.evaluate_alternative(start, p1, p2, end),
        }
    }

    /// Apply filter expression
    fn apply_filter(&self, bindings: &[Binding], expr: &FilterExpression) -> Vec<Binding> {
        bindings.iter().filter(|b| self.evaluate_filter(b, expr)).cloned().collect()
    }

    /// Evaluate filter for a binding
    fn evaluate_filter(&self, binding: &Binding, expr: &FilterExpression) -> bool {
        match expr {
            FilterExpression::Contains { var, literal } => {
                binding.variables.get(var)
                    .map_or(false, |v| v.as_str().contains(literal))
            }
            FilterExpression::StartsWith { var, literal } => {
                binding.variables.get(var)
                    .map_or(false, |v| v.as_str().starts_with(literal))
            }
            FilterExpression::EndsWith { var, literal } => {
                binding.variables.get(var)
                    .map_or(false, |v| v.as_str().ends_with(literal))
            }
            FilterExpression::Equals { var, value } => {
                binding.variables.get(var)
                    .map_or(false, |v| v.as_str() == value)
            }
            FilterExpression::NotEquals { var, value } => {
                binding.variables.get(var)
                    .map_or(false, |v| v.as_str() != value)
            }
            FilterExpression::GreaterThan { var, value } => {
                binding.variables.get(var)
                    .and_then(|v| v.as_str().parse::<f64>().ok())
                    .and_then(|v_num| value.parse::<f64>().ok().map(|val_num| v_num > val_num))
                    .unwrap_or(false)
            }
            FilterExpression::LessThan { var, value } => {
                binding.variables.get(var)
                    .and_then(|v| v.as_str().parse::<f64>().ok())
                    .and_then(|v_num| value.parse::<f64>().ok().map(|val_num| v_num < val_num))
                    .unwrap_or(false)
            }
            FilterExpression::And { left, right } => {
                self.evaluate_filter(binding, left) && self.evaluate_filter(binding, right)
            }
            FilterExpression::Or { left, right } => {
                self.evaluate_filter(binding, left) || self.evaluate_filter(binding, right)
            }
            FilterExpression::Not { expr } => {
                !self.evaluate_filter(binding, expr)
            }
        }
    }

    /// Apply join
    fn apply_join(&self, bindings: &[Binding], left_var: &str, right_var: &str, method: &JoinMethod) -> Result<Vec<Binding>, ExecutionError> {
        match method {
            JoinMethod::HashJoin { hash_on } => self.hash_join(bindings, left_var, right_var, hash_on),
            JoinMethod::NestedLoop => self.nested_loop_join(bindings, left_var, right_var),
        }
    }

    fn hash_join(&self, bindings: &[Binding], _left_var: &str, _right_var: &str, hash_on: &str) -> Result<Vec<Binding>, ExecutionError> {
        let mut hash_table: HashMap<String, Vec<Binding>> = HashMap::new();

        for binding in bindings {
            if let Some(value) = binding.variables.get(hash_on) {
                hash_table.entry(value.as_str().to_string())
                    .or_default()
                    .push(binding.clone());
            }
        }

        let mut results = Vec::new();
        for group in hash_table.values() {
            results.extend_from_slice(group);
        }

        Ok(results)
    }

    fn nested_loop_join(&self, bindings: &[Binding], _left_var: &str, _right_var: &str) -> Result<Vec<Binding>, ExecutionError> {
        Ok(bindings.to_vec())
    }

    /// Apply optional patterns
    fn apply_optional(&self, bindings: &[Binding], patterns: &[TriplePattern]) -> Result<Vec<Binding>, ExecutionError> {
        let mut results = Vec::new();

        for binding in bindings {
            let mut extended = vec![binding.clone()];

            for pattern in patterns {
                extended = self.scan_and_bind(pattern, extended)?;
            }

            if extended.is_empty() {
                results.push(binding.clone());
            } else {
                results.extend(extended);
            }
        }

        Ok(results)
    }

    /// Apply union
    fn apply_union(&self, bindings: &[Binding], branches: &[Vec<TriplePattern>]) -> Result<Vec<Binding>, ExecutionError> {
        let mut results = Vec::new();

        for branch in branches {
            let mut branch_results = bindings.to_vec();
            for pattern in branch {
                branch_results = self.scan_and_bind(pattern, branch_results)?;
            }
            results.extend(branch_results);
        }

        Ok(results)
    }

    /// Apply aggregations
    fn apply_aggregate(&self, bindings: &[Binding], aggregations: &[Aggregation], group_by: &[String]) -> Result<Vec<Binding>, ExecutionError> {
        if group_by.is_empty() {
            // No grouping - aggregate all
            let mut result = Binding { variables: HashMap::new() };

            for agg in aggregations {
                let value = self.compute_aggregation(agg, bindings)?;
                let alias = match agg {
                    Aggregation::Count { alias, .. } |
                    Aggregation::Sum { alias, .. } |
                    Aggregation::Min { alias, .. } |
                    Aggregation::Max { alias, .. } |
                    Aggregation::Avg { alias, .. } => alias.clone(),
                };
                result.variables.insert(alias, RdfValue::Literal(value));
            }

            Ok(vec![result])
        } else {
            // Group by variables
            let mut groups: HashMap<Vec<String>, Vec<Binding>> = HashMap::new();

            for binding in bindings {
                let key: Vec<String> = group_by.iter()
                    .filter_map(|var| binding.variables.get(var).map(|v| v.as_str().to_string()))
                    .collect();
                groups.entry(key).or_default().push(binding.clone());
            }

            let mut results = Vec::new();
            for (key, group) in groups {
                let mut result = Binding { variables: HashMap::new() };

                for (i, var) in group_by.iter().enumerate() {
                    if let Some(val) = key.get(i) {
                        result.variables.insert(var.clone(), RdfValue::Literal(val.clone()));
                    }
                }

                for agg in aggregations {
                    let value = self.compute_aggregation(agg, &group)?;
                    let alias = match agg {
                        Aggregation::Count { alias, .. } |
                        Aggregation::Sum { alias, .. } |
                        Aggregation::Min { alias, .. } |
                        Aggregation::Max { alias, .. } |
                        Aggregation::Avg { alias, .. } => alias.clone(),
                    };
                    result.variables.insert(alias, RdfValue::Literal(value));
                }

                results.push(result);
            }

            Ok(results)
        }
    }

    fn compute_aggregation(&self, agg: &Aggregation, bindings: &[Binding]) -> Result<String, ExecutionError> {
        match agg {
            Aggregation::Count { var, distinct, .. } => {
                let values: Vec<_> = bindings.iter()
                    .filter_map(|b| b.variables.get(var))
                    .collect();

                let count = if *distinct {
                    let unique: std::collections::HashSet<_> = values.into_iter().collect();
                    unique.len()
                } else {
                    values.len()
                };

                Ok(count.to_string())
            }
            Aggregation::Sum { var, .. } => {
                let sum: f64 = bindings.iter()
                    .filter_map(|b| b.variables.get(var))
                    .filter_map(|v| v.as_str().parse::<f64>().ok())
                    .sum();
                Ok(sum.to_string())
            }
            Aggregation::Min { var, .. } => {
                bindings.iter()
                    .filter_map(|b| b.variables.get(var))
                    .map(|v| v.as_str())
                    .min()
                    .map(|s| s.to_string())
                    .ok_or_else(|| ExecutionError::InvalidAggregation {
                        message: "No values for MIN".to_string(),
                    })
            }
            Aggregation::Max { var, .. } => {
                bindings.iter()
                    .filter_map(|b| b.variables.get(var))
                    .map(|v| v.as_str())
                    .max()
                    .map(|s| s.to_string())
                    .ok_or_else(|| ExecutionError::InvalidAggregation {
                        message: "No values for MAX".to_string(),
                    })
            }
            Aggregation::Avg { var, .. } => {
                let values: Vec<f64> = bindings.iter()
                    .filter_map(|b| b.variables.get(var))
                    .filter_map(|v| v.as_str().parse::<f64>().ok())
                    .collect();

                if values.is_empty() {
                    return Err(ExecutionError::InvalidAggregation {
                        message: "No values for AVG".to_string(),
                    });
                }

                let avg = values.iter().sum::<f64>() / values.len() as f64;
                Ok(avg.to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_ontology() -> Arc<Ontology> {
        let mut ont = Ontology::new();
        ont.add_triple("cmd:build", "rdf:type", "cnv:Verb");
        ont.add_triple("cmd:build", "rdfs:comment", "Build the project");
        ont.add_triple("cmd:test", "rdf:type", "cnv:Verb");
        ont.add_triple("cmd:test", "rdfs:comment", "Run tests");
        Arc::new(ont)
    }

    #[test]
    fn test_simple_scan() {
        let ont = create_test_ontology();
        let executor = QueryExecutor::new(ont);

        let pattern = TriplePattern::Simple {
            subject: PatternElement::Variable("?verb".to_string()),
            predicate: PatternElement::Constant("rdf:type".to_string()),
            object: PatternElement::Constant("cnv:Verb".to_string()),
        };

        let bindings = vec![Binding { variables: HashMap::new() }];
        let results = executor.scan_and_bind(&pattern, bindings).unwrap();

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_filter_contains() {
        let ont = create_test_ontology();
        let executor = QueryExecutor::new(ont);

        let mut binding = Binding { variables: HashMap::new() };
        binding.variables.insert("?comment".to_string(), RdfValue::Literal("Build the project".to_string()));

        let filter = FilterExpression::Contains {
            var: "?comment".to_string(),
            literal: "Build".to_string(),
        };

        assert!(executor.evaluate_filter(&binding, &filter));
    }
}
