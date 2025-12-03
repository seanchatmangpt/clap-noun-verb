# Tutorial 07: Async Operations - Tokio Integration

**Learning Path:** Synchronous CLIs → Async/Await Patterns
**Time:** 25 minutes
**Prerequisites:** [Tutorial 06: Autonomic Features](06-autonomic-features.md)

---

## What You'll Learn

How to build async CLI commands with clap-noun-verb:
- Using `#[async_verb]` macro
- Tokio runtime integration
- Concurrent operations
- Async error handling

---

## Why Async CLIs?

**Synchronous operations block:**
```rust
// ❌ Blocks entire CLI while waiting
fn fetch_data() -> Result<Data> {
    let response = reqwest::blocking::get("https://api.example.com/data")?;
    response.json()
}
```

**Async operations enable concurrency:**
```rust
// ✅ Can run multiple requests concurrently
async fn fetch_data() -> Result<Data> {
    let response = reqwest::get("https://api.example.com/data").await?;
    response.json().await
}
```

**Use async when:**
- Making network requests
- Querying databases
- Processing multiple items concurrently
- Long-running I/O operations

---

## Basic Async Command

### Setup Dependencies

```toml
[dependencies]
clap-noun-verb = "5.2"
clap-noun-verb-macros = "5.2"
tokio = { version = "1.0", features = ["full"] }
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### Simple Async Command

```rust
use clap_noun_verb_macros::async_verb;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct ApiResponse {
    data: String,
}

#[derive(Serialize)]
pub struct FetchResult {
    data: String,
    status: String,
}

#[async_verb(help = "Fetch data from API")]
pub async fn fetch(
    #[arg(help = "API endpoint URL")] url: String,
) -> Result<FetchResult, Box<dyn std::error::Error>> {
    // Async network call
    let response: ApiResponse = reqwest::get(&url)
        .await?
        .json()
        .await?;

    Ok(FetchResult {
        data: response.data,
        status: "success".to_string(),
    })
}
```

**Usage:**
```bash
$ myapp api fetch --url https://api.example.com/data
{"data":"...","status":"success"}
```

---

## Concurrent Operations

Run multiple async operations in parallel:

```rust
use clap_noun_verb_macros::async_verb;
use tokio::try_join;

#[async_verb(help = "Fetch data from multiple sources")]
pub async fn fetch_all(
    #[arg(help = "Comma-separated URLs")] urls: String,
) -> Result<MultiSourceResult, Box<dyn std::error::Error>> {
    let url_list: Vec<&str> = urls.split(',').collect();

    // Launch all requests concurrently
    let futures: Vec<_> = url_list
        .iter()
        .map(|url| async move {
            let response: ApiResponse = reqwest::get(*url)
                .await?
                .json()
                .await?;
            Ok::<_, Box<dyn std::error::Error>>(response.data)
        })
        .collect();

    // Wait for all to complete
    let results = futures::future::try_join_all(futures).await?;

    Ok(MultiSourceResult {
        sources: url_list.len(),
        results,
        status: "success".to_string(),
    })
}
```

**Usage:**
```bash
$ myapp api fetch-all --urls "https://api1.com/data,https://api2.com/data,https://api3.com/data"
{
  "sources": 3,
  "results": ["data1", "data2", "data3"],
  "status": "success"
}
```

---

## Async Database Operations

```rust
use clap_noun_verb_macros::async_verb;
use sqlx::{PgPool, FromRow};
use serde::Serialize;

#[derive(FromRow, Serialize)]
pub struct User {
    id: i64,
    username: String,
    email: String,
}

#[async_verb(help = "Query users from database")]
pub async fn list_users(
    #[arg(env = "DATABASE_URL", help = "Database connection string")]
    database_url: String,
) -> Result<UserListResult, Box<dyn std::error::Error>> {
    // Connect to database
    let pool = PgPool::connect(&database_url).await?;

    // Async query
    let users: Vec<User> = sqlx::query_as("SELECT id, username, email FROM users")
        .fetch_all(&pool)
        .await?;

    Ok(UserListResult {
        users,
        count: users.len(),
    })
}
```

**Dependencies:**
```toml
[dependencies]
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-native-tls"] }
```

---

## Timeout and Cancellation

Add timeouts to prevent hanging:

```rust
use clap_noun_verb_macros::async_verb;
use tokio::time::{timeout, Duration};

#[async_verb(help = "Fetch with timeout")]
pub async fn fetch_with_timeout(
    #[arg(help = "API URL")] url: String,
    #[arg(help = "Timeout in seconds", default = "30")] timeout_secs: u64,
) -> Result<FetchResult, Box<dyn std::error::Error>> {
    // Wrap async operation in timeout
    let result = timeout(
        Duration::from_secs(timeout_secs),
        async {
            let response: ApiResponse = reqwest::get(&url)
                .await?
                .json()
                .await?;
            Ok::<_, Box<dyn std::error::Error>>(response)
        }
    ).await;

    match result {
        Ok(Ok(response)) => Ok(FetchResult {
            data: response.data,
            status: "success".to_string(),
        }),
        Ok(Err(e)) => Err(e),
        Err(_) => Err(format!("Request timed out after {} seconds", timeout_secs).into()),
    }
}
```

---

## Streaming Responses

Stream large datasets:

```rust
use clap_noun_verb_macros::async_verb;
use tokio_stream::StreamExt;

