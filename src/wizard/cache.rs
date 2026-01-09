//! Response caching for wizard AI interactions
//!
//! This module provides LRU caching with TTL expiration for AI model responses.
//! It wraps GenAiClient to provide transparent caching with configurable policies.
//!
//! ## Features
//!
//! - LRU cache with configurable size
//! - TTL-based expiration
//! - Cache key generation from prompts
//! - Cache statistics (hits, misses, evictions)
//! - Thread-safe cache access

use crate::wizard::{
    client::GenAiClient,
    config::WizardConfig,
    error::{WizardError, WizardResult},
    types::{Prompt, WizardResponse},
};
use lru::LruCache;
use std::num::NonZeroUsize;
use std::time::{Duration, Instant};

/// Configuration for response caching
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CacheConfig {
    /// Maximum number of entries in the cache
    pub max_entries: usize,
    /// Time-to-live for cache entries (in seconds)
    pub ttl_seconds: u64,
    /// Enable cache statistics tracking
    pub enable_stats: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 100,
            ttl_seconds: 3600, // 1 hour
            enable_stats: true,
        }
    }
}

impl CacheConfig {
    /// Create a new cache configuration
    pub const fn new(max_entries: usize, ttl_seconds: u64) -> Self {
        Self { max_entries, ttl_seconds, enable_stats: true }
    }

    /// Set maximum cache entries
    pub const fn with_max_entries(mut self, max_entries: usize) -> Self {
        self.max_entries = max_entries;
        self
    }

    /// Set TTL in seconds
    pub const fn with_ttl(mut self, ttl_seconds: u64) -> Self {
        self.ttl_seconds = ttl_seconds;
        self
    }

    /// Enable or disable statistics tracking
    pub const fn with_stats(mut self, enable: bool) -> Self {
        self.enable_stats = enable;
        self
    }

    /// Validate the configuration
    pub fn validate(&self) -> WizardResult<()> {
        if self.max_entries == 0 {
            return Err(WizardError::Config("Cache max_entries must be > 0".to_string()));
        }
        if self.ttl_seconds == 0 {
            return Err(WizardError::Config("Cache TTL must be > 0".to_string()));
        }
        Ok(())
    }
}

/// A cached entry with expiration time
#[derive(Debug, Clone)]
struct CacheEntry {
    response: WizardResponse,
    expires_at: Instant,
}

impl CacheEntry {
    fn new(response: WizardResponse, ttl: Duration) -> Self {
        Self { response, expires_at: Instant::now() + ttl }
    }

    fn is_expired(&self) -> bool {
        Instant::now() >= self.expires_at
    }
}

/// Cache statistics
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CacheStats {
    /// Total cache hits
    pub hits: u64,
    /// Total cache misses
    pub misses: u64,
    /// Total cache evictions
    pub evictions: u64,
    /// Total entries added
    pub insertions: u64,
}

impl CacheStats {
    /// Calculate cache hit rate (0.0 to 1.0)
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    /// Get total requests (hits + misses)
    pub const fn total_requests(&self) -> u64 {
        self.hits + self.misses
    }
}

/// Cached client wrapper around GenAiClient
///
/// This struct wraps GenAiClient and provides transparent LRU caching
/// with TTL expiration. Cache hits return immediately without API calls.
pub struct CachedClient {
    /// The underlying GenAI client
    client: GenAiClient,
    /// LRU cache with TTL
    cache: LruCache<String, CacheEntry>,
    /// Cache configuration
    config: CacheConfig,
    /// Cache statistics
    stats: CacheStats,
}

impl CachedClient {
    /// Create a new cached client
    ///
    /// # Errors
    ///
    /// Returns `WizardError::Config` if cache configuration is invalid
    pub async fn new(wizard_config: WizardConfig, cache_config: CacheConfig) -> WizardResult<Self> {
        cache_config.validate()?;

        let client = GenAiClient::new(wizard_config).await?;

        let cache_size = NonZeroUsize::new(cache_config.max_entries)
            .ok_or_else(|| WizardError::Config("Cache size must be non-zero".to_string()))?;

        Ok(Self {
            client,
            cache: LruCache::new(cache_size),
            config: cache_config,
            stats: CacheStats::default(),
        })
    }

