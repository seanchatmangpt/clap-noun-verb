//! Tests for runtime interceptor

use clap_noun_verb::error::Result;
use clap_noun_verb::logic::{HandlerContext, HandlerInput, HandlerOutput};
use clap_noun_verb::runtime::interceptor::{Interceptor, NoOpInterceptor};
use std::collections::HashMap;

#[test]
fn test_noop_interceptor_pre_execute() -> Result<()> {
    let interceptor = NoOpInterceptor;

    let input = HandlerInput {
        args: HashMap::new(),
        opts: HashMap::new(),
        context: HandlerContext::new("test"),
    };

    let original_verb = input.context.verb.clone();

    let result = interceptor.pre_execute(input)?;

    assert_eq!(result.context.verb, original_verb);

    Ok(())
}

#[test]
fn test_noop_interceptor_post_execute() -> Result<()> {
    let interceptor = NoOpInterceptor;

    let output = HandlerOutput::from_data("Test output")?;

    let json_before = output.to_json()?;
    let result = interceptor.post_execute(output)?;
    let json_after = result.to_json()?;

    assert_eq!(json_before, json_after);

    Ok(())
}
