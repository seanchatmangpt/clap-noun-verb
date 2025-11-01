//! Tests for logic handler

use clap_noun_verb::logic::{HandlerContext, HandlerInput, HandlerOutput};
use std::collections::HashMap;

#[test]
fn test_handler_context_new() {
    let context = HandlerContext::new("status");
    assert_eq!(context.verb, "status");
    assert_eq!(context.noun, None);
    assert!(context.data.is_empty());
}

#[test]
fn test_handler_context_with_noun() {
    let context = HandlerContext::new("status").with_noun("services");

    assert_eq!(context.verb, "status");
    assert_eq!(context.noun, Some("services".to_string()));
}

#[test]
fn test_handler_context_with_data() {
    let context =
        HandlerContext::new("status").with_data("key1", "value1").with_data("key2", "value2");

    assert_eq!(context.data.get("key1"), Some(&"value1".to_string()));
    assert_eq!(context.data.get("key2"), Some(&"value2".to_string()));
}

#[test]
fn test_handler_context_chaining() {
    let context = HandlerContext::new("status")
        .with_noun("services")
        .with_data("env", "production")
        .with_data("region", "us-east-1");

    assert_eq!(context.verb, "status");
    assert_eq!(context.noun, Some("services".to_string()));
    assert_eq!(context.data.get("env"), Some(&"production".to_string()));
    assert_eq!(context.data.get("region"), Some(&"us-east-1".to_string()));
}

#[test]
fn test_handler_input_creation() {
    let context = HandlerContext::new("status").with_noun("services");
    let args = HashMap::new();
    let opts = HashMap::new();

    let input = HandlerInput { args, opts, context };

    assert_eq!(input.context.verb, "status");
    assert_eq!(input.context.noun, Some("services".to_string()));
    assert!(input.args.is_empty());
    assert!(input.opts.is_empty());
}

#[test]
fn test_handler_output_from_data_string() {
    let output = HandlerOutput::from_data("All services running".to_string())
        .expect("Failed to create output");

    assert_eq!(output.data.as_str(), Some("All services running"));
    assert_eq!(output.message, None);
}

#[test]
fn test_handler_output_from_data_structured() {
    let mut data = HashMap::new();
    data.insert("status".to_string(), "running".to_string());
    data.insert("count".to_string(), "5".to_string());

    let output = HandlerOutput::from_data(data.clone()).expect("Failed to create output");

    // Verify JSON contains the data
    let json_str = output.to_json().expect("Failed to serialize");
    assert!(json_str.contains("status"));
    assert!(json_str.contains("running"));
    assert!(json_str.contains("count"));
    assert!(json_str.contains("5"));
}

#[test]
fn test_handler_output_with_message() {
    let output = HandlerOutput::from_data("Test".to_string())
        .expect("Failed to create output")
        .with_message("Operation completed".to_string());

    assert_eq!(output.message, Some("Operation completed".to_string()));
    assert_eq!(output.data.as_str(), Some("Test"));
}

#[test]
fn test_handler_input_with_args_and_opts() {
    let mut args = HashMap::new();
    args.insert("service".to_string(), "api".to_string());

    let mut opts = HashMap::new();
    opts.insert("verbose".to_string(), "true".to_string());

    let context = HandlerContext::new("status");

    let input = HandlerInput { args: args.clone(), opts: opts.clone(), context };

    assert_eq!(input.args.get("service"), Some(&"api".to_string()));
    assert_eq!(input.opts.get("verbose"), Some(&"true".to_string()));
}
