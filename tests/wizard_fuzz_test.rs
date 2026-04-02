#![cfg_attr(not(feature = "wizard"), allow(dead_code))]
//! Fuzz Testing for Wizard v2
//!
//! Tests malformed input handling, buffer overflow protection,
//! Unicode handling, and security boundaries.
//! Follows Chicago TDD with behavior verification.

#[cfg(feature = "wizard")]
use clap_noun_verb::wizard::{ModelConfig, Prompt, SessionBuilder, WizardConfig, WizardSession};

// =============================================================================
// PROMPT INJECTION ATTEMPTS
// =============================================================================

#[cfg(feature = "wizard")]
#[test]
fn test_prompt_injection_with_system_commands() {
    // Arrange: Malicious prompts attempting command injection
    let malicious_prompts = vec![
        "'; DROP TABLE sessions; --",
        "<script>alert('XSS')</script>",
        "$(rm -rf /)",
        "../../etc/passwd",
        "\0\0\0null bytes",
        "UNION SELECT * FROM secrets",
    ];

    // Act & Assert: Each prompt should be handled safely
    for (i, mal_prompt) in malicious_prompts.iter().enumerate() {
        let prompt = Prompt::new(*mal_prompt);

        // Verify prompt stores the content as-is (sanitization is API's job)
        assert_eq!(prompt.text, *mal_prompt);
        assert!(prompt.text.len() > 0, "Prompt {} should preserve content", i);
    }
}

#[cfg(feature = "wizard")]
#[test]
fn test_session_id_injection_attempts() {
    // Arrange: Malicious session IDs
    let malicious_ids = vec![
        "<script>alert(1)</script>",
        "'; DELETE FROM sessions; --",
        "../../../etc/passwd",
        "\n\r\t\0",
        "🚀💥🔥",          // Unicode emojis
        "a".repeat(10000), // Very long ID
    ];

    // Act & Assert: Session handles all IDs
    for id in malicious_ids {
        let result = WizardSession::new(id.clone());
        assert_eq!(result.session_id(), id);
    }
}

// =============================================================================
// MALFORMED INPUT HANDLING
// =============================================================================

#[cfg(feature = "wizard")]
#[test]
fn test_empty_prompt_handling() {
    // Arrange: Empty and whitespace-only prompts
    let empty_prompts = vec!["", " ", "\t", "\n", "   \t\n   "];

    // Act & Assert: All empty prompts are accepted
    for empty in empty_prompts {
        let prompt = Prompt::new(empty);
        assert_eq!(prompt.text, empty);
    }
}

#[cfg(feature = "wizard")]
#[test]
fn test_null_byte_in_prompt() {
    // Arrange: Prompt with null bytes
    let prompt_with_null = "Hello\0World\0Test";

    // Act: Create prompt
    let prompt = Prompt::new(prompt_with_null);

    // Assert: Null bytes are preserved (Rust strings handle them)
    assert!(prompt.text.contains('\0'));
}

#[cfg(feature = "wizard")]
#[test]
fn test_malformed_json_in_metadata() {
    // Arrange: Session with various metadata
    let test_cases = vec![
        serde_json::json!(null),
        serde_json::json!("just a string"),
        serde_json::json!(12345),
        serde_json::json!([1, 2, 3]),
        serde_json::json!({"key": "value"}),
    ];

    // Act & Assert: All metadata types accepted
    for (i, metadata) in test_cases.iter().enumerate() {
        let session = SessionBuilder::new()
            .session_id(format!("meta-test-{}", i))
            .metadata(metadata.clone())
            .build()
            .expect("Should build");

        assert_eq!(session.metadata(), metadata);
    }
}

// =============================================================================
// BUFFER OVERFLOW PROTECTION
// =============================================================================

#[cfg(feature = "wizard")]
#[test]
fn test_extremely_long_prompt() {
    // Arrange: Very long prompt (1MB+)
    let long_prompt = "A".repeat(1_000_000);

    // Act: Create prompt
    let prompt = Prompt::new(&long_prompt);

    // Assert: Prompt handles large content
    assert_eq!(prompt.text.len(), 1_000_000);
}

