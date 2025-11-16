//! Phase 5.3: Transport-Neutral Invocation Protocol
//!
//! Defines a transport-agnostic protocol for sending CNV requests and receiving responses.
//!
//! ## Supported Transports
//!
//! - **StdioTransport**: Local process via stdin/stdout (current mode)
//! - **UnixSocketTransport**: Unix domain sockets for local IPC
//! - **QuicTransport**: QUIC-based RPC for cluster-level federation
//! - **TcpTransport**: TCP for remote deployments
//!
//! All share the same **binary frame format** and **multiplexing protocol**.

use crate::kernel::broker::{BrokerRequest, BrokerResponse};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;

/// Binary invocation frame - transport-agnostic message unit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvocationFrame {
    /// Protocol version
    pub version: u32,
    /// Correlation ID (for multiplexing)
    pub correlation_id: String,
    /// Tenant ID (for multi-tenant routing)
    pub tenant_id: String,
    /// Agent ID (for tracing)
    pub agent_id: String,
    /// Frame type
    pub frame_type: FrameType,
    /// Serialized payload (JSON)
    pub payload: Vec<u8>,
    /// Frame sequence number
    pub sequence_number: u64,
    /// Flags (backpressure, eof, etc.)
    pub flags: FrameFlags,
}

/// Frame type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FrameType {
    /// Request frame
    Request = 1,
    /// Response frame
    Response = 2,
    /// Streaming data frame
    Stream = 3,
    /// Control frame (heartbeat, backpressure, etc.)
    Control = 4,
    /// Error frame
    Error = 5,
}

impl FrameType {
    /// Convert u8 to FrameType
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            1 => Some(FrameType::Request),
            2 => Some(FrameType::Response),
            3 => Some(FrameType::Stream),
            4 => Some(FrameType::Control),
            5 => Some(FrameType::Error),
            _ => None,
        }
    }

    /// Convert to u8
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }
}

/// Frame flags
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct FrameFlags {
    /// Backpressure signal (pause sender)
    pub backpressure: bool,
    /// End of stream
    pub end_of_stream: bool,
    /// Priority bit
    pub priority: bool,
}

impl InvocationFrame {
    /// Create a new request frame
    pub fn new_request(
        correlation_id: String,
        tenant_id: String,
        agent_id: String,
        request: BrokerRequest,
    ) -> Result<Self, serde_json::Error> {
        let payload = serde_json::to_vec(&request)?;

        Ok(Self {
            version: 1,
            correlation_id,
            tenant_id,
            agent_id,
            frame_type: FrameType::Request,
            payload,
            sequence_number: 0,
            flags: FrameFlags::default(),
        })
    }

    /// Create a new response frame
    pub fn new_response(
        correlation_id: String,
        tenant_id: String,
        agent_id: String,
        response: BrokerResponse,
    ) -> Result<Self, serde_json::Error> {
        let payload = serde_json::to_vec(&response)?;

        Ok(Self {
            version: 1,
            correlation_id,
            tenant_id,
            agent_id,
            frame_type: FrameType::Response,
            payload,
            sequence_number: 0,
            flags: FrameFlags::default(),
        })
    }

    /// Decode payload as broker request
    pub fn decode_request(&self) -> Result<BrokerRequest, serde_json::Error> {
        serde_json::from_slice(&self.payload)
    }

    /// Decode payload as broker response
    pub fn decode_response(&self) -> Result<BrokerResponse, serde_json::Error> {
        serde_json::from_slice(&self.payload)
    }

    /// Get frame size
    pub fn frame_size(&self) -> usize {
        // Version (4) + correlation_id + tenant_id + agent_id + frame_type (1) + payload_len + sequence (8) + flags (1)
        4 + self.correlation_id.len() + self.tenant_id.len() + self.agent_id.len()
            + 1 + 4 + self.payload.len() + 8 + 1
    }
}

/// Transport error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransportError {
    ConnectionFailed(String),
    SendFailed(String),
    ReceiveFailed(String),
    TimeoutError,
    FrameFormatError(String),
    EncodingError(String),
}

impl fmt::Display for TransportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransportError::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
            TransportError::SendFailed(msg) => write!(f, "Send failed: {}", msg),
            TransportError::ReceiveFailed(msg) => write!(f, "Receive failed: {}", msg),
            TransportError::TimeoutError => write!(f, "Timeout"),
            TransportError::FrameFormatError(msg) => write!(f, "Frame format error: {}", msg),
            TransportError::EncodingError(msg) => write!(f, "Encoding error: {}", msg),
        }
    }
}

/// Transport trait - sealed interface for pluggable backends
#[async_trait]
pub trait InvocationTransport: Send + Sync {
    /// Send an invocation frame
    async fn send_frame(&self, frame: InvocationFrame) -> Result<(), TransportError>;

    /// Receive an invocation frame
    async fn recv_frame(&self) -> Result<InvocationFrame, TransportError>;

    /// Get transport name
    fn transport_name(&self) -> &'static str;

    /// Check if connected
    fn is_connected(&self) -> bool;

    /// Close connection
    async fn close(&self) -> Result<(), TransportError>;
}

/// Stdio transport - local process
pub struct StdioTransport {
    connected: std::sync::atomic::AtomicBool,
}

impl StdioTransport {
    pub fn new() -> Self {
        Self {
            connected: std::sync::atomic::AtomicBool::new(true),
        }
    }
}

