//! I/O Type Detection for Macro Expansion
//!
//! Provides advanced type detection for Input and Output types,
//! enabling auto-wiring of I/O parameters in the #[verb] macro.

use syn::{GenericArgument, PathArguments, Type};

/// Represents detected I/O types in function parameters
// FUTURE: v4.0 Phase 2 - I/O type detection for #[verb] macro auto-wiring
// This enum will be integrated into verb macro expansion for automatic
// clio::Input/Output parameter detection and clap configuration generation.
// Keeping this code for future macro enhancement work.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetectedIoType {
    /// clio::Input - required input file/stdin
    Input,
    /// clio::Output - required output file/stdout
    Output,
    /// Option<clio::Output> - optional output
    OutputOptional,
    /// Not an I/O type
    NonIo,
}

impl DetectedIoType {
    /// Check if this is an I/O type
    // FUTURE: v4.0 Phase 2 - Used for filtering I/O parameters during macro expansion
    #[allow(dead_code)]
    pub fn is_io(&self) -> bool {
        !matches!(self, Self::NonIo)
    }

    /// Get value parser expression for this type
    // FUTURE: v4.0 Phase 2 - Generates clap::value_parser() calls in macro output
    #[allow(dead_code)]
    pub fn value_parser(&self) -> &'static str {
        match self {
            Self::Input => "clio::Input::value_parser()",
            Self::Output | Self::OutputOptional => "clio::Output::value_parser()",
            Self::NonIo => "",
        }
    }

    /// Get help text for this type
    // FUTURE: v4.0 Phase 2 - Generates automatic help text in macro output
    #[allow(dead_code)]
    pub fn help_text(&self) -> &'static str {
        match self {
            Self::Input => "Input file or path (use '-' for stdin)",
            Self::Output => "Output file or path (use '-' for stdout)",
            Self::OutputOptional => "Optional output file or path (use '-' for stdout)",
            Self::NonIo => "",
        }
    }
}

/// Detect I/O type from a syn::Type
// FUTURE: v4.0 Phase 2 - Main detection function called during #[verb] macro expansion
#[allow(dead_code)]
pub fn detect_io_type(ty: &Type) -> DetectedIoType {
    // Check for Option<T> first
    if let Type::Path(type_path) = ty {
        if is_option_path(type_path) {
            // Extract inner type from Option<T>
            if let Some(inner) = extract_option_inner(type_path) {
                // Check if inner type is Output
                if is_output_type(&inner) {
                    return DetectedIoType::OutputOptional;
                }
            }
            return DetectedIoType::NonIo;
        }
    }

    // Check for direct Input/Output types
    if is_input_type(ty) {
        return DetectedIoType::Input;
    }

    if is_output_type(ty) {
        return DetectedIoType::Output;
    }

    DetectedIoType::NonIo
}

/// Check if a type is clio::Input
// FUTURE: v4.0 Phase 2 - Helper for detect_io_type()
#[allow(dead_code)]
fn is_input_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        // Check last segment for "Input"
        if let Some(last_seg) = type_path.path.segments.last() {
            return last_seg.ident == "Input";
        }
    }
    false
}

/// Check if a type is clio::Output
// FUTURE: v4.0 Phase 2 - Helper for detect_io_type()
#[allow(dead_code)]
fn is_output_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        // Check last segment for "Output"
        if let Some(last_seg) = type_path.path.segments.last() {
            return last_seg.ident == "Output";
        }
    }
    false
}

/// Check if a type path represents Option<T>
// FUTURE: v4.0 Phase 2 - Helper for detecting Option<clio::Output>
#[allow(dead_code)]
fn is_option_path(type_path: &syn::TypePath) -> bool {
    if let Some(last_seg) = type_path.path.segments.last() {
        return last_seg.ident == "Option";
    }
    false
}

/// Extract inner type from Option<T>
// FUTURE: v4.0 Phase 2 - Helper for extracting T from Option<T>
#[allow(dead_code)]
fn extract_option_inner(type_path: &syn::TypePath) -> Option<Type> {
    if let Some(last_seg) = type_path.path.segments.last() {
        if let PathArguments::AngleBracketed(args) = &last_seg.arguments {
            // Get first generic argument
            if let Some(GenericArgument::Type(inner_type)) = args.args.first() {
                return Some(inner_type.clone());
            }
        }
    }
    None
}

/// I/O argument configuration
// FUTURE: v4.0 Phase 2 - Configuration struct for generating clap attributes
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct IoArgConfig {
    pub io_type: DetectedIoType,
    pub value_parser: String,
    pub help: String,
    pub is_positional: bool,
}

impl IoArgConfig {
    /// Create configuration from detected type
    // FUTURE: v4.0 Phase 2 - Factory method for creating clap configuration
    #[allow(dead_code)]
    pub fn from_detected(detected: DetectedIoType, arg_name: &str) -> Option<Self> {
        if detected.is_io() {
            Some(Self {
                io_type: detected.clone(),
                value_parser: detected.value_parser().to_string(),
                help: detected.help_text().to_string(),
                is_positional: arg_name.starts_with("input")
                    || arg_name == "src"
                    || arg_name == "source",
            })
        } else {
            None
        }
    }

    /// Generate clap configuration tokens
    // FUTURE: v4.0 Phase 2 - Generates code for clap attribute methods
    #[allow(dead_code)]
    pub fn clap_config(&self) -> String {
        format!(".value_parser({})\n.help(\"{}\")", self.value_parser, self.help)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_detect_input() {
        let ty: Type = parse_quote!(Input);
        assert_eq!(detect_io_type(&ty), DetectedIoType::Input);
    }

    #[test]
    fn test_detect_output() {
        let ty: Type = parse_quote!(Output);
        assert_eq!(detect_io_type(&ty), DetectedIoType::Output);
    }

    #[test]
    fn test_detect_optional_output() {
        let ty: Type = parse_quote!(Option<Output>);
        assert_eq!(detect_io_type(&ty), DetectedIoType::OutputOptional);
    }

    #[test]
    fn test_non_io_type() {
        let ty: Type = parse_quote!(String);
        assert_eq!(detect_io_type(&ty), DetectedIoType::NonIo);
    }

    #[test]
    fn test_io_value_parser() {
        assert_eq!(DetectedIoType::Input.value_parser(), "clio::Input::value_parser()");
    }
}
