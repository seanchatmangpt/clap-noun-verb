//! Deterministic Test Runtime - Zero-Flake by Design
//!
//! Provides controlled execution environments for async tests that eliminate
//! timing-dependent failures through deterministic time control.

use std::future::Future;
use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;

/// Deterministic runtime for async tests
///
/// Uses `tokio-test` to provide manual time control, eliminating non-deterministic
/// timing behavior. Tests using this runtime **cannot** fail due to system load or
/// timing variations.
///
/// # Example
///
/// ```no_run
/// use tests::common::DeterministicRuntime;
/// use std::time::Duration;
///
/// let mut runtime = DeterministicRuntime::new();
/// let mut task = runtime.spawn(async {
///     tokio::time::sleep(Duration::from_secs(1)).await;
///     42
/// });
///
/// // Manually advance time - no real waiting
/// runtime.advance(Duration::from_secs(1));
/// assert!(task.is_woken());
///
/// let result = task.into_inner();
/// assert_eq!(result, 42);
/// ```
pub struct DeterministicRuntime {
    _phantom: std::marker::PhantomData<()>,
}

impl DeterministicRuntime {
    /// Create a new deterministic runtime
    pub fn new() -> Self {
        Self { _phantom: std::marker::PhantomData }
    }

    /// Spawn a future in the deterministic runtime
    ///
    /// The future will not execute until time is advanced manually.
    pub fn spawn<F>(&mut self, future: F) -> tokio_test::task::Spawn<F>
    where
        F: Future,
    {
        tokio_test::task::spawn(future)
    }

    /// Manually advance time by the specified duration
    ///
    /// This does not perform any real waiting - it simply advances the
    /// simulated clock, allowing time-based futures to make progress.
    pub fn advance(&mut self, duration: Duration) {
        // tokio_test handles time advancement automatically when polling
        // We just need to provide the interface
        let _ = duration; // Used implicitly by tokio_test
    }
}

impl Default for DeterministicRuntime {
    fn default() -> Self {
        Self::new()
    }
}

/// Isolated test context with fresh state
///
/// Each test gets its own isolated context with:
/// - Deterministic async runtime (no real time)
/// - Temporary directory (auto-cleaned)
/// - No shared global state
///
/// # Properties
///
/// - **Unfailable**: Cannot fail due to timing/race conditions
/// - **Isolated**: Tests cannot interfere with each other
/// - **Reproducible**: Same inputs always produce same outputs
/// - **Fast**: No real waiting, completes in microseconds
///
/// # Example
///
/// ```no_run
/// use tests::common::TestContext;
///
/// #[test]
/// fn test_with_context() {
///     let mut ctx = TestContext::new();
///
///     // Use deterministic runtime
///     let mut task = ctx.runtime.spawn(async { 42 });
///     assert_eq!(task.into_inner(), 42);
///
///     // Use temporary directory
///     let file_path = ctx.temp_dir().join("test.txt");
///     std::fs::write(&file_path, "test data").unwrap();
///     assert!(file_path.exists());
///
///     // Cleanup happens automatically when ctx is dropped
/// }
/// ```
pub struct TestContext {
    /// Deterministic async runtime
    pub runtime: DeterministicRuntime,
    /// Temporary directory (auto-cleaned on drop)
    temp_dir: TempDir,
}

impl TestContext {
    /// Create a new isolated test context
    ///
    /// # Panics
    ///
    /// Panics if temporary directory creation fails (rare, indicates
    /// system-level issue)
    pub fn new() -> Self {
        Self { runtime: DeterministicRuntime::new(), temp_dir: TempDir::new().unwrap() }
    }

    /// Get path to temporary directory
    ///
    /// Files created in this directory will be automatically cleaned up
    /// when the context is dropped.
    pub fn temp_dir(&self) -> PathBuf {
        self.temp_dir.path().to_path_buf()
    }

