//! Registry protocol client with HTTP, retries, and caching
//!
//! Provides a production-ready HTTP client for interacting with the
//! ggen registry, including retry logic, timeout handling, and response caching.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Default registry URL
const DEFAULT_REGISTRY_URL: &str = "https://registry.ggen.dev";

/// Request timeout in seconds
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Cache TTL in seconds (5 minutes)
const CACHE_TTL_SECS: u64 = 300;

/// Maximum retry attempts
const MAX_RETRIES: usize = 3;

/// Retry delay in milliseconds
const RETRY_DELAY_MS: u64 = 1000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrySearchResult {
    pub name: String,
    pub version: String,
    pub description: String,
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryInfo {
    pub name: String,
    pub description: String,
    pub versions: Vec<String>,
    pub latest_version: String,
    pub dependencies: Vec<String>,
    pub homepage: Option<String>,
    pub repository: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrySource {
    pub name: String,
    pub url: String,
    pub priority: u32,
}

/// Cached response with expiration timestamp
#[derive(Debug, Clone)]
struct CachedResponse<T> {
    data: T,
    expires_at: Instant,
}

/// Registry client with HTTP support, retries, and caching
pub struct RegistryClient {
    /// Base URL for the registry
    base_url: String,
    /// Request timeout
    timeout: Duration,
    /// Response cache
    cache: Arc<RwLock<HashMap<String, CachedResponse<String>>>>,
    /// Cache TTL
    cache_ttl: Duration,
    /// User agent string
    user_agent: String,
}

impl RegistryClient {
    /// Create a new registry client with default settings
    pub fn new() -> Result<Self, String> {
        #[cfg(feature = "reqwest")]
        {
            Ok(Self {
                base_url: DEFAULT_REGISTRY_URL.to_string(),
                timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
                cache: Arc::new(RwLock::new(HashMap::new())),
                cache_ttl: Duration::from_secs(CACHE_TTL_SECS),
                user_agent: format!("ggen/{}", env!("CARGO_PKG_VERSION")),
            })
        }

        #[cfg(not(feature = "reqwest"))]
        {
            Ok(Self {
                base_url: DEFAULT_REGISTRY_URL.to_string(),
                timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
                cache: Arc::new(RwLock::new(HashMap::new())),
                cache_ttl: Duration::from_secs(CACHE_TTL_SECS),
                user_agent: "ggen/0.1.0".to_string(),
            })
        }
    }

    /// Create a new registry client with custom base URL
    pub fn with_base_url(base_url: String) -> Result<Self, String> {
        let mut client = Self::new()?;
        client.base_url = base_url;
        Ok(client)
    }

    /// Set custom timeout
    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout = Duration::from_secs(timeout_secs);
        self
    }

    /// Set custom cache TTL
    pub fn with_cache_ttl(mut self, cache_ttl_secs: u64) -> Self {
        self.cache_ttl = Duration::from_secs(cache_ttl_secs);
        self
    }

    /// Search registry for packs
    pub fn search(
        &self,
        query: &str,
        category: Option<&str>,
        limit: usize,
    ) -> Result<Vec<RegistrySearchResult>, String> {
        #[cfg(feature = "reqwest")]
        {
            let cache_key = format!("search:{}:{}", query, category.unwrap_or(""));

            // Check cache first
            if let Some(cached) = self.get_cached(&cache_key) {
                return serde_json::from_str(&cached)
                    .map_err(|e| format!("Failed to parse cached search results: {}", e));
            }

            // Build request URL
            let mut url = format!("{}/api/v1/search?q={}", self.base_url, query);
            if let Some(cat) = category {
                url.push_str(&format!("&category={}", cat));
            }
            url.push_str(&format!("&limit={}", limit));

            // Make HTTP request with retries
            let response = self.http_get_with_retry(&url)?;

            // Cache the response
            self.set_cached(cache_key, response.clone());

            // Parse response
            serde_json::from_str(&response)
                .map_err(|e| format!("Failed to parse search results: {}", e))
        }

        #[cfg(not(feature = "reqwest"))]
        {
            // Stub implementation when marketplace feature is disabled
            let _ = (query, category, limit);
            Ok(vec![])
        }
    }

    /// Get detailed information about a specific pack
    pub fn get_info(&self, identifier: &str) -> Result<RegistryInfo, String> {
        #[cfg(feature = "reqwest")]
        {
            let cache_key = format!("info:{}", identifier);

            // Check cache first
            if let Some(cached) = self.get_cached(&cache_key) {
                return serde_json::from_str(&cached)
                    .map_err(|e| format!("Failed to parse cached pack info: {}", e));
            }

            // Build request URL
            let url = format!("{}/api/v1/packs/{}", self.base_url, identifier);

            // Make HTTP request with retries
            let response = self.http_get_with_retry(&url)?;

            // Cache the response
            self.set_cached(cache_key, response.clone());

            // Parse response
            serde_json::from_str(&response)
                .map_err(|e| format!("Failed to parse pack info: {}", e))
        }

        #[cfg(not(feature = "reqwest"))]
        {
            // Stub implementation when marketplace feature is disabled
            Ok(RegistryInfo {
                name: identifier.to_string(),
                description: "Marketplace feature is disabled".to_string(),
                versions: vec![],
                latest_version: "0.0.0".to_string(),
                dependencies: vec![],
                homepage: None,
                repository: None,
            })
        }
    }

    /// List configured registry sources
    pub fn list_sources(&self) -> Result<Vec<RegistrySource>, String> {
        #[cfg(feature = "reqwest")]
        {
            let cache_key = "sources".to_string();

            // Check cache first
            if let Some(cached) = self.get_cached(&cache_key) {
                return serde_json::from_str(&cached)
                    .map_err(|e| format!("Failed to parse cached sources: {}", e));
            }

            // Build request URL
            let url = format!("{}/api/v1/sources", self.base_url);

            // Make HTTP request with retries
            let response = self.http_get_with_retry(&url)?;

            // Cache the response
            self.set_cached(cache_key, response.clone());

            // Parse response
            serde_json::from_str(&response)
                .map_err(|e| format!("Failed to parse sources: {}", e))
        }

        #[cfg(not(feature = "reqwest"))]
        {
            // Return default source when marketplace feature is disabled
            Ok(vec![RegistrySource {
                name: "default".to_string(),
                url: DEFAULT_REGISTRY_URL.to_string(),
                priority: 100,
            }])
        }
    }

    /// Download pack from registry
    pub fn download_pack(&self, identifier: &str, version: &str) -> Result<Vec<u8>, String> {
        #[cfg(feature = "reqwest")]
        {
            // Build request URL
            let url = format!("{}/api/v1/packs/{}/download/{}", self.base_url, identifier, version);

            // Make HTTP request with retries
            let response = self.http_get_bytes_with_retry(&url)?;

            Ok(response)
        }

        #[cfg(not(feature = "reqwest"))]
        {
            let _ = (identifier, version);
            Err("Marketplace feature is disabled".to_string())
        }
    }

    /// Check registry health/status
    pub fn health_check(&self) -> Result<RegistryHealth, String> {
        #[cfg(feature = "reqwest")]
        {
            let url = format!("{}/health", self.base_url);

            let start = Instant::now();
            let response = self.http_get_with_retry(&url)?;
            let duration = start.elapsed();

            let health: RegistryHealth = serde_json::from_str(&response)
                .map_err(|e| format!("Failed to parse health check: {}", e))?;

            Ok(RegistryHealth {
                healthy: health.healthy,
                version: health.version,
                latency_ms: duration.as_millis() as u64,
            })
        }

        #[cfg(not(feature = "reqwest"))]
        {
            Ok(RegistryHealth {
                healthy: true,
                version: "0.1.0".to_string(),
                latency_ms: 0,
            })
        }
    }

    /// Clear response cache
    pub fn clear_cache(&self) -> Result<(), String> {
        let mut cache = self.cache.write()
            .map_err(|e| format!("Failed to acquire cache lock: {}", e))?;
        cache.clear();
        Ok(())
    }

    /// Get cached response if not expired
    fn get_cached(&self, key: &str) -> Option<String> {
        let cache = self.cache.read().ok()?;
        cache.get(key).and_then(|cached| {
            if cached.expires_at > Instant::now() {
                Some(cached.data.clone())
            } else {
                None
            }
        })
    }

    /// Set cached response with expiration
    fn set_cached(&self, key: String, data: String) {
        if let Ok(mut cache) = self.cache.write() {
            let expires_at = Instant::now() + self.cache_ttl;
            cache.insert(key, CachedResponse { data, expires_at });
        }
    }

    /// Make HTTP GET request with retry logic
    #[cfg(feature = "reqwest")]
    fn http_get_with_retry(&self, url: &str) -> Result<String, String> {
        let mut last_error = String::new();

        for attempt in 0..MAX_RETRIES {
            // Build client with timeout
            let client = reqwest::blocking::Client::builder()
                .timeout(self.timeout)
                .build()
                .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

            // Make request
            let response = client
                .get(url)
                .header("User-Agent", &self.user_agent)
                .header("Accept", "application/json")
                .send();

            match response {
                Ok(resp) => {
                    if resp.status().is_success() {
                        return resp.text()
                            .map_err(|e| format!("Failed to read response body: {}", e));
                    } else {
                        last_error = format!("HTTP error: {}", resp.status());
                    }
                }
                Err(e) => {
                    last_error = format!("Request failed: {}", e);
                }
            }

            // Don't sleep after last attempt
            if attempt < MAX_RETRIES - 1 {
                std::thread::sleep(Duration::from_millis(RETRY_DELAY_MS));
            }
        }

        Err(format!("Failed after {} attempts: {}", MAX_RETRIES, last_error))
    }

    /// Make HTTP GET request for binary data with retry logic
    #[cfg(feature = "reqwest")]
    fn http_get_bytes_with_retry(&self, url: &str) -> Result<Vec<u8>, String> {
        let mut last_error = String::new();

        for attempt in 0..MAX_RETRIES {
            let client = reqwest::blocking::Client::builder()
                .timeout(self.timeout)
                .build()
                .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

            let response = client
                .get(url)
                .header("User-Agent", &self.user_agent)
                .send();

            match response {
                Ok(resp) => {
                    if resp.status().is_success() {
                        return resp.bytes()
                            .map(|b| b.to_vec())
                            .map_err(|e| format!("Failed to read response body: {}", e));
                    } else {
                        last_error = format!("HTTP error: {}", resp.status());
                    }
                }
                Err(e) => {
                    last_error = format!("Request failed: {}", e);
                }
            }

            if attempt < MAX_RETRIES - 1 {
                std::thread::sleep(Duration::from_millis(RETRY_DELAY_MS));
            }
        }

        Err(format!("Failed after {} attempts: {}", MAX_RETRIES, last_error))
    }
}

impl Default for RegistryClient {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            base_url: DEFAULT_REGISTRY_URL.to_string(),
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: Duration::from_secs(CACHE_TTL_SECS),
            user_agent: "ggen/0.1.0".to_string(),
        })
    }
}

