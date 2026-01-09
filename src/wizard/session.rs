//! Session management for wizard workflows
//!
//! Provides type-safe state machine for managing multi-step wizard sessions
//! with compile-time guarantees about valid state transitions.

use super::error::{Result, WizardError};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

/// Trait representing a valid wizard state
///
/// Types implementing this trait can be used as states in the wizard state machine.
/// The type system ensures only valid state transitions are possible.
pub trait State: Send + Sync + 'static {
    /// Name of this state for debugging and logging
    fn name(&self) -> &'static str;

    /// Whether this is a terminal state
    fn is_terminal(&self) -> bool {
        false
    }
}

/// Initial state - session just created
#[derive(Debug, Clone, Copy)]
pub struct Init;

impl State for Init {
    fn name(&self) -> &'static str {
        "Init"
    }
}

/// Active state - session in progress
#[derive(Debug, Clone, Copy)]
pub struct Active;

impl State for Active {
    fn name(&self) -> &'static str {
        "Active"
    }
}

/// Paused state - session temporarily suspended
#[derive(Debug, Clone, Copy)]
pub struct Paused;

impl State for Paused {
    fn name(&self) -> &'static str {
        "Paused"
    }
}

/// Complete state - session finished successfully
#[derive(Debug, Clone, Copy)]
pub struct Complete;

impl State for Complete {
    fn name(&self) -> &'static str {
        "Complete"
    }

    fn is_terminal(&self) -> bool {
        true
    }
}

/// Failed state - session ended with error
#[derive(Debug, Clone, Copy)]
pub struct Failed;

impl State for Failed {
    fn name(&self) -> &'static str {
        "Failed"
    }

    fn is_terminal(&self) -> bool {
        true
    }
}

/// Session data stored across state transitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    /// Unique session identifier
    pub session_id: String,

    /// Step history - prompts and responses
    pub history: Vec<(String, String)>,

    /// Session metadata
    pub metadata: serde_json::Value,
}

impl SessionData {
    /// Create new session data with default capacity
    #[inline]
    pub fn new(session_id: String) -> Self {
        Self::with_capacity(session_id, 8)
    }

    /// Create new session data with specified history capacity
    #[inline]
    pub fn with_capacity(session_id: String, capacity: usize) -> Self {
        Self {
            session_id,
            history: Vec::with_capacity(capacity),
            metadata: serde_json::Value::Null,
        }
    }

    /// Add an interaction to the history
    #[inline]
    pub fn add_interaction(&mut self, prompt: String, response: String) {
        self.history.push((prompt, response));
    }

    /// Get the last interaction
    pub fn last_interaction(&self) -> Option<&(String, String)> {
        self.history.last()
    }
}

/// Type-safe wizard session with state machine
///
/// The type parameter `S` encodes the current state at the type level,
/// preventing invalid state transitions at compile time.
pub struct WizardSession<S: State> {
    /// Session data persisted across transitions
    data: SessionData,

    /// Current state (zero-cost phantom data)
    _state: PhantomData<S>,
}

impl<S: State> WizardSession<S> {
    /// Get session ID (zero-cost reference)
    #[inline(always)]
    pub fn session_id(&self) -> &str {
        &self.data.session_id
    }

    /// Get interaction history (zero-cost slice reference)
    #[inline(always)]
    pub fn history(&self) -> &[(String, String)] {
        &self.data.history
    }

    /// Get session metadata (zero-cost reference)
    #[inline(always)]
    pub fn metadata(&self) -> &serde_json::Value {
        &self.data.metadata
    }

    /// Set session metadata
    #[inline]
    pub fn set_metadata(&mut self, metadata: serde_json::Value) {
        self.data.metadata = metadata;
    }

    /// Check if current state is terminal
    pub fn is_terminal(&self) -> bool
    where
        S: Default,
    {
        S::default().is_terminal()
    }
}

impl WizardSession<Init> {
    /// Create a new wizard session in Init state with default capacity
    #[inline]
    pub fn new(session_id: String) -> Self {
        Self::with_capacity(session_id, 8)
    }

    /// Create a new wizard session with specified history capacity
    #[inline]
    pub fn with_capacity(session_id: String, capacity: usize) -> Self {
        Self {
            data: SessionData::with_capacity(session_id, capacity),
            _state: PhantomData,
        }
    }

