//! Integration: File System Operations
//!
//! Glue code for file I/O operations.
//! Domain logic stays pure - this handles all side effects.

use std::fs;
use std::path::Path;

/// Ensure output directory exists
pub fn ensure_output_dir(path: &str) -> Result<(), String> {
    fs::create_dir_all(path)
        .map_err(|e| format!("Failed to create output directory: {}", e))
}

/// Write paper content to file
pub fn write_paper(path: &str, content: &str) -> Result<(), String> {
    // Ensure parent directory exists
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    fs::write(path, content)
        .map_err(|e| format!("Failed to write paper: {}", e))
}

/// Read configuration file (if exists)
/// FUTURE: Used for persistent config loading
#[allow(dead_code)]
pub fn read_config_file(path: &str) -> Result<Option<String>, String> {
    match fs::read_to_string(path) {
        Ok(content) => Ok(Some(content)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("Failed to read config: {}", e)),
    }
}

/// Write configuration file
/// FUTURE: Used for persistent config saving
#[allow(dead_code)]
pub fn write_config_file(path: &str, content: &str) -> Result<(), String> {
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    fs::write(path, content)
        .map_err(|e| format!("Failed to write config: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_ensure_output_dir_creates_nested() {
        let temp_dir = env::temp_dir().join("playground_test_io");
        let nested = temp_dir.join("nested/deep/dir");

        let result = ensure_output_dir(nested.to_str().unwrap());
        assert!(result.is_ok());
        assert!(nested.exists());

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_read_config_file_missing_returns_none() {
        let result = read_config_file("/nonexistent/path/config.toml");
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }
}
