//! Integration tests for CNV Kernel Capabilities
//!
//! Tests for:
//! - Telemetry Profile
//! - Output Pipeline
//! - Grammar Model
//! - Manpage Generation
//! - File IO
//! - Test Harness

use clap_noun_verb::kernel::*;
use clap_noun_verb::OutputFormat;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestData {
    value: i32,
    name: String,
}

mod telemetry_tests {
    use super::*;

    #[test]
    fn test_verbosity_levels() {
        assert_eq!(VerbosityLevel::Silent.level(), 0);
        assert_eq!(VerbosityLevel::Normal.level(), 1);
        assert_eq!(VerbosityLevel::Verbose.level(), 2);
        assert_eq!(VerbosityLevel::Debug.level(), 3);
        assert_eq!(VerbosityLevel::Trace.level(), 4);
    }

    #[test]
    fn test_verbosity_from_counts() {
        // Normal (no flags)
        assert_eq!(VerbosityLevel::from_counts(0, false), VerbosityLevel::Normal);

        // Verbose (-v)
        assert_eq!(VerbosityLevel::from_counts(1, false), VerbosityLevel::Verbose);

        // Debug (-vv)
        assert_eq!(VerbosityLevel::from_counts(2, false), VerbosityLevel::Debug);

        // Trace (-vvv)
        assert_eq!(VerbosityLevel::from_counts(3, false), VerbosityLevel::Trace);

        // Quiet wins over verbose
        assert_eq!(VerbosityLevel::from_counts(5, true), VerbosityLevel::Silent);
    }

    #[test]
    fn test_verbosity_checks() {
        let normal = VerbosityLevel::Normal;
        assert!(!normal.is_verbose());
        assert!(!normal.is_debug());
        assert!(!normal.is_trace());

        let verbose = VerbosityLevel::Verbose;
        assert!(verbose.is_verbose());
        assert!(!verbose.is_debug());

        let debug = VerbosityLevel::Debug;
        assert!(debug.is_verbose());
        assert!(debug.is_debug());
        assert!(!debug.is_trace());

        let trace = VerbosityLevel::Trace;
        assert!(trace.is_verbose());
        assert!(trace.is_debug());
        assert!(trace.is_trace());
    }

    #[test]
    fn test_color_policy() {
        // Parse from string
        assert_eq!("auto".parse::<ColorPolicy>().ok(), Some(ColorPolicy::Auto));
        assert_eq!("always".parse::<ColorPolicy>().ok(), Some(ColorPolicy::Always));
        assert_eq!("never".parse::<ColorPolicy>().ok(), Some(ColorPolicy::Never));

        // Invalid parse
        assert!("invalid".parse::<ColorPolicy>().is_err());

        // Should colorize
        assert!(ColorPolicy::Always.should_colorize());
        assert!(!ColorPolicy::Never.should_colorize());
    }

    #[test]
    fn test_telemetry_profile_default() {
        let profile = TelemetryProfile::default();
        assert_eq!(profile.verbosity(), VerbosityLevel::Normal);
        assert_eq!(profile.color_policy(), ColorPolicy::Auto);
        assert_eq!(profile.format(), OutputFormat::Json);
        assert!(!profile.is_quiet());
    }

    #[test]
    fn test_telemetry_profile_from_args() {
        let profile =
            TelemetryProfile::from_args(2, false, ColorPolicy::Always, OutputFormat::Yaml);

        assert_eq!(profile.verbosity(), VerbosityLevel::Debug);
        assert!(profile.is_debug());
        assert!(profile.should_colorize());
        assert_eq!(profile.format(), OutputFormat::Yaml);
    }

    #[test]
    fn test_telemetry_profile_builder() {
        let profile = TelemetryProfile::builder()
            .verbose_count(1)
            .color(ColorPolicy::Never)
            .format(OutputFormat::Table)
            .build();

        assert_eq!(profile.verbosity(), VerbosityLevel::Verbose);
        assert!(profile.is_verbose());
        assert!(!profile.should_colorize());
        assert_eq!(profile.format(), OutputFormat::Table);
    }