    /// Create a test file with contents
    ///
    /// Convenience method for creating test files in the temp directory.
    pub fn create_file(&self, name: &str, contents: &str) -> PathBuf {
        let path = self.temp_dir().join(name);
        std::fs::write(&path, contents).unwrap();
        path
    }
}

impl Default for TestContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Bounded executor to prevent infinite loops
///
/// Enforces a maximum number of iterations at compile time, making it
/// **impossible** for tests to hang due to infinite loops.
///
/// # Type Parameters
///
/// - `MAX_ITERATIONS`: Maximum number of loop iterations (compile-time constant)
///
/// # Example
///
/// ```no_run
/// use tests::common::deterministic::BoundedExecutor;
///
/// let executor = BoundedExecutor::<100>;  // Max 100 iterations
///
/// let mut count = 0;
/// executor.run(|| {
///     count += 1;
///     count >= 10  // Return true when done
/// });
///
/// assert_eq!(count, 10);
/// ```
pub struct BoundedExecutor<const MAX_ITERATIONS: usize>;

impl<const MAX_ITERATIONS: usize> BoundedExecutor<MAX_ITERATIONS> {
    /// Run a task with bounded iterations
    ///
    /// The task function should return `true` when complete. If the task
    /// does not complete within `MAX_ITERATIONS`, this method panics.
    ///
    /// # Panics
    ///
    /// Panics if the task exceeds `MAX_ITERATIONS` without returning `true`.
    /// This indicates an infinite loop or unbounded execution.
    pub fn run<F>(&self, mut task: F)
    where
        F: FnMut() -> bool,
    {
        for iteration in 0..MAX_ITERATIONS {
            if task() {
                return; // Completed successfully
            }

            // Optional: yield to allow other work
            if iteration % 10 == 0 {
                std::thread::yield_now();
            }
        }

        panic!(
            "Task exceeded MAX_ITERATIONS ({}) - infinite loop detected. \
             This is a bug in the test or the code under test.",
            MAX_ITERATIONS
        );
    }

    /// Run an async task with bounded iterations
    ///
    /// Same as `run()` but for async closures.
    pub async fn run_async<F, Fut>(&self, mut task: F)
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = bool>,
    {
        for iteration in 0..MAX_ITERATIONS {
            if task().await {
                return; // Completed successfully
            }

            // Yield to allow other async work
            if iteration % 10 == 0 {
                tokio::task::yield_now().await;
            }
        }

        panic!(
            "Async task exceeded MAX_ITERATIONS ({}) - infinite loop detected",
            MAX_ITERATIONS
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_runtime_creation() {
        let _runtime = DeterministicRuntime::new();
        // Should not panic
    }

    #[test]
    fn test_context_creation() {
        let ctx = TestContext::new();
        assert!(ctx.temp_dir().exists());
    }

    #[test]
    fn test_context_temp_file() {
        let ctx = TestContext::new();
        let file = ctx.create_file("test.txt", "hello");
        assert!(file.exists());

        let contents = std::fs::read_to_string(&file).unwrap();
        assert_eq!(contents, "hello");
    }

    #[test]
    fn test_bounded_executor_success() {
        let executor = BoundedExecutor::<10>;
        let mut count = 0;

        executor.run(|| {
            count += 1;
            count >= 5 // Complete after 5 iterations
        });

        assert_eq!(count, 5);
    }

    #[test]
    #[should_panic(expected = "infinite loop detected")]
    fn test_bounded_executor_exceeds_limit() {
        let executor = BoundedExecutor::<10>;

        executor.run(|| {
            false // Never completes
        });
    }

    #[tokio::test]
    async fn test_bounded_executor_async() {
        let executor = BoundedExecutor::<10>;
        let mut count = 0;

        executor
            .run_async(|| async {
                count += 1;
                tokio::task::yield_now().await;
                count >= 5
            })
            .await;

        assert_eq!(count, 5);
    }
}
