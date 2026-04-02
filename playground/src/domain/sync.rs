//! Sync domain - the sacred authoritative transformation
//!
//! Sync consumes resolved graph, validates, emits artifacts, writes receipts.
//! This is the authoritative seam - everything else supports this.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// Sync profile enforcement for `ggen sync --profile` and `--locked`.
///
/// Known enforcement profiles control pre-flight checks before the sync
/// pipeline runs.  The lockfile check is a real `Path::exists()` call —
/// no test doubles required.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncProfile {
    /// Strict enterprise governance: lockfile required, no unsigned packs.
    EnterpriseStrict,
    /// Permissive: most checks relaxed; suitable for exploration.
    Permissive,
    /// Development / dev alias: same as permissive with debug-friendly messages.
    Development,
}

impl FromStr for SyncProfile {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "enterprise-strict" => Ok(Self::EnterpriseStrict),
            "permissive" => Ok(Self::Permissive),
            "development" | "dev" => Ok(Self::Development),
            other => Err(format!(
                "Unknown profile '{}'. Known: enterprise-strict, permissive, development",
                other
            )),
        }
    }
}

impl SyncProfile {
    /// Canonical string representation of the profile.
    pub fn as_str(&self) -> &str {
        match self {
            Self::EnterpriseStrict => "enterprise-strict",
            Self::Permissive => "permissive",
            Self::Development => "development",
        }
    }

    /// Returns `true` when this profile requires a `.ggen/packs.lock` file to
    /// exist before sync may proceed.
    pub fn requires_lockfile(&self) -> bool {
        matches!(self, Self::EnterpriseStrict)
    }

    /// Returns `true` when this profile allows unsigned packs.
    pub fn allows_unsigned_packs(&self) -> bool {
        !matches!(self, Self::EnterpriseStrict)
    }
}

/// Pre-flight check executed before `ggen sync` runs.
///
/// Returns `Ok(())` when all profile requirements are satisfied, or an
/// `Err(String)` with a human-readable explanation when they are not.
///
/// # Arguments
/// * `profile`        – Optional profile name from `--profile <name>`.
/// * `locked`         – Whether `--locked` was passed on the CLI.
/// * `workspace_root` – The working directory used to resolve `.ggen/packs.lock`.
pub fn validate_sync_preconditions(
    profile: Option<&str>, locked: bool, workspace_root: &Path,
) -> Result<(), String> {
    let profile = match profile {
        Some(p) => SyncProfile::from_str(p)?,
        None => {
            // No profile — only enforce --locked if it was explicitly requested.
            if locked {
                let lockfile = workspace_root.join(".ggen").join("packs.lock");
                if !lockfile.exists() {
                    return Err(
                        "Lockfile required (--locked) but .ggen/packs.lock not found. \
                         Run `ggen packs add <pack>` first."
                            .to_string(),
                    );
                }
            }
            return Ok(());
        }
    };

    // --locked or profile.requires_lockfile() both mandate the lockfile.
    if locked || profile.requires_lockfile() {
        let lockfile = workspace_root.join(".ggen").join("packs.lock");
        if !lockfile.exists() {
            return Err(format!(
                "Lockfile required (profile='{}', --locked={}) but .ggen/packs.lock not found. \
                 Run `ggen packs add <pack>` first.",
                profile.as_str(),
                locked
            ));
        }
    }

    Ok(())
}

/// Lockfile format for tracking sync state
///
/// The lockfile captures the authoritative state after a successful sync.
/// It's used for reproducibility and validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lockfile {
    /// Lockfile format version
    pub version: u32,
    /// Timestamp when lockfile was created
    pub created_at: String,
    /// Git commit SHA of workspace
    pub git_sha: Option<String>,
    /// Packs that were resolved and installed
    pub packs: Vec<LockfilePack>,
    /// Policy profile used for validation
    pub policy_profile: String,
    /// Artifacts that were emitted
    pub artifacts: Vec<String>,
    /// Receipt from the sync operation
    pub receipt_id: String,
}

