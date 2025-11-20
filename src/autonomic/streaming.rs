//! Streaming interactions and session support for swarm-native CLIs
//!
//! Enables:
//! - Commands that emit structured events over time
//! - Long-lived sessions with stateful context
//! - Partial results and incremental receipts
//! - Pause, resume, and cancellation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Session identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(String);

impl SessionId {
    /// Create a new session ID
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Generate a new random session ID
    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }

    /// Get the underlying ID string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for SessionId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for SessionId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Session context for stateful command execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionContext {
    /// Session ID
    pub session_id: SessionId,
    /// Session state
    pub state: SessionState,
    /// Session metadata
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub metadata: HashMap<String, serde_json::Value>,
    /// Noun scope for this session
    pub noun_scope: Option<String>,
    /// Created timestamp (ISO 8601)
    pub created_at: String,
    /// Last active timestamp (ISO 8601)
    pub last_active_at: String,
    /// Session timeout in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_seconds: Option<u64>,
}

impl SessionContext {
    /// Create a new session context
    pub fn new(session_id: SessionId) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            session_id,
            state: SessionState::Active,
            metadata: HashMap::new(),
            noun_scope: None,
            created_at: now.clone(),
            last_active_at: now,
            timeout_seconds: None,
        }
    }

    /// Create a new session with generated ID
    pub fn generate() -> Self {
        Self::new(SessionId::generate())
    }

    /// Set noun scope
    pub fn with_noun_scope(mut self, noun: impl Into<String>) -> Self {
        self.noun_scope = Some(noun.into());
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout_seconds = Some(seconds);
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Update last active timestamp
    pub fn touch(&mut self) {
        self.last_active_at = chrono::Utc::now().to_rfc3339();
    }

    /// Check if session is active
    pub fn is_active(&self) -> bool {
        matches!(self.state, SessionState::Active)
    }

    /// Pause the session
    pub fn pause(&mut self) {
        self.state = SessionState::Paused;
    }

    /// Resume the session
    pub fn resume(&mut self) {
        self.state = SessionState::Active;
        self.touch();
    }

    /// Terminate the session
    pub fn terminate(&mut self) {
        self.state = SessionState::Terminated;
    }
}

/// Session state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionState {
    /// Session is active
    Active,
    /// Session is paused
    Paused,
    /// Session is terminated
    Terminated,
    /// Session has expired
    Expired,
}

/// Streaming event from a command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamEvent {
    /// Event sequence number
    pub sequence: u64,
    /// Event timestamp (ISO 8601)
    pub timestamp: String,
    /// Event type
    pub event_type: StreamEventType,
    /// Event data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    /// Session ID (if part of a session)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<SessionId>,
}

impl StreamEvent {
    /// Create a new stream event
    pub fn new(sequence: u64, event_type: StreamEventType) -> Self {
        Self {
            sequence,
            timestamp: chrono::Utc::now().to_rfc3339(),
            event_type,
            data: None,
            session_id: None,
        }
    }

    /// Set event data
    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }

    /// Set session ID
    pub fn with_session(mut self, session_id: SessionId) -> Self {
        self.session_id = Some(session_id);
        self
    }
}

/// Type of streaming event
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StreamEventType {
    /// Command started
    Started,
    /// Progress update
    Progress,
    /// Partial result
    PartialResult,
    /// Log message
    Log,
    /// Warning
    Warning,
    /// Error
    Error,
    /// Command completed
    Completed,
    /// Command cancelled
    Cancelled,
}

/// Stream control command
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum StreamControl {
    /// Pause the stream
    Pause,
    /// Resume the stream
    Resume,
    /// Cancel the stream
    Cancel,
    /// Request status
    Status,
}

/// Incremental receipt for long-running operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncrementalReceipt {
    /// Receipt ID
    pub receipt_id: String,
    /// Milestone number
    pub milestone: u64,
    /// Timestamp (ISO 8601)
    pub timestamp: String,
    /// Progress percentage (0-100)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress_percent: Option<u8>,
    /// Milestone description
    pub description: String,
    /// Partial data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    /// Session ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<SessionId>,
}

impl IncrementalReceipt {
    /// Create a new incremental receipt
    pub fn new(
        receipt_id: impl Into<String>,
        milestone: u64,
        description: impl Into<String>,
    ) -> Self {
        Self {
            receipt_id: receipt_id.into(),
            milestone,
            timestamp: chrono::Utc::now().to_rfc3339(),
            progress_percent: None,
            description: description.into(),
            data: None,
            session_id: None,
        }
    }

