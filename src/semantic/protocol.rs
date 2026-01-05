//! MCP protocol adapter for agent-to-agent capability sharing

use super::capability::CapabilityMetadata;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors from protocol operations
#[derive(Debug, Error)]
pub enum ProtocolError {
    /// Serialization failed
    #[error("Serialization failed: {0}")]
    SerializationFailed(String),

    /// Deserialization failed
    #[error("Deserialization failed: {0}")]
    DeserializationFailed(String),

    /// Invalid MCP version
    #[error("Invalid MCP version: {0}")]
    InvalidVersion(String),

    /// Protocol mismatch
    #[error("Protocol mismatch: expected {expected}, got {actual}")]
    ProtocolMismatch { expected: String, actual: String },
}

/// MCP protocol message for capability sharing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct McpMessage {
    /// Protocol identifier
    pub protocol: String,
    /// Protocol version
    pub version: String,
    /// Capability URI
    pub capability_uri: String,
    /// Function name
    pub function_name: String,
    /// RDF metadata
    pub rdf_metadata: String,
}

/// Protocol adapter trait for capability serialization
pub trait ProtocolAdapter {
    /// Serialize capability to protocol format
    fn serialize(&self, capability: &CapabilityMetadata) -> Result<Vec<u8>, ProtocolError>;

    /// Deserialize capability from protocol format
    fn deserialize(&self, data: &[u8]) -> Result<CapabilityMetadata, ProtocolError>;
}

/// MCP protocol adapter
pub struct McpAdapter {
    /// Protocol version
    version: String,
}

impl McpAdapter {
    /// Create new MCP adapter with version
    pub fn new(version: impl Into<String>) -> Self {
        Self { version: version.into() }
    }

    /// Create adapter with default version (2024.1)
    pub fn default_version() -> Self {
        Self::new("2024.1")
    }
}

impl ProtocolAdapter for McpAdapter {
    fn serialize(&self, capability: &CapabilityMetadata) -> Result<Vec<u8>, ProtocolError> {
        let message = McpMessage {
            protocol: "mcp".to_string(),
            version: self.version.clone(),
            capability_uri: capability.uri.to_string(),
            function_name: capability.function_name.to_string(),
            rdf_metadata: capability.rdf_metadata.to_string(),
        };

        serde_json::to_vec(&message).map_err(|e| ProtocolError::SerializationFailed(e.to_string()))
    }

    fn deserialize(&self, data: &[u8]) -> Result<CapabilityMetadata, ProtocolError> {
        let message: McpMessage = serde_json::from_slice(data)
            .map_err(|e| ProtocolError::DeserializationFailed(e.to_string()))?;

        // Validate protocol
        if message.protocol != "mcp" {
            return Err(ProtocolError::ProtocolMismatch {
                expected: "mcp".to_string(),
                actual: message.protocol,
            });
        }

        // Note: This creates static strings by leaking - acceptable for capability metadata
        // In production, consider arena allocation or lazy_static
        let metadata = CapabilityMetadata {
            uri: Box::leak(message.capability_uri.into_boxed_str()),
            function_name: Box::leak(message.function_name.into_boxed_str()),
            rdf_metadata: Box::leak(message.rdf_metadata.into_boxed_str()),
            mcp_descriptor: "{}",
        };

        Ok(metadata)
    }
}

// =============================================================================
// Unit Tests - Chicago TDD
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_adapter_new() {
        // Arrange & Act: Create adapter
        let adapter = McpAdapter::new("2024.1");

        // Assert: Version set correctly
        assert_eq!(adapter.version, "2024.1");
    }

    #[test]
    fn test_mcp_adapter_default_version() {
        // Arrange & Act: Create adapter with default version
        let adapter = McpAdapter::default_version();

        // Assert: Uses default version
        assert_eq!(adapter.version, "2024.1");
    }

    #[test]
    fn test_serialize_capability() {
        // Arrange: Adapter and capability
        let adapter = McpAdapter::default_version();
        let capability = CapabilityMetadata {
            uri: "urn:test:cap",
            function_name: "test_fn",
            rdf_metadata: "test RDF",
            mcp_descriptor: "{}",
        };

        // Act: Serialize
        let result = adapter.serialize(&capability);

        // Assert: Serialization succeeds
        assert!(result.is_ok());
        let data = result.expect("serialization should succeed");
        assert!(!data.is_empty());
    }

    #[test]
    fn test_deserialize_capability() {
        // Arrange: Serialized message
        let message = McpMessage {
            protocol: "mcp".to_string(),
            version: "2024.1".to_string(),
            capability_uri: "urn:test:cap".to_string(),
            function_name: "test_fn".to_string(),
            rdf_metadata: "test RDF".to_string(),
        };
        let data = serde_json::to_vec(&message).expect("message serialization should work");
        let adapter = McpAdapter::default_version();

        // Act: Deserialize
        let result = adapter.deserialize(&data);

        // Assert: Deserialization succeeds
        assert!(result.is_ok());
        let capability = result.expect("deserialization should succeed");
        assert_eq!(capability.uri, "urn:test:cap");
        assert_eq!(capability.function_name, "test_fn");
    }

    #[test]
    fn test_roundtrip_serialization() {
        // Arrange: Adapter and capability
        let adapter = McpAdapter::default_version();
        let original = CapabilityMetadata {
            uri: "urn:test:roundtrip",
            function_name: "roundtrip_fn",
            rdf_metadata: "roundtrip RDF",
            mcp_descriptor: "{}",
        };

        // Act: Serialize then deserialize
        let serialized = adapter.serialize(&original).expect("serialization should succeed");
        let deserialized =
            adapter.deserialize(&serialized).expect("deserialization should succeed");

        // Assert: Roundtrip preserves data
        assert_eq!(original.uri, deserialized.uri);
        assert_eq!(original.function_name, deserialized.function_name);
        assert_eq!(original.rdf_metadata, deserialized.rdf_metadata);
    }

    #[test]
    fn test_deserialize_invalid_protocol() {
        // Arrange: Message with wrong protocol
        let message = McpMessage {
            protocol: "wrong".to_string(),
            version: "2024.1".to_string(),
            capability_uri: "urn:test".to_string(),
            function_name: "test".to_string(),
            rdf_metadata: "test".to_string(),
        };
        let data = serde_json::to_vec(&message).expect("message serialization should work");
        let adapter = McpAdapter::default_version();

        // Act: Deserialize
        let result = adapter.deserialize(&data);

        // Assert: Fails with protocol mismatch
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ProtocolError::ProtocolMismatch { .. }));
    }

    #[test]
    fn test_deserialize_invalid_json() {
        // Arrange: Invalid JSON data
        let adapter = McpAdapter::default_version();
        let invalid_data = b"not valid json";

        // Act: Deserialize
        let result = adapter.deserialize(invalid_data);

        // Assert: Fails with deserialization error
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ProtocolError::DeserializationFailed(_)));
    }
}