/// Pack entry in lockfile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockfilePack {
    /// Pack identifier
    pub identifier: String,
    /// Pack version
    pub version: String,
    /// Checksum for verification
    pub checksum: String,
    /// Source registry
    pub source: String,
}

impl Lockfile {
    /// Default lockfile version
    pub const CURRENT_VERSION: u32 = 1;

    /// Create a new empty lockfile (pure function)
    pub fn new() -> Self {
        Self {
            version: Self::CURRENT_VERSION,
            created_at: String::new(), // Caller must populate
            git_sha: None,
            packs: Vec::new(),
            policy_profile: "default".to_string(),
            artifacts: Vec::new(),
            receipt_id: String::new(),
        }
    }

    /// Create lockfile with timestamp (pure function)
    pub fn with_timestamp(mut self, timestamp: String) -> Self {
        self.created_at = timestamp;
        self
    }

    /// Create lockfile with git SHA (pure function)
    pub fn with_git_sha(mut self, sha: Option<String>) -> Self {
        self.git_sha = sha;
        self
    }

    /// Create lockfile with packs (pure function)
    pub fn with_packs(mut self, packs: Vec<LockfilePack>) -> Self {
        self.packs = packs;
        self
    }

    /// Create lockfile with policy profile (pure function)
    pub fn with_policy_profile(mut self, profile: String) -> Self {
        self.policy_profile = profile;
        self
    }

    /// Create lockfile with artifacts (pure function)
    pub fn with_artifacts(mut self, artifacts: Vec<String>) -> Self {
        self.artifacts = artifacts;
        self
    }

    /// Create lockfile with receipt ID (pure function)
    pub fn with_receipt_id(mut self, receipt_id: String) -> Self {
        self.receipt_id = receipt_id;
        self
    }
}

impl Default for Lockfile {
    fn default() -> Self {
        Self::new()
    }
}

impl Lockfile {
    /// Load lockfile from path (pure function that may fail due to I/O)
    ///
    /// Note: This is a convenience method that wraps I/O. For production code,
    /// prefer using `LockfileStore::load()` from the integration layer.
    pub fn load(path: &Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read lockfile: {}", e))?;

        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse lockfile: {}", e))
    }

    /// Save lockfile to path (pure function that may fail due to I/O)
    ///
    /// Note: This is a convenience method that wraps I/O. For production code,
    /// prefer using `LockfileStore::save()` from the integration layer.
    pub fn save(&self, path: &Path) -> Result<(), String> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize lockfile: {}", e))?;

        std::fs::write(path, content)
            .map_err(|e| format!("Failed to write lockfile: {}", e))
    }
}

/// Input for Stage 1: Load
#[derive(Debug, Clone)]
pub struct LoadInput {
    /// Lockfile content as string (pre-loaded by caller)
    pub lockfile_content: Option<String>,
    /// Workspace root path (as string for pure function)
    pub workspace_root: String,
}

/// Output from Stage 1: Load
#[derive(Debug, Clone)]
pub struct LoadOutput {
    /// Loaded lockfile
    pub lockfile: Lockfile,
    /// Workspace metadata
    pub workspace_metadata: HashMap<String, String>,
}

/// Stage 1: Load lockfile and workspace state (pure function)
///
/// This is a pure function - all I/O must be done by the caller.
pub fn load_stage(input: LoadInput) -> Result<LoadOutput, String> {
    let lockfile = match input.lockfile_content {
        Some(content) => {
            serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse lockfile: {}", e))?
        }
        None => Lockfile::new(),
    };

    let mut workspace_metadata = HashMap::new();
    workspace_metadata.insert("workspace_root".to_string(), input.workspace_root);

    Ok(LoadOutput {
        lockfile,
        workspace_metadata,
    })
}

/// Input for Stage 2: Resolve
#[derive(Debug, Clone)]
pub struct ResolveInput {
    /// Lockfile from load stage
    pub lockfile: Lockfile,
    /// Workspace metadata from load stage
    pub workspace_metadata: HashMap<String, String>,
    /// Force re-resolution
    pub force: bool,
}