    #[test]
    fn test_quiet_mode() {
        let profile = TelemetryProfile::from_args(0, true, ColorPolicy::Auto, OutputFormat::Json);
        assert!(profile.is_quiet());
        assert_eq!(profile.verbosity(), VerbosityLevel::Silent);
        assert!(!profile.is_verbose());
    }

    #[test]
    fn test_format_output() {
        let profile = TelemetryProfile::default();
        let data = TestData { value: 42, name: "test".to_string() };

        let output = profile.format_output(&data);
        assert!(output.is_ok());
        let json = output.unwrap();
        assert!(json.contains("\"value\""));
        assert!(json.contains("42"));
    }
}

mod output_pipeline_tests {
    use super::*;
    use clap_noun_verb::kernel::output::ExitCodeClass;

    #[test]
    fn test_structured_error_creation() {
        let err = StructuredError::new("test_error", "Something went wrong");
        assert_eq!(err.kind, "test_error");
        assert_eq!(err.message, "Something went wrong");
        assert_eq!(err.exit_code(), 1); // GeneralError
    }

    #[test]
    fn test_structured_error_with_context() {
        let err = StructuredError::new("validation_error", "Invalid input")
            .with_context("field", "username")
            .with_context("value", "ab")
            .with_exit_code(ExitCodeClass::UsageError);

        assert_eq!(err.exit_code(), 2);
        assert!(err.context.is_some());
        let ctx = err.context.unwrap();
        assert_eq!(ctx.len(), 2);
    }

    #[test]
    fn test_output_envelope_success() {
        let data = TestData { value: 42, name: "test".to_string() };
        let envelope = OutputEnvelope::success(data.clone());

        assert!(envelope.is_success());
        assert!(!envelope.is_error());

        match envelope {
            OutputEnvelope::Success { data: d, .. } => {
                assert_eq!(d, data);
            }
            _ => panic!("Expected success envelope"),
        }
    }

    #[test]
    fn test_output_envelope_error() {
        let error = StructuredError::new("test", "failed");
        let envelope: OutputEnvelope<TestData> = OutputEnvelope::error(error.clone());

        assert!(envelope.is_error());
        assert!(!envelope.is_success());

        match envelope {
            OutputEnvelope::Error { error: e, .. } => {
                assert_eq!(e.kind, error.kind);
            }
            _ => panic!("Expected error envelope"),
        }
    }

    #[test]
    fn test_output_envelope_from_result() {
        let data = TestData { value: 42, name: "test".to_string() };

        // Success case
        let result: StructuredResult<TestData> = Ok(data.clone());
        let envelope: OutputEnvelope<TestData> = result.into();
        assert!(envelope.is_success());

        // Error case
        let result: StructuredResult<TestData> = Err(StructuredError::new("test", "failed"));
        let envelope: OutputEnvelope<TestData> = result.into();
        assert!(envelope.is_error());
    }

    #[test]
    fn test_output_envelope_with_metadata() {
        let data = TestData { value: 42, name: "test".to_string() };
        let envelope = OutputEnvelope::success(data)
            .with_metadata("execution_time_ms", 100)
            .with_metadata("version", "1.0.0");

        match envelope {
            OutputEnvelope::Success { metadata, .. } => {
                assert!(metadata.is_some());
                let meta = metadata.unwrap();
                assert_eq!(meta.len(), 2);
            }
            _ => panic!("Expected success envelope"),
        }
    }

