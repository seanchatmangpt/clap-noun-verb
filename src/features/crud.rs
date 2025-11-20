//! CRUD Operations for Noun-Verb CLI Patterns
//!
//! Provides trait-based definitions for Create, Read, Update, Delete operations
//! with comprehensive error handling and type safety.

use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Result type for noun-verb operations
pub type OperationResult<T> = Result<T, OperationError>;

/// Errors that can occur during operations
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
pub enum OperationError {
    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Unauthorized operation: {0}")]
    Unauthorized(String),

    #[error("Conflict detected: {0}")]
    Conflict(String),

    #[error("Operation timed out")]
    Timeout,

    #[error("Operation failed: {0}")]
    Failed(String),
}

/// Enum for CRUD operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrudOperation {
    Create,
    Read,
    Update,
    Delete,
    List,
    Execute,
}

impl CrudOperation {
    /// Get string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Read => "read",
            Self::Update => "update",
            Self::Delete => "delete",
            Self::List => "list",
            Self::Execute => "execute",
        }
    }

    /// Parse from string
    pub fn from_str(s: &str) -> OperationResult<Self> {
        match s.to_lowercase().as_str() {
            "create" | "c" => Ok(Self::Create),
            "read" | "get" | "r" => Ok(Self::Read),
            "update" | "u" => Ok(Self::Update),
            "delete" | "remove" | "d" => Ok(Self::Delete),
            "list" | "ls" | "l" => Ok(Self::List),
            "execute" | "exec" | "e" => Ok(Self::Execute),
            _ => Err(OperationError::InvalidInput(format!(
                "Unknown operation: {}",
                s
            ))),
        }
    }
}

/// Core trait for noun-verb operations
#[async_trait::async_trait]
pub trait NounVerb: Send + Sync + Debug {
    /// Get the noun name (e.g., "User", "Product")
    fn noun_name(&self) -> &str;

    /// Create a new resource
    async fn create(&self, data: serde_json::Value) -> OperationResult<serde_json::Value>;

    /// Read an existing resource
    async fn read(&self, id: &str) -> OperationResult<serde_json::Value>;

    /// Update an existing resource
    async fn update(
        &self,
        id: &str,
        data: serde_json::Value,
    ) -> OperationResult<serde_json::Value>;

    /// Delete a resource
    async fn delete(&self, id: &str) -> OperationResult<()>;

    /// List all resources
    async fn list(&self) -> OperationResult<Vec<serde_json::Value>>;

    /// Execute a custom operation
    async fn execute(
        &self,
        operation: &str,
        data: serde_json::Value,
    ) -> OperationResult<serde_json::Value>;

