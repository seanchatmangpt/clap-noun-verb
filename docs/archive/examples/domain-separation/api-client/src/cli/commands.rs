//! CLI layer - thin wrapper around domain logic
//!
//! Responsibilities:
//! - Parse command-line arguments
//! - Make HTTP requests (I/O)
//! - Convert domain errors to CLI errors
//! - Format output for users

use crate::domain::{self, ApiRequest, ApiResponse, CircuitBreaker, RetryPolicy};
use anyhow::{Context, Result};
use std::time::Duration;

/// Query command - CLI entry point
///
/// This function is THIN - it only handles HTTP I/O and delegates to domain
pub async fn query(
    base_url: String,
    endpoint: String,
    query: String,
    max_results: usize,
) -> Result<()> {
    // Build domain request
    let request = ApiRequest {
        endpoint: endpoint.clone(),
        query: query.clone(),
        max_results,
    };

    // Validate using domain logic
    domain::validate_request(&request)
        .context("Invalid request parameters")?;

    // Set up circuit breaker and retry policy
    let mut circuit_breaker = CircuitBreaker::new(3, 2, Duration::from_secs(30));
    let retry_policy = RetryPolicy::default();

    // Execute request with retries - CLI handles HTTP
    let response = execute_with_retry(
        &base_url,
        &endpoint,
        &query,
        max_results,
        &mut circuit_breaker,
        &retry_policy,
    ).await?;

    // Validate response using domain logic
    domain::validate_response(&response)
        .context("Invalid API response")?;

    // Format output for user - CLI responsibility
    print_results(&response);

    Ok(())
}

/// Execute HTTP request with retry logic - CLI layer responsibility
async fn execute_with_retry(
    base_url: &str,
    endpoint: &str,
    query: &str,
    max_results: usize,
    circuit_breaker: &mut CircuitBreaker,
    retry_policy: &RetryPolicy,
) -> Result<ApiResponse> {
    let client = reqwest::Client::new();
    let url = format!("{}{}", base_url, endpoint);

    let mut attempt = 0;
    let mut delay = retry_policy.initial_delay;

    loop {
        attempt += 1;

        // Check circuit breaker
        if let Err(e) = circuit_breaker.can_request() {
            anyhow::bail!("Circuit breaker open: {}", e);
        }

        // Make HTTP request
        match client
            .get(&url)
            .query(&[("q", query), ("limit", &max_results.to_string())])
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                let response: ApiResponse = resp.json().await?;
                circuit_breaker.record_success();
                return Ok(response);
            }
            Ok(resp) => {
                circuit_breaker.record_failure();
                let status = resp.status();

                if attempt >= retry_policy.max_attempts {
                    anyhow::bail!("Request failed with status: {}", status);
                }

                println!("Retry {}/{} after {} (status: {})",
                    attempt, retry_policy.max_attempts,
                    format_duration(delay), status);
            }
            Err(e) => {
                circuit_breaker.record_failure();

                if attempt >= retry_policy.max_attempts {
                    anyhow::bail!("Request failed: {}", e);
                }

                println!("Retry {}/{} after {} (error: {})",
                    attempt, retry_policy.max_attempts,
                    format_duration(delay), e);
            }
        }

        // Exponential backoff
        tokio::time::sleep(delay).await;
        delay = std::cmp::min(
            Duration::from_secs_f64(delay.as_secs_f64() * retry_policy.backoff_multiplier),
            retry_policy.max_delay,
        );
    }
}

fn format_duration(d: Duration) -> String {
    if d.as_secs() > 0 {
        format!("{}s", d.as_secs())
    } else {
        format!("{}ms", d.as_millis())
    }
}

fn print_results(response: &ApiResponse) {
    println!("✓ Query successful!");
    println!("  Total results: {}", response.total);
    println!("  Returned: {}", response.results.len());
    println!("  Has more: {}", response.has_more);
    println!();

    for (i, item) in response.results.iter().enumerate() {
        println!("{}. {} (score: {:.2})", i + 1, item.title, item.score);
        println!("   ID: {}", item.id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_query_command_success() {
        // Arrange - mock HTTP server
        let mut server = mockito::Server::new_async().await;
        let _m = server.mock("GET", "/search")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("q".into(), "test".into()),
                mockito::Matcher::UrlEncoded("limit".into(), "10".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "results": [
                    {"id": "1", "title": "Test Result", "score": 0.95}
                ],
                "total": 1,
                "has_more": false
            }"#)
            .create_async()
            .await;

        // Act
        let result = query(
            server.url(),
            "/search".to_string(),
            "test".to_string(),
            10,
        ).await;

        // Assert
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_query_command_validation_fails() {
        // Arrange - empty query
        let result = query(
            "http://example.com".to_string(),
            "/search".to_string(),
            "".to_string(),
            10,
        ).await;

        // Assert
        assert!(result.is_err());
        let err_msg = format!("{:#}", result.unwrap_err());
        assert!(err_msg.contains("Invalid request parameters"));
    }
}
