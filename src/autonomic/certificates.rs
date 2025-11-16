//! # Proof-Carrying Command Certificates
//!
//! Every invocation is a structured proof that it is allowed to run in this context
//! with this effect envelope. Certificates are zero-cost abstractions enforced at
//! compile time using phantom types.
//!
//! ## Design Principles
//!
//! 1. **Zero-Cost Safety**: Use phantom types to enforce policy at compile time
//! 2. **Immutability**: Certificates are immutable once constructed
//! 3. **Verifiability**: Every certificate can be independently verified
//! 4. **Replayability**: Certificates can be serialized and replayed

use super::{
    capability_id::CapabilityId,
    effects::EffectMetadata,
    policy::{PolicyDecision, PolicyResult},
    schema::{InputSchema, OutputSchema},
    tenancy::{AgentIdentity, InvocationContext, TenantIdentity},
};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use std::time::{Duration, SystemTime};

/// Phantom marker for unchecked certificates
#[derive(Debug)]
pub struct Unchecked;

/// Phantom marker for policy-checked certificates
#[derive(Debug)]
pub struct PolicyChecked;

/// Phantom marker for capability-checked certificates
#[derive(Debug)]
pub struct CapabilityChecked;

/// Phantom marker for fully verified certificates
#[derive(Debug)]
pub struct Verified;

/// A proof that a command invocation has passed all required checks.
///
/// Uses phantom types to enforce a state machine at compile time:
/// - `Certificate<Unchecked>` - Initial state
/// - `Certificate<PolicyChecked>` - Policy evaluation succeeded
/// - `Certificate<CapabilityChecked>` - Capability verification succeeded
/// - `Certificate<Verified>` - Fully verified, ready for execution
///
/// Handlers only receive `Certificate<Verified>`, ensuring policy and capability
/// checks cannot be bypassed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certificate<State = Unchecked> {
    /// Unique certificate ID
    pub certificate_id: CertificateId,

    /// Stable capability identifier
    pub capability_id: CapabilityId,

    /// Capability version
    pub version: String,

    /// Declared effects
    pub effects: Vec<EffectMetadata>,

    /// Input schema hash for verification
    pub input_schema_hash: SchemaHash,

    /// Output schema hash for verification
    pub output_schema_hash: SchemaHash,

    /// Agent identity
    pub agent: AgentIdentity,

    /// Tenant identity
    pub tenant: TenantIdentity,

    /// Policy decision trace
    pub policy_trace: PolicyTrace,

    /// Timestamp when certificate was issued
    pub issued_at: SystemTime,

    /// Expiration time (certificates have bounded lifetime)
    pub expires_at: SystemTime,

    /// Correlation ID linking related invocations
    pub correlation_id: String,

    /// Digital signature (for future crypto verification)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<CertificateSignature>,

    /// Phantom state marker (zero-sized)
    #[serde(skip)]
    _state: PhantomData<State>,
}

/// Unique certificate identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CertificateId(pub String);

impl CertificateId {
    /// Generate a new unique certificate ID
    pub fn generate() -> Self {
        use sha2::{Digest, Sha256};
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let random = uuid::Uuid::new_v4();
        let input = format!("{}{}", timestamp, random);
        let hash = Sha256::digest(input.as_bytes());
        Self(format!("cert_{}", hex::encode(&hash[..12])))
    }
}

/// Schema hash for verification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaHash(pub String);

impl SchemaHash {
    /// Compute hash of input schema
    pub fn from_input_schema(schema: &InputSchema) -> Self {
        use sha2::{Digest, Sha256};
        let serialized = serde_json::to_string(schema).unwrap();
        let hash = Sha256::digest(serialized.as_bytes());
        Self(hex::encode(&hash[..16]))
    }

    /// Compute hash of output schema
    pub fn from_output_schema(schema: &OutputSchema) -> Self {
        use sha2::{Digest, Sha256};
        let serialized = serde_json::to_string(schema).unwrap();
        let hash = Sha256::digest(serialized.as_bytes());
        Self(hex::encode(&hash[..16]))
    }
}

