#![cfg_attr(not(feature = "wizard"), allow(dead_code))]
//! Example: Response caching with wizard
//!
//! This example demonstrates LRU caching with TTL expiration
//! to reduce API calls and improve response times.
//!
//! Usage:
//!   cargo run --example wizard_caching --features wizard
//!
//! Environment variables required:
//!   ANTHROPIC_API_KEY=your-api-key

#[cfg(feature = "wizard")]
use clap_noun_verb::wizard::{
    cache::{CacheConfig, CachedClient},
    types::Prompt,
    WizardConfig,
};

#[cfg(feature = "wizard")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Wizard Caching Example ===\n");

    // Load configuration from environment
    let wizard_config = WizardConfig::from_env()?;
    println!("Using model: {:?}\n", wizard_config.model_config.model);

    // Create cached client with custom cache settings
    let cache_config = CacheConfig::new(50, 1800) // 50 entries, 30 min TTL
        .with_max_entries(100)
        .with_ttl(3600) // 1 hour
        .with_stats(true);

    let mut client = CachedClient::new(wizard_config, cache_config).await?;

    // Example 1: First request (cache miss)
    println!("Example 1: First request (cache miss)");
    let prompt = Prompt::new("What is Rust's ownership system?");

    let start = std::time::Instant::now();
    let response1 = client.generate(prompt.clone()).await?;
    let duration1 = start.elapsed();

    println!("Response: {}", response1.text);
    println!("Latency: {:?}", duration1);
    println!("From cache: {}", response1.metadata.from_cache);
    println!("Cache stats: {:?}\n", client.stats());

    // Example 2: Second request (cache hit)
    println!("Example 2: Same request (cache hit)");

    let start = std::time::Instant::now();
    let response2 = client.generate(prompt.clone()).await?;
    let duration2 = start.elapsed();

    println!("Response: {}", response2.text);
    println!("Latency: {:?}", duration2);
    println!("From cache: {}", response2.metadata.from_cache);
    println!("Cache stats: {:?}", client.stats());
    println!("Speedup: {}x faster\n", duration1.as_millis() / duration2.as_millis().max(1));

    // Example 3: Multiple different requests
    println!("Example 3: Multiple different requests");

    let prompts = vec![
        Prompt::new("What is a trait in Rust?"),
        Prompt::new("What is a lifetime in Rust?"),
        Prompt::new("What is a closure in Rust?"),
    ];

    for (i, prompt) in prompts.iter().enumerate() {
        let response = client.generate(prompt.clone()).await?;
        println!("Request {}: {} chars", i + 1, response.text.len());
    }

    println!("\nCache stats after 3 requests: {:?}", client.stats());
    println!("Cache size: {}", client.cache_size());
    println!("Hit rate: {:.2}%\n", client.stats().hit_rate() * 100.0);

    // Example 4: Cache hit on previously seen prompt
    println!("Example 4: Request first prompt again (cache hit)");

    let start = std::time::Instant::now();
    let response = client.generate(prompts[0].clone()).await?;
    let duration = start.elapsed();

    println!("Response: {} chars", response.text.len());
    println!("Latency: {:?}", duration);
    println!("From cache: {}", response.metadata.from_cache);
    println!("Final cache stats: {:?}", client.stats());
    println!("Final hit rate: {:.2}%", client.stats().hit_rate() * 100.0);

    // Example 5: Prune expired entries
    println!("\nExample 5: Cache management");
    println!("Cache size before pruning: {}", client.cache_size());
    client.prune_expired();
    println!("Cache size after pruning: {}", client.cache_size());

    // Clear cache
    client.clear_cache();
    println!("Cache size after clearing: {}", client.cache_size());

    Ok(())
}