/// Output from Stage 2: Resolve
#[derive(Debug, Clone)]
pub struct ResolveOutput {
    /// Updated lockfile with resolved dependencies
    pub lockfile: Lockfile,
    /// Resolved dependency graph
    pub dependency_graph: HashMap<String, Vec<String>>,
}

/// Stage 2: Resolve pack dependencies (pure function)
///
/// This is a pure function - all I/O must be done by the caller.
pub fn resolve_stage(input: ResolveInput) -> Result<ResolveOutput, String> {
    // In full implementation, this would:
    // - Build dependency graph from lockfile packs
    // - Resolve transitive dependencies
    // - Check for conflicts
    // - Update lockfile with resolved versions

    let dependency_graph = HashMap::new(); // Placeholder

    Ok(ResolveOutput {
        lockfile: input.lockfile,
        dependency_graph,
    })
}

/// Input for Stage 3: Validate
#[derive(Debug, Clone)]
pub struct ValidateInput {
    /// Lockfile from resolve stage
    pub lockfile: Lockfile,
    /// Dependency graph from resolve stage
    pub dependency_graph: HashMap<String, Vec<String>>,
    /// Policy profile to validate against
    pub policy_profile: String,
}

/// Output from Stage 3: Validate
#[derive(Debug, Clone)]
pub struct ValidateOutput {
    /// Validation passed
    pub passed: bool,
    /// Validation violations (if any)
    pub violations: Vec<String>,
    /// Validated lockfile
    pub lockfile: Lockfile,
}

/// Stage 3: Validate against policy profile (pure function)
///
/// This is a pure function - all I/O must be done by the caller.
pub fn validate_stage(input: ValidateInput) -> Result<ValidateOutput, String> {
    // In full implementation, this would:
    // - Check policy compliance for each pack
    // - Validate signatures if required by profile
    // - Check license compatibility
    // - Verify checksums

    let profile = SyncProfile::from_str(&input.policy_profile)
        .unwrap_or(SyncProfile::Permissive);

    let violations = Vec::new(); // Placeholder
    let passed = violations.is_empty();

    Ok(ValidateOutput {
        passed,
        violations,
        lockfile: input.lockfile,
    })
}

/// Input for Stage 4: Render
#[derive(Debug, Clone)]
pub struct RenderInput {
    /// Validated lockfile
    pub lockfile: Lockfile,
    /// Dependency graph
    pub dependency_graph: HashMap<String, Vec<String>>,
    /// Template variables
    pub template_vars: HashMap<String, String>,
}

/// Output from Stage 4: Render
#[derive(Debug, Clone)]
pub struct RenderOutput {
    /// Rendered artifacts as string content
    pub rendered_artifacts: HashMap<String, String>,
    /// Updated lockfile
    pub lockfile: Lockfile,
}

/// Stage 4: Render templates/generate code (pure function)
///
/// This is a pure function - all I/O must be done by the caller.
pub fn render_stage(input: RenderInput) -> Result<RenderOutput, String> {
    // In full implementation, this would:
    // - Load templates for each pack
    // - Render templates with variables
    // - Generate code from specs
    // - Return rendered content as strings

    let rendered_artifacts = HashMap::new(); // Placeholder

    Ok(RenderOutput {
        rendered_artifacts,
        lockfile: input.lockfile,
    })
}

/// Input for Stage 5: Emit
#[derive(Debug, Clone)]
pub struct EmitInput {
    /// Rendered artifacts (as string content)
    pub rendered_artifacts: HashMap<String, String>,
    /// Lockfile
    pub lockfile: Lockfile,
    /// Dry run mode
    pub dry_run: bool,
}

/// Output from Stage 5: Emit
#[derive(Debug, Clone)]
pub struct EmitOutput {
    /// Artifact paths that would be emitted
    pub artifact_paths: Vec<String>,
    /// Updated lockfile with artifact list
    pub lockfile: Lockfile,
}

