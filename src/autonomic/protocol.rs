//! Protocol Negotiation for Trillion-Agent Swarms
//!
//! This module implements zero-copy protocol negotiation enabling heterogeneous
//! agent swarms to establish compatibility and feature sets.
//!
//! ## Key Features
//!
//! - **Version Negotiation**: Automatic protocol version selection
//! - **Capability Discovery**: Zero-copy capability advertisement
//! - **Feature Flags**: Efficient feature negotiation via bitfields
//! - **Backward Compatibility**: Support for legacy protocol versions
//! - **Forward Compatibility**: Graceful handling of unknown features
//!
//! ## Protocol Versions
//!
//! - **v1.0**: Base protocol (legacy, deprecated)
//! - **v2.0**: Adds streaming support
//! - **v3.0**: Adds distributed tracing
//! - **v4.0**: Current - Adds swarm phases and SIMD optimizations

use std::collections::HashSet;

/// Protocol version
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProtocolVersion {
    /// Major version
    pub major: u16,

    /// Minor version
    pub minor: u16,

    /// Patch version
    pub patch: u16,
}

impl ProtocolVersion {
    /// Create a new protocol version
    pub const fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self { major, minor, patch }
    }

    /// Protocol v1.0.0 (legacy)
    pub const V1_0: Self = Self::new(1, 0, 0);

    /// Protocol v2.0.0 (streaming)
    pub const V2_0: Self = Self::new(2, 0, 0);

    /// Protocol v3.0.0 (tracing)
    pub const V3_0: Self = Self::new(3, 0, 0);

    /// Protocol v4.0.0 (2027 swarm-native)
    pub const V4_0: Self = Self::new(4, 0, 0);

    /// Latest protocol version
    pub const LATEST: Self = Self::V4_0;

    /// Check if this version is compatible with another
    pub fn is_compatible_with(&self, other: &ProtocolVersion) -> bool {
        // Major version must match for compatibility
        self.major == other.major
    }

    /// Get minimum compatible version
    pub fn min_compatible(&self) -> Self {
        Self::new(self.major, 0, 0)
    }
}

impl std::fmt::Display for ProtocolVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "v{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Protocol features (bitfield for efficient negotiation)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProtocolFeatures(u64);

impl ProtocolFeatures {
    /// No features
    pub const NONE: Self = Self(0);

    /// Streaming support
    pub const STREAMING: Self = Self(1 << 0);

    /// Distributed tracing
    pub const TRACING: Self = Self(1 << 1);

    /// Certificate verification
    pub const CERTIFICATES: Self = Self(1 << 2);

    /// Delegation chains
    pub const DELEGATION: Self = Self(1 << 3);

    /// Policy enforcement
    pub const POLICY: Self = Self(1 << 4);

    /// Capability composition
    pub const COMPOSITION: Self = Self(1 << 5);

    /// SIMD optimizations
    pub const SIMD: Self = Self(1 << 6);

    /// Phase management
    pub const PHASES: Self = Self(1 << 7);

    /// Async runtime
    pub const ASYNC_RUNTIME: Self = Self(1 << 8);

    /// Formal verification
    pub const VERIFICATION: Self = Self(1 << 9);

    /// Advanced telemetry
    pub const TELEMETRY: Self = Self(1 << 10);

    /// All 2027 features
    pub const ALL_2027: Self = Self(
        Self::STREAMING.0
            | Self::TRACING.0
            | Self::CERTIFICATES.0
            | Self::DELEGATION.0
            | Self::POLICY.0
            | Self::COMPOSITION.0
            | Self::SIMD.0
            | Self::PHASES.0
            | Self::ASYNC_RUNTIME.0
            | Self::VERIFICATION.0
            | Self::TELEMETRY.0,
    );

    /// Check if feature is enabled
    pub const fn has(&self, feature: Self) -> bool {
        (self.0 & feature.0) != 0
    }

    /// Enable a feature
    pub const fn with(&self, feature: Self) -> Self {
        Self(self.0 | feature.0)
    }

    /// Disable a feature
    pub const fn without(&self, feature: Self) -> Self {
        Self(self.0 & !feature.0)
    }

    /// Intersection of two feature sets
    pub const fn intersect(&self, other: &Self) -> Self {
        Self(self.0 & other.0)
    }

    /// Union of two feature sets
    pub const fn union(&self, other: &Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Count enabled features
    pub const fn count(&self) -> u32 {
        self.0.count_ones()
    }
}

/// Protocol capabilities advertised by an agent
#[derive(Debug, Clone)]
pub struct ProtocolCapabilities {
    /// Supported protocol versions (in preference order)
    pub versions: Vec<ProtocolVersion>,

    /// Supported features
    pub features: ProtocolFeatures,

    /// Maximum message size
    pub max_message_size: usize,

    /// Maximum concurrent invocations
    pub max_concurrent_invocations: u32,

    /// Supported compression algorithms
    pub compression: Vec<CompressionAlgorithm>,

