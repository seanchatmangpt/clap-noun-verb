//! Tests for runtime executor

use clap_noun_verb::error::{NounVerbError, Result};
use clap_noun_verb::logic::{HandlerContext, HandlerInput, HandlerOutput};
use clap_noun_verb::runtime::executor::Executor;
use clap_noun_verb::runtime::interceptor::{Interceptor, NoOpInterceptor};
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

    assert_eq!(output.data.as_str(), Some("Processed: test"));

    Ok(())
}

#[test]
fn test_executor_execute_with_error() {
    let executor = Executor::new();

    let handler = |_input: HandlerInput| -> Result<HandlerOutput> {
        Err(NounVerbError::execution_error("Handler failed"))
    };

    let input = HandlerInput {
        args: HashMap::new(),
        opts: HashMap::new(),
        context: HandlerContext::new("test"),
    };

    let result = executor.execute(handler, input);

    assert!(result.is_err());
    match result {
        Err(NounVerbError::ExecutionError { message }) => {
            assert_eq!(message, "Handler failed");
        }
        _ => {
            panic!("Expected ExecutionError");
        }
    }
}

#[test]
fn test_executor_with_noop_interceptor() -> Result<()> {
    let mut executor = Executor::new();
    executor.add_interceptor(Box::new(NoOpInterceptor));

    let handler = |_input: HandlerInput| -> Result<HandlerOutput> {
        HandlerOutput::from_data("Success".to_string())
    };

    let input = HandlerInput {
        args: HashMap::new(),
        opts: HashMap::new(),
        context: HandlerContext::new("test"),
    };

    let output = executor.execute(handler, input)?;

    assert_eq!(output.data.as_str(), Some("Success"));

    Ok(())
}

#[test]
fn test_executor_with_interceptors() -> Result<()> {
    let interceptors: Vec<Box<dyn Interceptor>> =
        vec![Box::new(NoOpInterceptor), Box::new(NoOpInterceptor)];

    let executor = Executor::with_interceptors(interceptors);

    let handler = |input: HandlerInput| -> Result<HandlerOutput> {
        HandlerOutput::from_data(format!("Processed: {}", input.context.verb))
    };

    let input = HandlerInput {
        args: HashMap::new(),
        opts: HashMap::new(),
        context: HandlerContext::new("test"),
    };

    let output = executor.execute(handler, input)?;

    assert_eq!(output.data.as_str(), Some("Processed: test"));

    Ok(())
}

#[test]
fn test_noop_interceptor_pre_execute() -> Result<()> {
    let interceptor = NoOpInterceptor;

    let input = HandlerInput {
        args: HashMap::new(),
        opts: HashMap::new(),
        context: HandlerContext::new("test"),
    };

    let result = interceptor.pre_execute(input.clone())?;

    // No-op interceptor should return input unchanged
    assert_eq!(result.context.verb, input.context.verb);

    Ok(())
}

#[test]
fn test_noop_interceptor_post_execute() -> Result<()> {
    let interceptor = NoOpInterceptor;

    let output = HandlerOutput::from_data("Test".to_string())?;

    let result = interceptor.post_execute(output.clone())?;

    // No-op interceptor should return output unchanged
    assert_eq!(result.data, output.data);

    Ok(())
}
