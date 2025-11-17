//! Integration tests for I/O module
//!
//! Tests basic and advanced I/O functionality with clio integration.

#[cfg(test)]
mod io_tests {
    use clap_noun_verb::io::{IoType, IoTypeRegistry};

    #[test]
    fn test_io_type_detection() {
        let registry = IoTypeRegistry::new();
        assert!(registry.is_io_type("Input"));
        assert!(registry.is_io_type("Output"));
        assert!(!registry.is_io_type("String"));
    }

    #[test]
    fn test_io_type_properties() {
        assert!(IoType::Input.is_input());
        assert!(IoType::Output.is_output());
        assert!(IoType::OutputOptional.is_optional());
    }

    #[test]
    fn test_io_type_value_parser() {
        assert_eq!(
            IoType::Input.value_parser_expr(),
            "clio::Input::value_parser()"
        );
        assert_eq!(
            IoType::Output.value_parser_expr(),
            "clio::Output::value_parser()"
        );
    }

    #[test]
    fn test_io_error_creation() {
        use clap_noun_verb::io::IoError;
        use std::path::PathBuf;

        let err = IoError::NotFound(PathBuf::from("test.txt"));
        assert!(err.to_string().contains("test.txt"));
        assert!(err.path().is_some());
    }

    #[test]
    fn test_io_error_with_context() {
        use clap_noun_verb::io::IoError;

        let err = IoError::custom("base error").with_context("additional info");
        let err_str = err.to_string();
        assert!(err_str.contains("base error"));
        assert!(err_str.contains("additional info"));
    }

    #[test]
    fn test_registry_registration() {
        let registry = IoTypeRegistry::new();
        let result = registry.register(
            "CustomIO".to_string(),
            IoType::Custom {
                name: "CustomIO".to_string(),
                properties: std::collections::HashMap::new(),
            },
        );
        assert!(result.is_ok());
        assert!(registry.is_io_type("CustomIO"));
    }

    #[test]
    fn test_io_type_list() {
        let registry = IoTypeRegistry::new();
        let types = registry.list_types();
        assert!(!types.is_empty());
        assert!(types.iter().any(|(name, _)| name == "Input"));
        assert!(types.iter().any(|(name, _)| name == "Output"));
    }

    #[test]
    fn test_io_help_text() {
        assert!(IoType::Input.help_text().contains("stdin"));
        assert!(IoType::Output.help_text().contains("stdout"));
    }

    #[test]
    fn test_io_type_optional_detection() {
        let output_opt = IoType::OutputOptional;
        assert!(output_opt.is_output());
        assert!(output_opt.is_optional());
        assert!(!output_opt.is_input());
    }

    #[test]
    fn test_custom_io_type() {
        let mut props = std::collections::HashMap::new();
        props.insert("format".to_string(), "json".to_string());

        let custom = IoType::Custom {
            name: "JsonInput".to_string(),
            properties: props,
        };

        assert!(!custom.is_input());
        assert!(!custom.is_output());
        assert!(custom.to_string().contains("JsonInput"));
    }

    #[test]
    fn test_io_error_io_conversion() {
        use clap_noun_verb::io::IoError;
        use std::io;

        let io_err = io::Error::new(io::ErrorKind::NotFound, "test");
        let io_error: IoError = io_err.into();
        assert!(matches!(io_error, IoError::Io(_)));
    }

    #[test]
    fn test_io_type_registry_default() {
        let registry1 = IoTypeRegistry::new();
        let registry2 = IoTypeRegistry::default();

        let types1 = registry1.list_types();
        let types2 = registry2.list_types();

        assert_eq!(types1.len(), types2.len());
    }

    #[test]
    fn test_io_module_version() {
        assert_eq!(clap_noun_verb::io::IO_MODULE_VERSION, "4.0.0");
        assert!(clap_noun_verb::io::CLIO_AVAILABLE);
    }

    #[test]
    fn test_io_type_clone() {
        let io_type = IoType::Input;
        let cloned = io_type.clone();
        assert_eq!(io_type, cloned);
    }

    #[test]
    fn test_io_type_custom_clone() {
        let custom = IoType::Custom {
            name: "Test".to_string(),
            properties: std::collections::HashMap::new(),
        };
        let cloned = custom.clone();
        assert_eq!(custom, cloned);
    }
}

#[cfg(test)]
mod io_pipeline_tests {
    use clap_noun_verb::io::pipeline;

    #[test]
    fn test_pipeline_builder() {
        let pipe = pipeline()
            .buffer_size(16384)
            .build();

        assert_eq!(pipe.buffer_size(), 16384);
        assert!(pipe.inputs().is_empty());
        assert!(pipe.output().is_none());
    }

    #[test]
    fn test_pipeline_default() {
        let pipe = pipeline().build();
        assert_eq!(pipe.buffer_size(), 8192);
    }

    #[test]
    fn test_pipeline_buffer_size() {
        let sizes = [1024, 4096, 8192, 16384, 65536];
        for size in &sizes {
            let pipe = pipeline().buffer_size(*size).build();
            assert_eq!(pipe.buffer_size(), *size);
        }
    }
}

#[cfg(test)]
mod io_error_tests {
    use clap_noun_verb::io::IoError;
    use std::path::PathBuf;

    #[test]
    fn test_error_path_variants() {
        let path = PathBuf::from("/tmp/test.txt");

        let errors = vec![
            IoError::Path {
                path: path.clone(),
                reason: "test".to_string(),
            },
            IoError::Format {
                path: path.clone(),
                reason: "invalid format".to_string(),
            },
            IoError::PermissionDenied {
                path: path.clone(),
                operation: "read".to_string(),
            },
            IoError::NotFound(path.clone()),
        ];

        for err in errors {
            assert_eq!(err.path(), Some(&path));
        }
    }

    #[test]
    fn test_error_reason_extraction() {
        let err = IoError::NotFound(PathBuf::from("file.txt"));
        let reason = err.reason();
        assert_eq!(reason, "file not found");
    }

    #[test]
    fn test_error_custom_display() {
        let err = IoError::custom("test error");
        assert!(err.to_string().contains("test error"));
    }

    #[test]
    fn test_error_encoding() {
        let err = IoError::Encoding {
            path: PathBuf::from("file.txt"),
            expected: "UTF-8".to_string(),
            found: "Latin-1".to_string(),
        };

        let err_str = err.to_string();
        assert!(err_str.contains("UTF-8"));
        assert!(err_str.contains("Latin-1"));
    }
}
