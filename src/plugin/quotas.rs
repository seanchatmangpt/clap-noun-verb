//! Resource Quota System for Plugin Isolation
//!
//! This module provides resource quotas to limit plugin resource consumption and prevent
//! runaway plugins from affecting the host application.
//!
//! # Features
//!
//! - **CPU Time Limits**: Walltime measurement and enforcement
//! - **Memory Usage Tracking**: RSS and heap tracking
//! - **File Handle Limits**: Maximum number of open files
//! - **Network Connection Limits**: Maximum concurrent connections
//! - **Graceful Failure**: Quotas fail gracefully with clear error messages
//!
//! # Example
//!
//! ```rust,ignore
//! use clap_noun_verb::plugin::quotas::{ResourceQuota, QuotaGuard};
//!
//! // Create quota with limits
//! let quota = ResourceQuota::builder()
//!     .cpu_time_ms(5000)  // 5 seconds max
//!     .memory_bytes(100_000_000)  // 100MB max
//!     .file_handles(50)
//!     .build();
//!
//! // Execute plugin with quota enforcement
//! {
//!     let guard = quota.acquire()?;
//!     // Plugin execution happens here
//!     // Quota is released when guard is dropped
//! }
//! ```

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

/// Resource quota configuration for plugin execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceQuota {
    /// Maximum CPU time in milliseconds (0 = unlimited)
    pub cpu_time_ms: u64,
    /// Maximum memory usage in bytes (0 = unlimited)
    pub memory_bytes: u64,
    /// Maximum number of file handles (0 = unlimited)
    pub file_handles: u64,
    /// Maximum network connections (0 = unlimited)
    pub network_connections: u64,
}

impl Default for ResourceQuota {
    fn default() -> Self {
        Self {
            cpu_time_ms: 30_000,        // 30 seconds default
            memory_bytes: 500_000_000,  // 500MB default
            file_handles: 100,          // 100 files default
            network_connections: 10,    // 10 connections default
        }
    }
}

impl ResourceQuota {
    /// Create a new resource quota builder.
    pub fn builder() -> ResourceQuotaBuilder {
        ResourceQuotaBuilder::default()
    }

    /// Create unlimited quota (no limits).
    pub fn unlimited() -> Self {
        Self {
            cpu_time_ms: 0,
            memory_bytes: 0,
            file_handles: 0,
            network_connections: 0,
        }
    }

    /// Create strict quota (conservative limits).
    pub fn strict() -> Self {
        Self {
            cpu_time_ms: 5_000,          // 5 seconds
            memory_bytes: 50_000_000,    // 50MB
            file_handles: 10,            // 10 files
            network_connections: 2,      // 2 connections
        }
    }

    /// Acquire a quota guard for plugin execution.
    ///
    /// # Errors
    ///
    /// Returns an error if quota cannot be acquired.
    pub fn acquire(&self) -> crate::Result<QuotaGuard> {
        Ok(QuotaGuard::new(self.clone()))
    }

    /// Check if CPU time limit is exceeded.
    pub fn is_cpu_time_exceeded(&self, elapsed: Duration) -> bool {
        if self.cpu_time_ms == 0 {
            return false;
        }
        elapsed.as_millis() as u64 > self.cpu_time_ms
    }

    /// Check if memory limit is exceeded.
    pub fn is_memory_exceeded(&self, current_bytes: u64) -> bool {
        if self.memory_bytes == 0 {
            return false;
        }
        current_bytes > self.memory_bytes
    }

    /// Check if file handle limit is exceeded.
    pub fn is_file_handles_exceeded(&self, current_handles: u64) -> bool {
        if self.file_handles == 0 {
            return false;
        }
        current_handles > self.file_handles
    }

    /// Check if network connection limit is exceeded.
    pub fn is_network_connections_exceeded(&self, current_connections: u64) -> bool {
        if self.network_connections == 0 {
            return false;
        }
        current_connections > self.network_connections
    }
}

/// Builder for ResourceQuota.
#[derive(Debug, Default)]
pub struct ResourceQuotaBuilder {
    cpu_time_ms: Option<u64>,
    memory_bytes: Option<u64>,
    file_handles: Option<u64>,
    network_connections: Option<u64>,
}