    /// Agent metadata
    pub metadata: std::collections::HashMap<String, String>,
}

impl ProtocolCapabilities {
    /// Create capabilities for 2027 swarm-native agent
    pub fn swarm_native_2027() -> Self {
        Self {
            versions: vec![ProtocolVersion::V4_0, ProtocolVersion::V3_0, ProtocolVersion::V2_0],
            features: ProtocolFeatures::ALL_2027,
            max_message_size: 16 * 1024 * 1024, // 16MB
            max_concurrent_invocations: 10_000,
            compression: vec![
                CompressionAlgorithm::Zstd,
                CompressionAlgorithm::Lz4,
                CompressionAlgorithm::None,
            ],
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Create minimal legacy capabilities
    pub fn legacy() -> Self {
        Self {
            versions: vec![ProtocolVersion::V1_0],
            features: ProtocolFeatures::NONE,
            max_message_size: 1024 * 1024, // 1MB
            max_concurrent_invocations: 100,
            compression: vec![CompressionAlgorithm::None],
            metadata: std::collections::HashMap::new(),
        }
    }
}

impl Default for ProtocolCapabilities {
    fn default() -> Self {
        Self::swarm_native_2027()
    }
}

/// Compression algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompressionAlgorithm {
    /// No compression
    None,

    /// LZ4 - Fast compression
    Lz4,

    /// Zstandard - High compression ratio
    Zstd,

    /// Brotli - Web-optimized
    Brotli,
}

/// Protocol negotiation result
#[derive(Debug, Clone)]
pub struct NegotiationResult {
    /// Selected protocol version
    pub version: ProtocolVersion,

    /// Agreed feature set (intersection)
    pub features: ProtocolFeatures,

    /// Selected compression
    pub compression: CompressionAlgorithm,

    /// Negotiated message size limit
    pub max_message_size: usize,

    /// Negotiated concurrency limit
    pub max_concurrent_invocations: u32,
}

/// Protocol negotiator
pub struct ProtocolNegotiator {
    /// Local capabilities
    local: ProtocolCapabilities,
}

impl ProtocolNegotiator {
    /// Create a new protocol negotiator
    pub fn new(local: ProtocolCapabilities) -> Self {
        Self { local }
    }

    /// Negotiate protocol with remote agent
    pub fn negotiate(&self, remote: &ProtocolCapabilities) -> Result<NegotiationResult, String> {
        // Find highest mutually supported version
        let version = self
            .negotiate_version(remote)
            .ok_or_else(|| "No compatible protocol version".to_string())?;

        // Intersect feature sets
        let features = self.local.features.intersect(&remote.features);

        // Select best compression
        let compression = self
            .negotiate_compression(remote)
            .ok_or_else(|| "No compatible compression algorithm".to_string())?;

        // Use minimum message size
        let max_message_size = self.local.max_message_size.min(remote.max_message_size);

        // Use minimum concurrency
        let max_concurrent_invocations =
            self.local.max_concurrent_invocations.min(remote.max_concurrent_invocations);

        Ok(NegotiationResult {
            version,
            features,
            compression,
            max_message_size,
            max_concurrent_invocations,
        })
    }

    /// Negotiate protocol version
    fn negotiate_version(&self, remote: &ProtocolCapabilities) -> Option<ProtocolVersion> {
        // Build set of remote versions for fast lookup
        let remote_versions: HashSet<_> = remote.versions.iter().copied().collect();

        // Find highest version in local preference order that remote also supports
        self.local.versions.iter().find(|v| remote_versions.contains(v)).copied()
    }

    /// Negotiate compression algorithm
    fn negotiate_compression(&self, remote: &ProtocolCapabilities) -> Option<CompressionAlgorithm> {
        // Build set of remote algorithms
        let remote_algos: HashSet<_> = remote.compression.iter().copied().collect();

        // Find first match in local preference order
        self.local.compression.iter().find(|a| remote_algos.contains(a)).copied()
    }
}

/// Zero-copy protocol message
///
/// Messages use borrowed slices to avoid allocations
pub struct ProtocolMessage<'a> {
    /// Message type
    pub msg_type: MessageType,

    /// Message payload (zero-copy)
    pub payload: &'a [u8],

    /// Correlation ID for request/response matching
    pub correlation_id: u64,
}

impl<'a> ProtocolMessage<'a> {
    /// Create a new protocol message
    pub fn new(msg_type: MessageType, payload: &'a [u8], correlation_id: u64) -> Self {
        Self { msg_type, payload, correlation_id }
    }

    /// Get message size
    pub fn size(&self) -> usize {
        std::mem::size_of::<MessageType>() + std::mem::size_of::<u64>() + self.payload.len()
    }
}

/// Protocol message types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MessageType {
    /// Capability advertisement
    Capabilities = 0,

    /// Negotiation request
    NegotiationRequest = 1,

    /// Negotiation response
    NegotiationResponse = 2,

    /// Invocation request
    Invocation = 3,

