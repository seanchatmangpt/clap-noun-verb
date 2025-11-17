//! Session Streaming Protocol
//!
//! Server/Client streaming mode for long-lived command streams.
//! Uses advanced async patterns: channels, backpressure, and multiplexing.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use std::collections::VecDeque;

/// Frame types in the streaming protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StreamFrame {
    /// Output data frame
    #[serde(rename = "data")]
    Data {
        session_id: String,
        sequence: u64,
        payload: serde_json::Value,
    },
    /// Diagnostic log frame
    #[serde(rename = "log")]
    Log {
        session_id: String,
        level: LogLevel,
        message: String,
        timestamp_ns: u64,
    },
    /// Metrics collection frame
    #[serde(rename = "metrics")]
    Metrics {
        session_id: String,
        cpu_us: u64,
        memory_bytes: u64,
        io_read_bytes: u64,
        io_write_bytes: u64,
    },
    /// Control frame (cancellation, pause, etc.)
    #[serde(rename = "control")]
    Control {
        session_id: String,
        action: ControlAction,
    },
    /// Error frame
    #[serde(rename = "error")]
    Error {
        session_id: String,
        code: u32,
        message: String,
    },
    /// Session completion frame
    #[serde(rename = "done")]
    Done {
        session_id: String,
        exit_code: i32,
    },
}

/// Log level for diagnostic frames
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// Control actions in the streaming protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum ControlAction {
    /// Cancel the running command
    #[serde(rename = "cancel")]
    Cancel { reason: Option<String> },
    /// Pause execution (not all commands support this)
    #[serde(rename = "pause")]
    Pause,
    /// Resume execution
    #[serde(rename = "resume")]
    Resume,
    /// Request metrics update
    #[serde(rename = "get_metrics")]
    GetMetrics,
    /// Request status update
    #[serde(rename = "get_status")]
    GetStatus,
}

/// Session-scoped state for streaming execution
#[derive(Debug, Clone)]
pub struct StreamingSession {
    pub id: String,
    pub command: String,
    pub args: Vec<String>,
    pub start_time: u64,
    pub frame_count: u64,
    pub byte_count: u64,
    pub is_cancelled: bool,
}

impl StreamingSession {
    pub fn new(id: String, command: String, args: Vec<String>) -> Self {
        Self {
            id,
            command,
            args,
            start_time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            frame_count: 0,
            byte_count: 0,
            is_cancelled: false,
        }
    }
}

/// Backpressure-aware frame sink
/// Uses bounded channels to prevent memory overflow
#[derive(Clone)]
pub struct FrameSink {
    tx: mpsc::Sender<StreamFrame>,
    buffer: Arc<RwLock<VecDeque<StreamFrame>>>,
    max_buffered_frames: usize,
}

impl FrameSink {
    /// Create a frame sink with backpressure limit
    pub fn new(buffer_size: usize) -> (Self, mpsc::Receiver<StreamFrame>) {
        let (tx, rx) = mpsc::channel(buffer_size);
        (
            Self {
                tx,
                buffer: Arc::new(RwLock::new(VecDeque::with_capacity(buffer_size))),
                max_buffered_frames: buffer_size,
            },
            rx,
        )
    }

    /// Send frame with backpressure handling
    /// Returns Ok if queued, Err if backpressure limit exceeded
    pub async fn send(&self, frame: StreamFrame) -> Result<(), BackpressureError> {
        // Async send with bounded queue provides backpressure
        self.tx.send(frame).await.map_err(|_| BackpressureError {
            message: "Channel closed or full".to_string(),
        })?;
        Ok(())
    }

    /// Try to send without blocking
    pub fn try_send(&self, frame: StreamFrame) -> Result<(), BackpressureError> {
        self.tx.try_send(frame).map_err(|_| BackpressureError {
            message: "Channel full or closed".to_string(),
        })?;
        Ok(())
    }

    /// Get number of pending frames
    pub async fn pending_frames(&self) -> usize {
        self.buffer.read().await.len()
    }
}

/// Backpressure error from frame sink
#[derive(Debug, Clone)]
pub struct BackpressureError {
    pub message: String,
}

/// Server-side streaming session handler
/// Manages a long-lived command stream with multiplexing
pub struct ServerStreamingHandler {
    sessions: Arc<RwLock<std::collections::HashMap<String, StreamingSession>>>,
    frame_sinks: Arc<RwLock<std::collections::HashMap<String, FrameSink>>>,
}

