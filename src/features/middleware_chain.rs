//! MiddlewareChain for Request/Response Processing Pipeline
//!
//! Provides a flexible middleware pattern for processing requests and responses
//! with composable middleware components.

use super::executor::ExecutionContext;
use serde_json::{json, Value};
use std::sync::Arc;

/// Middleware request/response pair
#[derive(Debug, Clone)]
pub struct MiddlewareMessage {
    pub context: ExecutionContext,
    pub request_data: Value,
    pub response_data: Option<Value>,
}

impl MiddlewareMessage {
    /// Create new middleware message
    pub fn new(context: ExecutionContext, request_data: Value) -> Self {
        Self {
            context,
            request_data,
            response_data: None,
        }
    }

    /// Set response data
    pub fn with_response(mut self, response_data: Value) -> Self {
        self.response_data = Some(response_data);
        self
    }
}

/// Middleware trait for processing requests and responses
#[async_trait::async_trait]
pub trait Middleware: Send + Sync {
    /// Process a message (request or response)
    async fn process(&self, message: MiddlewareMessage) -> Result<MiddlewareMessage, String>;

    /// Get middleware name
    fn name(&self) -> &str;

    /// Check if middleware should process this message
    fn should_process(&self, message: &MiddlewareMessage) -> bool {
        let _ = message;
        true
    }
}

/// Middleware chain for composing multiple middleware
#[derive(Clone)]
pub struct MiddlewareChain {
    middlewares: Vec<Arc<dyn Middleware>>,
}

impl std::fmt::Debug for MiddlewareChain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MiddlewareChain")
            .field("middleware_count", &self.middlewares.len())
            .finish()
    }
}

impl MiddlewareChain {
    /// Create new middleware chain
    pub fn new() -> Self {
        Self {
            middlewares: Vec::new(),
        }
    }

    /// Add middleware to chain
    pub fn add(mut self, middleware: Arc<dyn Middleware>) -> Self {
        self.middlewares.push(middleware);
        self
    }

    /// Add multiple middleware
    pub fn add_all(mut self, middlewares: Vec<Arc<dyn Middleware>>) -> Self {
        self.middlewares.extend(middlewares);
        self
    }

    /// Process message through entire chain
    pub async fn process(&self, mut message: MiddlewareMessage) -> Result<MiddlewareMessage, String> {
        for middleware in &self.middlewares {
            if middleware.should_process(&message) {
                message = middleware.process(message).await?;
            }
        }
        Ok(message)
    }

    /// Get number of middleware
    pub fn len(&self) -> usize {
        self.middlewares.len()
    }

    /// Check if chain is empty
    pub fn is_empty(&self) -> bool {
        self.middlewares.is_empty()
    }

    /// Get middleware names
    pub fn names(&self) -> Vec<&str> {
        self.middlewares.iter().map(|m| m.name()).collect()
    }
}

impl Default for MiddlewareChain {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test middleware that logs messages
    #[derive(Debug)]
    struct LoggingMiddleware;

    #[async_trait::async_trait]
    impl Middleware for LoggingMiddleware {
        async fn process(&self, mut message: MiddlewareMessage) -> Result<MiddlewareMessage, String> {
            message.request_data["logged"] = json!(true);
            Ok(message)
        }

        fn name(&self) -> &str {
            "LoggingMiddleware"
        }
    }

    /// Test middleware that validates data
    #[derive(Debug)]
    struct ValidationMiddleware;

    #[async_trait::async_trait]
    impl Middleware for ValidationMiddleware {
        async fn process(&self, message: MiddlewareMessage) -> Result<MiddlewareMessage, String> {
            if message.request_data.is_null() {
                Err("Validation failed: null data".to_string())
            } else {
                Ok(message)
            }
        }

        fn name(&self) -> &str {
            "ValidationMiddleware"
        }
    }

    /// Test middleware that transforms data
    #[derive(Debug)]
    struct TransformMiddleware;

    #[async_trait::async_trait]
    impl Middleware for TransformMiddleware {
        async fn process(&self, mut message: MiddlewareMessage) -> Result<MiddlewareMessage, String> {
            if let Some(obj) = message.request_data.as_object_mut() {
                obj.insert("transformed".to_string(), json!(true));
            }
            Ok(message)
        }

        fn name(&self) -> &str {
            "TransformMiddleware"
        }
    }

    #[test]
    fn test_middleware_message_new() {
        let context = ExecutionContext::new("user", "create");
        let data = json!({"name": "alice"});
        let msg = MiddlewareMessage::new(context, data);

        assert_eq!(msg.context.noun, "user");
        assert_eq!(msg.context.verb, "create");
        assert_eq!(msg.request_data["name"], "alice");
        assert!(msg.response_data.is_none());
    }

