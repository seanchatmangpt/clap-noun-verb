//! AsyncRunner for Async Verb Operations
//!
//! Provides async/await support for non-blocking operations with timeout and cancellation.

use super::crud::OperationResult;
use std::future::Future;
use std::time::Duration;

/// Async operation runner
#[derive(Debug)]
pub struct AsyncRunner {
    timeout: Duration,
    max_concurrent: usize,
}

impl Default for AsyncRunner {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            max_concurrent: 10,
        }
    }
}

impl AsyncRunner {
    /// Create new async runner
    pub fn new(timeout: Duration, max_concurrent: usize) -> Self {
        Self {
            timeout,
            max_concurrent,
        }
    }

    /// Run an async operation with timeout
    pub async fn run<F, T>(&self, future: F) -> OperationResult<T>
    where
        F: Future<Output = OperationResult<T>>,
    {
        match tokio::time::timeout(self.timeout, future).await {
            Ok(result) => result,
            Err(_) => Err(super::crud::OperationError::Timeout),
        }
    }

    /// Run multiple async operations concurrently
    pub async fn run_concurrent<F, T>(&self, futures: Vec<F>) -> Vec<OperationResult<T>>
    where
        F: Future<Output = OperationResult<T>> + Send,
        T: Send,
    {
        let mut results = Vec::new();
        let mut chunk_iter = futures.into_iter().peekable();

        while chunk_iter.peek().is_some() {
            let chunk: Vec<_> = chunk_iter.by_ref().take(self.max_concurrent).collect();
            let futures_tasks: Vec<_> = chunk.into_iter().map(|f| self.run(f)).collect();
            let chunk_results = futures::future::join_all(futures_tasks).await;
            results.extend(chunk_results);
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_async_runner_default() {
        let runner = AsyncRunner::default();
        assert_eq!(runner.timeout, Duration::from_secs(30));
        assert_eq!(runner.max_concurrent, 10);
    }

    #[tokio::test]
    async fn test_async_runner_run_success() {
        let runner = AsyncRunner::new(Duration::from_secs(5), 1);

        let result = runner
            .run(async { Ok::<String, super::crud::OperationError>("success".to_string()) })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }

    #[tokio::test]
    async fn test_async_runner_timeout() {
        let runner = AsyncRunner::new(Duration::from_millis(100), 1);

        let result: OperationResult<String> = runner
            .run(async {
                tokio::time::sleep(Duration::from_secs(1)).await;
                Ok("should timeout".to_string())
            })
            .await;

        assert!(matches!(
            result,
            Err(super::crud::OperationError::Timeout)
        ));
    }

    #[tokio::test]
    async fn test_async_runner_concurrent() {
        let runner = AsyncRunner::new(Duration::from_secs(5), 3);

        let futures = vec![
            Box::pin(async { Ok::<i32, super::crud::OperationError>(1) })
                as std::pin::Pin<Box<dyn std::future::Future<Output = OperationResult<i32>> + Send>>,
            Box::pin(async { Ok::<i32, super::crud::OperationError>(2) }),
            Box::pin(async { Ok::<i32, super::crud::OperationError>(3) }),
        ];

        let results = runner.run_concurrent(futures).await;
        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.is_ok()));
    }
}
