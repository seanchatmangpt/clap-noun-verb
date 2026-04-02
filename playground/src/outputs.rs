//! Output types for CLI commands
//!
//! All CLI commands return these types for JSON serialization.
//! Following domain separation: CLI validates, domain computes, integration connects.

use serde::Serialize;

/// Output from paper generation
#[derive(Serialize)]
pub struct PaperGeneratedOutput {
    pub family: String,
    pub output_path: String,
    pub template_engine: String,
    pub sections: usize,
}

/// Output from paper family listing
#[derive(Serialize)]
pub struct PaperFamilyOutput {
    pub name: String,
    pub description: String,
}

/// Output from paper validation
#[derive(Serialize)]
pub struct ValidationResultOutput {
    pub is_valid: bool,
    pub structure_valid: bool,
    pub citations_valid: bool,
    pub formatting_valid: bool,
    pub errors: Vec<String>,
}

/// Output from config get operation
#[derive(Serialize)]
pub struct ConfigValueOutput {
    pub key: String,
    pub value: Option<String>,
    pub valid_key: bool,
}

/// Output from config set operation
#[derive(Serialize)]
pub struct ConfigSetOutput {
    pub key: String,
    pub value: String,
    pub valid_key: bool,
    pub saved: bool,
}

/// Output from config show operation
#[derive(Serialize)]
pub struct ConfigAllOutput {
    pub entries: std::collections::HashMap<String, String>,
}

/// Output from health check
#[derive(Serialize)]
pub struct HealthOutput {
    pub status: String,
    pub version: String,
    pub timestamp: u64,
}

/// Output from ontology export
#[derive(Serialize)]
pub struct OntologyOutput {
    pub format: String,
    pub triples: usize,
    pub content: String,
}

/// Output from SPARQL query
#[derive(Serialize)]
pub struct SparqlResultOutput {
    pub query: String,
    pub rows: usize,
    pub results: Vec<Vec<String>>,
}

/// Output from shell completion generation
#[derive(Serialize)]
pub struct CompletionScriptOutput {
    pub shell: String,
    pub cli: String,
    pub script: String,
}

/// Output from middleware display
#[derive(Serialize)]
pub struct MiddlewareOutput {
    pub config: crate::domain::MiddlewareConfig,
    pub stats: crate::domain::MiddlewareStats,
}

/// Output from telemetry display
#[derive(Serialize)]
pub struct TelemetryOutput {
    pub span: crate::domain::ExecutionSpan,
    pub receipt: crate::domain::ExecutionReceipt,
    pub trace_id: String,
}

/// Output from format listing
#[derive(Serialize)]
pub struct FormatInfoOutput {
    pub name: String,
    pub description: String,
}

// ============================================================================
// ggen v26.4.2 Output Types
// ============================================================================

/// Output from sync command
#[derive(Serialize)]
pub struct SyncOutput {
    pub lockfile: String,
    pub operations: usize,
    pub artifacts: Vec<String>,
    pub receipt: String,
    pub duration_ms: u64,
}

/// Output from receipt verification
#[derive(Serialize)]
pub struct ReceiptVerifyOutput {
    pub receipt_id: String,
    pub is_valid: bool,
    pub signature_valid: bool,
    pub chain_valid: bool,
    pub warnings: Vec<String>,
}

/// Output from receipt info
#[derive(Serialize)]
pub struct ReceiptInfoOutput {
    pub id: String,
    pub timestamp: String,
    pub operations: usize,
    pub artifacts: Vec<String>,
    pub agent_type: String,
    pub agent_id: String,
    pub agent_version: String,
}

/// Output from receipt chain verification
#[derive(Serialize)]
pub struct ReceiptChainVerifyOutput {
    pub chain_length: usize,
    pub all_valid: bool,
    pub broken_links: Vec<String>,
}

/// Output from doctor run command
#[derive(Serialize)]
pub struct DoctorRunOutput {
    pub checks_run: usize,
    pub passed: usize,
    pub failed: usize,
    pub results: Vec<DoctorCheckResult>,
}

/// Individual check result for doctor output
#[derive(Serialize)]
pub struct DoctorCheckResult {
    pub name: String,
    pub passed: bool,
    pub output: String,
    pub suggestions: Vec<String>,
}

// Implement From conversion for convenience
impl From<crate::domain::doctor::DiagnosticCheck> for DoctorCheckResult {
    fn from(check: crate::domain::doctor::DiagnosticCheck) -> Self {
        Self {
            name: check.name,
            passed: check.passed,
            output: check.output,
            suggestions: check.suggestions,
        }
    }
}

/// Output from doctor check command
#[derive(Serialize)]
pub struct DoctorCheckOutput {
    pub name: String,
    pub passed: bool,
    pub output: String,
    pub suggestions: Vec<String>,
}