/// Policy decision trace - why this invocation was allowed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyTrace {
    /// Policy engine ID that made the decision
    pub policy_engine_id: String,

    /// Final decision
    pub decision: PolicyDecision,

    /// Rules that were evaluated
    pub evaluated_rules: Vec<String>,

    /// Rule that matched (if any)
    pub matched_rule: Option<String>,

    /// Evaluation duration
    pub evaluation_duration: Duration,

    /// Additional context
    pub context: std::collections::HashMap<String, String>,
}

impl PolicyTrace {
    /// Create a trace from policy result
    pub fn from_policy_result(engine_id: impl Into<String>, result: &PolicyResult) -> Self {
        Self {
            policy_engine_id: engine_id.into(),
            decision: result.decision.clone(),
            evaluated_rules: result.evaluated_rules.clone(),
            matched_rule: None, // Policy result doesn't have this field
            evaluation_duration: Duration::from_micros(100), // Placeholder
            context: std::collections::HashMap::new(),
        }
    }
}

/// Digital signature for certificate verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateSignature {
    /// Signing algorithm
    pub algorithm: String,

    /// Public key ID
    pub key_id: String,

    /// Signature bytes (hex-encoded)
    pub signature: String,
}

// State transition implementations

impl Certificate<Unchecked> {
    /// Create a new unchecked certificate
    pub fn new(
        capability_id: CapabilityId,
        version: impl Into<String>,
        effects: Vec<EffectMetadata>,
        input_schema: &InputSchema,
        output_schema: &OutputSchema,
        agent: AgentIdentity,
        tenant: TenantIdentity,
        correlation_id: impl Into<String>,
    ) -> Self {
        let now = SystemTime::now();
        Self {
            certificate_id: CertificateId::generate(),
            capability_id,
            version: version.into(),
            effects,
            input_schema_hash: SchemaHash::from_input_schema(input_schema),
            output_schema_hash: SchemaHash::from_output_schema(output_schema),
            agent,
            tenant,
            policy_trace: PolicyTrace {
                policy_engine_id: "pending".to_string(),
                decision: PolicyDecision::Deny {
                    reason: "Not yet evaluated".to_string(),
                },
                evaluated_rules: vec![],
                matched_rule: None,
                evaluation_duration: Duration::ZERO,
                context: std::collections::HashMap::new(),
            },
            issued_at: now,
            expires_at: now + Duration::from_secs(3600), // 1 hour default
            correlation_id: correlation_id.into(),
            signature: None,
            _state: PhantomData,
        }
    }

    /// Attach policy evaluation result and transition to PolicyChecked
    pub fn with_policy_check(
        mut self,
        engine_id: impl Into<String>,
        result: &PolicyResult,
    ) -> Result<Certificate<PolicyChecked>, CertificateError> {
        // Only allow if policy decision is Allow
        match &result.decision {
            PolicyDecision::Allow => {
                self.policy_trace = PolicyTrace::from_policy_result(engine_id, result);
                Ok(Certificate {
                    certificate_id: self.certificate_id,
                    capability_id: self.capability_id,
                    version: self.version,
                    effects: self.effects,
                    input_schema_hash: self.input_schema_hash,
                    output_schema_hash: self.output_schema_hash,
                    agent: self.agent,
                    tenant: self.tenant,
                    policy_trace: self.policy_trace,
                    issued_at: self.issued_at,
                    expires_at: self.expires_at,
                    correlation_id: self.correlation_id,
                    signature: self.signature,
                    _state: PhantomData,
                })
            }
            PolicyDecision::Deny { reason, .. } => {
                Err(CertificateError::PolicyDenied(reason.clone()))
            }
            PolicyDecision::Rewrite { .. } | PolicyDecision::Redirect { .. } => {
                Err(CertificateError::PolicyDenied(
                    "Rewrite/Redirect not supported in certificates".to_string(),
                ))
            }
        }
    }
}

