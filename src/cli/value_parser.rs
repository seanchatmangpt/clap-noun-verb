// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Value parser pattern matching and application
//!
//! This module handles parsing and applying value_parser expressions
//! from their string representations using pattern matching.
//!
//! For range patterns like `clap::value_parser!(u16).range(1..=65535)`,
//! users should use the `#[validate(min = ..., max = ...)]` attribute instead,
//! which is already supported via the existing validation system.

/// Apply value parser from string representation
///
/// This function parses common value_parser patterns and applies them.
/// For range patterns, it extracts min/max bounds which are then handled
/// by the existing validation system.
pub fn apply_value_parser(arg: &mut clap::Arg, vp_str: &str) -> bool {
    // Skip placeholder for explicit but unsupported expressions
    if vp_str == "__explicit__" {
        return false;
    }

    // Match: clap::value_parser!(T) - simple type parser (no range)
    // Example: clap::value_parser!(PathBuf)
    if vp_str.contains("value_parser!") {
        if vp_str.contains("PathBuf") {
            *arg = arg.clone().value_parser(clap::value_parser!(std::path::PathBuf));
            return true;
        } else if vp_str.contains("IpAddr") && !vp_str.contains("Ipv4") && !vp_str.contains("Ipv6")
        {
            *arg = arg.clone().value_parser(clap::value_parser!(std::net::IpAddr));
            return true;
        } else if vp_str.contains("Ipv4Addr") {
            *arg = arg.clone().value_parser(clap::value_parser!(std::net::Ipv4Addr));
            return true;
        } else if vp_str.contains("Ipv6Addr") {
            *arg = arg.clone().value_parser(clap::value_parser!(std::net::Ipv6Addr));
            return true;
        } else if vp_str.contains("Url") {
            // Url requires url crate - only apply if available
            // For now, skip - users must ensure url feature is enabled
            return false;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::apply_value_parser;

    #[test]
    fn test_explicit_placeholder_returns_false_and_attaches_nothing() {
        // Arrange
        let mut arg = clap::Arg::new("input");

        // Act
        let applied = apply_value_parser(&mut arg, "__explicit__");

        // Assert
        assert!(!applied, "__explicit__ placeholder must report no parser applied");
    }

    #[test]
    fn test_pathbuf_value_parser_returns_true() {
        // Arrange
        let mut arg = clap::Arg::new("path");

        // Act
        let applied = apply_value_parser(&mut arg, "clap::value_parser!(PathBuf)");

        // Assert
        assert!(applied, "PathBuf value_parser must report applied");
    }

    #[test]
    fn test_pathbuf_value_parser_actually_attaches_pathbuf_parser() {
        // Arrange: an arg that takes a value, mutated by apply_value_parser.
        let mut arg = clap::Arg::new("path").long("path");
        let applied = apply_value_parser(&mut arg, "clap::value_parser!(PathBuf)");
        assert!(applied);

        // Build a command containing the mutated arg.
        let cmd = clap::Command::new("test").no_binary_name(true).arg(arg);

        // Act: parse a value through the command.
        let matches = cmd
            .try_get_matches_from(["--path", "/tmp/some/file.txt"])
            .expect("parsing a valid path value should succeed");

        // Assert: round-trip — the value is typed as PathBuf, proving the
        // parser was actually attached (not merely a true return value).
        let parsed = matches
            .get_one::<std::path::PathBuf>("path")
            .expect("path value should be retrievable as PathBuf");
        assert_eq!(parsed, &std::path::PathBuf::from("/tmp/some/file.txt"));
    }

    #[test]
    fn test_ipaddr_value_parser_returns_true() {
        // Arrange
        let mut arg = clap::Arg::new("addr");

        // Act
        let applied = apply_value_parser(&mut arg, "clap::value_parser!(IpAddr)");

        // Assert
        assert!(applied, "IpAddr value_parser must report applied");
    }

    #[test]
    fn test_ipv4addr_value_parser_returns_true() {
        // Arrange
        let mut arg = clap::Arg::new("addr");

        // Act
        let applied = apply_value_parser(&mut arg, "clap::value_parser!(Ipv4Addr)");

        // Assert
        assert!(applied, "Ipv4Addr value_parser must report applied");
    }

    #[test]
    fn test_ipv6addr_value_parser_returns_true() {
        // Arrange
        let mut arg = clap::Arg::new("addr");

        // Act
        let applied = apply_value_parser(&mut arg, "clap::value_parser!(Ipv6Addr)");

        // Assert
        assert!(applied, "Ipv6Addr value_parser must report applied");
    }

    #[test]
    fn test_url_value_parser_is_documented_skip_returns_false() {
        // Arrange
        let mut arg = clap::Arg::new("url");

        // Act
        let applied = apply_value_parser(&mut arg, "clap::value_parser!(Url)");

        // Assert
        assert!(!applied, "Url is a documented skip and must report not applied");
    }

    #[test]
    fn test_unknown_type_in_value_parser_returns_false() {
        // Arrange
        let mut arg = clap::Arg::new("count");

        // Act
        let applied = apply_value_parser(&mut arg, "clap::value_parser!(u16)");

        // Assert
        assert!(!applied, "unrecognized value_parser type must report not applied");
    }

    #[test]
    fn test_non_value_parser_string_returns_false() {
        // Arrange
        let mut arg = clap::Arg::new("name");

        // Act
        let applied = apply_value_parser(&mut arg, "some plain string");

        // Assert
        assert!(!applied, "a string without value_parser! must report not applied");
    }
}