    /// Generate a response (with caching)
    ///
    /// # Errors
    ///
    /// Returns `WizardError::Request` if the API request fails
    pub async fn generate(&mut self, prompt: impl Into<Prompt>) -> WizardResult<WizardResponse> {
        let prompt = prompt.into();
        let cache_key = Self::cache_key(&prompt);

        // Check cache first
        if let Some(entry) = self.cache.get(&cache_key) {
            if !entry.is_expired() {
                // Cache hit
                if self.config.enable_stats {
                    self.stats.hits += 1;
                }

                let mut response = entry.response.clone();
                response.metadata.from_cache = true;
                return Ok(response);
            } else {
                // Expired entry, remove it
                self.cache.pop(&cache_key);
                if self.config.enable_stats {
                    self.stats.evictions += 1;
                }
            }
        }

        // Cache miss - generate from client
        if self.config.enable_stats {
            self.stats.misses += 1;
        }

        let response = self.client.generate(prompt.clone()).await?;

        // Store in cache
        let ttl = Duration::from_secs(self.config.ttl_seconds);
        let entry = CacheEntry::new(response.clone(), ttl);

        // Check if we're evicting an entry
        if self.cache.len() >= self.config.max_entries && self.config.enable_stats {
            self.stats.evictions += 1;
        }

        self.cache.put(cache_key, entry);

        if self.config.enable_stats {
            self.stats.insertions += 1;
        }

        Ok(response)
    }

    /// Clear all cache entries
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get cache statistics
    pub const fn stats(&self) -> &CacheStats {
        &self.stats
    }

    /// Get current cache size (number of entries)
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }

    /// Prune expired entries from the cache
    pub fn prune_expired(&mut self) {
        let mut expired_keys = Vec::new();

        // Find expired keys
        for (key, entry) in self.cache.iter() {
            if entry.is_expired() {
                expired_keys.push(key.clone());
            }
        }

        // Remove expired entries
        for key in expired_keys {
            self.cache.pop(&key);
            if self.config.enable_stats {
                self.stats.evictions += 1;
            }
        }
    }

    /// Generate a cache key from a prompt
    ///
    /// Uses ahash for fast, non-cryptographic hashing.
    fn cache_key(prompt: &Prompt) -> String {
        use std::hash::{Hash, Hasher};

        let mut hasher = ahash::AHasher::default();
        prompt.text.hash(&mut hasher);
        prompt.system.hash(&mut hasher);
        for msg in &prompt.history {
            msg.content.hash(&mut hasher);
            format!("{:?}", msg.role).hash(&mut hasher);
        }
        format!("{:016x}", hasher.finish())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_config_default() {
        // Arrange + Act
        let config = CacheConfig::default();

        // Assert
        assert_eq!(config.max_entries, 100);
        assert_eq!(config.ttl_seconds, 3600);
        assert!(config.enable_stats);
    }

    #[test]
    fn test_cache_config_builder() {
        // Arrange + Act
        let config = CacheConfig::new(50, 1800).with_max_entries(200).with_stats(false);

        // Assert
        assert_eq!(config.max_entries, 200);
        assert_eq!(config.ttl_seconds, 1800);
        assert!(!config.enable_stats);
    }

    #[test]
    fn test_cache_config_validation() {
        // Arrange
        let invalid_config = CacheConfig { max_entries: 0, ttl_seconds: 3600, enable_stats: true };

        // Act
        let result = invalid_config.validate();

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn test_cache_stats_hit_rate() {
        // Arrange
        let stats = CacheStats { hits: 75, misses: 25, evictions: 10, insertions: 100 };

        // Act
        let hit_rate = stats.hit_rate();

        // Assert
        assert!((hit_rate - 0.75).abs() < 0.001); // 75/100 = 0.75
    }

    #[test]
    fn test_cache_stats_zero_requests() {
        // Arrange
        let stats = CacheStats::default();

        // Act
        let hit_rate = stats.hit_rate();

        // Assert
        assert_eq!(hit_rate, 0.0);
    }

    #[test]
    fn test_cache_entry_expiration() {
        // Arrange
        let response = WizardResponse::new("test", "model");
        let ttl = Duration::from_millis(10); // 10ms TTL

        // Act
        let entry = CacheEntry::new(response, ttl);

        // Assert - not expired immediately
        assert!(!entry.is_expired());

        // Wait for expiration
        std::thread::sleep(Duration::from_millis(20));
        assert!(entry.is_expired());
    }

    #[test]
    fn test_cache_key_deterministic() {
        // Arrange
        let prompt1 = Prompt::new("Hello, world!");
        let prompt2 = Prompt::new("Hello, world!");

        // Act
        let key1 = CachedClient::cache_key(&prompt1);
        let key2 = CachedClient::cache_key(&prompt2);

        // Assert - same prompts should generate same keys
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_cache_key_different_prompts() {
        // Arrange
        let prompt1 = Prompt::new("Hello");
        let prompt2 = Prompt::new("World");

        // Act
        let key1 = CachedClient::cache_key(&prompt1);
        let key2 = CachedClient::cache_key(&prompt2);

        // Assert - different prompts should generate different keys
        assert_ne!(key1, key2);
    }
}