    #[test]
    fn test_output_pipeline_format() {
        let data = TestData { value: 42, name: "test".to_string() };

        // Format success
        let result: StructuredResult<TestData> = Ok(data.clone());
        let json = OutputPipeline::format(result, OutputFormat::Json);
        assert!(json.is_ok());
        assert!(json.unwrap().contains("\"value\""));

        // Format error
        let result: StructuredResult<TestData> = Err(StructuredError::new("test", "failed"));
        let json = OutputPipeline::format(result, OutputFormat::Json);
        assert!(json.is_ok());
        assert!(json.unwrap().contains("\"kind\""));
    }

    #[test]
    fn test_output_pipeline_format_with_envelope() {
        let data = TestData { value: 42, name: "test".to_string() };
        let result: StructuredResult<TestData> = Ok(data);

        let json = OutputPipeline::format_with_envelope(result, OutputFormat::Json);
        assert!(json.is_ok());
        let output = json.unwrap();
        assert!(output.contains("\"status\""));
        assert!(output.contains("\"data\""));
    }

    #[test]
    fn test_exit_code_classes() {
        assert_eq!(ExitCodeClass::Success as u8, 0);
        assert_eq!(ExitCodeClass::GeneralError as u8, 1);
        assert_eq!(ExitCodeClass::UsageError as u8, 2);
        assert_eq!(ExitCodeClass::InputError as u8, 3);
        assert_eq!(ExitCodeClass::NotFound as u8, 4);
        assert_eq!(ExitCodeClass::PermissionDenied as u8, 5);
        assert_eq!(ExitCodeClass::Timeout as u8, 6);
    }
}

mod file_io_tests {
    use super::*;
    use clap_noun_verb::kernel::io::FileIOBuilder;
    use std::path::Path;

    #[test]
    fn test_input_source_from_path() {
        // Stdin
        assert!(InputSource::from_path(None).is_stdin());
        assert!(InputSource::from_path(Some("-")).is_stdin());

        // File
        let file_input = InputSource::from_path(Some("test.txt"));
        assert!(!file_input.is_stdin());
        assert_eq!(file_input.path(), Some(Path::new("test.txt")));
    }

    #[test]
    fn test_output_sink_from_path() {
        // Stdout
        assert!(OutputSink::from_path(None).is_stdout());
        assert!(OutputSink::from_path(Some("-")).is_stdout());

        // File
        let file_output = OutputSink::from_path(Some("output.txt"));
        assert!(!file_output.is_stdout());
        assert_eq!(file_output.path(), Some(Path::new("output.txt")));
    }

    #[test]
    fn test_file_io_from_args() {
        // Both stdin/stdout
        let io = FileIO::from_args(None, None);
        assert!(io.input().is_stdin());
        assert!(io.output().is_stdout());

        // File input/output
        let io = FileIO::from_args(Some("input.txt"), Some("output.txt"));
        assert!(!io.input().is_stdin());
        assert!(!io.output().is_stdout());
        assert_eq!(io.input().path(), Some(Path::new("input.txt")));
        assert_eq!(io.output().path(), Some(Path::new("output.txt")));
    }

    #[test]
    fn test_file_io_builder() {
        let io = FileIOBuilder::new().input(Some("input.txt")).output(Some("output.txt")).build();

        assert!(!io.input().is_stdin());
        assert!(!io.output().is_stdout());
    }

    #[test]
    fn test_file_io_multiple_inputs() {
        let io = FileIO::from_args(Some("main.txt"), None)
            .add_input(InputSource::from_path(Some("extra1.txt")))
            .add_input(InputSource::from_path(Some("extra2.txt")));

        assert_eq!(io.additional_inputs().len(), 2);
        assert!(!io.additional_inputs()[0].is_stdin());
        assert!(!io.additional_inputs()[1].is_stdin());
    }

    #[test]
    fn test_file_io_default() {
        let io = FileIO::default();
        assert!(io.input().is_stdin());
        assert!(io.output().is_stdout());
        assert_eq!(io.additional_inputs().len(), 0);
    }
}

mod grammar_tests {
    use super::*;
    use clap_noun_verb::kernel::grammar::*;