    /// Invocation response
    Response = 4,

    /// Heartbeat
    Heartbeat = 5,

    /// Error message
    Error = 6,

    /// Shutdown notification
    Shutdown = 7,
}

impl MessageType {
    /// Convert from u8
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(MessageType::Capabilities),
            1 => Some(MessageType::NegotiationRequest),
            2 => Some(MessageType::NegotiationResponse),
            3 => Some(MessageType::Invocation),
            4 => Some(MessageType::Response),
            5 => Some(MessageType::Heartbeat),
            6 => Some(MessageType::Error),
            7 => Some(MessageType::Shutdown),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_version_compatibility() {
        let v4_0 = ProtocolVersion::V4_0;
        let v4_1 = ProtocolVersion::new(4, 1, 0);
        let v3_0 = ProtocolVersion::V3_0;

        assert!(v4_0.is_compatible_with(&v4_1));
        assert!(!v4_0.is_compatible_with(&v3_0));
    }

    #[test]
    fn test_protocol_features() {
        let features = ProtocolFeatures::NONE
            .with(ProtocolFeatures::STREAMING)
            .with(ProtocolFeatures::TRACING);

        assert!(features.has(ProtocolFeatures::STREAMING));
        assert!(features.has(ProtocolFeatures::TRACING));
        assert!(!features.has(ProtocolFeatures::SIMD));

        let features = features.without(ProtocolFeatures::STREAMING);
        assert!(!features.has(ProtocolFeatures::STREAMING));
    }

    #[test]
    fn test_feature_intersection() {
        let f1 = ProtocolFeatures::STREAMING
            .with(ProtocolFeatures::TRACING)
            .with(ProtocolFeatures::SIMD);

        let f2 = ProtocolFeatures::STREAMING
            .with(ProtocolFeatures::CERTIFICATES)
            .with(ProtocolFeatures::SIMD);

        let intersection = f1.intersect(&f2);

        assert!(intersection.has(ProtocolFeatures::STREAMING));
        assert!(intersection.has(ProtocolFeatures::SIMD));
        assert!(!intersection.has(ProtocolFeatures::TRACING));
        assert!(!intersection.has(ProtocolFeatures::CERTIFICATES));
    }

    #[test]
    fn test_protocol_negotiation() {
        let local = ProtocolCapabilities::swarm_native_2027();
        let remote = ProtocolCapabilities {
            versions: vec![ProtocolVersion::V3_0, ProtocolVersion::V2_0],
            features: ProtocolFeatures::STREAMING.with(ProtocolFeatures::TRACING),
            max_message_size: 8 * 1024 * 1024,
            max_concurrent_invocations: 5_000,
            compression: vec![CompressionAlgorithm::Lz4],
            metadata: std::collections::HashMap::new(),
        };

        let negotiator = ProtocolNegotiator::new(local);
        let result = negotiator.negotiate(&remote).unwrap();

        assert_eq!(result.version, ProtocolVersion::V3_0);
        assert!(result.features.has(ProtocolFeatures::STREAMING));
        assert!(result.features.has(ProtocolFeatures::TRACING));
        assert!(!result.features.has(ProtocolFeatures::SIMD));
        assert_eq!(result.compression, CompressionAlgorithm::Lz4);
        assert_eq!(result.max_message_size, 8 * 1024 * 1024);
        assert_eq!(result.max_concurrent_invocations, 5_000);
    }

    #[test]
    fn test_negotiation_failure() {
        let local = ProtocolCapabilities {
            versions: vec![ProtocolVersion::V4_0],
            features: ProtocolFeatures::ALL_2027,
            max_message_size: 1024,
            max_concurrent_invocations: 100,
            compression: vec![CompressionAlgorithm::Zstd],
            metadata: std::collections::HashMap::new(),
        };

        let remote = ProtocolCapabilities {
            versions: vec![ProtocolVersion::V1_0],
            features: ProtocolFeatures::NONE,
            max_message_size: 1024,
            max_concurrent_invocations: 100,
            compression: vec![CompressionAlgorithm::Lz4],
            metadata: std::collections::HashMap::new(),
        };

        let negotiator = ProtocolNegotiator::new(local);
        let result = negotiator.negotiate(&remote);

        assert!(result.is_err());
    }

    #[test]
    fn test_zero_copy_message() {
        let payload = b"test payload data";
        let msg = ProtocolMessage::new(MessageType::Invocation, payload, 42);

        assert_eq!(msg.msg_type, MessageType::Invocation);
        assert_eq!(msg.payload, payload);
        assert_eq!(msg.correlation_id, 42);
        assert!(msg.size() > payload.len());
    }

    #[test]
    fn test_feature_count() {
        let features = ProtocolFeatures::ALL_2027;
        assert_eq!(features.count(), 11); // All 11 2027 features

        let minimal = ProtocolFeatures::STREAMING.with(ProtocolFeatures::TRACING);
        assert_eq!(minimal.count(), 2);
    }
}