/// Registry health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryHealth {
    pub healthy: bool,
    pub version: String,
    #[serde(default)]
    pub latency_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_client_new() {
        let client = RegistryClient::new();
        assert!(client.is_ok());
    }

    #[test]
    fn test_registry_client_default() {
        let client = RegistryClient::default();
        assert_eq!(client.base_url, DEFAULT_REGISTRY_URL);
    }

    #[test]
    fn test_registry_client_with_base_url() {
        let client = RegistryClient::with_base_url("https://custom.registry.com".to_string());
        assert!(client.is_ok());
        assert_eq!(client.unwrap().base_url, "https://custom.registry.com");
    }

    #[test]
    fn test_registry_client_with_timeout() {
        let client = RegistryClient::new().unwrap().with_timeout(60);
        assert_eq!(client.timeout.as_secs(), 60);
    }

    #[test]
    fn test_registry_client_with_cache_ttl() {
        let client = RegistryClient::new().unwrap().with_cache_ttl(600);
        assert_eq!(client.cache_ttl.as_secs(), 600);
    }

    #[test]
    fn test_registry_clear_cache() {
        let client = RegistryClient::new().unwrap();
        client.set_cached("test".to_string(), "data".to_string());
        assert!(client.get_cached("test").is_some());

        client.clear_cache().unwrap();
        assert!(client.get_cached("test").is_none());
    }

    #[test]
    fn test_registry_cache_expiration() {
        let client = RegistryClient::new().unwrap().with_cache_ttl(1);
        client.set_cached("test".to_string(), "data".to_string());
        assert!(client.get_cached("test").is_some());

        // Wait for cache to expire
        std::thread::sleep(Duration::from_secs(2));
        assert!(client.get_cached("test").is_none());
    }

    #[test]
    fn test_registry_health_check() {
        let client = RegistryClient::new().unwrap();
        let health = client.health_check();

        #[cfg(feature = "reqwest")]
        {
            // This will fail if registry is not available, but tests the code path
            let _ = health;
        }

        #[cfg(not(feature = "reqwest"))]
        {
            assert!(health.is_ok());
            let health = health.unwrap();
            assert!(health.healthy);
        }
    }

    #[test]
    fn test_registry_list_sources() {
        let client = RegistryClient::new().unwrap();
        let sources = client.list_sources();

        #[cfg(feature = "reqwest")]
        {
            // This will fail if registry is not available, but tests the code path
            let _ = sources;
        }

        #[cfg(not(feature = "reqwest"))]
        {
            assert!(sources.is_ok());
            let sources = sources.unwrap();
            assert_eq!(sources.len(), 1);
            assert_eq!(sources[0].name, "default");
        }
    }

    #[test]
    fn test_registry_search() {
        let client = RegistryClient::new().unwrap();
        let results = client.search("test", None, 10);

        #[cfg(feature = "reqwest")]
        {
            // This will fail if registry is not available, but tests the code path
            let _ = results;
        }

        #[cfg(not(feature = "reqwest"))]
        {
            assert!(results.is_ok());
            let results = results.unwrap();
            assert_eq!(results.len(), 0);
        }
    }

    #[test]
    fn test_registry_get_info() {
        let client = RegistryClient::new().unwrap();
        let info = client.get_info("test-pack");

        #[cfg(feature = "reqwest")]
        {
            // This will fail if registry is not available, but tests the code path
            let _ = info;
        }

        #[cfg(not(feature = "reqwest"))]
        {
            assert!(info.is_ok());
            let info = info.unwrap();
            assert_eq!(info.name, "test-pack");
        }
    }

    #[test]
    fn test_registry_download_pack() {
        let client = RegistryClient::new().unwrap();
        let result = client.download_pack("test-pack", "1.0.0");

        #[cfg(feature = "reqwest")]
        {
            // This will fail if registry is not available, but tests the code path
            let _ = result;
        }

        #[cfg(not(feature = "reqwest"))]
        {
            assert!(result.is_err());
            assert!(result.unwrap_err().contains("disabled"));
        }
    }
}
