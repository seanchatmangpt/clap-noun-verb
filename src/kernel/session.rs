//! CNV Session Kernel
//!
//! Long-lived, multiplexed, back-pressured command streams for agent workloads.
//!
//! # Design
//!
//! The session kernel extends CNV from invocation-oriented (parse argv, run, exit)
//! to session-oriented (long-lived RPC-like protocol) while preserving:
//! - Deterministic output
//! - Structured telemetry
//! - Strict capability contracts
//!
//! # Features
//!
//! - **Session abstraction**: Long-lived command contexts
//! - **Multiplexed protocol**: Multiple logical streams over stdio
//! - **Backpressure**: Cooperative flow control
//! - **Cancellation**: Graceful command termination
//! - **Session-scoped telemetry**: Per-session metrics and tracing
//!
//! # Example
//!
//! ```rust,ignore
//! use clap_noun_verb::kernel::{Session, SessionConfig};
//!
//! // Create a session
//! let session = Session::new(SessionConfig::default())?;
//!
//! // Run a command in session mode
//! session.execute("noun", "verb", &args)?;
//!
//! // Stream output frames
//! for frame in session.output_stream() {
//!     println!("Frame: {:?}", frame);
//! }
//! ```

use crate::kernel::capability::CapabilityContract;
use crate::kernel::output::StructuredResult;
use crate::kernel::telemetry::TelemetryProfile;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use uuid::Uuid;

/// Session ID - unique identifier for a session
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(Uuid);

impl SessionId {
    /// Create a new random session ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create from UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Get the underlying UUID
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for SessionId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

/// Stream ID - identifies a logical stream within a session
///
/// Sessions support multiple logical streams:
/// - Stdout: Standard output
/// - Stderr: Standard error
/// - Logs: Structured logs
/// - Metrics: Telemetry metrics
/// - Control: Control messages (backpressure, cancellation)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StreamId {
    /// Standard output stream
    Stdout,
    /// Standard error stream
    Stderr,
    /// Structured log stream
    Logs,
    /// Metrics stream
    Metrics,
    /// Control message stream
    Control,
    /// Custom stream
    Custom(u8),
}

impl fmt::Display for StreamId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stdout => write!(f, "stdout"),
            Self::Stderr => write!(f, "stderr"),
            Self::Logs => write!(f, "logs"),
            Self::Metrics => write!(f, "metrics"),
            Self::Control => write!(f, "control"),
            Self::Custom(id) => write!(f, "custom_{}", id),
        }
    }
}

/// Frame - a single message in the multiplexed protocol
///
/// Each frame contains:
/// - Session ID
/// - Stream ID
/// - Sequence number (for ordering)
/// - Payload (structured data)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frame {
    /// Session ID
    pub session_id: SessionId,
    /// Stream ID
    pub stream_id: StreamId,
    /// Sequence number (monotonic within stream)
    pub sequence: u64,
    /// Timestamp (milliseconds since epoch)
    pub timestamp_ms: u64,
    /// Payload
    pub payload: FramePayload,
}

impl Frame {
    /// Create a new frame
    pub fn new(
        session_id: SessionId,
        stream_id: StreamId,
        sequence: u64,
        payload: FramePayload,
    ) -> Self {
        Self {
            session_id,
            stream_id,
            sequence,
            timestamp_ms: chrono::Utc::now().timestamp_millis() as u64,
            payload,
        }
    }

    /// Serialize frame to JSON
    pub fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }

    /// Deserialize frame from JSON
    pub fn from_json(json: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(serde_json::from_str(json)?)
    }
}