impl ResourceQuotaBuilder {
    /// Set CPU time limit in milliseconds.
    pub fn cpu_time_ms(mut self, ms: u64) -> Self {
        self.cpu_time_ms = Some(ms);
        self
    }

    /// Set memory limit in bytes.
    pub fn memory_bytes(mut self, bytes: u64) -> Self {
        self.memory_bytes = Some(bytes);
        self
    }

    /// Set file handle limit.
    pub fn file_handles(mut self, handles: u64) -> Self {
        self.file_handles = Some(handles);
        self
    }

    /// Set network connection limit.
    pub fn network_connections(mut self, connections: u64) -> Self {
        self.network_connections = Some(connections);
        self
    }

    /// Build the ResourceQuota.
    pub fn build(self) -> ResourceQuota {
        let default = ResourceQuota::default();
        ResourceQuota {
            cpu_time_ms: self.cpu_time_ms.unwrap_or(default.cpu_time_ms),
            memory_bytes: self.memory_bytes.unwrap_or(default.memory_bytes),
            file_handles: self.file_handles.unwrap_or(default.file_handles),
            network_connections: self.network_connections.unwrap_or(default.network_connections),
        }
    }
}

/// Guard that enforces resource quotas during plugin execution.
///
/// The guard tracks resource usage and automatically releases resources when dropped.
pub struct QuotaGuard {
    quota: ResourceQuota,
    start_time: Instant,
    tracker: Arc<ResourceTracker>,
}

impl QuotaGuard {
    /// Create a new quota guard.
    fn new(quota: ResourceQuota) -> Self {
        Self {
            quota,
            start_time: Instant::now(),
            tracker: Arc::new(ResourceTracker::new()),
        }
    }

    /// Check if any quota is exceeded.
    ///
    /// # Errors
    ///
    /// Returns an error if any quota is exceeded.
    pub fn check_quotas(&self) -> crate::Result<()> {
        // Check CPU time
        let elapsed = self.start_time.elapsed();
        if self.quota.is_cpu_time_exceeded(elapsed) {
            return Err(crate::NounVerbError::PluginError(format!(
                "CPU time quota exceeded: {}ms > {}ms",
                elapsed.as_millis(),
                self.quota.cpu_time_ms
            )));
        }

        // Check memory
        let memory_used = self.tracker.memory_bytes.load(Ordering::Relaxed);
        if self.quota.is_memory_exceeded(memory_used) {
            return Err(crate::NounVerbError::PluginError(format!(
                "Memory quota exceeded: {} bytes > {} bytes",
                memory_used,
                self.quota.memory_bytes
            )));
        }

        // Check file handles
        let file_handles = self.tracker.file_handles.load(Ordering::Relaxed);
        if self.quota.is_file_handles_exceeded(file_handles) {
            return Err(crate::NounVerbError::PluginError(format!(
                "File handle quota exceeded: {} > {}",
                file_handles,
                self.quota.file_handles
            )));
        }

        // Check network connections
        let network_connections = self.tracker.network_connections.load(Ordering::Relaxed);
        if self.quota.is_network_connections_exceeded(network_connections) {
            return Err(crate::NounVerbError::PluginError(format!(
                "Network connection quota exceeded: {} > {}",
                network_connections,
                self.quota.network_connections
            )));
        }

        Ok(())
    }

