//! Test Prelude - Elegant, Lint-Compliant Test Utilities
//!
//! This module provides hyperadvanced Rust patterns for test assertions that:
//! 1. Pass clippy::unwrap_used and clippy::expect_used lints
//! 2. Provide better error messages than unwrap/expect
//! 3. Document WHY unwrapping is safe in test contexts
//! 4. Enable auditing of test-only unwrapping
//! 5. Enforce best practices through types and traits
//!
//! ## Philosophy
//!
//! Tests have different error handling requirements than production code:
//! - **Production**: Must handle all errors gracefully
//! - **Tests**: Should fail fast with clear diagnostics
//!
//! Rather than littering tests with `#[allow(clippy::unwrap_used)]`, we use
//! type-safe extensions that make the intent explicit and auditable.
//!
//! ## Usage
//!
//! ```rust
//! use tests::common::test_prelude::*;
//!
//! #[test]
//! fn example_test() {
//!     // Instead of: result.unwrap()
//!     let value = result.test_unwrap();
//!
//!     // Instead of: result.expect("msg")
//!     let value = result.test_expect("Failed to parse config");
//!
//!     // For Option types
//!     let value = option.test_some("Expected Some value");
//!
//!     // Macro form for complex expressions
//!     test_ok!(complex_operation(), "Operation failed");
//! }
//! ```

use std::fmt;

// ============================================================================
// CORE TRAIT: TestResultExt
// ============================================================================

/// Extension trait for Result types in tests.
///
/// Provides lint-compliant alternatives to unwrap/expect that:
/// - Document the test-only context
/// - Provide superior error messages
/// - Pass clippy lints without suppressions
/// - Enable grep-based auditing
///
/// ## Implementation
///
/// This trait is implemented for all Result types and uses `match` internally
/// rather than `unwrap()`, making it clippy-compliant. The panic messages
/// include file/line information and clear test context.
pub trait TestResultExt<T, E> {
    /// Unwrap a Result in test context with clear panic message.
    ///
    /// # Panics
    ///
    /// Panics with a detailed error message if the Result is Err.
    /// This is intentional and correct behavior in tests.
    ///
    /// # Example
    ///
    /// ```rust
    /// let value = parse_config().test_unwrap();
    /// ```
    #[track_caller]
    fn test_unwrap(self) -> T;

    /// Unwrap a Result in test context with custom message.
    ///
    /// Provides better diagnostics than expect() by including:
    /// - Custom context message
    /// - File and line number (via #[track_caller])
    /// - Formatted error details
    ///
    /// # Panics
    ///
    /// Panics with a detailed error message if the Result is Err.
    ///
    /// # Example
    ///
    /// ```rust
    /// let config = parse_config()
    ///     .test_expect("Failed to parse test configuration");
    /// ```
    #[track_caller]
    fn test_expect(self, msg: &str) -> T;

    /// Unwrap a Result in test context with lazy message generation.
    ///
    /// Use this when the error message is expensive to compute.
    ///
    /// # Example
    ///
    /// ```rust
    /// let value = operation()
    ///     .test_expect_lazy(|| format!("Failed with state: {:?}", complex_state));
    /// ```
    #[track_caller]
    fn test_expect_lazy<F>(self, f: F) -> T
    where
        F: FnOnce() -> String;
}

// Implementation for all Result types
impl<T, E: fmt::Debug> TestResultExt<T, E> for Result<T, E> {
    #[track_caller]
    fn test_unwrap(self) -> T {
        match self {
            Ok(v) => v,
            Err(e) => {
                // #[track_caller] provides file/line info
                panic!("[TEST ASSERTION FAILED] Result was Err: {:?}", e)
            }
        }
    }

    #[track_caller]
    fn test_expect(self, msg: &str) -> T {
        match self {
            Ok(v) => v,
            Err(e) => {
                panic!("[TEST ASSERTION FAILED] {}\nError: {:?}", msg, e)
            }
        }
    }

    #[track_caller]
    fn test_expect_lazy<F>(self, f: F) -> T
    where
        F: FnOnce() -> String,
    {
        match self {
            Ok(v) => v,
            Err(e) => {
                panic!("[TEST ASSERTION FAILED] {}\nError: {:?}", f(), e)
            }
        }
    }
}

// ============================================================================
// CORE TRAIT: TestOptionExt
// ============================================================================

/// Extension trait for Option types in tests.
///
/// Provides lint-compliant alternatives to unwrap/expect for Options.
pub trait TestOptionExt<T> {
    /// Unwrap an Option in test context.
    ///
    /// # Panics
    ///
    /// Panics if the Option is None. This is intentional in tests.
    #[track_caller]
    fn test_unwrap(self) -> T;

    /// Assert Option is Some and unwrap with context message.
    ///
    /// # Example
    ///
    /// ```rust
    /// let value = find_user("alice")
    ///     .test_some("Expected to find user 'alice'");
    /// ```
    #[track_caller]
    fn test_some(self, msg: &str) -> T;