    #[test]
    fn test_middleware_message_with_response() {
        let context = ExecutionContext::new("user", "read");
        let req = json!({"id": "123"});
        let res = json!({"id": "123", "name": "alice"});

        let msg = MiddlewareMessage::new(context, req).with_response(res.clone());

        assert_eq!(msg.response_data, Some(res));
    }

    #[tokio::test]
    async fn test_middleware_chain_empty() {
        let chain = MiddlewareChain::new();
        let context = ExecutionContext::new("user", "list");
        let data = json!({"filter": "active"});
        let msg = MiddlewareMessage::new(context, data.clone());

        let result = chain.process(msg).await.unwrap();
        assert_eq!(result.request_data, data);
    }

    #[tokio::test]
    async fn test_middleware_chain_single() {
        let chain = MiddlewareChain::new().add(Arc::new(LoggingMiddleware));
        let context = ExecutionContext::new("user", "create");
        let msg = MiddlewareMessage::new(context, json!({"name": "bob"}));

        let result = chain.process(msg).await.unwrap();
        assert_eq!(result.request_data["logged"], true);
    }

    #[tokio::test]
    async fn test_middleware_chain_multiple() {
        let middlewares = vec![
            Arc::new(ValidationMiddleware) as Arc<dyn Middleware>,
            Arc::new(LoggingMiddleware),
            Arc::new(TransformMiddleware),
        ];

        let chain = MiddlewareChain::new().add_all(middlewares);
        let context = ExecutionContext::new("product", "update");
        let msg = MiddlewareMessage::new(context, json!({"id": "456", "price": 99.99}));

        let result = chain.process(msg).await.unwrap();
        assert_eq!(result.request_data["logged"], true);
        assert_eq!(result.request_data["transformed"], true);
        assert_eq!(result.request_data["id"], "456");
    }

    #[tokio::test]
    async fn test_middleware_chain_order() {
        let middlewares = vec![
            Arc::new(LoggingMiddleware) as Arc<dyn Middleware>,
            Arc::new(TransformMiddleware),
        ];

        let chain = MiddlewareChain::new().add_all(middlewares);
        let context = ExecutionContext::new("order", "create");
        let msg = MiddlewareMessage::new(context, json!({"items": []}));

        let result = chain.process(msg).await.unwrap();
        // Verify execution order: logging sets logged, then transform adds transformed
        assert_eq!(result.request_data["logged"], true);
        assert_eq!(result.request_data["transformed"], true);
    }

    #[tokio::test]
    async fn test_middleware_chain_error_handling() {
        let middlewares = vec![
            Arc::new(ValidationMiddleware) as Arc<dyn Middleware>,
            Arc::new(LoggingMiddleware),
        ];

        let chain = MiddlewareChain::new().add_all(middlewares);
        let context = ExecutionContext::new("user", "delete");
        let msg = MiddlewareMessage::new(context, json!(null));

        let result = chain.process(msg).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Validation failed"));
    }

    #[test]
    fn test_middleware_chain_metadata() {
        let middlewares = vec![
            Arc::new(LoggingMiddleware) as Arc<dyn Middleware>,
            Arc::new(ValidationMiddleware),
            Arc::new(TransformMiddleware),
        ];

        let chain = MiddlewareChain::new().add_all(middlewares);
        assert_eq!(chain.len(), 3);
        assert!(!chain.is_empty());
        assert_eq!(
            chain.names(),
            vec!["LoggingMiddleware", "ValidationMiddleware", "TransformMiddleware"]
        );
    }

    #[tokio::test]
    async fn test_middleware_selective_processing() {
        #[derive(Debug)]
        struct SelectiveMiddleware;

        #[async_trait::async_trait]
        impl Middleware for SelectiveMiddleware {
            async fn process(&self, mut message: MiddlewareMessage) -> Result<MiddlewareMessage, String> {
                message.request_data["selective"] = json!(true);
                Ok(message)
            }

            fn name(&self) -> &str {
                "SelectiveMiddleware"
            }

            fn should_process(&self, message: &MiddlewareMessage) -> bool {
                // Only process if noun is "user"
                message.context.noun == "user"
            }
        }

        let chain = MiddlewareChain::new().add(Arc::new(SelectiveMiddleware));

        // Test with matching noun
        let msg1 = MiddlewareMessage::new(ExecutionContext::new("user", "create"), json!({}));
        let result1 = chain.process(msg1).await.unwrap();
        assert_eq!(result1.request_data["selective"], true);

        // Test with non-matching noun
        let msg2 = MiddlewareMessage::new(ExecutionContext::new("product", "create"), json!({}));
        let result2 = chain.process(msg2).await.unwrap();
        assert!(result2.request_data["selective"].is_null());
    }
}
