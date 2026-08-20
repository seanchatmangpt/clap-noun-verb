//! Deterministic OCI/container build projection.
//!
//! This module renders a Dockerfile-compatible build artifact. It never invokes
//! Docker, Podman, BuildKit, a registry, or any external process.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerConfig {
    pub rust_image: String,
    pub runtime_image: String,
    pub package: String,
    pub binary: String,
    pub port: u16,
    pub args: Vec<String>,
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

    /// Render a deterministic multi-stage Dockerfile.
    #[must_use]
    pub fn render_dockerfile(&self) -> String {
        let entrypoint = json_array(
            std::iter::once(self.binary.as_str()).chain(self.args.iter().map(String::as_str)),
        );
        format!(
            "FROM {rust_image} AS build\nWORKDIR /src\nCOPY . .\nRUN cargo build --release --locked -p {package}\n\nFROM {runtime_image}\nCOPY --from=build /src/target/release/{binary} /usr/local/bin/{binary}\nEXPOSE {port}\nENTRYPOINT {entrypoint}\n",
            rust_image = self.rust_image,
            package = self.package,
            runtime_image = self.runtime_image,
            binary = self.binary,
            port = self.port,
            entrypoint = entrypoint,
        )
    }
}

fn json_array<'a>(values: impl Iterator<Item = &'a str>) -> String {
    let encoded =
        values.map(|value| format!("\"{}\"", escape(value))).collect::<Vec<_>>().join(", ");
    format!("[{encoded}]")
}

fn escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