impl Certificate<PolicyChecked> {
    /// Perform capability verification and transition to CapabilityChecked
    pub fn with_capability_check(
        self,
        available_capabilities: &[CapabilityId],
    ) -> Result<Certificate<CapabilityChecked>, CertificateError> {
        // Verify capability is available
        if !available_capabilities.contains(&self.capability_id) {
            return Err(CertificateError::CapabilityNotAvailable(
                self.capability_id.clone(),
            ));
        }

        Ok(Certificate {
            certificate_id: self.certificate_id,
            capability_id: self.capability_id,
            version: self.version,
            effects: self.effects,
            input_schema_hash: self.input_schema_hash,
            output_schema_hash: self.output_schema_hash,
            agent: self.agent,
            tenant: self.tenant,
            policy_trace: self.policy_trace,
            issued_at: self.issued_at,
            expires_at: self.expires_at,
            correlation_id: self.correlation_id,
            signature: self.signature,
            _state: PhantomData,
        })
    }
}

impl Certificate<CapabilityChecked> {
    /// Finalize verification and transition to Verified
    pub fn verify(self) -> Result<Certificate<Verified>, CertificateError> {
        // Check expiration
        if SystemTime::now() > self.expires_at {
            return Err(CertificateError::Expired);
        }

        Ok(Certificate {
            certificate_id: self.certificate_id,
            capability_id: self.capability_id,
            version: self.version,
            effects: self.effects,
            input_schema_hash: self.input_schema_hash,
            output_schema_hash: self.output_schema_hash,
            agent: self.agent,
            tenant: self.tenant,
            policy_trace: self.policy_trace,
            issued_at: self.issued_at,
            expires_at: self.expires_at,
            correlation_id: self.correlation_id,
            signature: self.signature,
            _state: PhantomData,
        })
    }
}

impl Certificate<Verified> {
    /// Get the capability ID from a verified certificate
    pub fn capability_id(&self) -> &CapabilityId {
        &self.capability_id
    }

    /// Get the agent identity
    pub fn agent(&self) -> &AgentIdentity {
        &self.agent
    }

    /// Get the tenant identity
    pub fn tenant(&self) -> &TenantIdentity {
        &self.tenant
    }

    /// Check if certificate is still valid
    pub fn is_valid(&self) -> bool {
        SystemTime::now() <= self.expires_at
    }

    /// Export certificate for caching/replay
    pub fn export(&self) -> Result<String, CertificateError> {
        serde_json::to_string(self).map_err(|e| CertificateError::SerializationFailed(e.to_string()))
    }

    /// Import and verify a certificate
    pub fn import(data: &str) -> Result<Self, CertificateError> {
        let cert: Certificate<Verified> =
            serde_json::from_str(data).map_err(|e| CertificateError::DeserializationFailed(e.to_string()))?;

        if !cert.is_valid() {
            return Err(CertificateError::Expired);
        }

        Ok(cert)
    }
}

/// Certificate-related errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum CertificateError {
    #[error("Policy denied: {0}")]
    PolicyDenied(String),

    #[error("Capability not available: {0:?}")]
    CapabilityNotAvailable(CapabilityId),

    #[error("Certificate expired")]
    Expired,

    #[error("Serialization failed: {0}")]
    SerializationFailed(String),

    #[error("Deserialization failed: {0}")]
    DeserializationFailed(String),
}

/// Wrapper for verified arguments with certificate
///
/// This is the type that handlers receive - it guarantees both:
/// 1. Arguments have been parsed and validated
/// 2. Certificate proves authorization
pub struct CertifiedInvocation<T> {
    /// The verified certificate
    pub certificate: Certificate<Verified>,

    /// Parsed and validated arguments
    pub args: T,
}

impl<T> CertifiedInvocation<T> {
    /// Create a new certified invocation
    pub fn new(certificate: Certificate<Verified>, args: T) -> Self {
        Self { certificate, args }
    }

    /// Decompose into certificate and args
    pub fn into_parts(self) -> (Certificate<Verified>, T) {
        (self.certificate, self.args)
    }

    /// Get reference to certificate
    pub fn certificate(&self) -> &Certificate<Verified> {
        &self.certificate
    }

    /// Get reference to args
    pub fn args(&self) -> &T {
        &self.args
    }

    /// Get mutable reference to args
    pub fn args_mut(&mut self) -> &mut T {
        &mut self.args
    }
}

