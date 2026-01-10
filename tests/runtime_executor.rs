//! Tests for runtime executor

use clap_noun_verb::error::{NounVerbError, Result};
use clap_noun_verb::logic::{HandlerContext, HandlerInput, HandlerOutput};
use clap_noun_verb::runtime::executor::Executor;
use clap_noun_verb::runtime::interceptor::NoOpInterceptor;
use std::collections::HashMap;

#[test]
fn test_executor_new() {
    let _executor = Executor::new();
    // Executor should be created successfully
    assert!(true); // Executor state is internal, just verify it compiles
}

#[test]
fn test_executor_default() {
    let _executor = Executor::default();
    // Default should work
    assert!(true);
}

#[test]
fn test_executor_execute_success() -> Result<()> {
    let executor = Executor::new();

    let handler = |input: HandlerInput| -> Result<HandlerOutput> {
        HandlerOutput::from_data(format!("Processed: {}", input.context.verb))
    };

    let input = HandlerInput {
        args: HashMap::new(),
        opts: HashMap::new(),
        context: HandlerContext::new("test"),
    };

    let output = executor.execute(handler, input)?;

    // Verify JSON output contains expected data
    let json_str = output.to_json()?;
    assert!(json_str.contains("Processed: test"));

    Ok(())
}

#[test]
fn test_executor_execute_with_error() -> Result<()> {
    let executor = Executor::new();

    let handler = |_input: HandlerInput| -> Result<HandlerOutput> {
        Err(NounVerbError::execution_error("Test error"))
    };

    let input = HandlerInput {
        args: HashMap::new(),
        opts: HashMap::new(),
        context: HandlerContext::new("test"),
    };

    let result = executor.execute(handler, input);
    assert!(result.is_err());

    Ok(())
}

#[test]
fn test_executor_with_interceptors() -> Result<()> {
    let mut executor = Executor::new();
    executor.add_interceptor(Box::new(NoOpInterceptor));

    let handler =
        |_input: HandlerInput| -> Result<HandlerOutput> { HandlerOutput::from_data("Success") };

    let input = HandlerInput {
        args: HashMap::new(),
        opts: HashMap::new(),
        context: HandlerContext::new("test"),
    };

    let output = executor.execute(handler, input)?;

    // Verify JSON output
    let json_str = output.to_json()?;
    assert!(json_str.contains("Success"));

    Ok(())
}

#[test]
fn test_executor_interceptor_chain() -> Result<()> {
    let mut executor = Executor::new();
    executor.add_interceptor(Box::new(NoOpInterceptor));
    executor.add_interceptor(Box::new(NoOpInterceptor));

    let handler = |input: HandlerInput| -> Result<HandlerOutput> {
        HandlerOutput::from_data(format!("Processed: {}", input.context.verb))
    };

    let input = HandlerInput {
        args: HashMap::new(),
        opts: HashMap::new(),
        context: HandlerContext::new("test"),
    };

    let output = executor.execute(handler, input)?;

    // Verify JSON output
    let json_str = output.to_json()?;
    assert!(json_str.contains("Processed: test"));

    Ok(())
}

#[test]
fn test_handler_output_serialization() -> Result<()> {
    use serde::Serialize;

    #[derive(Serialize)]
    struct TestData {
        message: String,
        count: usize,
    }

    let output = HandlerOutput::from_data(TestData { message: "Test".to_string(), count: 42 })?;

    let json_str = output.to_json()?;
    assert!(json_str.contains("Test"));
    assert!(json_str.contains("42"));

    Ok(())
}
