//! A real OCI image builder: shells out to a real, already-installed
//! container build tool (`docker`, `podman`, or any OCI-compatible CLI) to
//! actually build an image from a rendered Dockerfile.
//!
//! Unlike [`crate::container::ContainerConfig`]'s pure, CONSTRUCT-only
//! Dockerfile projection (which "never invokes Docker, Podman, BuildKit, a
//! registry, or any external process"), this module is explicitly
//! effectful: [`OciBuilder::build`] spawns a real subprocess. The two stay
//! deliberately separate -- render the Dockerfile text with
//! `ContainerConfig::render_dockerfile`, write it to disk yourself, then
//! hand this module the resulting build context directory.

use std::path::{Path, PathBuf};
use std::process::Command;
use thiserror::Error;

/// Which real OCI build tool [`OciBuilder`] invokes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OciBuildTool {
    Docker,
    Podman,
    /// Any other OCI-compatible CLI accepting the same
    /// `build -t <tag> -f <dockerfile> <context>` argument shape (e.g.
    /// `nerdctl`, `buildah bud`).
    Custom(String),
}

impl OciBuildTool {
    #[must_use]
    pub fn program(&self) -> &str {
        match self {
            Self::Docker => "docker",
            Self::Podman => "podman",
            Self::Custom(program) => program.as_str(),
        }
    }
}

/// A real, immediately-executable build request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OciBuildRequest {
    /// The build context directory (passed as the builder's final
    /// positional argument).
    pub context_dir: PathBuf,
    /// Path to the Dockerfile, relative to `context_dir` or absolute.
    pub dockerfile_path: PathBuf,
    /// The image tag to build (e.g. `"my-cli:sha-123"`).
    pub tag: String,
    /// `--build-arg KEY=VALUE` pairs, in a stable (`BTreeMap`) order.
    pub build_args: std::collections::BTreeMap<String, String>,
}

impl OciBuildRequest {
    #[must_use]
    pub fn new(context_dir: impl Into<PathBuf>, dockerfile_path: impl Into<PathBuf>, tag: impl Into<String>) -> Self {
        Self {
            context_dir: context_dir.into(),
            dockerfile_path: dockerfile_path.into(),
            tag: tag.into(),
            build_args: std::collections::BTreeMap::new(),
        }
    }
}

/// The real, captured result of a build.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OciBuildResult {
    pub tag: String,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl OciBuildResult {
    #[must_use]
    pub const fn success(&self) -> bool {
        self.exit_code == 0
    }
}

#[derive(Debug, Error)]
pub enum OciBuildError {
    #[error("failed to spawn '{program}': {source}")]
    Spawn { program: String, #[source] source: std::io::Error },
}

/// Builds real OCI images by shelling out to a real, already-installed
/// build tool.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OciBuilder {
    tool: OciBuildTool,
}

impl OciBuilder {
    #[must_use]
    pub const fn new(tool: OciBuildTool) -> Self {
        Self { tool }
    }