/// Output from doctor env command
#[derive(Serialize)]
pub struct DoctorEnvOutput {
    pub workspace_root: String,
    pub ggen_version: String,
    pub lockfile_valid: bool,
    pub pack_integrity: bool,
    pub policy_conflicts: Vec<String>,
}

// ============================================================================
// Policy Domain Output Types
// ============================================================================

/// Output from policy list command
#[derive(Serialize)]
pub struct PolicyListOutput {
    pub name: String,
    pub description: String,
    pub strict: bool,
}

/// Output from policy show command
#[derive(Serialize)]
pub struct PolicyShowOutput {
    pub name: String,
    pub description: String,
    pub strict: bool,
    pub rules: usize,
}

/// Output from policy validate command
#[derive(Serialize)]
pub struct PolicyValidateOutput {
    pub profile: String,
    pub valid: bool,
    pub violations: Vec<String>,
    pub warnings: Vec<String>,
}

// ============================================================================
// Registry Domain Output Types
// ============================================================================

/// Output from registry search command
#[derive(Serialize)]
pub struct RegistrySearchOutput {
    pub query: String,
    pub category: Option<String>,
    pub results: Vec<RegistrySearchResultItem>,
    pub count: usize,
}

/// Individual search result item
#[derive(Serialize)]
pub struct RegistrySearchResultItem {
    pub name: String,
    pub version: String,
    pub description: String,
    pub category: Option<String>,
}

/// Output from registry info command
#[derive(Serialize)]
pub struct RegistryInfoOutput {
    pub name: String,
    pub description: String,
    pub versions: Vec<String>,
    pub latest_version: String,
    pub dependencies: Vec<String>,
    pub homepage: Option<String>,
    pub repository: Option<String>,
}

/// Output from registry list sources command
#[derive(Serialize)]
pub struct RegistrySourcesOutput {
    pub sources: Vec<RegistrySourceItem>,
    pub count: usize,
}

/// Individual registry source item
#[derive(Serialize)]
pub struct RegistrySourceItem {
    pub name: String,
    pub url: String,
    pub priority: u32,
}

// ============================================================================
// Capability Domain Output Types
// ============================================================================

/// Output from capability enable command
#[derive(Serialize)]
pub struct CapabilityEnabledOutput {
    pub capability: String,
    pub packs_required: Vec<String>,
    pub install_actions: Vec<String>,
    pub validation_required: bool,
}

/// Output from capability list command
#[derive(Serialize)]
pub struct CapabilityListOutput {
    pub capabilities: Vec<CapabilityInfoItem>,
}

/// Individual capability info for list output
#[derive(Serialize)]
pub struct CapabilityInfoItem {
    pub name: String,
    pub description: String,
}

/// Output from capability show command
#[derive(Serialize)]
pub struct CapabilityShowOutput {
    pub name: String,
    pub description: String,
}

// ============================================================================
// Pack Domain Output Types
// ============================================================================

/// Output from pack add command
#[derive(Serialize)]
pub struct PackAddedOutput {
    pub name: String,
    pub version: String,
    pub dependencies: Vec<String>,
    pub installed_at: String,
}

/// Output from pack remove command
#[derive(Serialize)]
pub struct PackRemovedOutput {
    pub identifier: String,
    pub removed_at: String,
}

/// Output from pack list command
#[derive(Serialize)]
pub struct PackListOutput {
    pub packs: Vec<PackInfo>,
}

/// Individual pack info for list output
#[derive(Serialize)]
pub struct PackInfo {
    pub name: String,
    pub version: String,
}

/// Output from pack show command
#[derive(Serialize)]
pub struct PackShowOutput {
    pub name: String,
    pub version: String,
    pub description: String,
    pub dependencies: Vec<String>,
    pub capabilities: Vec<String>,
}

/// Output from pack verify command
#[derive(Serialize)]
pub struct PackVerifyOutput {
    pub identifier: String,
    pub is_valid: bool,
    pub checksum: String,
    pub signature_valid: bool,
    pub errors: Vec<String>,
}

/// Output from pack graph command
#[derive(Serialize)]
pub struct PackGraphOutput {
    pub graph: String,
}

/// Output from pack update command
#[derive(Serialize)]
pub struct PackUpdateOutput {
    pub dry_run: bool,
    pub updates_available: usize,
    pub updated: Vec<String>,
    pub failed: Vec<String>,
}

impl PackUpdateOutput {
    pub fn dry_run(updates: Vec<crate::domain::pack::UpdateInfo>) -> Self {
        Self {
            dry_run: true,
            updates_available: updates.len(),
            updated: vec![],
            failed: vec![],
        }
    }

    pub fn applied(result: crate::domain::pack::UpdateResult) -> Self {
        Self {
            dry_run: false,
            updates_available: result.updated.len() + result.failed.len(),
            updated: result.updated,
            failed: result.failed,
        }
    }
}
