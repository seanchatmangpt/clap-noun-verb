// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Bounded federation primitives.
//!
//! The module manages admitted node and capability metadata in memory. It does
//! not perform socket or HTTP I/O; [`InvocationProxy`] manufactures a serialized
//! invocation envelope for an integration adapter to actuate.

use crate::error::{NounVerbError, Result};
use linkme::distributed_slice;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};

/// Interface for a node that participates in a federated capability network.
pub trait Federated {
    /// Discovery endpoint URL used to find peers.
    fn discovery_url(&self) -> &str;
    /// Identity of this federation node.
    fn identity(&self) -> &str;
    /// Trust anchor used to validate peers.
    fn trust_anchor(&self) -> &str;
    /// Bring the federation node online.
    fn initialize_federation(&self) -> Result<()>;
    /// Take the federation node offline.
    fn shutdown_federation(&self) -> Result<()>;
}

#[derive(Debug, Clone, Default)]
struct NodeRecord {
    discovery_url: String,
    online: bool,
    capabilities: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Default)]
struct FederationState {
    nodes: BTreeMap<String, NodeRecord>,
    current: Option<(String, String)>,
}

static FEDERATION: OnceLock<Mutex<FederationState>> = OnceLock::new();

fn federation() -> &'static Mutex<FederationState> {
    FEDERATION.get_or_init(|| Mutex::new(FederationState::default()))
}

fn lock_federation() -> Result<std::sync::MutexGuard<'static, FederationState>> {
    federation().lock().map_err(|_| NounVerbError::ExecutionError {
        message: "federated-network: registry lock poisoned".to_string(),
    })
}

/// Advertises this node's capabilities to the bounded discovery registry.
#[derive(Debug, Clone)]
pub struct CapabilityAdvertiser {
    identity: String,
    discovery_url: String,
}

impl CapabilityAdvertiser {
    /// Create and select an advertiser for the given identity and discovery URL.
    pub fn new(identity: &str, discovery_url: &str) -> Result<Self> {
        if identity.trim().is_empty() || discovery_url.trim().is_empty() {
            return Err(NounVerbError::ExecutionError {
                message: "federated-network: identity and discovery URL are required".to_string(),
            });
        }
        let advertiser = Self {
            identity: identity.to_string(),
            discovery_url: discovery_url.trim_end_matches('/').to_string(),
        };
        lock_federation()?.current = Some((advertiser.identity.clone(), advertiser.discovery_url.clone()));
        Ok(advertiser)
    }

    /// Get the most recently configured advertiser.
    pub fn get_instance() -> Result<Self> {
        let state = lock_federation()?;
        state
            .current
            .as_ref()
            .map(|(identity, discovery_url)| Self {
                identity: identity.clone(),
                discovery_url: discovery_url.clone(),
            })
            .ok_or_else(|| NounVerbError::ExecutionError {
                message: "federated-network: federation not initialized".to_string(),
            })
    }

    /// Mark this node online in the bounded registry.
    pub fn advertise_startup(&self) -> Result<()> {
        let mut state = lock_federation()?;
        let node = state.nodes.entry(self.identity.clone()).or_default();
        node.discovery_url = self.discovery_url.clone();
        node.online = true;
        Ok(())
    }

    /// Mark this node offline while preserving its declared capabilities.
    pub fn advertise_shutdown(&self) -> Result<()> {
        let mut state = lock_federation()?;
        let node = state.nodes.get_mut(&self.identity).ok_or_else(|| {
            NounVerbError::ExecutionError {
                message: format!("federated-network: node not registered: {}", self.identity),
            }
        })?;
        node.online = false;
        Ok(())
    }

    /// Advertise a single capability to the federation registry.
    pub fn advertise_capability(&self, capability: &CapabilityDescriptor) -> Result<()> {
        if capability.id.trim().is_empty() || capability.handler.trim().is_empty() {
            return Err(NounVerbError::ExecutionError {
                message: "federated-network: capability id and handler are required".to_string(),
            });
        }
        let mut state = lock_federation()?;
        let node = state.nodes.entry(self.identity.clone()).or_default();
        node.discovery_url = self.discovery_url.clone();
        let endpoint = format!(
            "{}/capabilities/{}?handler={}",
            self.discovery_url, capability.id, capability.handler
        );
        node.capabilities.insert(capability.id.to_string(), endpoint);
        Ok(())
    }
}

/// Validates peers against a configured trust anchor.
#[derive(Debug, Clone)]
pub struct TrustValidator {
    trust_anchor: String,
}

impl TrustValidator {
    /// Create a validator for a non-empty trust anchor.
    pub fn new(trust_anchor: &str) -> Result<Self> {
        if trust_anchor.trim().is_empty() {
            return Err(NounVerbError::ExecutionError {
                message: "federated-network: trust anchor cannot be empty".to_string(),
            });
        }
        Ok(Self { trust_anchor: trust_anchor.to_string() })
    }