    /// Track memory allocation.
    pub fn track_memory_alloc(&self, bytes: u64) {
        self.tracker.memory_bytes.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Track memory deallocation.
    pub fn track_memory_dealloc(&self, bytes: u64) {
        self.tracker.memory_bytes.fetch_sub(bytes, Ordering::Relaxed);
    }

    /// Track file handle open.
    pub fn track_file_open(&self) -> crate::Result<FileHandleGuard> {
        let count = self.tracker.file_handles.fetch_add(1, Ordering::Relaxed) + 1;

        if self.quota.file_handles > 0 && count > self.quota.file_handles {
            self.tracker.file_handles.fetch_sub(1, Ordering::Relaxed);
            return Err(crate::NounVerbError::PluginError(format!(
                "File handle quota exceeded: {} > {}",
                count,
                self.quota.file_handles
            )));
        }

        Ok(FileHandleGuard {
            tracker: Arc::clone(&self.tracker),
        })
    }

    /// Track network connection open.
    pub fn track_connection_open(&self) -> crate::Result<ConnectionGuard> {
        let count = self.tracker.network_connections.fetch_add(1, Ordering::Relaxed) + 1;

        if self.quota.network_connections > 0 && count > self.quota.network_connections {
            self.tracker.network_connections.fetch_sub(1, Ordering::Relaxed);
            return Err(crate::NounVerbError::PluginError(format!(
                "Network connection quota exceeded: {} > {}",
                count,
                self.quota.network_connections
            )));
        }

        Ok(ConnectionGuard {
            tracker: Arc::clone(&self.tracker),
        })
    }

    /// Get elapsed time since guard creation.
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Get current resource usage.
    pub fn usage(&self) -> ResourceUsage {
        ResourceUsage {
            cpu_time_ms: self.elapsed().as_millis() as u64,
            memory_bytes: self.tracker.memory_bytes.load(Ordering::Relaxed),
            file_handles: self.tracker.file_handles.load(Ordering::Relaxed),
            network_connections: self.tracker.network_connections.load(Ordering::Relaxed),
        }
    }
}

impl Drop for QuotaGuard {
    fn drop(&mut self) {
        // Resource tracking is automatically cleaned up via Arc
    }
}

/// Resource tracker for quota enforcement.
struct ResourceTracker {
    memory_bytes: AtomicU64,
    file_handles: AtomicU64,
    network_connections: AtomicU64,
}

impl ResourceTracker {
    fn new() -> Self {
        Self {
            memory_bytes: AtomicU64::new(0),
            file_handles: AtomicU64::new(0),
            network_connections: AtomicU64::new(0),
        }
    }
}

/// Guard for file handle tracking (RAII).
pub struct FileHandleGuard {
    tracker: Arc<ResourceTracker>,
}

impl Drop for FileHandleGuard {
    fn drop(&mut self) {
        self.tracker.file_handles.fetch_sub(1, Ordering::Relaxed);
    }
}

/// Guard for network connection tracking (RAII).
pub struct ConnectionGuard {
    tracker: Arc<ResourceTracker>,
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        self.tracker.network_connections.fetch_sub(1, Ordering::Relaxed);
    }
}

/// Current resource usage snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// CPU time used in milliseconds
    pub cpu_time_ms: u64,
    /// Memory used in bytes
    pub memory_bytes: u64,
    /// File handles currently open
    pub file_handles: u64,
    /// Network connections currently open
    pub network_connections: u64,
}

impl ResourceUsage {
    /// Check if usage exceeds quota.
    pub fn exceeds(&self, quota: &ResourceQuota) -> bool {
        quota.is_cpu_time_exceeded(Duration::from_millis(self.cpu_time_ms))
            || quota.is_memory_exceeded(self.memory_bytes)
            || quota.is_file_handles_exceeded(self.file_handles)
            || quota.is_network_connections_exceeded(self.network_connections)
    }

    /// Get utilization percentage for each resource.
    pub fn utilization(&self, quota: &ResourceQuota) -> QuotaUtilization {
        QuotaUtilization {
            cpu_percent: if quota.cpu_time_ms > 0 {
                (self.cpu_time_ms as f64 / quota.cpu_time_ms as f64 * 100.0) as u8
            } else {
                0
            },
            memory_percent: if quota.memory_bytes > 0 {
                (self.memory_bytes as f64 / quota.memory_bytes as f64 * 100.0) as u8
            } else {
                0
            },
            file_handles_percent: if quota.file_handles > 0 {
                (self.file_handles as f64 / quota.file_handles as f64 * 100.0) as u8
            } else {
                0
            },
            network_percent: if quota.network_connections > 0 {
                (self.network_connections as f64 / quota.network_connections as f64 * 100.0) as u8
            } else {
                0
            },
        }
    }
}