#[async_verb(help = "Stream large dataset")]
pub async fn stream_data(
    #[arg(help = "Data source URL")] url: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client.get(&url).send().await?;

    // Stream response body
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        // Process chunk
        let record: DataRecord = serde_json::from_slice(&chunk)?;
        println!("{}", serde_json::to_string(&record)?);
    }

    Ok(())
}
```

---

## Error Handling

Handle async errors properly:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AsyncCommandError {
    #[error("Network request failed: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Timeout after {0} seconds")]
    Timeout(u64),

    #[error("Invalid response format")]
    InvalidFormat,
}

#[async_verb(help = "Fetch with proper error handling")]
pub async fn fetch_safe(
    #[arg(help = "API URL")] url: String,
) -> Result<FetchResult, AsyncCommandError> {
    let response = reqwest::get(&url)
        .await
        .map_err(AsyncCommandError::NetworkError)?;

    if !response.status().is_success() {
        return Err(AsyncCommandError::InvalidFormat);
    }

    let data: ApiResponse = response
        .json()
        .await
        .map_err(AsyncCommandError::NetworkError)?;

    Ok(FetchResult {
        data: data.data,
        status: "success".to_string(),
    })
}
```

---

## Exercise: Build Async Health Checker

**Goal:** Create a service health checker that pings multiple endpoints concurrently

**Arrange:** Define domain logic

```rust
// domain/health.rs
use tokio::time::{timeout, Duration};

pub struct HealthCheck {
    pub endpoint: String,
    pub healthy: bool,
    pub response_time_ms: u64,
}

pub async fn check_health(endpoint: &str) -> Result<HealthCheck, HealthError> {
    let start = std::time::Instant::now();

    let result = timeout(
        Duration::from_secs(5),
        reqwest::get(endpoint)
    ).await;

    let response_time_ms = start.elapsed().as_millis() as u64;

    match result {
        Ok(Ok(response)) => Ok(HealthCheck {
            endpoint: endpoint.to_string(),
            healthy: response.status().is_success(),
            response_time_ms,
        }),
        Ok(Err(_)) | Err(_) => Ok(HealthCheck {
            endpoint: endpoint.to_string(),
            healthy: false,
            response_time_ms,
        }),
    }
}
```

**Act:** Create async CLI command

```rust
// commands/health.rs
use clap_noun_verb_macros::async_verb;
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthReport {
    total_endpoints: usize,
    healthy_count: usize,
    unhealthy_count: usize,
    checks: Vec<HealthCheckResult>,
}

#[derive(Serialize)]
pub struct HealthCheckResult {
    endpoint: String,
    healthy: bool,
    response_time_ms: u64,
}

#[async_verb(help = "Check health of multiple services")]
pub async fn check(
    #[arg(help = "Comma-separated service URLs")] endpoints: String,
) -> Result<HealthReport, Box<dyn std::error::Error>> {
    let endpoint_list: Vec<&str> = endpoints.split(',').collect();

    // Check all endpoints concurrently
    let futures: Vec<_> = endpoint_list
        .iter()
        .map(|endpoint| crate::domain::health::check_health(endpoint))
        .collect();

    let results = futures::future::join_all(futures).await;

    let checks: Vec<HealthCheckResult> = results
        .into_iter()
        .filter_map(|r| r.ok())
        .map(|check| HealthCheckResult {
            endpoint: check.endpoint,
            healthy: check.healthy,
            response_time_ms: check.response_time_ms,
        })
        .collect();

    let healthy_count = checks.iter().filter(|c| c.healthy).count();
    let unhealthy_count = checks.len() - healthy_count;

    Ok(HealthReport {
        total_endpoints: checks.len(),
        healthy_count,
        unhealthy_count,
        checks,
    })
}
```

**Assert:** Test concurrent health checks

```bash
$ myapp health check --endpoints "https://api.example.com/health,https://db.example.com/ping,https://cache.example.com/status"
{
  "total_endpoints": 3,
  "healthy_count": 2,
  "unhealthy_count": 1,
  "checks": [
    {
      "endpoint": "https://api.example.com/health",
      "healthy": true,
      "response_time_ms": 45
    },
    {
      "endpoint": "https://db.example.com/ping",
      "healthy": true,
      "response_time_ms": 12
    },
    {
      "endpoint": "https://cache.example.com/status",
      "healthy": false,
      "response_time_ms": 5001
    }
  ]
}
```

---

## Testing Async Commands

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_success() {
        // Arrange
        let url = "https://api.example.com/data".to_string();

        // Act
        let result = fetch(url).await;

        // Assert
        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.status, "success");
    }

    #[tokio::test]
    async fn test_fetch_timeout() {
        // Arrange
        let url = "https://slow.example.com/data".to_string();

        // Act
        let result = fetch_with_timeout(url, 1).await;

        // Assert
        assert!(result.is_err());
    }
}
```

---

## Key Takeaways

✅ **`#[async_verb]`** - Async command support
✅ **Tokio runtime** - Async I/O execution
✅ **Concurrent operations** - Run multiple tasks in parallel
✅ **Timeout handling** - Prevent hanging operations
✅ **Error handling** - Proper async error propagation

---

## Next Steps

- **[Tutorial 08: Error Handling](08-error-handling.md)** - Comprehensive error strategies
- **[Tutorial 09: Deployment Basics](09-deployment-basics.md)** - Production deployment
- **[How-To: Async Patterns](../howto/advanced/async-patterns.md)** - Advanced async techniques

**Estimated time to next tutorial:** 20 minutes

---

*Part of the [clap-noun-verb Tutorial Series](README.md) - Learning-oriented documentation*
