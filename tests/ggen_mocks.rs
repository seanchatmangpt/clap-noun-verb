//! Mock implementation of ggen library for testing
//!
//! This module provides complete mocks of the ggen library's external interfaces
//! to test the clap-noun-verb ggen CLI integration without external dependencies.

use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// Mock ggen API client for AI generation
pub struct MockGgenApi {
    /// API responses queued for returning
    responses: Arc<Mutex<Vec<ApiResponse>>>,
    /// Call history for verification
    call_history: Arc<Mutex<Vec<ApiCall>>>,
}

/// Recorded API call for testing
#[derive(Debug, Clone)]
pub struct ApiCall {
    pub endpoint: String,
    pub params: HashMap<String, String>,
    pub timestamp: u64,
}

/// Mock API response
#[derive(Debug, Clone)]
pub enum ApiResponse {
    GenerateSuccess {
        content: String,
        tokens: u32,
    },
    ProjectSuccess {
        files: Vec<String>,
    },
    GraphSuccess {
        triples: usize,
    },
    SearchSuccess {
        results: Vec<PackageResult>,
    },
    Error {
        code: u32,
        message: String,
    },
}

/// Mock package search result
#[derive(Debug, Clone)]
pub struct PackageResult {
    pub name: String,
    pub version: String,
    pub description: String,
    pub downloads: u64,
}