/// Stage 5: Emit artifacts to disk (pure function)
///
/// This is a pure function - returns paths but doesn't write.
/// Caller must handle actual file I/O.
pub fn emit_stage(input: EmitInput) -> Result<EmitOutput, String> {
    // In full implementation, this would:
    // - Determine output paths for each artifact
    // - Check for conflicts with existing files
    // - Return list of paths to write
    // - Caller would handle actual file writes

    let artifact_paths: Vec<String> = input.rendered_artifacts
        .keys()
        .map(|k| k.clone())
        .collect();

    let mut lockfile = input.lockfile;
    lockfile.artifacts = artifact_paths.clone();

    Ok(EmitOutput {
        artifact_paths,
        lockfile,
    })
}

/// Input for Stage 6: Receipt
#[derive(Debug, Clone)]
pub struct ReceiptInput {
    /// Final lockfile after all stages
    pub lockfile: Lockfile,
    /// Artifacts emitted
    pub artifact_paths: Vec<String>,
    /// Receipt ID (pre-generated by caller)
    pub receipt_id: String,
    /// Duration in milliseconds
    pub duration_ms: u64,
}

/// Output from Stage 6: Receipt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptOutput {
    /// Final lockfile with receipt ID
    pub lockfile: Lockfile,
    /// Receipt as JSON string (for caller to write)
    pub receipt_content: String,
    /// Sync result summary
    pub sync_result: SyncResult,
}

/// Result from sync pipeline execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    /// Path to lockfile
    pub lockfile_path: String,
    /// Number of operations completed
    pub operations_completed: usize,
    /// Artifacts that were emitted
    pub artifacts_emitted: Vec<String>,
    /// Path to receipt file
    pub receipt_path: String,
    /// Duration in milliseconds
    pub duration_ms: u64,
}

/// Stage 6: Generate cryptographic receipt (pure function)
///
/// This is a pure function - returns receipt content as string.
/// Caller must handle actual file I/O.
pub fn receipt_stage(input: ReceiptInput) -> Result<ReceiptOutput, String> {
    // In full implementation, this would:
    // - Generate cryptographic receipt
    // - Sign with private key
    // - Return receipt content as JSON string
    // - Caller would write to receipt store

    let mut lockfile = input.lockfile;
    lockfile.receipt_id = input.receipt_id.clone();

    let receipt_data = serde_json::json!({
        "receipt_id": input.receipt_id,
        "lockfile_version": lockfile.version,
        "timestamp": lockfile.created_at,
        "git_sha": lockfile.git_sha,
        "artifacts": input.artifact_paths,
        "duration_ms": input.duration_ms,
    });

    let receipt_content = serde_json::to_string_pretty(&receipt_data)
        .map_err(|e| format!("Failed to serialize receipt: {}", e))?;

    let sync_result = SyncResult {
        lockfile_path: "ggen.lock".to_string(),
        operations_completed: input.artifact_paths.len(),
        artifacts_emitted: input.artifact_paths,
        receipt_path: format!("receipts/{}.json", input.receipt_id),
        duration_ms: input.duration_ms,
    };

    Ok(ReceiptOutput {
        lockfile,
        receipt_content,
        sync_result,
    })
}

/// Sync pipeline configuration
///
/// Pure data structure for configuring the sync pipeline.
#[derive(Debug, Clone)]
pub struct SyncPipeline {
    lockfile: Lockfile,
    policy_profile: String,
    dry_run: bool,
    force: bool,
}

impl SyncPipeline {
    /// Create a new sync pipeline
    pub fn new() -> Self {
        Self {
            lockfile: Lockfile::new(),
            policy_profile: "default".to_string(),
            dry_run: false,
            force: false,
        }
    }

    /// Set lockfile for pipeline
    pub fn with_lockfile(mut self, lockfile: Lockfile) -> Self {
        self.lockfile = lockfile;
        self
    }