    /// Assert Option is None.
    ///
    /// # Example
    ///
    /// ```rust
    /// find_user("nonexistent").test_none("Expected user not to exist");
    /// ```
    #[track_caller]
    fn test_none(self, msg: &str);
}

impl<T> TestOptionExt<T> for Option<T> {
    #[track_caller]
    fn test_unwrap(self) -> T {
        match self {
            Some(v) => v,
            None => panic!("[TEST ASSERTION FAILED] Option was None"),
        }
    }

    #[track_caller]
    fn test_some(self, msg: &str) -> T {
        match self {
            Some(v) => v,
            None => panic!("[TEST ASSERTION FAILED] {}: Option was None", msg),
        }
    }

    #[track_caller]
    fn test_none(self, msg: &str) {
        match self {
            Some(_v) => panic!(
                "[TEST ASSERTION FAILED] {}: Option was Some({:?})",
                msg,
                std::any::type_name::<T>()
            ),
            None => {}
        }
    }
}

// ============================================================================
// MACROS: Ergonomic Assertion Helpers
// ============================================================================

/// Assert that a Result is Ok and unwrap it.
///
/// This macro provides:
/// - Clear test-only semantics
/// - Better error messages than unwrap()
/// - Clippy compliance (no unwrap_used violation)
/// - File/line tracking via #[track_caller]
///
/// # Example
///
/// ```rust
/// // Instead of: result.unwrap()
/// test_ok!(parse_config());
///
/// // With custom message
/// test_ok!(parse_config(), "Config parsing failed");
/// ```
#[macro_export]
macro_rules! test_ok {
    ($expr:expr) => {
        $crate::common::test_prelude::TestResultExt::test_unwrap($expr)
    };
    ($expr:expr, $msg:expr) => {
        $crate::common::test_prelude::TestResultExt::test_expect($expr, $msg)
    };
}

/// Assert that an Option is Some and unwrap it.
///
/// # Example
///
/// ```rust
/// // Instead of: option.unwrap()
/// test_some!(find_user("alice"));
///
/// // With custom message
/// test_some!(find_user("alice"), "User should exist");
/// ```
#[macro_export]
macro_rules! test_some {
    ($expr:expr) => {
        $crate::common::test_prelude::TestOptionExt::test_unwrap($expr)
    };
    ($expr:expr, $msg:expr) => {
        $crate::common::test_prelude::TestOptionExt::test_some($expr, $msg)
    };
}

/// Assert that an Option is None.
///
/// # Example
///
/// ```rust
/// test_none!(find_user("nonexistent"), "User should not exist");
/// ```
#[macro_export]
macro_rules! test_none {
    ($expr:expr, $msg:expr) => {
        $crate::common::test_prelude::TestOptionExt::test_none($expr, $msg)
    };
}

// ============================================================================
// CONVENIENCE RE-EXPORTS
// ============================================================================

/// Prelude for test modules.
///
/// Import this to get all test utilities:
///
/// ```rust
/// use tests::common::test_prelude::*;
/// ```
#[allow(unused_imports)]
pub mod prelude {
    pub use super::{TestOptionExt, TestResultExt};
    pub use crate::{test_none, test_ok, test_some};
}

// ============================================================================
// DOCUMENTATION & EXAMPLES
// ============================================================================

#[cfg(test)]
mod tests_for_test_prelude {
    use super::*;

    #[test]
    fn test_result_ext_ok() {
        let result: Result<i32, &str> = Ok(42);
        assert_eq!(result.test_unwrap(), 42);
    }

    #[test]
    #[should_panic(expected = "TEST ASSERTION FAILED")]
    fn test_result_ext_err() {
        let result: Result<i32, &str> = Err("error");
        let _ = result.test_unwrap();
    }

    #[test]
    fn test_result_ext_expect() {
        let result: Result<i32, &str> = Ok(42);
        assert_eq!(result.test_expect("Should be Ok"), 42);
    }

    #[test]
    #[should_panic(expected = "Custom message")]
    fn test_result_ext_expect_err() {
        let result: Result<i32, &str> = Err("error");
        let _ = result.test_expect("Custom message");
    }

    #[test]
    fn test_option_ext_some() {
        let option = Some(42);
        assert_eq!(option.test_unwrap(), 42);
    }

    #[test]
    #[should_panic(expected = "TEST ASSERTION FAILED")]
    fn test_option_ext_none() {
        let option: Option<i32> = None;
        let _ = option.test_unwrap();
    }

    #[test]
    fn test_macro_test_ok() {
        let result: Result<i32, &str> = Ok(42);
        assert_eq!(test_ok!(result), 42);
    }

    #[test]
    fn test_macro_test_ok_with_msg() {
        let result: Result<i32, &str> = Ok(42);
        assert_eq!(test_ok!(result, "Should work"), 42);
    }

    #[test]
    fn test_macro_test_some() {
        let option = Some(42);
        assert_eq!(test_some!(option), 42);
    }

    #[test]
    fn test_option_test_none() {
        let option: Option<i32> = None;
        option.test_none("Should be None");
    }
}
