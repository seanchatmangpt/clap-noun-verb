// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Deterministic process-data transformations.
//!
//! The `process-data` capability is deliberately pure: it transforms admitted
//! JSON values and never performs I/O or actuation. This keeps data construction
//! separate from command execution while providing a concrete implementation for
//! the capability crown's `transform` verb.

use crate::{NounVerbError, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// One deterministic transformation in a [`ProcessDataPipeline`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcessDataStep {
    /// Select a value with an RFC 6901 JSON Pointer.
    SelectPointer(String),
    /// Rename one field on the current JSON object.
    RenameField { from: String, to: String },
    /// Recursively remove object fields whose value is JSON `null`.
    RemoveNullFields,
}

/// A replayable sequence of pure JSON transformations.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessDataPipeline {
    steps: Vec<ProcessDataStep>,
}

impl ProcessDataPipeline {
    /// Create an empty pipeline. An empty pipeline is the identity transform.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Append one transformation step.
    #[must_use]
    pub fn with_step(mut self, step: ProcessDataStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Return the admitted transformation steps in execution order.
    #[must_use]
    pub fn steps(&self) -> &[ProcessDataStep] {
        &self.steps
    }

    /// Transform one JSON value without mutating the input or performing I/O.
    ///
    /// # Errors
    ///
    /// Returns [`NounVerbError`] when a JSON Pointer does not resolve, a rename
    /// is requested for a non-object value, the source field is absent, or the
    /// destination field would overwrite an existing field.
    pub fn transform(&self, input: &Value) -> Result<Value> {
        let mut current = input.clone();
        for step in &self.steps {
            current = apply_step(current, step)?;
        }
        Ok(current)
    }
}

fn apply_step(mut value: Value, step: &ProcessDataStep) -> Result<Value> {
    match step {
        ProcessDataStep::SelectPointer(pointer) => value
            .pointer(pointer)
            .cloned()
            .ok_or_else(|| NounVerbError::argument_error(format!("JSON pointer did not resolve: {pointer}"))),
        ProcessDataStep::RenameField { from, to } => {
            let object = value.as_object_mut().ok_or_else(|| {
                NounVerbError::argument_error("rename-field requires a JSON object")
            })?;
            if from == to {
                if object.contains_key(from) {
                    return Ok(value);
                }
                return Err(NounVerbError::argument_error(format!(
                    "rename source field does not exist: {from}"
                )));
            }
            if object.contains_key(to) {
                return Err(NounVerbError::argument_error(format!(
                    "rename destination field already exists: {to}"
                )));
            }
            let field_value = object.remove(from).ok_or_else(|| {
                NounVerbError::argument_error(format!("rename source field does not exist: {from}"))
            })?;
            object.insert(to.clone(), field_value);
            Ok(value)
        }
        ProcessDataStep::RemoveNullFields => {
            remove_null_fields(&mut value);
            Ok(value)
        }
    }
}

fn remove_null_fields(value: &mut Value) {
    match value {
        Value::Object(object) => {
            object.retain(|_, child| !child.is_null());
            for child in object.values_mut() {
                remove_null_fields(child);
            }
        }
        Value::Array(values) => {
            for child in values {
                remove_null_fields(child);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn empty_pipeline_is_identity() {
        let input = json!({"name": "cnv"});
        assert_eq!(ProcessDataPipeline::new().transform(&input).expect("identity"), input);
    }

    #[test]
    fn pipeline_composes_pointer_rename_and_null_removal() {
        let input = json!({"payload": {"old": 7, "drop": null, "nested": {"drop": null, "keep": true}}});
        let pipeline = ProcessDataPipeline::new()
            .with_step(ProcessDataStep::SelectPointer("/payload".to_string()))
            .with_step(ProcessDataStep::RenameField {
                from: "old".to_string(),
                to: "new".to_string(),
            })
            .with_step(ProcessDataStep::RemoveNullFields);

        assert_eq!(
            pipeline.transform(&input).expect("bounded transform"),
            json!({"new": 7, "nested": {"keep": true}})
        );
        assert_eq!(input["payload"]["old"], 7, "input remains unchanged");
    }

    #[test]
    fn missing_pointer_is_typed_error() {
        let error = ProcessDataPipeline::new()
            .with_step(ProcessDataStep::SelectPointer("/missing".to_string()))
            .transform(&json!({"present": true}))
            .expect_err("missing pointer must be refused");
        assert!(error.to_string().contains("JSON pointer did not resolve"));
    }

    #[test]
    fn rename_refuses_overwrite() {
        let error = ProcessDataPipeline::new()
            .with_step(ProcessDataStep::RenameField {
                from: "a".to_string(),
                to: "b".to_string(),
            })
            .transform(&json!({"a": 1, "b": 2}))
            .expect_err("overwrites are not admitted");
        assert!(error.to_string().contains("destination field already exists"));
    }
}