/// Frame payload - typed content of a frame
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FramePayload {
    /// Data output (structured result)
    Data {
        /// The actual data
        data: serde_json::Value,
    },
    /// Log message
    Log {
        /// Log level
        level: String,
        /// Log message
        message: String,
        /// Additional context
        #[serde(skip_serializing_if = "Option::is_none")]
        context: Option<HashMap<String, serde_json::Value>>,
    },
    /// Metrics snapshot
    Metrics {
        /// Metrics data
        metrics: HashMap<String, f64>,
    },
    /// Control message
    Control {
        /// Control command
        command: ControlCommand,
    },
    /// Error
    Error {
        /// Error kind
        kind: String,
        /// Error message
        message: String,
    },
}

/// Control commands for session management
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum ControlCommand {
    /// Pause the session
    Pause,
    /// Resume the session
    Resume,
    /// Cancel the current operation
    Cancel,
    /// Request backpressure (slow down)
    Backpressure {
        /// Target rate (frames per second)
        target_rate: Option<f64>,
    },
    /// Session ended
    End {
        /// Exit code
        exit_code: u8,
    },
}

/// Session state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionState {
    /// Session is active
    Active,
    /// Session is paused
    Paused,
    /// Session is cancelled
    Cancelled,
    /// Session has ended
    Ended,
}

/// Session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Enable multiplexed protocol
    pub enable_multiplexing: bool,
    /// Enable backpressure control
    pub enable_backpressure: bool,
    /// Enable cancellation support
    pub enable_cancellation: bool,
    /// Maximum frames per second (0 = unlimited)
    pub max_frames_per_second: u64,
    /// Frame buffer size
    pub frame_buffer_size: usize,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            enable_multiplexing: true,
            enable_backpressure: true,
            enable_cancellation: true,
            max_frames_per_second: 0, // Unlimited by default
            frame_buffer_size: 1000,
        }
    }
}

/// Session metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionMetrics {
    /// Total frames sent
    pub frames_sent: u64,
    /// Total bytes sent
    pub bytes_sent: u64,
    /// Total errors
    pub errors: u64,
    /// Average latency per frame (milliseconds)
    pub avg_latency_ms: f64,
    /// Session duration (milliseconds)
    pub duration_ms: u64,
}

impl SessionMetrics {
    /// Create new metrics
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a frame sent
    pub fn record_frame(&mut self, bytes: u64, latency_ms: f64) {
        self.frames_sent += 1;
        self.bytes_sent += bytes;

        // Update rolling average
        let n = self.frames_sent as f64;
        self.avg_latency_ms = (self.avg_latency_ms * (n - 1.0) + latency_ms) / n;
    }

    /// Record an error
    pub fn record_error(&mut self) {
        self.errors += 1;
    }
}

/// Session handle - owned reference to a session
///
/// Provides APIs for:
/// - Yielding output frames
/// - Checking cancellation
/// - Receiving control messages
/// - Reporting metrics
pub struct SessionHandle {
    /// Session ID
    id: SessionId,
    /// Session state
    state: Arc<AtomicBool>, // true = active, false = cancelled
    /// Telemetry profile
    profile: TelemetryProfile,
    /// Capability contract
    capability: CapabilityContract,
    /// Sequence counters per stream
    sequences: HashMap<StreamId, AtomicU64>,
    /// Metrics
    metrics: Arc<std::sync::Mutex<SessionMetrics>>,
    /// Configuration
    config: SessionConfig,
}

impl SessionHandle {
    /// Create a new session handle
    pub fn new(
        id: SessionId,
        profile: TelemetryProfile,
        capability: CapabilityContract,
        config: SessionConfig,
    ) -> Self {
        Self {
            id,
            state: Arc::new(AtomicBool::new(true)),
            profile,
            capability,
            sequences: HashMap::new(),
            metrics: Arc::new(std::sync::Mutex::new(SessionMetrics::new())),
            config,
        }
    }

    /// Get the session ID
    pub fn id(&self) -> SessionId {
        self.id
    }

    /// Get the telemetry profile
    pub fn profile(&self) -> &TelemetryProfile {
        &self.profile
    }

    /// Get the capability contract
    pub fn capability(&self) -> &CapabilityContract {
        &self.capability
    }

