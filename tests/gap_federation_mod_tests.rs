// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Behavioral integration tests for the completed bounded federation feature.

#![cfg(feature = "federated-network")]

use clap_noun_verb::error::{NounVerbError, Result};
use clap_noun_verb::federation::{
    deserialize_result, serialize_param, CapabilityAdvertiser, CapabilityDescriptor, Federated,
    FederationRegistry, InvocationEnvelope, InvocationParams, InvocationProxy, RemoteResolver,
    TrustValidator,
};
use std::time::Duration;

fn sample_descriptor() -> CapabilityDescriptor {
    CapabilityDescriptor {
        id: "cap.echo",
        description: "echoes input",
        inputs: vec!["text".to_string()],
        outputs: vec!["text".to_string()],
        handler: "echo_handler",
    }
}

#[test]
fn advertiser_lifecycle_and_resolution_are_executable() {
    let advertiser = CapabilityAdvertiser::new("node-a", "https://disco.example")
        .expect("valid advertiser");
    advertiser.advertise_startup().expect("startup");
    advertiser
        .advertise_capability(&sample_descriptor())
        .expect("capability advertisement");

    let current = CapabilityAdvertiser::get_instance().expect("configured instance");
    current.advertise_startup().expect("idempotent startup");

    let resolver = RemoteResolver::new().expect("resolver");
    let endpoint = resolver
        .resolve_capability("node-a", "cap.echo")
        .expect("advertised capability");
    assert!(endpoint.contains("cap.echo"));
    assert!(endpoint.contains("echo_handler"));

    advertiser.advertise_shutdown().expect("shutdown");
    assert!(resolver.resolve_capability("node-a", "cap.echo").is_err());
}

#[test]
fn trust_registry_registers_self() {
    let validator = TrustValidator::new("anchor").expect("validator");
    assert!(validator.validates("anchor"));
    assert!(!validator.validates("other"));
    let registry = FederationRegistry::new("node-b", validator).expect("registry");
    registry.register_self().expect("self registration");
}

#[test]
fn invocation_proxy_manufactures_bounded_envelope() {
    let proxy = InvocationProxy::new(
        "https://peer.example/capabilities/cap.echo".to_string(),
        Duration::from_millis(250),
    )
    .expect("proxy");
    let params = InvocationParams {
        capability: "cap.echo".to_string(),
        args: vec![("text".to_string(), b"hi".to_vec())],
    };

    let bytes = proxy.invoke(&params).expect("envelope manufacture");
    let envelope: InvocationEnvelope = deserialize_result(&bytes).expect("valid envelope");
    assert_eq!(envelope.timeout_ms, 250);
    assert_eq!(envelope.params, params);
    assert!(envelope.endpoint.contains("cap.echo"));
}

#[test]
fn serialization_round_trips_values() {
    let original = "federation".to_string();
    let bytes = serialize_param(&original).expect("serialize");
    let restored: String = deserialize_result(&bytes).expect("deserialize");
    assert_eq!(restored, original);
    assert_eq!(bytes, b"\"federation\"");
}

#[test]
fn invalid_json_is_a_typed_error() {
    let result: Result<i32> = deserialize_result(b"not-json");
    let error = result.expect_err("invalid JSON must fail");
    match error {
        NounVerbError::ExecutionError { message } => {
            assert!(message.contains("deserialization error"));
        }
        other => panic!("expected ExecutionError, got: {other:?}"),
    }
}

struct TestNode {
    fail_initialization: bool,
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
        if self.fail_initialization {
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
fn federated_trait_remains_object_safe() {
    let node = TestNode { fail_initialization: false };
    let dynamic: &dyn Federated = &node;
    assert_eq!(dynamic.identity(), "test-node");
    assert!(dynamic.initialize_federation().is_ok());
    assert!(dynamic.shutdown_federation().is_ok());
}