    /// Transition to Active state (zero-cost state transition)
    #[inline(always)]
    pub fn start(self) -> WizardSession<Active> {
        WizardSession {
            data: self.data,
            _state: PhantomData,
        }
    }
}

impl WizardSession<Active> {
    /// Add an interaction to the session
    #[inline]
    pub fn add_interaction(&mut self, prompt: String, response: String) {
        self.data.add_interaction(prompt, response);
    }

    /// Pause the session (zero-cost state transition)
    #[inline(always)]
    pub fn pause(self) -> WizardSession<Paused> {
        WizardSession {
            data: self.data,
            _state: PhantomData,
        }
    }

    /// Complete the session successfully (zero-cost state transition)
    #[inline(always)]
    pub fn complete(self) -> WizardSession<Complete> {
        WizardSession {
            data: self.data,
            _state: PhantomData,
        }
    }

    /// Fail the session (zero-cost state transition)
    #[inline(always)]
    pub fn fail(self) -> WizardSession<Failed> {
        WizardSession {
            data: self.data,
            _state: PhantomData,
        }
    }
}

impl WizardSession<Paused> {
    /// Resume a paused session (zero-cost state transition)
    #[inline(always)]
    pub fn resume(self) -> WizardSession<Active> {
        WizardSession {
            data: self.data,
            _state: PhantomData,
        }
    }

    /// Fail a paused session (zero-cost state transition)
    #[inline(always)]
    pub fn fail(self) -> WizardSession<Failed> {
        WizardSession {
            data: self.data,
            _state: PhantomData,
        }
    }
}

/// Builder pattern for creating and configuring wizard sessions
pub struct SessionBuilder {
    session_id: Option<String>,
    metadata: serde_json::Value,
}

impl SessionBuilder {
    /// Create a new session builder
    pub fn new() -> Self {
        Self {
            session_id: None,
            metadata: serde_json::Value::Null,
        }
    }

    /// Set session ID
    pub fn session_id(mut self, id: String) -> Self {
        self.session_id = Some(id);
        self
    }

    /// Set session metadata
    pub fn metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    /// Build the session
    pub fn build(self) -> Result<WizardSession<Init>> {
        let session_id = self.session_id.ok_or_else(|| {
            WizardError::ConfigError("session_id is required".to_string())
        })?;

        let mut session = WizardSession::new(session_id);
        session.set_metadata(self.metadata);
        Ok(session)
    }
}

impl Default for SessionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// Implement Default for state types (required for is_terminal check)
impl Default for Init {
    fn default() -> Self {
        Self
    }
}

impl Default for Active {
    fn default() -> Self {
        Self
    }
}

impl Default for Paused {
    fn default() -> Self {
        Self
    }
}

impl Default for Complete {
    fn default() -> Self {
        Self
    }
}

impl Default for Failed {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_names() {
        assert_eq!(Init.name(), "Init");
        assert_eq!(Active.name(), "Active");
        assert_eq!(Complete.name(), "Complete");
    }

    #[test]
    fn test_terminal_states() {
        assert!(!Init.is_terminal());
        assert!(!Active.is_terminal());
        assert!(Complete.is_terminal());
        assert!(Failed.is_terminal());
    }

    #[test]
    fn test_session_lifecycle() {
        // Create session in Init state
        let session = WizardSession::new("test-123".to_string());
        assert_eq!(session.session_id(), "test-123");

        // Transition to Active
        let mut session = session.start();
        session.add_interaction("prompt1".to_string(), "response1".to_string());
        assert_eq!(session.history().len(), 1);

        // Transition to Complete
        let session = session.complete();
        assert_eq!(session.history().len(), 1);
    }

    #[test]
    fn test_session_builder() {
        let session = SessionBuilder::new()
            .session_id("test-456".to_string())
            .metadata(serde_json::json!({"key": "value"}))
            .build()
            .expect("Failed to build session");

        assert_eq!(session.session_id(), "test-456");
    }

    #[test]
    fn test_session_builder_missing_id() {
        let result = SessionBuilder::new().build();
        assert!(result.is_err());
    }

    #[test]
    fn test_pause_resume() {
        let session = WizardSession::new("test-789".to_string());
        let session = session.start();
        let session = session.pause();
        let session = session.resume();
        let _session = session.complete();
        // Compile-time guarantee that state transitions are valid
    }
}