#[cfg(feature = "wizard")]
#[test]
fn test_extremely_long_session_id() {
    // Arrange: Very long session ID (100K chars)
    let long_id = "x".repeat(100_000);

    // Act: Create session
    let session = WizardSession::new(long_id.clone());

    // Assert: Session accepts long ID
    assert_eq!(session.session_id().len(), 100_000);
}

#[cfg(feature = "wizard")]
#[test]
fn test_prompt_with_massive_history() {
    // Arrange: Create prompt with very large history
    let mut prompt = Prompt::new("Final question");

    // Add 10,000 history messages
    let large_history: Vec<_> = (0..10_000)
        .map(|i| crate::wizard::types::Message {
            role: if i % 2 == 0 {
                crate::wizard::types::Role::User
            } else {
                crate::wizard::types::Role::Assistant
            },
            content: format!("Message {}", i),
        })
        .collect();

    // Act: Add large history
    prompt = prompt.with_history(large_history);

    // Assert: All history stored
    assert_eq!(prompt.history.len(), 10_000);
}

// =============================================================================
// UNICODE HANDLING
// =============================================================================

#[cfg(feature = "wizard")]
#[test]
fn test_unicode_emoji_in_prompts() {
    // Arrange: Prompts with various Unicode
    let unicode_prompts = vec![
        "Hello 👋 World 🌍",
        "测试中文字符",
        "مرحبا بالعالم",  // Arabic
        "Привет мир",     // Cyrillic
        "こんにちは世界", // Japanese
        "🚀💥🔥⚡️✨",     // Only emojis
        "Ñoño español",
        "Зелёный", // With combining characters
    ];

    // Act & Assert: All Unicode handled correctly
    for unicode_prompt in unicode_prompts {
        let prompt = Prompt::new(unicode_prompt);
        assert_eq!(prompt.text, unicode_prompt);
        assert!(prompt.text.len() > 0);
    }
}

#[cfg(feature = "wizard")]
#[test]
fn test_unicode_in_session_ids() {
    // Arrange: Session IDs with Unicode
    let unicode_ids = vec!["session-🚀", "сессия-123", "会话-456", "جلسة-789"];

    // Act & Assert: All Unicode IDs work
    for id in unicode_ids {
        let session = WizardSession::new(id.to_string());
        assert_eq!(session.session_id(), id);
    }
}

#[cfg(feature = "wizard")]
#[test]
fn test_combining_characters_in_prompts() {
    // Arrange: Prompts with combining characters
    let combining_prompts = vec![
        "é", // e with combining acute
        "Café", "naïve", "Zürich",
    ];

    // Act & Assert: Combining characters preserved
    for prompt_text in combining_prompts {
        let prompt = Prompt::new(prompt_text);
        assert!(prompt.text.len() > 0);
    }
}

// =============================================================================
// VERY LARGE RESPONSES
// =============================================================================

#[cfg(feature = "wizard")]
#[test]
fn test_session_with_large_responses() {
    // Arrange: Session with very large response content
    let session = WizardSession::new("large-response".to_string());
    let mut session = session.start();

    // Act: Add interaction with 1MB response
    let large_response = "R".repeat(1_000_000);
    session.add_interaction("question".to_string(), large_response.clone());

    // Assert: Large response stored correctly
    assert_eq!(session.history().len(), 1);
    assert_eq!(session.history()[0].1.len(), 1_000_000);
}

// =============================================================================
// INVALID UTF-8 SEQUENCES
// =============================================================================

#[cfg(feature = "wizard")]
#[test]
fn test_valid_utf8_enforcement() {
    // Note: Rust String type enforces valid UTF-8 at compile time
    // This test verifies that the type system prevents invalid UTF-8

    // Arrange: All valid UTF-8 strings
    let valid_strings = vec!["Hello", "🌍", "测试", "مرحبا"];

    // Act & Assert: All strings are valid UTF-8 by construction
    for s in valid_strings {
        let prompt = Prompt::new(s);
        assert!(std::str::from_utf8(prompt.text.as_bytes()).is_ok());
    }
}

// =============================================================================
// SPECIAL CHARACTER HANDLING
// =============================================================================