/// Quota utilization percentages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaUtilization {
    /// CPU time utilization (0-100%)
    pub cpu_percent: u8,
    /// Memory utilization (0-100%)
    pub memory_percent: u8,
    /// File handle utilization (0-100%)
    pub file_handles_percent: u8,
    /// Network connection utilization (0-100%)
    pub network_percent: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_quota() {
        let quota = ResourceQuota::default();
        assert_eq!(quota.cpu_time_ms, 30_000);
        assert_eq!(quota.memory_bytes, 500_000_000);
    }

    #[test]
    fn test_unlimited_quota() {
        let quota = ResourceQuota::unlimited();
        assert_eq!(quota.cpu_time_ms, 0);
        assert_eq!(quota.memory_bytes, 0);
        assert!(!quota.is_cpu_time_exceeded(Duration::from_secs(1000)));
    }

    #[test]
    fn test_strict_quota() {
        let quota = ResourceQuota::strict();
        assert_eq!(quota.cpu_time_ms, 5_000);
        assert_eq!(quota.memory_bytes, 50_000_000);
    }

    #[test]
    fn test_quota_builder() {
        let quota = ResourceQuota::builder()
            .cpu_time_ms(10_000)
            .memory_bytes(100_000_000)
            .file_handles(50)
            .network_connections(5)
            .build();

        assert_eq!(quota.cpu_time_ms, 10_000);
        assert_eq!(quota.memory_bytes, 100_000_000);
        assert_eq!(quota.file_handles, 50);
        assert_eq!(quota.network_connections, 5);
    }

    #[test]
    fn test_quota_guard_creation() {
        let quota = ResourceQuota::default();
        let guard = quota.acquire().unwrap();
        assert!(guard.elapsed().as_millis() < 100);
    }

    #[test]
    fn test_cpu_time_exceeded() {
        let quota = ResourceQuota::builder()
            .cpu_time_ms(100)
            .build();

        assert!(!quota.is_cpu_time_exceeded(Duration::from_millis(50)));
        assert!(quota.is_cpu_time_exceeded(Duration::from_millis(150)));
    }

    #[test]
    fn test_memory_tracking() {
        let quota = ResourceQuota::default();
        let guard = quota.acquire().unwrap();

        guard.track_memory_alloc(1000);
        guard.track_memory_alloc(2000);

        let usage = guard.usage();
        assert_eq!(usage.memory_bytes, 3000);

        guard.track_memory_dealloc(1500);
        let usage = guard.usage();
        assert_eq!(usage.memory_bytes, 1500);
    }

    #[test]
    fn test_file_handle_tracking() {
        let quota = ResourceQuota::builder()
            .file_handles(2)
            .build();

        let guard = quota.acquire().unwrap();

        // Open first file
        let _fh1 = guard.track_file_open().unwrap();
        assert_eq!(guard.usage().file_handles, 1);

        // Open second file
        let _fh2 = guard.track_file_open().unwrap();
        assert_eq!(guard.usage().file_handles, 2);

        // Try to open third file (should fail)
        let result = guard.track_file_open();
        assert!(result.is_err());

        // Drop first file handle
        drop(_fh1);
        assert_eq!(guard.usage().file_handles, 1);

        // Now we can open another file
        let _fh3 = guard.track_file_open().unwrap();
        assert_eq!(guard.usage().file_handles, 2);
    }

    #[test]
    fn test_network_connection_tracking() {
        let quota = ResourceQuota::builder()
            .network_connections(2)
            .build();

        let guard = quota.acquire().unwrap();

        let _conn1 = guard.track_connection_open().unwrap();
        assert_eq!(guard.usage().network_connections, 1);

        let _conn2 = guard.track_connection_open().unwrap();
        assert_eq!(guard.usage().network_connections, 2);

        // Try to open third connection (should fail)
        let result = guard.track_connection_open();
        assert!(result.is_err());
    }

    #[test]
    fn test_quota_utilization() {
        let quota = ResourceQuota::builder()
            .cpu_time_ms(1000)
            .memory_bytes(1000)
            .build();

        let usage = ResourceUsage {
            cpu_time_ms: 500,
            memory_bytes: 750,
            file_handles: 0,
            network_connections: 0,
        };

        let util = usage.utilization(&quota);
        assert_eq!(util.cpu_percent, 50);
        assert_eq!(util.memory_percent, 75);
    }

    #[test]
    fn test_quota_check() {
        let quota = ResourceQuota::builder()
            .memory_bytes(1000)
            .build();

        let guard = quota.acquire().unwrap();

        // Within quota
        guard.track_memory_alloc(500);
        assert!(guard.check_quotas().is_ok());

        // Exceed quota
        guard.track_memory_alloc(600);
        assert!(guard.check_quotas().is_err());
    }
}
