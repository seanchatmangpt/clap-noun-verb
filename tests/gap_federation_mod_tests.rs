// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Behavioral integration tests for `src/federation/mod.rs`.
//!
//! The entire `federation` module is gated behind the `federated-network`
//! feature, so every test here is compiled only under that feature.
//! Run with: `cargo test --test gap_federation_mod_tests --features federated-network`

#![cfg(feature = "federated-network")]

use clap_noun_verb::error::{NounVerbError, Result};
use clap_noun_verb::federation::{
    deserialize_result, serialize_param, CapabilityAdvertiser, CapabilityDescriptor, Federated,
    FederationRegistry, InvocationParams, InvocationProxy, RemoteResolver, TrustValidator,
};
use std::time::Duration;

// ---- helpers ---------------------------------------------------------------

fn assert_not_implemented(err: &NounVerbError) {
    match err {
        NounVerbError::ExecutionError { message } => {
            assert!(
                message.contains("federated-network"),
                "expected federated-network message, got: {message}"
            );
        }
        other => panic!("expected ExecutionError, got: {other:?}"),
    }
}

fn sample_descriptor() -> CapabilityDescriptor {
    CapabilityDescriptor {
        id: "cap.echo",
        description: "echoes input",
        inputs: vec!["text".to_string()],
        outputs: vec!["text".to_string()],
        handler: "echo_handler",
    }
}

// ---- CapabilityAdvertiser --------------------------------------------------

#[test]
fn test_capability_advertiser_new_with_valid_args_returns_ok() {
    // Arrange / Act
    let result = CapabilityAdvertiser::new("node-a", "https://disco.example");

    // Assert
    assert!(result.is_ok(), "new should succeed for valid args");
    // Construct again to confirm it is repeatable / not a singleton side effect.
    assert!(CapabilityAdvertiser::new("node-b", "https://disco2.example").is_ok());
}

#[test]
fn test_capability_advertiser_get_instance_uninitialized_errors() {
    // Act
    let result = CapabilityAdvertiser::get_instance();

    // Assert
    let err = result.unwrap_err();
    match &err {
        NounVerbError::ExecutionError { message } => {
            assert!(message.contains("not initialized"), "got: {message}");
        }
        other => panic!("expected ExecutionError, got: {other:?}"),
    }
}

#[test]
fn test_capability_advertiser_advertise_startup_errors_not_implemented() {
    // Arrange
    let adv = CapabilityAdvertiser::new("node", "https://d").expect("new ok");

    // Act
    let err = adv.advertise_startup().unwrap_err();

    // Assert
    assert_not_implemented(&err);
}

#[test]
fn test_capability_advertiser_advertise_shutdown_errors_not_implemented() {
    let adv = CapabilityAdvertiser::new("node", "https://d").expect("new ok");
    let err = adv.advertise_shutdown().unwrap_err();
    assert_not_implemented(&err);
}

#[test]
fn test_capability_advertiser_advertise_capability_errors_not_implemented() {
    // Arrange
    let adv = CapabilityAdvertiser::new("node", "https://d").expect("new ok");
    let cap = sample_descriptor();

    // Act
    let err = adv.advertise_capability(&cap).unwrap_err();

    // Assert
    assert_not_implemented(&err);
}

// ---- TrustValidator --------------------------------------------------------

#[test]
fn test_trust_validator_new_with_anchor_returns_ok() {
    let result = TrustValidator::new("anchor-pem");
    assert!(result.is_ok(), "TrustValidator::new should succeed");
}

// ---- FederationRegistry ----------------------------------------------------

#[test]
fn test_federation_registry_new_with_validator_returns_ok() {
    // Arrange
    let validator = TrustValidator::new("anchor").expect("validator ok");

    // Act
    let result = FederationRegistry::new("node-id", validator);

    // Assert
    assert!(result.is_ok(), "registry construction should succeed");
}

#[test]
fn test_federation_registry_register_self_errors_not_implemented() {
    // Arrange
    let validator = TrustValidator::new("anchor").expect("validator ok");
    let registry = FederationRegistry::new("node-id", validator).expect("registry ok");

    // Act
    let err = registry.register_self().unwrap_err();

    // Assert
    assert_not_implemented(&err);
}

// ---- RemoteResolver --------------------------------------------------------

#[test]
fn test_remote_resolver_new_errors_not_implemented() {
    // Act
    let err = RemoteResolver::new().unwrap_err();

    // Assert
    assert_not_implemented(&err);
}

