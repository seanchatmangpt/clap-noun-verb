//! RouteRegistry for Command Routing and Discovery
//!
//! Provides a registry for mapping noun-verb commands to their implementations
//! with support for dynamic registration and routing.

use super::crud::NounVerb;
use std::collections::HashMap;
use std::sync::Arc;

/// Route information for a noun-verb command
#[derive(Debug, Clone)]
pub struct RouteInfo {
    pub noun: String,
    pub verbs: Vec<String>,
    pub description: Option<String>,
}

impl RouteInfo {
    /// Create new route info
    pub fn new(noun: impl Into<String>) -> Self {
        Self {
            noun: noun.into(),
            verbs: Vec::new(),
            description: None,
        }
    }

    /// Add a supported verb
    pub fn with_verb(mut self, verb: impl Into<String>) -> Self {
        self.verbs.push(verb.into());
        self
    }

    /// Add description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Check if verb is supported
    pub fn supports_verb(&self, verb: &str) -> bool {
        self.verbs.contains(&verb.to_string())
    }
}

/// Route registry for command dispatch
#[derive(Debug)]
pub struct RouteRegistry {
    routes: HashMap<String, Arc<dyn NounVerb>>,
    route_info: HashMap<String, RouteInfo>,
}

impl RouteRegistry {
    /// Create new route registry
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
            route_info: HashMap::new(),
        }
    }

    /// Register a noun handler
    pub fn register(mut self, noun: impl Into<String>, handler: Arc<dyn NounVerb>, info: RouteInfo) -> Self {
        let noun_str = noun.into();
        self.routes.insert(noun_str.clone(), handler);
        self.route_info.insert(noun_str, info);
        self
    }

    /// Unregister a noun handler
    pub fn unregister(&mut self, noun: &str) -> Option<Arc<dyn NounVerb>> {
        self.route_info.remove(noun);
        self.routes.remove(noun)
    }

    /// Get handler for noun
    pub fn get_handler(&self, noun: &str) -> Option<Arc<dyn NounVerb>> {
        self.routes.get(noun).cloned()
    }

    /// Get route info for noun
    pub fn get_route_info(&self, noun: &str) -> Option<RouteInfo> {
        self.route_info.get(noun).cloned()
    }

    /// Check if noun is registered
    pub fn has_noun(&self, noun: &str) -> bool {
        self.routes.contains_key(noun)
    }

    /// Check if verb is supported for noun
    pub fn supports_verb(&self, noun: &str, verb: &str) -> bool {
        self.route_info
            .get(noun)
            .map(|info| info.supports_verb(verb))
            .unwrap_or(false)
    }

    /// Get all registered nouns
    pub fn get_nouns(&self) -> Vec<String> {
        self.routes.keys().cloned().collect()
    }

    /// Get verbs for noun
    pub fn get_verbs(&self, noun: &str) -> Vec<String> {
        self.route_info
            .get(noun)
            .map(|info| info.verbs.clone())
            .unwrap_or_default()
    }

    /// Get number of registered nouns
    pub fn len(&self) -> usize {
        self.routes.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.routes.is_empty()
    }

    /// Get all route info
    pub fn get_all_routes(&self) -> Vec<RouteInfo> {
        self.route_info.values().cloned().collect()
    }

    /// Find routes by verb
    pub fn find_by_verb(&self, verb: &str) -> Vec<(String, RouteInfo)> {
        self.route_info
            .iter()
            .filter(|(_, info)| info.supports_verb(verb))
            .map(|(noun, info)| (noun.clone(), info.clone()))
            .collect()
    }
}