    /// Check if the session is active
    pub fn is_active(&self) -> bool {
        self.state.load(Ordering::SeqCst)
    }

    /// Check if the session is cancelled
    pub fn is_cancelled(&self) -> bool {
        !self.is_active()
    }

    /// Cancel the session
    pub fn cancel(&self) {
        self.state.store(false, Ordering::SeqCst);
    }

    /// Yield a data frame to the output stream
    pub fn yield_data(
        &mut self,
        stream: StreamId,
        data: impl Serialize,
    ) -> Result<Frame, Box<dyn std::error::Error>> {
        self.check_cancellation()?;

        let sequence = self.next_sequence(stream);
        let json_data = serde_json::to_value(data)?;

        let frame = Frame::new(
            self.id,
            stream,
            sequence,
            FramePayload::Data { data: json_data },
        );

        self.record_frame(&frame)?;

        Ok(frame)
    }

    /// Yield a log frame
    pub fn yield_log(
        &mut self,
        level: impl Into<String>,
        message: impl Into<String>,
        context: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<Frame, Box<dyn std::error::Error>> {
        self.check_cancellation()?;

        let sequence = self.next_sequence(StreamId::Logs);
        let frame = Frame::new(
            self.id,
            StreamId::Logs,
            sequence,
            FramePayload::Log {
                level: level.into(),
                message: message.into(),
                context,
            },
        );

        self.record_frame(&frame)?;

        Ok(frame)
    }

    /// Yield a metrics frame
    pub fn yield_metrics(
        &mut self,
        metrics: HashMap<String, f64>,
    ) -> Result<Frame, Box<dyn std::error::Error>> {
        self.check_cancellation()?;

        let sequence = self.next_sequence(StreamId::Metrics);
        let frame = Frame::new(
            self.id,
            StreamId::Metrics,
            sequence,
            FramePayload::Metrics { metrics },
        );

        self.record_frame(&frame)?;

        Ok(frame)
    }

    /// Send a control message
    pub fn send_control(
        &mut self,
        command: ControlCommand,
    ) -> Result<Frame, Box<dyn std::error::Error>> {
        let sequence = self.next_sequence(StreamId::Control);
        let frame = Frame::new(
            self.id,
            StreamId::Control,
            sequence,
            FramePayload::Control { command },
        );

        self.record_frame(&frame)?;

        Ok(frame)
    }

    /// Check for cancellation and return error if cancelled
    pub fn check_cancellation(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.is_cancelled() {
            Err("Session cancelled".into())
        } else {
            Ok(())
        }
    }

    /// Get the next sequence number for a stream
    fn next_sequence(&mut self, stream: StreamId) -> u64 {
        self.sequences
            .entry(stream)
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(1, Ordering::SeqCst)
    }

    /// Record a frame in metrics
    fn record_frame(&self, frame: &Frame) -> Result<(), Box<dyn std::error::Error>> {
        let json = frame.to_json()?;
        let bytes = json.len() as u64;

        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.record_frame(bytes, 0.0); // TODO: Measure actual latency
        }

        Ok(())
    }

    /// Get current metrics
    pub fn metrics(&self) -> SessionMetrics {
        self.metrics
            .lock()
            .ok()
            .map(|m| m.clone())
            .unwrap_or_default()
    }

    /// Get configuration
    pub fn config(&self) -> &SessionConfig {
        &self.config
    }
}

/// Session-aware verb handler trait
///
/// Verbs can implement this trait to opt into session mode:
/// - Receive a SessionHandle
/// - Yield frames incrementally
/// - Cooperate with cancellation
pub trait SessionVerb {
    /// Execute the verb in session mode
    fn execute_session(
        &self,
        handle: &mut SessionHandle,
    ) -> StructuredResult<()>;
}

/// Session builder for creating configured sessions
#[derive(Debug, Default)]
pub struct SessionBuilder {
    config: SessionConfig,
    profile: Option<TelemetryProfile>,
    capability: Option<CapabilityContract>,
}