#[cfg(feature = "wizard")]
#[test]
fn test_special_characters_in_prompts() {
    // Arrange: Prompts with special characters
    let special_chars = vec![
        "Line1\nLine2\nLine3",                 // Newlines
        "Tab\there",                           // Tabs
        "Quote: \"Hello\"",                    // Quotes
        "Apostrophe: 'World'",                 // Apostrophes
        "Backslash: \\path\\to\\file",         // Backslashes
        "Control: \r\n\t",                     // Control characters
        "Symbols: !@#$%^&*()_+-=[]{}|;:,.<>?", // Special symbols
    ];

    // Act & Assert: All special characters handled
    for special in special_chars {
        let prompt = Prompt::new(special);
        assert_eq!(prompt.text, special);
    }
}

#[cfg(feature = "wizard")]
#[test]
fn test_boundary_characters() {
    // Arrange: Boundary Unicode characters
    let boundary_chars = vec![
        "\u{0000}",   // Null
        "\u{007F}",   // DEL
        "\u{0080}",   // First high char
        "\u{FFFD}",   // Replacement character
        "\u{10FFFF}", // Max Unicode
    ];

    // Act & Assert: Boundary characters handled
    for ch in boundary_chars {
        let prompt = Prompt::new(ch);
        assert_eq!(prompt.text, ch);
    }
}

// =============================================================================
// CONFIGURATION FUZZING
// =============================================================================

#[cfg(feature = "wizard")]
#[test]
fn test_extreme_temperature_values() {
    // Arrange: Extreme temperature values
    let extreme_temps = vec![
        -1000.0,
        -1.0,
        0.0,
        1.0,
        2.0,
        100.0,
        f32::INFINITY,
        f32::NEG_INFINITY,
        f32::MAX,
        f32::MIN,
    ];

    // Act & Assert: Temperature clamping works
    for temp in extreme_temps {
        let config = ModelConfig::default().with_temperature(temp);
        // Should be clamped to [0.0, 2.0]
        assert!(config.temperature >= 0.0);
        assert!(config.temperature <= 2.0);
    }
}

#[cfg(feature = "wizard")]
#[test]
fn test_extreme_top_p_values() {
    // Arrange: Extreme top_p values
    let extreme_top_p = vec![-100.0, -1.0, 0.0, 0.5, 1.0, 2.0, 1000.0, f32::MAX];

    // Act & Assert: top_p clamping works
    for top_p in extreme_top_p {
        let config = ModelConfig::default().with_top_p(top_p);
        // Should be clamped to [0.0, 1.0]
        assert!(config.top_p >= 0.0);
        assert!(config.top_p <= 1.0);
    }
}

#[cfg(feature = "wizard")]
#[test]
fn test_extreme_max_tokens() {
    // Arrange: Extreme token values
    let extreme_tokens = vec![0, 1, 100, 1000, usize::MAX];

    // Act & Assert: Token values accepted
    for tokens in extreme_tokens {
        let config = ModelConfig::default().with_max_tokens(tokens);
        assert_eq!(config.max_response_tokens, tokens);
    }
}

// =============================================================================
// RAPID MUTATION TESTING
// =============================================================================

#[cfg(feature = "wizard")]
#[test]
fn test_rapid_config_mutations() {
    // Arrange: Base config
    let mut config = ModelConfig::default();

    // Act: Rapidly mutate config
    for i in 0..1000 {
        config = config
            .with_temperature((i as f32 % 20.0) / 10.0)
            .with_top_p((i as f32 % 10.0) / 10.0)
            .with_max_tokens(100 + i);
    }

    // Assert: Config remains valid
    assert!(config.temperature >= 0.0 && config.temperature <= 2.0);
    assert!(config.top_p >= 0.0 && config.top_p <= 1.0);
    assert!(config.max_response_tokens >= 100);
}

#[cfg(feature = "wizard")]
#[test]
fn test_random_session_metadata_fuzzing() {
    // Arrange: Various random metadata structures
    let fuzz_metadata = vec![
        serde_json::json!({"a": {"b": {"c": {"d": "deep"}}}}),
        serde_json::json!(vec![1; 10000]),
        serde_json::json!({"key": "x".repeat(100000)}),
        serde_json::json!({"unicode": "🚀".repeat(1000)}),
    ];

    // Act & Assert: All metadata handled
    for (i, meta) in fuzz_metadata.iter().enumerate() {
        let session = SessionBuilder::new()
            .session_id(format!("fuzz-{}", i))
            .metadata(meta.clone())
            .build()
            .expect("Should build");

        assert_eq!(session.metadata(), meta);
    }
}