    /// Set progress percentage
    pub fn with_progress(mut self, percent: u8) -> Self {
        self.progress_percent = Some(percent.min(100));
        self
    }

    /// Set partial data
    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }

    /// Set session ID
    pub fn with_session(mut self, session_id: SessionId) -> Self {
        self.session_id = Some(session_id);
        self
    }
}

/// Stream producer trait for commands that emit events over time
pub trait StreamProducer {
    /// Emit a stream event
    fn emit(&mut self, event: StreamEvent) -> Result<(), std::io::Error>;

    /// Emit progress
    fn emit_progress(
        &mut self,
        sequence: u64,
        percent: u8,
        message: impl Into<String>,
    ) -> Result<(), std::io::Error> {
        self.emit(StreamEvent::new(sequence, StreamEventType::Progress).with_data(
            serde_json::json!({
                "percent": percent,
                "message": message.into(),
            }),
        ))
    }

    /// Emit partial result
    fn emit_partial_result(
        &mut self,
        sequence: u64,
        data: serde_json::Value,
    ) -> Result<(), std::io::Error> {
        self.emit(StreamEvent::new(sequence, StreamEventType::PartialResult).with_data(data))
    }

    /// Emit log message
    fn emit_log(
        &mut self,
        sequence: u64,
        level: impl Into<String>,
        message: impl Into<String>,
    ) -> Result<(), std::io::Error> {
        self.emit(StreamEvent::new(sequence, StreamEventType::Log).with_data(serde_json::json!({
            "level": level.into(),
            "message": message.into(),
        })))
    }
}

/// Session manager for tracking active sessions
pub struct SessionManager {
    sessions: HashMap<SessionId, SessionContext>,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new() -> Self {
        Self { sessions: HashMap::new() }
    }

    /// Create a new session
    pub fn create_session(&mut self) -> SessionContext {
        let session = SessionContext::generate();
        let session_id = session.session_id.clone();
        self.sessions.insert(session_id, session.clone());
        session
    }

    /// Get a session by ID
    pub fn get_session(&self, session_id: &SessionId) -> Option<&SessionContext> {
        self.sessions.get(session_id)
    }

    /// Get a mutable session by ID
    pub fn get_session_mut(&mut self, session_id: &SessionId) -> Option<&mut SessionContext> {
        self.sessions.get_mut(session_id)
    }

    /// Remove a session
    pub fn remove_session(&mut self, session_id: &SessionId) -> Option<SessionContext> {
        self.sessions.remove(session_id)
    }

    /// List active sessions
    pub fn active_sessions(&self) -> Vec<&SessionContext> {
        self.sessions.values().filter(|s| s.is_active()).collect()
    }

    /// Clean up expired sessions
    pub fn cleanup_expired(&mut self) -> usize {
        let now = chrono::Utc::now();
        let mut expired = Vec::new();

        for (id, session) in &self.sessions {
            if let Some(timeout) = session.timeout_seconds {
                if let Ok(last_active) =
                    chrono::DateTime::parse_from_rfc3339(&session.last_active_at)
                {
                    let elapsed = now.signed_duration_since(last_active).num_seconds() as u64;
                    if elapsed > timeout {
                        expired.push(id.clone());
                    }
                }
            }
        }

        let count = expired.len();
        for id in expired {
            if let Some(session) = self.sessions.get_mut(&id) {
                session.state = SessionState::Expired;
            }
        }

        count
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_context() {
        let mut session = SessionContext::generate().with_noun_scope("services").with_timeout(3600);

        assert!(session.is_active());
        assert_eq!(session.noun_scope, Some("services".to_string()));
        assert_eq!(session.timeout_seconds, Some(3600));

        session.pause();
        assert!(!session.is_active());

        session.resume();
        assert!(session.is_active());
    }

    #[test]
    fn test_stream_event() {
        let event = StreamEvent::new(1, StreamEventType::Progress)
            .with_data(serde_json::json!({"percent": 50}))
            .with_session(SessionId::from("session-123"));

        assert_eq!(event.sequence, 1);
        assert_eq!(event.event_type, StreamEventType::Progress);
        assert_eq!(event.session_id, Some(SessionId::from("session-123")));
    }

    #[test]
    fn test_session_manager() {
        let mut manager = SessionManager::new();

        let session = manager.create_session();
        assert!(manager.get_session(&session.session_id).is_some());

        let active = manager.active_sessions();
        assert_eq!(active.len(), 1);

        manager.remove_session(&session.session_id);
        assert!(manager.get_session(&session.session_id).is_none());
    }
}