// ---- InvocationProxy -------------------------------------------------------

#[test]
fn test_invocation_proxy_new_with_endpoint_returns_ok() {
    let result = InvocationProxy::new("https://peer.example".to_string(), Duration::from_secs(5));
    assert!(result.is_ok(), "proxy construction should succeed");
}

#[test]
fn test_invocation_proxy_invoke_errors_not_implemented() {
    // Arrange
    let proxy = InvocationProxy::new("https://peer".to_string(), Duration::from_millis(10))
        .expect("proxy ok");
    let params = InvocationParams {
        capability: "cap.echo".to_string(),
        args: vec![("text".to_string(), b"hi".to_vec())],
    };

    // Act
    let err = proxy.invoke(&params).unwrap_err();

    // Assert
    assert_not_implemented(&err);
}

// ---- serialize_param / deserialize_result round-trip -----------------------

#[test]
fn test_serialize_then_deserialize_string_round_trips_value() {
    // Arrange
    let original = "federation".to_string();

    // Act
    let bytes = serialize_param(&original).expect("serialize ok");
    let restored: String = deserialize_result(&bytes).expect("deserialize ok");

    // Assert
    assert_eq!(restored, original);
    // JSON-encoded string includes quotes.
    assert_eq!(bytes, b"\"federation\"");
}

#[test]
fn test_serialize_then_deserialize_struct_round_trips_fields() {
    // Arrange
    let original = InvocationParams {
        capability: "cap.sum".to_string(),
        args: vec![("a".to_string(), vec![1, 2, 3])],
    };

    // Act
    let bytes = serialize_param(&original).expect("serialize ok");
    let restored: InvocationParams = deserialize_result(&bytes).expect("deserialize ok");

    // Assert
    assert_eq!(restored.capability, "cap.sum");
    assert_eq!(restored.args, vec![("a".to_string(), vec![1u8, 2, 3])]);
}

#[test]
fn test_deserialize_result_with_invalid_json_errors() {
    // Arrange: bytes that are not valid JSON for an i32
    let bad = b"not-json";

    // Act
    let result: Result<i32> = deserialize_result(bad);

    // Assert
    let err = result.unwrap_err();
    match &err {
        NounVerbError::ExecutionError { message } => {
            assert!(message.contains("deserialization error"), "got: {message}");
        }
        other => panic!("expected ExecutionError, got: {other:?}"),
    }
}

// ---- Federated trait contract ----------------------------------------------

struct TestNode {
    initialized_err: bool,
}

impl Federated for TestNode {
    fn discovery_url(&self) -> &str {
        "https://disco.test"
    }
    fn identity(&self) -> &str {
        "test-node"
    }
    fn trust_anchor(&self) -> &str {
        "test-anchor"
    }
    fn initialize_federation(&self) -> Result<()> {
        if self.initialized_err {
            Err(NounVerbError::ExecutionError { message: "boom".to_string() })
        } else {
            Ok(())
        }
    }
    fn shutdown_federation(&self) -> Result<()> {
        Ok(())
    }
}

#[test]
fn test_federated_trait_accessors_return_configured_values() {
    // Arrange
    let node = TestNode { initialized_err: false };

    // Act / Assert
    assert_eq!(node.discovery_url(), "https://disco.test");
    assert_eq!(node.identity(), "test-node");
    assert_eq!(node.trust_anchor(), "test-anchor");
}

#[test]
fn test_federated_trait_lifecycle_methods_reflect_implementation() {
    // Arrange
    let ok_node = TestNode { initialized_err: false };
    let bad_node = TestNode { initialized_err: true };

    // Act / Assert: success path
    assert!(ok_node.initialize_federation().is_ok());
    assert!(ok_node.shutdown_federation().is_ok());

    // Failure path propagates the implementor's error
    let err = bad_node.initialize_federation().unwrap_err();
    match err {
        NounVerbError::ExecutionError { message } => assert_eq!(message, "boom"),
        other => panic!("expected ExecutionError, got: {other:?}"),
    }
}

#[test]
fn test_federated_trait_is_object_safe_via_dyn() {
    // Arrange: trait must be usable as a trait object (dyn-compatible).
    let node = TestNode { initialized_err: false };
    let dyn_node: &dyn Federated = &node;

    // Act / Assert
    assert_eq!(dyn_node.identity(), "test-node");
    assert!(dyn_node.initialize_federation().is_ok());
}
