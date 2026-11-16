//! Deterministic OCI/container build projection.
//!
//! This module renders a Dockerfile-compatible build artifact. It never invokes
//! Docker, Podman, BuildKit, a registry, or any external process.

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerConfig {
    pub rust_image: String,
    pub runtime_image: String,
    pub package: String,
    pub binary: String,
    pub port: u16,
    pub args: Vec<String>,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ContainerRenderError {
    #[error("invalid container field '{field}': {reason}")]
    InvalidField { field: &'static str, reason: &'static str },
}

impl ContainerConfig {
    #[must_use]
    pub fn new(package: impl Into<String>, binary: impl Into<String>) -> Self {
        Self {
            rust_image: "rust:1.74-slim".to_owned(),
            runtime_image: "debian:bookworm-slim".to_owned(),
            package: package.into(),
            binary: binary.into(),
            port: 8080,
            args: Vec::new(),
        }
    }

    /// Render a deterministic multi-stage Dockerfile after validating every raw
    /// token that can affect Dockerfile grammar.
    pub fn render_dockerfile(&self) -> Result<String, ContainerRenderError> {
        validate_image(&self.rust_image, "rust_image")?;
        validate_image(&self.runtime_image, "runtime_image")?;
        validate_identifier(&self.package, "package")?;
        validate_identifier(&self.binary, "binary")?;

        let entrypoint = json_array(
            std::iter::once(self.binary.as_str()).chain(self.args.iter().map(String::as_str)),
        );
        Ok(format!(
            "FROM {rust_image} AS build\nWORKDIR /src\nCOPY . .\nRUN cargo build --release --locked -p {package}\n\nFROM {runtime_image}\nCOPY --from=build /src/target/release/{binary} /usr/local/bin/{binary}\nEXPOSE {port}\nENTRYPOINT {entrypoint}\n",
            rust_image = self.rust_image,
            package = self.package,
            runtime_image = self.runtime_image,
            binary = self.binary,
            port = self.port,
            entrypoint = entrypoint,
        ))
    }
}

fn validate_image(value: &str, field: &'static str) -> Result<(), ContainerRenderError> {
    if value.is_empty() {
        return Err(ContainerRenderError::InvalidField { field, reason: "must not be empty" });
    }
    if value.chars().any(char::is_whitespace) || value.chars().any(char::is_control) {
        return Err(ContainerRenderError::InvalidField {
            field,
            reason: "must not contain whitespace or control characters",
        });
    }
    Ok(())
}

fn validate_identifier(value: &str, field: &'static str) -> Result<(), ContainerRenderError> {
    if value.is_empty() {
        return Err(ContainerRenderError::InvalidField { field, reason: "must not be empty" });
    }
    if !value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.'))
    {
        return Err(ContainerRenderError::InvalidField {
            field,
            reason: "must contain only ASCII alphanumerics, '-', '_' or '.'",
        });
    }
    Ok(())
}

fn json_array<'a>(values: impl Iterator<Item = &'a str>) -> String {
    let encoded =
        values.map(|value| format!("\"{}\"", escape(value))).collect::<Vec<_>>().join(", ");
    format!("[{encoded}]")
}

fn escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
