use clap_noun_verb::cli::preprocessor::preprocess_args;
use clap_noun_verb::cli::registry::{ArgMetadata, CommandRegistry};
use clap_noun_verb::logic::{HandlerInput, HandlerOutput};
use clap_noun_verb::Result;
use serde_json::json;

#[test]
fn test_preprocessor_step_references() -> Result<()> {
    let step_results = vec![
        json!({
            "token": "secret-abc-123",
            "user": {
                "id": 42,
                "role": "admin"
            }
        }),
        json!({
            "status": "success",
            "code": 200
        }),
    ];

    // Test simple reference
    let args1 = vec!["--token".to_string(), "@{1.token}".to_string()];
    let processed1 = preprocess_args(&args1, &None, &step_results)?;
    assert_eq!(processed1, vec!["--token".to_string(), "secret-abc-123".to_string()]);

    // Test nested reference
    let args2 = vec!["--user-id".to_string(), "@{1.user.id}".to_string()];
    let processed2 = preprocess_args(&args2, &None, &step_results)?;
    assert_eq!(processed2, vec!["--user-id".to_string(), "42".to_string()]);

    // Test reference inside other text
    let args3 = vec!["--header".to_string(), "Bearer @{1.token}".to_string()];
    let processed3 = preprocess_args(&args3, &None, &step_results)?;
    assert_eq!(processed3, vec!["--header".to_string(), "Bearer secret-abc-123".to_string()]);

    Ok(())
}

#[test]
fn test_preprocessor_stdin_bindings() -> Result<()> {
    let stdin_val = Some("hello-stdin-world".to_string());

    // Test basic stdin binding @-
    let args1 = vec!["--message".to_string(), "@-".to_string()];
    let processed1 = preprocess_args(&args1, &stdin_val, &[])?;
    assert_eq!(processed1, vec!["--message".to_string(), "hello-stdin-world".to_string()]);

    // Test JSON stdin binding @-::key
    let json_stdin = Some(r#"{"session": {"id": "session-xyz", "expired": false}}"#.to_string());
    let args2 = vec!["--session-id".to_string(), "@-::session.id".to_string()];
    let processed2 = preprocess_args(&args2, &json_stdin, &[])?;
    assert_eq!(processed2, vec!["--session-id".to_string(), "session-xyz".to_string()]);

    Ok(())
}

#[test]
fn test_end_to_end_chaining() -> Result<()> {
    // Register commands for testing chaining
    CommandRegistry::register_noun("session", "Session commands");

    let username_arg = ArgMetadata {
        name: "username".to_string(),
        required: true,
        is_flag: false,
        help: Some("Username".to_string()),
        min_value: None,
        max_value: None,
        min_length: None,
        max_length: None,
        short: None,
        default_value: None,
        env: None,
        multiple: false,
        value_name: None,
        aliases: Vec::new(),
        positional: Some(1),
        action: None,
        group: None,
        requires: Vec::new(),
        conflicts_with: Vec::new(),
        value_parser: None,
        hide: false,
        next_help_heading: None,
        long_help: None,
        next_line_help: false,
        display_order: None,
        exclusive: None,
        trailing_vararg: false,
        allow_negative_numbers: false,
        value_hint: None,
        global: false,
    };

    CommandRegistry::register_verb_with_args(
        "session",
        "login",
        "Login to session",
        vec![username_arg],
        |input: HandlerInput| {
            let username = input.args.get("username").cloned().unwrap_or_default();
            HandlerOutput::from_data(json!({
                "token": "token-for-".to_string() + &username,
                "status": "active"
            }))
        },
    );

    let token_arg = ArgMetadata {
        name: "token".to_string(),
        required: true,
        is_flag: false,
        help: Some("Token".to_string()),
        min_value: None,
        max_value: None,
        min_length: None,
        max_length: None,
        short: None,
        default_value: None,
        env: None,
        multiple: false,
        value_name: None,
        aliases: Vec::new(),
        positional: Some(1),
        action: None,
        group: None,
        requires: Vec::new(),
        conflicts_with: Vec::new(),
        value_parser: None,
        hide: false,
        next_help_heading: None,
        long_help: None,
        next_line_help: false,
        display_order: None,
        exclusive: None,
        trailing_vararg: false,
        allow_negative_numbers: false,
        value_hint: None,
        global: false,
    };

    CommandRegistry::register_verb_with_args(
        "session",
        "verify",
        "Verify session token",
        vec![token_arg],
        |input: HandlerInput| {
            let token = input.args.get("token").cloned().unwrap_or_default();
            HandlerOutput::from_data(json!({
                "verified": true,
                "token_used": token
            }))
        },
    );

    // Act: Run chained execution
    let reg = CommandRegistry::get().lock().unwrap();
    let run_args = vec![
        "app".to_string(),
        "session".to_string(),
        "login".to_string(),
        "john_doe".to_string(),
        "++".to_string(),
        "session".to_string(),
        "verify".to_string(),
        "@{1.token}".to_string(),
    ];

    // This should run step 1 and step 2, substituting john_doe's token into step 2!
    let res = reg.run(run_args);
    assert!(res.is_ok(), "Chained execution failed: {:?}", res);

    Ok(())
}

#[test]
fn test_preprocessor_infinite_loop_prevention() -> Result<()> {
    let step_results = vec![json!({
        "recursive": "looping @{1.recursive}",
        "normal": "value"
    })];

    let args = vec!["@{1.recursive}".to_string()];
    let processed = preprocess_args(&args, &None, &step_results)?;
    assert_eq!(processed, vec!["looping @{1.recursive}".to_string()]);

    Ok(())
}