impl SessionBuilder {
    /// Create a new session builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the session configuration
    pub fn config(mut self, config: SessionConfig) -> Self {
        self.config = config;
        self
    }

    /// Set the telemetry profile
    pub fn profile(mut self, profile: TelemetryProfile) -> Self {
        self.profile = Some(profile);
        self
    }

    /// Set the capability contract
    pub fn capability(mut self, capability: CapabilityContract) -> Self {
        self.capability = Some(capability);
        self
    }

    /// Build a session handle
    pub fn build(self) -> SessionHandle {
        SessionHandle::new(
            SessionId::new(),
            self.profile.unwrap_or_default(),
            self.capability.unwrap_or_default(),
            self.config,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_id() {
        let id1 = SessionId::new();
        let id2 = SessionId::new();
        assert_ne!(id1, id2);

        let id_str = id1.to_string();
        let parsed: SessionId = id_str.parse().ok().unwrap();
        assert_eq!(id1, parsed);
    }

    #[test]
    fn test_stream_id_display() {
        assert_eq!(StreamId::Stdout.to_string(), "stdout");
        assert_eq!(StreamId::Stderr.to_string(), "stderr");
        assert_eq!(StreamId::Custom(42).to_string(), "custom_42");
    }

    #[test]
    fn test_frame_creation() {
        let session_id = SessionId::new();
        let frame = Frame::new(
            session_id,
            StreamId::Stdout,
            0,
            FramePayload::Data {
                data: serde_json::json!({"test": "value"}),
            },
        );

        assert_eq!(frame.session_id, session_id);
        assert_eq!(frame.stream_id, StreamId::Stdout);
        assert_eq!(frame.sequence, 0);
    }

    #[test]
    fn test_frame_serialization() {
        let session_id = SessionId::new();
        let frame = Frame::new(
            session_id,
            StreamId::Stdout,
            0,
            FramePayload::Data {
                data: serde_json::json!({"test": "value"}),
            },
        );

        let json = frame.to_json().ok().unwrap();
        let parsed = Frame::from_json(&json).ok().unwrap();

        assert_eq!(frame.session_id, parsed.session_id);
        assert_eq!(frame.sequence, parsed.sequence);
    }

    #[test]
    fn test_session_handle_cancellation() {
        let mut handle = SessionBuilder::new().build();
        assert!(handle.is_active());
        assert!(!handle.is_cancelled());

        handle.cancel();
        assert!(!handle.is_active());
        assert!(handle.is_cancelled());
        assert!(handle.check_cancellation().is_err());
    }

    #[test]
    fn test_session_handle_yield_data() {
        let mut handle = SessionBuilder::new().build();

        let frame = handle.yield_data(StreamId::Stdout, serde_json::json!({"test": "data"}));
        assert!(frame.is_ok());

        let frame = frame.ok().unwrap();
        assert_eq!(frame.stream_id, StreamId::Stdout);
        assert_eq!(frame.sequence, 0);
    }

    #[test]
    fn test_session_metrics() {
        let mut metrics = SessionMetrics::new();
        assert_eq!(metrics.frames_sent, 0);

        metrics.record_frame(100, 10.0);
        assert_eq!(metrics.frames_sent, 1);
        assert_eq!(metrics.bytes_sent, 100);
        assert_eq!(metrics.avg_latency_ms, 10.0);

        metrics.record_frame(200, 20.0);
        assert_eq!(metrics.frames_sent, 2);
        assert_eq!(metrics.bytes_sent, 300);
        assert_eq!(metrics.avg_latency_ms, 15.0);
    }

    #[test]
    fn test_session_config_defaults() {
        let config = SessionConfig::default();
        assert!(config.enable_multiplexing);
        assert!(config.enable_backpressure);
        assert!(config.enable_cancellation);
    }
}