    /// Perform a CRUD operation
    async fn perform_operation(
        &self,
        op: CrudOperation,
        id: Option<&str>,
        data: Option<serde_json::Value>,
    ) -> OperationResult<serde_json::Value> {
        match op {
            CrudOperation::Create => {
                let data = data.ok_or_else(|| {
                    OperationError::InvalidInput("Create requires data".to_string())
                })?;
                self.create(data).await
            }
            CrudOperation::Read => {
                let id = id.ok_or_else(|| {
                    OperationError::InvalidInput("Read requires id".to_string())
                })?;
                self.read(id).await
            }
            CrudOperation::Update => {
                let id = id.ok_or_else(|| {
                    OperationError::InvalidInput("Update requires id".to_string())
                })?;
                let data = data.ok_or_else(|| {
                    OperationError::InvalidInput("Update requires data".to_string())
                })?;
                self.update(id, data).await
            }
            CrudOperation::Delete => {
                let id = id.ok_or_else(|| {
                    OperationError::InvalidInput("Delete requires id".to_string())
                })?;
                self.delete(id).await?;
                Ok(serde_json::json!({"status": "deleted"}))
            }
            CrudOperation::List => self.list().await.map(serde_json::Value::Array),
            CrudOperation::Execute => {
                let operation = id.ok_or_else(|| {
                    OperationError::InvalidInput("Execute requires operation name".to_string())
                })?;
                let data = data.unwrap_or(serde_json::json!({}));
                self.execute(operation, data).await
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crud_operation_as_str() {
        assert_eq!(CrudOperation::Create.as_str(), "create");
        assert_eq!(CrudOperation::Read.as_str(), "read");
        assert_eq!(CrudOperation::Update.as_str(), "update");
        assert_eq!(CrudOperation::Delete.as_str(), "delete");
        assert_eq!(CrudOperation::List.as_str(), "list");
        assert_eq!(CrudOperation::Execute.as_str(), "execute");
    }

    #[test]
    fn test_crud_operation_from_str() {
        assert_eq!(CrudOperation::from_str("create"), Ok(CrudOperation::Create));
        assert_eq!(CrudOperation::from_str("read"), Ok(CrudOperation::Read));
        assert_eq!(CrudOperation::from_str("get"), Ok(CrudOperation::Read));
        assert_eq!(CrudOperation::from_str("delete"), Ok(CrudOperation::Delete));
        assert_eq!(CrudOperation::from_str("remove"), Ok(CrudOperation::Delete));
        assert_eq!(CrudOperation::from_str("list"), Ok(CrudOperation::List));
        assert_eq!(CrudOperation::from_str("ls"), Ok(CrudOperation::List));

        assert!(CrudOperation::from_str("invalid").is_err());
    }

    #[test]
    fn test_operation_error_display() {
        let err = OperationError::NotFound("user:123".to_string());
        assert_eq!(err.to_string(), "Resource not found: user:123");

        let err = OperationError::InvalidInput("bad data".to_string());
        assert_eq!(err.to_string(), "Invalid input: bad data");
    }

    #[tokio::test]
    async fn test_mock_noun_verb_implementation() {
        #[derive(Debug)]
        struct MockUser;

        #[async_trait::async_trait]
        impl NounVerb for MockUser {
            fn noun_name(&self) -> &str {
                "User"
            }

            async fn create(&self, data: serde_json::Value) -> OperationResult<serde_json::Value> {
                Ok(serde_json::json!({
                    "id": "1",
                    "data": data
                }))
            }

            async fn read(&self, id: &str) -> OperationResult<serde_json::Value> {
                Ok(serde_json::json!({
                    "id": id,
                    "name": "John Doe"
                }))
            }

            async fn update(
                &self,
                id: &str,
                data: serde_json::Value,
            ) -> OperationResult<serde_json::Value> {
                Ok(serde_json::json!({
                    "id": id,
                    "data": data
                }))
            }

            async fn delete(&self, _id: &str) -> OperationResult<()> {
                Ok(())
            }

            async fn list(&self) -> OperationResult<Vec<serde_json::Value>> {
                Ok(vec![
                    serde_json::json!({"id": "1", "name": "John"}),
                    serde_json::json!({"id": "2", "name": "Jane"}),
                ])
            }

            async fn execute(
                &self,
                operation: &str,
                data: serde_json::Value,
            ) -> OperationResult<serde_json::Value> {
                Ok(serde_json::json!({
                    "operation": operation,
                    "data": data
                }))
            }
        }

        let user = MockUser;

        // Test create
        let result = user
            .create(serde_json::json!({"name": "Alice"}))
            .await
            .unwrap();
        assert_eq!(result["data"]["name"], "Alice");

        // Test read
        let result = user.read("1").await.unwrap();
        assert_eq!(result["id"], "1");

        // Test update
        let result = user
            .update("1", serde_json::json!({"name": "Bob"}))
            .await
            .unwrap();
        assert_eq!(result["id"], "1");

        // Test delete
        assert!(user.delete("1").await.is_ok());

        // Test list
        let result = user.list().await.unwrap();
        assert_eq!(result.len(), 2);

        // Test execute
        let result = user.execute("custom", serde_json::json!({"foo": "bar"})).await.unwrap();
        assert_eq!(result["operation"], "custom");

        // Test perform_operation
        let result = user
            .perform_operation(CrudOperation::Create, None, Some(serde_json::json!({"test": true})))
            .await
            .unwrap();
        assert_eq!(result["data"]["test"], true);
    }
}