impl Default for StdioTransport {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl InvocationTransport for StdioTransport {
    async fn send_frame(&self, _frame: InvocationFrame) -> Result<(), TransportError> {
        // In a real implementation, serialize to JSON and write to stdout
        if !self.is_connected() {
            return Err(TransportError::ConnectionFailed(
                "Stdio transport not connected".to_string(),
            ));
        }
        Ok(())
    }

    async fn recv_frame(&self) -> Result<InvocationFrame, TransportError> {
        // In a real implementation, read from stdin and deserialize
        Err(TransportError::ReceiveFailed(
            "Stdio transport recv not implemented".to_string(),
        ))
    }

    fn transport_name(&self) -> &'static str {
        "stdio"
    }

    fn is_connected(&self) -> bool {
        self.connected.load(std::sync::atomic::Ordering::SeqCst)
    }

    async fn close(&self) -> Result<(), TransportError> {
        self.connected.store(false, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }
}

/// Unix socket transport for local IPC
pub struct UnixSocketTransport {
    connected: std::sync::atomic::AtomicBool,
    path: String,
}

impl UnixSocketTransport {
    pub fn new(path: String) -> Self {
        Self {
            connected: std::sync::atomic::AtomicBool::new(false),
            path,
        }
    }

    pub async fn connect(&self) -> Result<(), TransportError> {
        // In a real implementation, connect to Unix socket
        self.connected.store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }
}

#[async_trait]
impl InvocationTransport for UnixSocketTransport {
    async fn send_frame(&self, _frame: InvocationFrame) -> Result<(), TransportError> {
        if !self.is_connected() {
            return Err(TransportError::ConnectionFailed(format!(
                "Unix socket not connected to {}",
                self.path
            )));
        }
        Ok(())
    }

    async fn recv_frame(&self) -> Result<InvocationFrame, TransportError> {
        Err(TransportError::ReceiveFailed(
            "Unix socket recv not implemented".to_string(),
        ))
    }

    fn transport_name(&self) -> &'static str {
        "unix-socket"
    }

    fn is_connected(&self) -> bool {
        self.connected.load(std::sync::atomic::Ordering::SeqCst)
    }

    async fn close(&self) -> Result<(), TransportError> {
        self.connected.store(false, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }
}

/// QUIC transport for cluster-level federation
pub struct QuicTransport {
    connected: std::sync::atomic::AtomicBool,
    peer_addr: String,
}

impl QuicTransport {
    pub fn new(peer_addr: String) -> Self {
        Self {
            connected: std::sync::atomic::AtomicBool::new(false),
            peer_addr,
        }
    }

    pub async fn connect(&self) -> Result<(), TransportError> {
        // In a real implementation, establish QUIC connection
        self.connected.store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }
}

#[async_trait]
impl InvocationTransport for QuicTransport {
    async fn send_frame(&self, _frame: InvocationFrame) -> Result<(), TransportError> {
        if !self.is_connected() {
            return Err(TransportError::ConnectionFailed(format!(
                "QUIC not connected to {}",
                self.peer_addr
            )));
        }
        Ok(())
    }

    async fn recv_frame(&self) -> Result<InvocationFrame, TransportError> {
        Err(TransportError::ReceiveFailed(
            "QUIC recv not implemented".to_string(),
        ))
    }

    fn transport_name(&self) -> &'static str {
        "quic"
    }

    fn is_connected(&self) -> bool {
        self.connected.load(std::sync::atomic::Ordering::SeqCst)
    }

    async fn close(&self) -> Result<(), TransportError> {
        self.connected.store(false, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }
}

/// TCP transport for remote deployments
pub struct TcpTransport {
    connected: std::sync::atomic::AtomicBool,
    peer_addr: String,
}

impl TcpTransport {
    pub fn new(peer_addr: String) -> Self {
        Self {
            connected: std::sync::atomic::AtomicBool::new(false),
            peer_addr,
        }
    }

    pub async fn connect(&self) -> Result<(), TransportError> {
        // In a real implementation, establish TCP connection
        self.connected.store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }
}

#[async_trait]
impl InvocationTransport for TcpTransport {
    async fn send_frame(&self, _frame: InvocationFrame) -> Result<(), TransportError> {
        if !self.is_connected() {
            return Err(TransportError::ConnectionFailed(format!(
                "TCP not connected to {}",
                self.peer_addr
            )));
        }
        Ok(())
    }

    async fn recv_frame(&self) -> Result<InvocationFrame, TransportError> {
        Err(TransportError::ReceiveFailed(
            "TCP recv not implemented".to_string(),
        ))
    }

    fn transport_name(&self) -> &'static str {
        "tcp"
    }

    fn is_connected(&self) -> bool {
        self.connected.load(std::sync::atomic::Ordering::SeqCst)
    }

    async fn close(&self) -> Result<(), TransportError> {
        self.connected.store(false, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_type_conversion() {
        assert_eq!(FrameType::Request.as_u8(), 1);
        assert_eq!(FrameType::from_u8(1), Some(FrameType::Request));
        assert_eq!(FrameType::from_u8(99), None);
    }

    #[test]
    fn test_invocation_frame_size() {
        let frame = InvocationFrame {
            version: 1,
            correlation_id: "abc".to_string(),
            tenant_id: "t1".to_string(),
            agent_id: "a1".to_string(),
            frame_type: FrameType::Request,
            payload: vec![1, 2, 3],
            sequence_number: 0,
            flags: FrameFlags::default(),
        };
        assert!(frame.frame_size() > 0);
    }

    #[test]
    fn test_stdio_transport() {
        let transport = StdioTransport::new();
        assert!(transport.is_connected());
        assert_eq!(transport.transport_name(), "stdio");
    }
}