impl ServerStreamingHandler {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(std::collections::HashMap::new())),
            frame_sinks: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Create and register a new streaming session
    pub async fn create_session(
        &self,
        id: String,
        command: String,
        args: Vec<String>,
        buffer_size: usize,
    ) -> Result<(StreamingSession, FrameSink), String> {
        let session = StreamingSession::new(id.clone(), command, args);

        let (sink, _rx) = FrameSink::new(buffer_size);

        let mut sessions = self.sessions.write().await;
        let mut sinks = self.frame_sinks.write().await;

        sessions.insert(id.clone(), session.clone());
        sinks.insert(id, sink.clone());

        Ok((session, sink))
    }

    /// Get active session
    pub async fn get_session(&self, id: &str) -> Option<StreamingSession> {
        self.sessions.read().await.get(id).cloned()
    }

    /// Cancel a session
    pub async fn cancel_session(&self, id: &str, reason: Option<String>) -> Result<(), String> {
        let mut sessions = self.sessions.write().await;

        if let Some(session) = sessions.get_mut(id) {
            session.is_cancelled = true;
        } else {
            return Err(format!("Session not found: {}", id));
        }

        Ok(())
    }

    /// List all active sessions
    pub async fn list_sessions(&self) -> Vec<StreamingSession> {
        self.sessions.read().await.values().cloned().collect()
    }

    /// Cleanup completed session
    pub async fn close_session(&self, id: &str) {
        let mut sessions = self.sessions.write().await;
        let mut sinks = self.frame_sinks.write().await;

        sessions.remove(id);
        sinks.remove(id);
    }
}

impl Default for ServerStreamingHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Client-side streaming protocol handler
/// Consumes frames from server and provides convenient APIs
pub struct ClientStreamingHandler {
    session_id: String,
    rx: tokio::sync::mpsc::Receiver<StreamFrame>,
}

impl ClientStreamingHandler {
    pub fn new(session_id: String, rx: tokio::sync::mpsc::Receiver<StreamFrame>) -> Self {
        Self { session_id, rx }
    }

    /// Consume next frame from server
    pub async fn next_frame(&mut self) -> Option<StreamFrame> {
        self.rx.recv().await
    }

    /// Consume all frames until Done
    pub async fn consume_all(&mut self) -> Result<CollectedOutput, String> {
        let mut output = CollectedOutput::new(self.session_id.clone());

        while let Some(frame) = self.next_frame().await {
            match frame {
                StreamFrame::Data { payload, .. } => {
                    output.data.push(payload);
                }
                StreamFrame::Log {
                    level, message, ..
                } => {
                    output.logs.push((level, message));
                }
                StreamFrame::Metrics {
                    cpu_us,
                    memory_bytes,
                    ..
                } => {
                    output.cpu_us = cpu_us;
                    output.memory_bytes = memory_bytes;
                }
                StreamFrame::Error { code, message, .. } => {
                    return Err(format!("Error {}: {}", code, message));
                }
                StreamFrame::Done { exit_code, .. } => {
                    output.exit_code = exit_code;
                    break;
                }
                _ => {}
            }
        }

        Ok(output)
    }
}

/// Collected output from a streaming session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectedOutput {
    pub session_id: String,
    pub data: Vec<serde_json::Value>,
    pub logs: Vec<(LogLevel, String)>,
    pub cpu_us: u64,
    pub memory_bytes: u64,
    pub exit_code: i32,
}

impl CollectedOutput {
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            data: Vec::new(),
            logs: Vec::new(),
            cpu_us: 0,
            memory_bytes: 0,
            exit_code: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streaming_session_creation() {
        let session = StreamingSession::new(
            "session-1".to_string(),
            "list".to_string(),
            vec![],
        );

        assert_eq!(session.id, "session-1");
        assert_eq!(session.command, "list");
        assert!(!session.is_cancelled);
    }

    #[test]
    fn test_frame_sink_creation() {
        let (_sink, _rx) = FrameSink::new(100);
        // Sink created successfully
    }

    #[tokio::test]
    async fn test_server_streaming_handler() {
        let handler = ServerStreamingHandler::new();

        let (session, _sink) = handler
            .create_session(
                "session-1".to_string(),
                "list".to_string(),
                vec![],
                100,
            )
            .await
            .unwrap();

        assert_eq!(session.id, "session-1");

        let retrieved = handler.get_session("session-1").await;
        assert!(retrieved.is_some());

        handler.close_session("session-1").await;
        let retrieved = handler.get_session("session-1").await;
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_cancel_session() {
        let handler = ServerStreamingHandler::new();

        handler
            .create_session(
                "session-1".to_string(),
                "long-op".to_string(),
                vec![],
                100,
            )
            .await
            .unwrap();

        let result = handler
            .cancel_session("session-1", Some("User cancelled".to_string()))
            .await;
        assert!(result.is_ok());

        let session = handler.get_session("session-1").await.unwrap();
        assert!(session.is_cancelled);
    }

    #[test]
    fn test_collected_output() {
        let output = CollectedOutput::new("session-1".to_string());
        assert_eq!(output.session_id, "session-1");
        assert!(output.data.is_empty());
    }
}