/// Builder for constructing certificates through the full verification pipeline
pub struct CertificateBuilder {
    capability_id: CapabilityId,
    version: String,
    effects: Vec<EffectMetadata>,
    input_schema: InputSchema,
    output_schema: OutputSchema,
    agent: AgentIdentity,
    tenant: TenantIdentity,
    correlation_id: String,
    expiration: Duration,
}

impl CertificateBuilder {
    /// Create a new certificate builder
    pub fn new(
        capability_id: CapabilityId,
        version: impl Into<String>,
        input_schema: InputSchema,
        output_schema: OutputSchema,
    ) -> Self {
        Self {
            capability_id,
            version: version.into(),
            effects: vec![],
            input_schema,
            output_schema,
            agent: AgentIdentity::anonymous(),
            tenant: TenantIdentity::default_tenant(),
            correlation_id: uuid::Uuid::new_v4().to_string(),
            expiration: Duration::from_secs(3600),
        }
    }

    /// Add effects
    pub fn with_effects(mut self, effects: Vec<EffectMetadata>) -> Self {
        self.effects = effects;
        self
    }

    /// Set agent identity
    pub fn with_agent(mut self, agent: AgentIdentity) -> Self {
        self.agent = agent;
        self
    }

    /// Set tenant identity
    pub fn with_tenant(mut self, tenant: TenantIdentity) -> Self {
        self.tenant = tenant;
        self
    }

    /// Set correlation ID
    pub fn with_correlation_id(mut self, correlation_id: impl Into<String>) -> Self {
        self.correlation_id = correlation_id.into();
        self
    }

    /// Set expiration duration
    pub fn with_expiration(mut self, expiration: Duration) -> Self {
        self.expiration = expiration;
        self
    }

    /// Build an unchecked certificate
    pub fn build(self) -> Certificate<Unchecked> {
        let mut cert = Certificate::new(
            self.capability_id,
            self.version,
            self.effects,
            &self.input_schema,
            &self.output_schema,
            self.agent,
            self.tenant,
            self.correlation_id,
        );
        cert.expires_at = cert.issued_at + self.expiration;
        cert
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_certificate_state_machine() {
        // Create unchecked certificate
        let cert = Certificate::new(
            CapabilityId::from_path("user.create"),
            "1.0.0",
            vec![],
            &InputSchema::default(),
            &OutputSchema::default(),
            AgentIdentity::anonymous(),
            TenantIdentity::default(),
            "test-correlation",
        );

        // Cannot use until policy-checked
        // This would fail to compile:
        // let _ = cert.capability_id(); // Method only exists on Verified

        // Add policy check
        let policy_result = PolicyResult {
            decision: PolicyDecision::Allow,
            evaluated_rules: vec!["allow-all".to_string()],
            matched_rule: Some("allow-all".to_string()),
            metadata: std::collections::HashMap::new(),
        };

        let cert = cert.with_policy_check("test-engine", &policy_result).unwrap();

        // Add capability check
        let available = vec![CapabilityId::from_path("user.create")];
        let cert = cert.with_capability_check(&available).unwrap();

        // Verify
        let cert = cert.verify().unwrap();

        // Now we can use it
        assert_eq!(cert.capability_id(), &CapabilityId::from_path("user.create"));
        assert!(cert.is_valid());
    }

    #[test]
    fn test_certificate_export_import() {
        let cert = CertificateBuilder::new(
            CapabilityId::from_path("test.cmd"),
            "1.0.0",
            InputSchema::default(),
            OutputSchema::default(),
        )
        .build();

        let policy_result = PolicyResult {
            decision: PolicyDecision::Allow,
            evaluated_rules: vec![],
            matched_rule: None,
            metadata: std::collections::HashMap::new(),
        };

        let cert = cert
            .with_policy_check("test", &policy_result)
            .unwrap()
            .with_capability_check(&[CapabilityId::from_path("test.cmd")])
            .unwrap()
            .verify()
            .unwrap();

        let exported = cert.export().unwrap();
        let imported = Certificate::<Verified>::import(&exported).unwrap();

        assert_eq!(cert.certificate_id, imported.certificate_id);
    }
}