impl MockGgenApi {
    /// Create new mock API
    pub fn new() -> Self {
        Self {
            responses: Arc::new(Mutex::new(Vec::new())),
            call_history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Queue a response for the next call
    pub fn queue_response(&self, response: ApiResponse) {
        self.responses.lock().unwrap().push(response);
    }

    /// Get next queued response
    pub fn get_response(&self) -> Option<ApiResponse> {
        let mut responses = self.responses.lock().unwrap();
        if !responses.is_empty() {
            Some(responses.remove(0))
        } else {
            None
        }
    }

    /// Record an API call
    pub fn record_call(&self, endpoint: String, params: HashMap<String, String>) {
        let mut history = self.call_history.lock().unwrap();
        history.push(ApiCall {
            endpoint,
            params,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        });
    }

    /// Get call history
    pub fn get_history(&self) -> Vec<ApiCall> {
        self.call_history.lock().unwrap().clone()
    }

    /// Clear history
    pub fn clear_history(&self) {
        self.call_history.lock().unwrap().clear();
    }
}

impl Default for MockGgenApi {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Mock Template Engine
// ============================================================================

/// Mock template engine for rendering
pub struct MockTemplateEngine {
    templates: Arc<Mutex<HashMap<String, String>>>,
    rendered: Arc<Mutex<Vec<RenderedTemplate>>>,
}

/// Rendered template result
#[derive(Debug, Clone)]
pub struct RenderedTemplate {
    pub name: String,
    pub content: String,
    pub variables: HashMap<String, String>,
}

impl MockTemplateEngine {
    /// Create new mock template engine
    pub fn new() -> Self {
        Self {
            templates: Arc::new(Mutex::new(HashMap::new())),
            rendered: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Register a template
    pub fn register_template(&self, name: String, content: String) {
        self.templates.lock().unwrap().insert(name, content);
    }

    /// Render a template
    pub fn render(&self, name: &str, variables: HashMap<String, String>) -> Result<String, String> {
        let templates = self.templates.lock().unwrap();
        match templates.get(name) {
            Some(content) => {
                let mut rendered = content.clone();
                for (key, value) in &variables {
                    rendered = rendered.replace(&format!("{{{{{}}}}}", key), value);
                }

                let mut history = self.rendered.lock().unwrap();
                history.push(RenderedTemplate {
                    name: name.to_string(),
                    content: rendered.clone(),
                    variables: variables.clone(),
                });

                Ok(rendered)
            }
            None => Err(format!("Template '{}' not found", name)),
        }
    }

    /// Get rendered templates
    pub fn get_rendered(&self) -> Vec<RenderedTemplate> {
        self.rendered.lock().unwrap().clone()
    }
}

impl Default for MockTemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Mock Package Registry
// ============================================================================

/// Mock package registry for marketplace
pub struct MockPackageRegistry {
    packages: Arc<Mutex<HashMap<String, MockPackage>>>,
    search_history: Arc<Mutex<Vec<SearchQuery>>>,
}

/// Mock package entry
#[derive(Debug, Clone)]
pub struct MockPackage {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub downloads: u64,
    pub tags: Vec<String>,
}

/// Recorded search query
#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub query: String,
    pub results_count: usize,
}

impl MockPackageRegistry {
    /// Create new mock registry
    pub fn new() -> Self {
        let registry = Self {
            packages: Arc::new(Mutex::new(HashMap::new())),
            search_history: Arc::new(Mutex::new(Vec::new())),
        };
        registry.init_default_packages();
        registry
    }

    /// Initialize with default test packages
    fn init_default_packages(&self) {
        let packages = vec![
            MockPackage {
                id: "io.ggen.rest-api".to_string(),
                name: "REST API Template".to_string(),
                version: "1.0.0".to_string(),
                description: "Complete REST API template".to_string(),
                downloads: 5000,
                tags: vec!["api".to_string(), "rust".to_string()],
            },
            MockPackage {
                id: "io.ggen.web-ui".to_string(),
                name: "Web UI Template".to_string(),
                version: "2.1.0".to_string(),
                description: "React-based web UI".to_string(),
                downloads: 3200,
                tags: vec!["web".to_string(), "react".to_string()],
            },
            MockPackage {
                id: "io.ggen.auth".to_string(),
                name: "Auth Module".to_string(),
                version: "1.5.0".to_string(),
                description: "JWT authentication module".to_string(),
                downloads: 2100,
                tags: vec!["auth".to_string(), "security".to_string()],
            },
        ];

        let mut pkg_map = self.packages.lock().unwrap();
        for pkg in packages {
            pkg_map.insert(pkg.id.clone(), pkg);
        }
    }

    /// Search packages
    pub fn search(&self, query: &str) -> Vec<MockPackage> {
        let packages = self.packages.lock().unwrap();
        let query_lower = query.to_lowercase();

        let results: Vec<_> = packages
            .values()
            .filter(|pkg| {
                // Empty query matches nothing (not all packages)
                if query_lower.is_empty() {
                    false
                } else {
                    pkg.name.to_lowercase().contains(&query_lower)
                        || pkg.description.to_lowercase().contains(&query_lower)
                        || pkg.tags.iter().any(|t| t.to_lowercase().contains(&query_lower))
                }
            })
            .cloned()
            .collect();

        let mut history = self.search_history.lock().unwrap();
        history.push(SearchQuery {
            query: query.to_string(),
            results_count: results.len(),
        });

        results
    }

    /// Get or install package
    pub fn get_package(&self, id: &str) -> Option<MockPackage> {
        self.packages.lock().unwrap().get(id).cloned()
    }

    /// Get search history
    pub fn get_search_history(&self) -> Vec<SearchQuery> {
        self.search_history.lock().unwrap().clone()
    }

    /// Clear search history
    pub fn clear_history(&self) {
        self.search_history.lock().unwrap().clear();
    }
}

impl Default for MockPackageRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Global Mock State for Testing
// ============================================================================

lazy_static::lazy_static! {
    /// Global mock API instance
    pub static ref MOCK_API: MockGgenApi = MockGgenApi::new();

    /// Global mock template engine
    pub static ref MOCK_TEMPLATES: MockTemplateEngine = MockTemplateEngine::new();

    /// Global mock package registry
    pub static ref MOCK_REGISTRY: MockPackageRegistry = MockPackageRegistry::new();
}

/// Reset all mocks to clean state
pub fn reset_mocks() {
    MOCK_API.clear_history();
    MOCK_REGISTRY.clear_history();
    // Note: MOCK_TEMPLATES cannot be reset easily due to lazy_static limitations
    // Each test that needs clean templates should use a new instance
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_api_queue_and_retrieve() {
        let api = MockGgenApi::new();
        api.queue_response(ApiResponse::GenerateSuccess {
            content: "test".to_string(),
            tokens: 100,
        });

        match api.get_response() {
            Some(ApiResponse::GenerateSuccess { tokens, .. }) => assert_eq!(tokens, 100),
            _ => panic!("Expected GenerateSuccess"),
        }
    }

    #[test]
    fn test_mock_template_rendering() {
        let engine = MockTemplateEngine::new();
        engine.register_template(
            "test".to_string(),
            "Hello {{name}}!".to_string(),
        );

        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "World".to_string());

        let result = engine.render("test", vars).unwrap();
        assert_eq!(result, "Hello World!");
    }

    #[test]
    fn test_mock_registry_search() {
        let registry = MockPackageRegistry::new();
        let results = registry.search("api");

        assert!(!results.is_empty());
        assert!(results.iter().any(|p| p.name.contains("API")));
    }

    #[test]
    fn test_mock_registry_get_package() {
        let registry = MockPackageRegistry::new();
        let pkg = registry.get_package("io.ggen.rest-api").unwrap();

        assert_eq!(pkg.name, "REST API Template");
        assert_eq!(pkg.version, "1.0.0");
    }
}
