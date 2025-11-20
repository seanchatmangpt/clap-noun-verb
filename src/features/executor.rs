//! VerbExecutor for Executing Action Verbs
//!
//! Handles execution of verbs (Create, Read, Update, Delete, List, Execute)
//! with context management and result handling.

use super::crud::{CrudOperation, NounVerb, OperationError, OperationResult};
use serde_json::{json, Value};

/// Context for verb execution
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub noun: String,
    pub verb: String,
    pub resource_id: Option<String>,
    pub data: Option<Value>,
    pub metadata: Value,
}

impl ExecutionContext {
    /// Create new execution context
    pub fn new(noun: impl Into<String>, verb: impl Into<String>) -> Self {
        Self {
            noun: noun.into(),
            verb: verb.into(),
            resource_id: None,
            data: None,
            metadata: json!({}),
        }
    }

    /// Set resource ID
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.resource_id = Some(id.into());
        self
    }

    /// Set data payload
    pub fn with_data(mut self, data: Value) -> Self {
        self.data = Some(data);
        self
    }

    /// Set metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: Value) -> Self {
        if let Value::Object(ref mut obj) = self.metadata {
            obj.insert(key.into(), value);
        }
        self
    }
}

/// Executes verbs on nouns
#[derive(Debug)]
pub struct VerbExecutor;

impl VerbExecutor {
    /// Execute a verb on a noun
    pub async fn execute(
        context: ExecutionContext,
        noun_impl: &dyn NounVerb,
    ) -> OperationResult<Value> {
        let op = CrudOperation::from_str(&context.verb)?;

        let result = noun_impl
            .perform_operation(op, context.resource_id.as_deref(), context.data)
            .await?;

        Ok(json!({
            "noun": context.noun,
            "verb": context.verb,
            "result": result,
            "metadata": context.metadata
        }))
    }

    /// Validate verb syntax
    pub fn validate_verb(verb: &str) -> OperationResult<CrudOperation> {
        CrudOperation::from_str(verb)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_context_builder() {
        let ctx = ExecutionContext::new("user", "create")
            .with_id("123")
            .with_data(json!({"name": "Alice"}))
            .with_metadata("timestamp", json!("2024-11-20"));

        assert_eq!(ctx.noun, "user");
        assert_eq!(ctx.verb, "create");
        assert_eq!(ctx.resource_id, Some("123".to_string()));
        assert_eq!(ctx.data["name"], "Alice");
        assert_eq!(ctx.metadata["timestamp"], "2024-11-20");
    }

    #[tokio::test]
    async fn test_verb_executor() {
        #[derive(Debug)]
        struct TestNoun;

        #[async_trait::async_trait]
        impl NounVerb for TestNoun {
            fn noun_name(&self) -> &str {
                "Test"
            }

            async fn create(&self, data: Value) -> OperationResult<Value> {
                Ok(json!({"id": "1", "data": data}))
            }

            async fn read(&self, id: &str) -> OperationResult<Value> {
                Ok(json!({"id": id}))
            }

            async fn update(
                &self,
                id: &str,
                data: Value,
            ) -> OperationResult<Value> {
                Ok(json!({"id": id, "data": data}))
            }

            async fn delete(&self, _id: &str) -> OperationResult<()> {
                Ok(())
            }

            async fn list(&self) -> OperationResult<Vec<Value>> {
                Ok(vec![])
            }

            async fn execute(
                &self,
                _operation: &str,
                data: Value,
            ) -> OperationResult<Value> {
                Ok(data)
            }
        }

        let test_noun = TestNoun;
        let context = ExecutionContext::new("test", "create")
            .with_data(json!({"test": true}));

        let result = VerbExecutor::execute(context, &test_noun).await.unwrap();
        assert_eq!(result["noun"], "test");
        assert_eq!(result["verb"], "create");
    }

    #[test]
    fn test_verb_executor_validate() {
        assert!(VerbExecutor::validate_verb("create").is_ok());
        assert!(VerbExecutor::validate_verb("read").is_ok());
        assert!(VerbExecutor::validate_verb("invalid").is_err());
    }
}
