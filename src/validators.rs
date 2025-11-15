//! Common value validators for CLI arguments
//!
//! This module provides reusable validators for common data types.
//!
//! # Example
//!
//! ```rust,ignore
//! use clap_noun_verb::validators::validate_port;
//!
//! fn handler(port: u16) -> Result<()> {
//!     validate_port(port)?;
//!     println!("Valid port: {}", port);
//!     Ok(())
//! }
//! ```

use crate::Result;
use regex::Regex;
use std::path::Path;
use url::Url;

/// Validate that a port number is in valid range (1-65535)
///
/// # Example
///
/// ```rust,ignore
/// validate_port(8080)?;  // OK
/// validate_port(65536)?; // Error
/// ```
pub fn validate_port(port: u16) -> Result<()> {
    if port == 0 {
        return Err(crate::error::NounVerbError::execution_error(
            "Port must be between 1 and 65535",
        ));
    }
    Ok(())
}

/// Validate that a string is a valid URL
///
/// # Example
///
/// ```rust,ignore
/// validate_url("https://example.com")?;  // OK
/// validate_url("not a url")?;            // Error
/// ```
pub fn validate_url(url_str: &str) -> Result<()> {
    Url::parse(url_str).map_err(|e| {
        crate::error::NounVerbError::execution_error(format!("Invalid URL: {}", e))
    })?;
    Ok(())
}

/// Validate that a string is a valid IPv4 address
///
/// # Example
///
/// ```rust,ignore
/// validate_ipv4("192.168.1.1")?;  // OK
/// validate_ipv4("999.999.999.999")?; // Error
/// ```
pub fn validate_ipv4(ip: &str) -> Result<()> {
    let parts: Vec<&str> = ip.split('.').collect();
    if parts.len() != 4 {
        return Err(crate::error::NounVerbError::execution_error(
            "IPv4 must have 4 octets",
        ));
    }

    for part in parts {
        match part.parse::<u8>() {
            Ok(_) => continue,
            Err(_) => {
                return Err(crate::error::NounVerbError::execution_error(
                    "Each IPv4 octet must be 0-255",
                ))
            }
        }
    }
    Ok(())
}

/// Validate that a string is a valid IPv6 address
pub fn validate_ipv6(ip: &str) -> Result<()> {
    ip.parse::<std::net::Ipv6Addr>()
        .map_err(|_| {
            crate::error::NounVerbError::execution_error("Invalid IPv6 address")
        })?;
    Ok(())
}

/// Validate that a path exists
///
/// # Example
///
/// ```rust,ignore
/// validate_path_exists("/etc/hosts")?;  // OK
/// validate_path_exists("/nonexistent")?; // Error
/// ```
pub fn validate_path_exists(path_str: &str) -> Result<()> {
    if !Path::new(path_str).exists() {
        return Err(crate::error::NounVerbError::execution_error(
            format!("Path does not exist: {}", path_str),
        ));
    }
    Ok(())
}

/// Validate that a path can be created (parent exists)
pub fn validate_path_creatable(path_str: &str) -> Result<()> {
    let path = Path::new(path_str);
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            return Err(crate::error::NounVerbError::execution_error(
                format!("Parent directory does not exist for: {}", path_str),
            ));
        }
    }
    Ok(())
}

/// Validate email address format
///
/// # Example
///
/// ```rust,ignore
/// validate_email("user@example.com")?;  // OK
/// validate_email("invalid")?;           // Error
/// ```
pub fn validate_email(email: &str) -> Result<()> {
    // Simple email validation regex (RFC 5322 simplified)
    let email_regex = Regex::new(r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$")
        .map_err(|_| crate::error::NounVerbError::execution_error("Email regex error"))?;

    if !email_regex.is_match(email) {
        return Err(crate::error::NounVerbError::execution_error(
            "Invalid email format",
        ));
    }
    Ok(())
}

/// Validate that value is not empty
pub fn validate_not_empty(value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(crate::error::NounVerbError::execution_error(
            "Value cannot be empty",
        ));
    }
    Ok(())
}

/// Validate string length is within range
pub fn validate_length(value: &str, min: usize, max: usize) -> Result<()> {
    let len = value.len();
    if len < min {
        return Err(crate::error::NounVerbError::execution_error(
            format!("Value must be at least {} characters", min),
        ));
    }
    if len > max {
        return Err(crate::error::NounVerbError::execution_error(
            format!("Value must be at most {} characters", max),
        ));
    }
    Ok(())
}

/// Validate using a regex pattern
pub fn validate_regex(value: &str, pattern: &str) -> Result<()> {
    let regex = Regex::new(pattern).map_err(|e| {
        crate::error::NounVerbError::execution_error(format!("Invalid regex: {}", e))
    })?;

    if !regex.is_match(value) {
        return Err(crate::error::NounVerbError::execution_error(
            format!("Value does not match pattern: {}", pattern),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_port() {
        assert!(validate_port(8080).is_ok());
        assert!(validate_port(1).is_ok());
        assert!(validate_port(65535).is_ok());
        assert!(validate_port(0).is_err());
    }

    #[test]
    fn test_validate_ipv4() {
        assert!(validate_ipv4("192.168.1.1").is_ok());
        assert!(validate_ipv4("127.0.0.1").is_ok());
        assert!(validate_ipv4("999.999.999.999").is_err());
        assert!(validate_ipv4("192.168.1").is_err());
    }

    #[test]
    fn test_validate_email() {
        assert!(validate_email("user@example.com").is_ok());
        assert!(validate_email("invalid").is_err());
    }

    #[test]
    fn test_validate_not_empty() {
        assert!(validate_not_empty("hello").is_ok());
        assert!(validate_not_empty("").is_err());
        assert!(validate_not_empty("   ").is_err());
    }

    #[test]
    fn test_validate_length() {
        assert!(validate_length("hello", 1, 10).is_ok());
        assert!(validate_length("hello", 5, 10).is_ok());
        assert!(validate_length("hello", 6, 10).is_err());
        assert!(validate_length("hello", 1, 4).is_err());
    }
}