    #[test]
    fn test_grammar_model_creation() {
        let model = GrammarModel::new("test-app").with_version("1.0.0");
        assert_eq!(model.app_name, "test-app");
        assert_eq!(model.app_version, Some("1.0.0".to_string()));
        assert_eq!(model.schema_version, GRAMMAR_SCHEMA_VERSION);
    }

    #[test]
    fn test_grammar_model_nouns() {
        let mut model = GrammarModel::new("test-app");
        let noun = GrammarNoun {
            name: "data".to_string(),
            help: Some("Data commands".to_string()),
            long_help: None,
            verbs: Vec::new(),
            sub_nouns: Vec::new(),
            metadata: std::collections::HashMap::new(),
        };

        model.add_noun(noun);
        assert_eq!(model.nouns().len(), 1);
        assert!(model.find_noun("data").is_some());
        assert!(model.find_noun("other").is_none());
    }

    #[test]
    fn test_argument_types() {
        assert_eq!(ArgumentType::Positional, ArgumentType::Positional);
        assert_ne!(ArgumentType::Positional, ArgumentType::Named);
        assert_ne!(ArgumentType::Flag, ArgumentType::Count);
    }
}

mod test_harness_tests {
    use super::*;
    use clap_noun_verb::kernel::test_harness::*;

    #[test]
    fn test_validation_report() {
        let mut report = ValidationReport::default();
        assert!(report.is_valid());
        assert_eq!(report.issue_count(), 0);

        report.errors.push("Error 1".to_string());
        assert!(!report.is_valid());
        assert_eq!(report.issue_count(), 1);

        report.warnings.push("Warning 1".to_string());
        assert_eq!(report.issue_count(), 2);
    }

    #[test]
    fn test_test_expectation() {
        assert_eq!(TestExpectation::Success, TestExpectation::Success);
        assert_ne!(TestExpectation::Success, TestExpectation::Error);

        let custom = TestExpectation::ExitCode(42);
        match custom {
            TestExpectation::ExitCode(code) => assert_eq!(code, 42),
            _ => panic!("Expected ExitCode"),
        }
    }

    #[test]
    fn test_test_case_creation() {
        let test_case = TestCase {
            name: "test-help".to_string(),
            args: vec!["app".to_string(), "--help".to_string()],
            expected_result: TestExpectation::Success,
            description: Some("Test help flag".to_string()),
        };

        assert_eq!(test_case.name, "test-help");
        assert_eq!(test_case.args.len(), 2);
        assert_eq!(test_case.expected_result, TestExpectation::Success);
    }
}

mod integration_tests {
    use super::*;
    use clap_noun_verb::kernel::output::ExitCodeClass;

    #[test]
    fn test_full_pipeline() {
        // Create telemetry profile
        let profile =
            TelemetryProfile::builder().verbose_count(1).format(OutputFormat::Json).build();

        assert!(profile.is_verbose());

        // Create test data
        let data = TestData { value: 42, name: "integration_test".to_string() };

        // Format with profile
        let output = profile.format_output(&data);
        assert!(output.is_ok());

        // Create structured result
        let result: StructuredResult<TestData> = Ok(data);

        // Format through pipeline
        let formatted = OutputPipeline::format(result, profile.format());
        assert!(formatted.is_ok());

        let json = formatted.unwrap();
        assert!(json.contains("\"value\""));
        assert!(json.contains("42"));
    }

    #[test]
    fn test_error_pipeline() {
        let profile = TelemetryProfile::default();

        let error = StructuredError::new("test_error", "Test failure")
            .with_context("test_id", "integration_001")
            .with_exit_code(ExitCodeClass::InputError);

        let result: StructuredResult<TestData> = Err(error);

        let formatted = OutputPipeline::format(result, profile.format());
        assert!(formatted.is_ok());

        let json = formatted.unwrap();
        assert!(json.contains("\"kind\""));
        assert!(json.contains("test_error"));
    }
}