    /// Return true when the candidate anchor matches exactly.
    #[must_use]
    pub fn validates(&self, candidate: &str) -> bool {
        self.trust_anchor == candidate
    }
}

/// Registry of federation members for this node's identity.
#[derive(Debug, Clone)]
pub struct FederationRegistry {
    identity: String,
    validator: TrustValidator,
}

impl FederationRegistry {
    /// Create a registry for the given identity and validator.
    pub fn new(identity: &str, validator: TrustValidator) -> Result<Self> {
        if identity.trim().is_empty() {
            return Err(NounVerbError::ExecutionError {
                message: "federated-network: registry identity cannot be empty".to_string(),
            });
        }
        Ok(Self { identity: identity.to_string(), validator })
    }

    /// Register this node with the bounded federation.
    pub fn register_self(&self) -> Result<()> {
        if self.validator.trust_anchor.trim().is_empty() {
            return Err(NounVerbError::ExecutionError {
                message: "federated-network: invalid trust anchor".to_string(),
            });
        }
        lock_federation()?.nodes.entry(self.identity.clone()).or_default();
        Ok(())
    }
}

/// Describes a federated capability that can be advertised and invoked.
pub struct CapabilityDescriptor {
    /// Unique capability identifier.
    pub id: &'static str,
    /// Human-readable capability description.
    pub description: &'static str,
    /// Names of the inputs the capability accepts.
    pub inputs: Vec<String>,
    /// Names of the outputs the capability produces.
    pub outputs: Vec<String>,
    /// Identifier of the handler implementing the capability.
    pub handler: &'static str,
}

/// Distributed slice of capability registration functions collected at link time.
#[distributed_slice]
pub static __CAPABILITY_REGISTRY: [fn()] = [..];

/// Resolves capabilities advertised by federation nodes.
#[derive(Debug, Clone, Default)]
pub struct RemoteResolver;

impl RemoteResolver {
    /// Create a resolver over the bounded registry.
    pub fn new() -> Result<Self> {
        let _guard = lock_federation()?;
        Ok(Self)
    }

    /// Resolve a capability on an online target node to an invocation endpoint.
    pub fn resolve_capability(&self, target: &str, capability: &str) -> Result<String> {
        let state = lock_federation()?;
        let node = state.nodes.get(target).ok_or_else(|| NounVerbError::ExecutionError {
            message: format!("federated-network: target node not found: {target}"),
        })?;
        if !node.online {
            return Err(NounVerbError::ExecutionError {
                message: format!("federated-network: target node is offline: {target}"),
            });
        }
        node.capabilities
            .get(capability)
            .cloned()
            .ok_or_else(|| NounVerbError::ExecutionError {
                message: format!(
                    "federated-network: capability not advertised: {target}/{capability}"
                ),
            })
    }
}

/// Proxy that manufactures a remote invocation envelope.
#[derive(Debug, Clone)]
pub struct InvocationProxy {
    endpoint: String,
    timeout: std::time::Duration,
}

impl InvocationProxy {
    /// Create a proxy targeting `endpoint` with a non-zero request timeout.
    pub fn new(endpoint: String, timeout: std::time::Duration) -> Result<Self> {
        if endpoint.trim().is_empty() || timeout.is_zero() {
            return Err(NounVerbError::ExecutionError {
                message: "federated-network: endpoint and non-zero timeout are required".to_string(),
            });
        }
        Ok(Self { endpoint, timeout })
    }

    /// Serialize an admitted invocation envelope. No network I/O is performed.
    pub fn invoke(&self, params: &InvocationParams) -> Result<Vec<u8>> {
        if params.capability.trim().is_empty() {
            return Err(NounVerbError::ExecutionError {
                message: "federated-network: capability cannot be empty".to_string(),
            });
        }
        serialize_param(&InvocationEnvelope {
            endpoint: self.endpoint.clone(),
            timeout_ms: self.timeout.as_millis() as u64,
            params: params.clone(),
        })
    }
}

/// Parameters for a remote capability invocation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvocationParams {
    /// Identifier of the capability to invoke.
    pub capability: String,
    /// Named arguments as serialized byte payloads.
    pub args: Vec<(String, Vec<u8>)>,
}

/// Integration-boundary envelope manufactured by [`InvocationProxy`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvocationEnvelope {
    /// Resolved endpoint.
    pub endpoint: String,
    /// Bounded timeout in milliseconds.
    pub timeout_ms: u64,
    /// Invocation parameters.
    pub params: InvocationParams,
}

/// Serialize a value to JSON bytes for transmission.
pub fn serialize_param<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    serde_json::to_vec(value).map_err(|error| NounVerbError::ExecutionError {
        message: format!("federated-network: serialization error: {error}"),
    })
}

/// Deserialize a result from JSON bytes received from a remote node.
pub fn deserialize_result<T: for<'de> Deserialize<'de>>(bytes: &[u8]) -> Result<T> {
    serde_json::from_slice(bytes).map_err(|error| NounVerbError::ExecutionError {
        message: format!("federated-network: deserialization error: {error}"),
    })
}