    /// Whether the configured tool's binary is actually present on `PATH`
    /// and reports a version (a real, cheap `<program> --version` probe,
    /// not an assumption). Use this to skip a real build in an
    /// environment without the tool installed, rather than letting
    /// `build` fail with a confusing spawn error.
    #[must_use]
    pub fn is_tool_available(&self) -> bool {
        Command::new(self.tool.program())
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// The exact real argv this builder would execute for `request` --
    /// pure and deterministic, so its construction is testable without a
    /// real build tool installed. `-f` is always resolved to an absolute
    /// path (join of `context_dir`/`dockerfile_path` when the latter is
    /// relative) so the build works regardless of the spawning process's
    /// own current working directory -- `docker build` resolves a
    /// relative `-f` against its OWN process cwd, not the build context,
    /// which is a real footgun this builder closes rather than leaving
    /// for a caller to discover.
    #[must_use]
    pub fn command_line(&self, request: &OciBuildRequest) -> Vec<String> {
        let mut argv = vec![self.tool.program().to_owned(), "build".to_owned()];
        argv.push("-t".to_owned());
        argv.push(request.tag.clone());
        argv.push("-f".to_owned());
        argv.push(self.resolved_dockerfile_path(request).display().to_string());
        for (key, value) in &request.build_args {
            argv.push("--build-arg".to_owned());
            argv.push(format!("{key}={value}"));
        }
        argv.push(request.context_dir.display().to_string());
        argv
    }

    fn resolved_dockerfile_path(&self, request: &OciBuildRequest) -> PathBuf {
        if request.dockerfile_path.is_absolute() {
            request.dockerfile_path.clone()
        } else {
            request.context_dir.join(&request.dockerfile_path)
        }
    }

    /// Actually run the real build as a real subprocess, capturing real
    /// stdout/stderr/exit code. Never panics on a non-zero exit -- a
    /// failed build is a normal, real `Ok(OciBuildResult)` with
    /// `success() == false`; only a failure to spawn the process at all
    /// (the tool binary is missing, permissions, etc.) is an `Err`.
    pub fn build(&self, request: &OciBuildRequest) -> Result<OciBuildResult, OciBuildError> {
        let argv = self.command_line(request);
        let output = Command::new(&argv[0]).args(&argv[1..]).output().map_err(|source| {
            OciBuildError::Spawn { program: self.tool.program().to_owned(), source }
        })?;
        Ok(OciBuildResult {
            tag: request.tag.clone(),
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }
}

/// Write `dockerfile_contents` (from
/// `ContainerConfig::render_dockerfile()`) to `dockerfile_path` inside
/// `context_dir`, creating parent directories as needed -- the small,
/// real bridge between the pure `container.rs` projection and a real
/// `OciBuildRequest`.
pub fn write_dockerfile(
    context_dir: &Path,
    dockerfile_path: &Path,
    dockerfile_contents: &str,
) -> std::io::Result<PathBuf> {
    let full_path = context_dir.join(dockerfile_path);
    if let Some(parent) = full_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&full_path, dockerfile_contents)?;
    Ok(full_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_line_builds_the_real_argv_docker_would_receive() {
        let builder = OciBuilder::new(OciBuildTool::Docker);
        let mut request =
            OciBuildRequest::new("/tmp/build-ctx", "Dockerfile", "my-cli:sha-123");
        request.build_args.insert("VERSION".to_owned(), "26.9.1".to_owned());

        let argv = builder.command_line(&request);

        assert_eq!(
            argv,
            vec![
                "docker".to_owned(),
                "build".to_owned(),
                "-t".to_owned(),
                "my-cli:sha-123".to_owned(),
                "-f".to_owned(),
                "/tmp/build-ctx/Dockerfile".to_owned(),
                "--build-arg".to_owned(),
                "VERSION=26.9.1".to_owned(),
                "/tmp/build-ctx".to_owned(),
            ]
        );
    }

    #[test]
    fn command_line_uses_the_configured_custom_tool_program() {
        let builder = OciBuilder::new(OciBuildTool::Custom("nerdctl".to_owned()));
        let request = OciBuildRequest::new("/tmp/build-ctx", "Dockerfile", "my-cli:sha-123");
        let argv = builder.command_line(&request);
        assert_eq!(argv[0], "nerdctl");
    }

    #[test]
    fn write_dockerfile_writes_real_content_to_a_real_path_creating_parents() {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let context_dir = std::env::temp_dir().join(format!("cnv-oci-builder-test-{nanos}"));
        let full_path =
            write_dockerfile(&context_dir, Path::new("docker/Dockerfile"), "FROM scratch\n")
                .expect("write real Dockerfile");

        assert_eq!(std::fs::read_to_string(&full_path).expect("read it back"), "FROM scratch\n");
        std::fs::remove_dir_all(&context_dir).ok();
    }

    #[test]
    fn build_reports_a_real_spawn_error_for_a_nonexistent_tool() {
        let builder = OciBuilder::new(OciBuildTool::Custom(
            "definitely-not-a-real-oci-builder-binary".to_owned(),
        ));
        let request = OciBuildRequest::new(".", "Dockerfile", "nonexistent:latest");
        let error = builder.build(&request).expect_err("a missing tool binary must error");
        assert!(matches!(error, OciBuildError::Spawn { .. }));
    }

    #[test]
    fn is_tool_available_reports_false_for_a_nonexistent_tool() {
        let builder = OciBuilder::new(OciBuildTool::Custom(
            "definitely-not-a-real-oci-builder-binary".to_owned(),
        ));
        assert!(!builder.is_tool_available());
    }

    #[test]
    #[ignore = "requires a real, installed docker/podman binary; run with `cargo test -- --ignored`"]
    fn real_docker_build_actually_builds_a_trivial_real_image() {
        let builder = OciBuilder::new(OciBuildTool::Docker);
        if !builder.is_tool_available() {
            eprintln!("skipping: no real docker binary available on PATH");
            return;
        }

        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let context_dir = std::env::temp_dir().join(format!("cnv-real-oci-build-{nanos}"));
        write_dockerfile(&context_dir, Path::new("Dockerfile"), "FROM scratch\n")
            .expect("write real Dockerfile");

        let request = OciBuildRequest::new(&context_dir, "Dockerfile", "cnv-oci-builder-test:local");
        let result = builder.build(&request).expect("real docker build subprocess must spawn");

        assert!(result.success(), "real docker build failed: {}", result.stderr);

        Command::new("docker")
            .args(["rmi", "-f", "cnv-oci-builder-test:local"])
            .output()
            .ok();
        std::fs::remove_dir_all(&context_dir).ok();
    }
}