impl Default for RouteRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    #[derive(Debug)]
    struct MockUser;

    #[async_trait::async_trait]
    impl NounVerb for MockUser {
        fn noun_name(&self) -> &str {
            "User"
        }

        async fn create(&self, data: Value) -> super::super::crud::OperationResult<Value> {
            Ok(json!({"id": "1", "data": data}))
        }

        async fn read(&self, id: &str) -> super::super::crud::OperationResult<Value> {
            Ok(json!({"id": id}))
        }

        async fn update(
            &self,
            id: &str,
            data: Value,
        ) -> super::super::crud::OperationResult<Value> {
            Ok(json!({"id": id, "data": data}))
        }

        async fn delete(&self, _id: &str) -> super::super::crud::OperationResult<()> {
            Ok(())
        }

        async fn list(&self) -> super::super::crud::OperationResult<Vec<Value>> {
            Ok(vec![])
        }

        async fn execute(
            &self,
            _operation: &str,
            data: Value,
        ) -> super::super::crud::OperationResult<Value> {
            Ok(data)
        }
    }

    #[derive(Debug)]
    struct MockProduct;

    #[async_trait::async_trait]
    impl NounVerb for MockProduct {
        fn noun_name(&self) -> &str {
            "Product"
        }

        async fn create(&self, data: Value) -> super::super::crud::OperationResult<Value> {
            Ok(json!({"sku": "P1", "data": data}))
        }

        async fn read(&self, id: &str) -> super::super::crud::OperationResult<Value> {
            Ok(json!({"sku": id}))
        }

        async fn update(
            &self,
            id: &str,
            data: Value,
        ) -> super::super::crud::OperationResult<Value> {
            Ok(json!({"sku": id, "data": data}))
        }

        async fn delete(&self, _id: &str) -> super::super::crud::OperationResult<()> {
            Ok(())
        }

        async fn list(&self) -> super::super::crud::OperationResult<Vec<Value>> {
            Ok(vec![])
        }

        async fn execute(
            &self,
            _operation: &str,
            data: Value,
        ) -> super::super::crud::OperationResult<Value> {
            Ok(data)
        }
    }

    #[test]
    fn test_route_info_new() {
        let info = RouteInfo::new("user");
        assert_eq!(info.noun, "user");
        assert!(info.verbs.is_empty());
        assert!(info.description.is_none());
    }

    #[test]
    fn test_route_info_with_verb() {
        let info = RouteInfo::new("user")
            .with_verb("create")
            .with_verb("read")
            .with_verb("update");

        assert_eq!(info.verbs.len(), 3);
        assert!(info.supports_verb("create"));
        assert!(!info.supports_verb("delete"));
    }

    #[test]
    fn test_route_info_with_description() {
        let info = RouteInfo::new("user").with_description("Manages user resources");
        assert_eq!(info.description, Some("Manages user resources".to_string()));
    }

    #[test]
    fn test_registry_register_single() {
        let user_info = RouteInfo::new("user")
            .with_verb("create")
            .with_verb("read")
            .with_description("User management");

        let registry = RouteRegistry::new().register("user", Arc::new(MockUser), user_info);

        assert!(registry.has_noun("user"));
        assert!(registry.get_handler("user").is_some());
        assert!(registry.supports_verb("user", "create"));
    }

    #[test]
    fn test_registry_register_multiple() {
        let user_info = RouteInfo::new("user")
            .with_verb("create")
            .with_verb("read");
        let product_info = RouteInfo::new("product")
            .with_verb("create")
            .with_verb("update");

        let registry = RouteRegistry::new()
            .register("user", Arc::new(MockUser), user_info)
            .register("product", Arc::new(MockProduct), product_info);

        assert_eq!(registry.len(), 2);
        assert!(registry.has_noun("user"));
        assert!(registry.has_noun("product"));
    }

    #[test]
    fn test_registry_get_handler() {
        let user_info = RouteInfo::new("user").with_verb("create");
        let registry = RouteRegistry::new().register("user", Arc::new(MockUser), user_info);

        let handler = registry.get_handler("user");
        assert!(handler.is_some());
        assert_eq!(handler.unwrap().noun_name(), "User");
    }

    #[test]
    fn test_registry_get_route_info() {
        let user_info = RouteInfo::new("user")
            .with_verb("create")
            .with_verb("read")
            .with_description("User ops");

        let registry = RouteRegistry::new().register("user", Arc::new(MockUser), user_info);

        let info = registry.get_route_info("user").unwrap();
        assert_eq!(info.noun, "user");
        assert_eq!(info.verbs.len(), 2);
        assert_eq!(info.description, Some("User ops".to_string()));
    }

    #[test]
    fn test_registry_supports_verb() {
        let user_info = RouteInfo::new("user")
            .with_verb("create")
            .with_verb("read");

        let registry = RouteRegistry::new().register("user", Arc::new(MockUser), user_info);

        assert!(registry.supports_verb("user", "create"));
        assert!(registry.supports_verb("user", "read"));
        assert!(!registry.supports_verb("user", "delete"));
        assert!(!registry.supports_verb("product", "create"));
    }

    #[test]
    fn test_registry_get_nouns() {
        let user_info = RouteInfo::new("user").with_verb("create");
        let product_info = RouteInfo::new("product").with_verb("create");
        let order_info = RouteInfo::new("order").with_verb("create");

        let registry = RouteRegistry::new()
            .register("user", Arc::new(MockUser), user_info)
            .register("product", Arc::new(MockProduct), product_info)
            .register("order", Arc::new(MockUser), order_info);

        let nouns = registry.get_nouns();
        assert_eq!(nouns.len(), 3);
        assert!(nouns.contains(&"user".to_string()));
        assert!(nouns.contains(&"product".to_string()));
    }

    #[test]
    fn test_registry_get_verbs() {
        let user_info = RouteInfo::new("user")
            .with_verb("create")
            .with_verb("read")
            .with_verb("update");

        let registry = RouteRegistry::new().register("user", Arc::new(MockUser), user_info);

        let verbs = registry.get_verbs("user");
        assert_eq!(verbs.len(), 3);
        assert!(verbs.contains(&"create".to_string()));
    }

    #[test]
    fn test_registry_find_by_verb() {
        let user_info = RouteInfo::new("user")
            .with_verb("create")
            .with_verb("read");
        let product_info = RouteInfo::new("product")
            .with_verb("create")
            .with_verb("update");

        let registry = RouteRegistry::new()
            .register("user", Arc::new(MockUser), user_info)
            .register("product", Arc::new(MockProduct), product_info);

        let routes = registry.find_by_verb("create");
        assert_eq!(routes.len(), 2);

        let routes = registry.find_by_verb("read");
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].0, "user");
    }

    #[test]
    fn test_registry_unregister() {
        let user_info = RouteInfo::new("user").with_verb("create");
        let registry = RouteRegistry::new().register("user", Arc::new(MockUser), user_info);

        assert!(registry.has_noun("user"));

        let mut registry = registry;
        let removed = registry.unregister("user");
        assert!(removed.is_some());
        assert!(!registry.has_noun("user"));
    }

    #[test]
    fn test_registry_metadata() {
        let user_info = RouteInfo::new("user").with_verb("create");
        let product_info = RouteInfo::new("product").with_verb("create");

        let registry = RouteRegistry::new()
            .register("user", Arc::new(MockUser), user_info)
            .register("product", Arc::new(MockProduct), product_info);

        assert_eq!(registry.len(), 2);
        assert!(!registry.is_empty());

        let empty_registry: RouteRegistry = RouteRegistry::new();
        assert!(empty_registry.is_empty());
    }

    #[test]
    fn test_registry_get_all_routes() {
        let user_info = RouteInfo::new("user")
            .with_verb("create")
            .with_description("User management");
        let product_info = RouteInfo::new("product")
            .with_verb("create")
            .with_description("Product management");

        let registry = RouteRegistry::new()
            .register("user", Arc::new(MockUser), user_info)
            .register("product", Arc::new(MockProduct), product_info);

        let all_routes = registry.get_all_routes();
        assert_eq!(all_routes.len(), 2);
        assert!(all_routes.iter().any(|r| r.noun == "user"));
        assert!(all_routes.iter().any(|r| r.noun == "product"));
    }
}