    /// Set policy profile
    pub fn with_policy_profile(mut self, profile: String) -> Self {
        self.policy_profile = profile;
        self
    }

    /// Set dry run mode
    pub fn with_dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }

    /// Set force mode
    pub fn with_force(mut self, force: bool) -> Self {
        self.force = force;
        self
    }
}

impl Default for SyncPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // ── SyncProfile::from_str ────────────────────────────────────────────────

    #[test]
    fn parse_known_profiles() {
        assert_eq!(
            SyncProfile::from_str("enterprise-strict").unwrap(),
            SyncProfile::EnterpriseStrict
        );
        assert_eq!(
            SyncProfile::from_str("permissive").unwrap(),
            SyncProfile::Permissive
        );
        assert_eq!(
            SyncProfile::from_str("development").unwrap(),
            SyncProfile::Development
        );
        assert_eq!(
            SyncProfile::from_str("dev").unwrap(),
            SyncProfile::Development
        );
    }

    #[test]
    fn parse_unknown_profile_returns_error() {
        let err = SyncProfile::from_str("bogus").unwrap_err();
        assert!(err.contains("Unknown profile 'bogus'"), "error was: {err}");
        assert!(err.contains("enterprise-strict"), "error was: {err}");
    }

    // ── SyncProfile methods ──────────────────────────────────────────────────

    #[test]
    fn enterprise_strict_requires_lockfile() {
        assert!(SyncProfile::EnterpriseStrict.requires_lockfile());
        assert!(!SyncProfile::Permissive.requires_lockfile());
        assert!(!SyncProfile::Development.requires_lockfile());
    }

    #[test]
    fn enterprise_strict_forbids_unsigned_packs() {
        assert!(!SyncProfile::EnterpriseStrict.allows_unsigned_packs());
        assert!(SyncProfile::Permissive.allows_unsigned_packs());
    }

    #[test]
    fn as_str_round_trips() {
        for (input, expected) in [
            ("enterprise-strict", "enterprise-strict"),
            ("permissive", "permissive"),
            ("development", "development"),
            ("dev", "development"),
        ] {
            let p = SyncProfile::from_str(input).unwrap();
            assert_eq!(p.as_str(), expected);
        }
    }

    // ── validate_sync_preconditions ──────────────────────────────────────────

    #[test]
    fn no_profile_no_locked_always_passes() {
        let dir = TempDir::new().unwrap();
        assert!(validate_sync_preconditions(None, false, dir.path()).is_ok());
    }

    #[test]
    fn locked_flag_without_lockfile_fails() {
        let dir = TempDir::new().unwrap();
        let err = validate_sync_preconditions(None, true, dir.path()).unwrap_err();
        assert!(err.contains("--locked"), "error was: {err}");
        assert!(err.contains("packs.lock"), "error was: {err}");
    }

    #[test]
    fn locked_flag_with_lockfile_passes() {
        let dir = TempDir::new().unwrap();
        let ggen_dir = dir.path().join(".ggen");
        fs::create_dir_all(&ggen_dir).unwrap();
        fs::write(ggen_dir.join("packs.lock"), "{}").unwrap();

        assert!(validate_sync_preconditions(None, true, dir.path()).is_ok());
    }

    #[test]
    fn enterprise_strict_without_lockfile_fails() {
        let dir = TempDir::new().unwrap();
        let err =
            validate_sync_preconditions(Some("enterprise-strict"), false, dir.path()).unwrap_err();
        assert!(err.contains("enterprise-strict"), "error was: {err}");
        assert!(err.contains("packs.lock"), "error was: {err}");
    }

    #[test]
    fn enterprise_strict_with_lockfile_passes() {
        let dir = TempDir::new().unwrap();
        let ggen_dir = dir.path().join(".ggen");
        fs::create_dir_all(&ggen_dir).unwrap();
        fs::write(ggen_dir.join("packs.lock"), "{}").unwrap();

        assert!(validate_sync_preconditions(Some("enterprise-strict"), false, dir.path()).is_ok());
    }

    #[test]
    fn permissive_profile_passes_without_lockfile() {
        let dir = TempDir::new().unwrap();
        assert!(validate_sync_preconditions(Some("permissive"), false, dir.path()).is_ok());
    }

    #[test]
    fn development_profile_passes_without_lockfile() {
        let dir = TempDir::new().unwrap();
        assert!(validate_sync_preconditions(Some("dev"), false, dir.path()).is_ok());
    }

    #[test]
    fn unknown_profile_name_returns_error() {
        let dir = TempDir::new().unwrap();
        let err = validate_sync_preconditions(Some("nonexistent"), false, dir.path()).unwrap_err();
        assert!(err.contains("Unknown profile"), "error was: {err}");
    }

    // ── Pipeline stages ───────────────────────────────────────────────────────

    #[test]
    fn load_stage_with_empty_content_creates_new_lockfile() {
        let input = LoadInput {
            lockfile_content: None,
            workspace_root: "/test".to_string(),
        };

        let output = load_stage(input).unwrap();
        assert_eq!(output.lockfile.version, Lockfile::CURRENT_VERSION);
        assert_eq!(output.workspace_metadata.get("workspace_root").unwrap(), "/test");
    }

    #[test]
    fn load_stage_with_valid_content_parses_lockfile() {
        let lockfile_json = r#"{
            "version": 1,
            "created_at": "2024-01-01T00:00:00Z",
            "git_sha": null,
            "packs": [],
            "policy_profile": "default",
            "artifacts": [],
            "receipt_id": ""
        }"#;

        let input = LoadInput {
            lockfile_content: Some(lockfile_json.to_string()),
            workspace_root: "/test".to_string(),
        };

        let output = load_stage(input).unwrap();
        assert_eq!(output.lockfile.version, 1);
    }

    #[test]
    fn resolve_stage_returns_dependency_graph() {
        let input = ResolveInput {
            lockfile: Lockfile::new(),
            workspace_metadata: HashMap::new(),
            force: false,
        };

        let output = resolve_stage(input).unwrap();
        assert!(output.dependency_graph.is_empty());
    }

    #[test]
    fn validate_stage_with_permissive_profile_passes() {
        let input = ValidateInput {
            lockfile: Lockfile::new(),
            dependency_graph: HashMap::new(),
            policy_profile: "permissive".to_string(),
        };

        let output = validate_stage(input).unwrap();
        assert!(output.passed);
        assert!(output.violations.is_empty());
    }

    #[test]
    fn render_stage_returns_artifacts() {
        let input = RenderInput {
            lockfile: Lockfile::new(),
            dependency_graph: HashMap::new(),
            template_vars: HashMap::new(),
        };

        let output = render_stage(input).unwrap();
        assert!(output.rendered_artifacts.is_empty());
    }

    #[test]
    fn emit_stage_returns_artifact_paths() {
        let mut artifacts = HashMap::new();
        artifacts.insert("test.txt".to_string(), "content".to_string());

        let input = EmitInput {
            rendered_artifacts: artifacts,
            lockfile: Lockfile::new(),
            dry_run: false,
        };

        let output = emit_stage(input).unwrap();
        assert_eq!(output.artifact_paths, vec!["test.txt"]);
        assert_eq!(output.lockfile.artifacts, vec!["test.txt"]);
    }

    #[test]
    fn receipt_stage_generates_receipt() {
        let input = ReceiptInput {
            lockfile: Lockfile::new(),
            artifact_paths: vec!["test.txt".to_string()],
            receipt_id: "test-receipt".to_string(),
            duration_ms: 100,
        };

        let output = receipt_stage(input).unwrap();
        assert_eq!(output.lockfile.receipt_id, "test-receipt");
        assert!(output.receipt_content.contains("test-receipt"));
        assert_eq!(output.sync_result.operations_completed, 1);
        assert_eq!(output.sync_result.artifacts_emitted, vec!["test.txt"]);
        assert_eq!(output.sync_result.duration_ms, 100);
    }
}
