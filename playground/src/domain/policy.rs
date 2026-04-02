//! Policy domain - governance profiles, validation rules, and constraints
//!
//! This module provides policy enforcement for sync operations, including
//! profile-based validation (enterprise-strict, permissive, development) and
//! pre-flight checks for lockfile requirements.

use std::path::Path;
use std::str::FromStr;
use serde::{Deserialize, Serialize};

// ============================================================================
// Pure Data Structures (Serialize/Deserialize)
// ============================================================================

/// Policy profile enforcement level
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PolicyProfile {
    /// Strict enterprise governance: lockfile required, no unsigned packs
    EnterpriseStrict,
    /// Permissive: most checks relaxed; suitable for exploration
    Permissive,
    /// Development / dev alias: same as permissive with debug-friendly messages
    Development,
}

impl FromStr for PolicyProfile {
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

impl PolicyProfile {
    /// Canonical string representation of the profile
    pub fn as_str(&self) -> &str {
        match self {
            Self::EnterpriseStrict => "enterprise-strict",
            Self::Permissive => "permissive",
            Self::Development => "development",
        }
    }

    /// Returns `true` when this profile requires a lockfile to exist
    pub fn requires_lockfile(&self) -> bool {
        matches!(self, Self::EnterpriseStrict)
    }

    /// Returns `true` when this profile allows unsigned packs
    pub fn allows_unsigned_packs(&self) -> bool {
        !matches!(self, Self::EnterpriseStrict)
    }

    /// Get all available policy profiles
    pub fn all_available() -> Vec<PolicyProfileInfo> {
        vec![
            PolicyProfileInfo {
                name: "enterprise-strict".to_string(),
                description: "Strict enterprise governance: lockfile required, no unsigned packs".to_string(),
                strict: true,
            },
            PolicyProfileInfo {
                name: "permissive".to_string(),
                description: "Permissive: most checks relaxed; suitable for exploration".to_string(),
                strict: false,
            },
            PolicyProfileInfo {
                name: "development".to_string(),
                description: "Development: same as permissive with debug-friendly messages".to_string(),
                strict: false,
            },
        ]
    }

    /// Find a specific policy profile by name
    pub fn find(name: &str) -> Result<PolicyProfile, String> {
        Self::from_str(name)
    }

    /// Get profile info for display (static method)
    pub fn info(&self) -> PolicyProfileInfo {
        PolicyProfileInfo {
            name: self.name(),
            description: self.description(),
            strict: self.strict(),
        }
    }

    /// Get profile metadata for display
    pub fn name(&self) -> String {
        self.as_str().to_string()
    }

    /// Get profile description
    pub fn description(&self) -> String {
        match self {
            Self::EnterpriseStrict => "Strict enterprise governance: lockfile required, no unsigned packs".to_string(),
            Self::Permissive => "Permissive: most checks relaxed; suitable for exploration".to_string(),
            Self::Development => "Development: same as permissive with debug-friendly messages".to_string(),
        }
    }

    /// Get profile strictness level
    pub fn strict(&self) -> bool {
        matches!(self, Self::EnterpriseStrict)
    }

    /// Get profile rules
    pub fn rules(&self) -> Vec<PolicyRule> {
        match self {
            Self::EnterpriseStrict => vec![
                PolicyRule {
                    name: "lockfile-required".to_string(),
                    description: "Lockfile must exist before sync".to_string(),
                    enforced: true,
                    constraint: Constraint::LockfileRequired,
                },
                PolicyRule {
                    name: "signed-packs-only".to_string(),
                    description: "Only signed packs allowed".to_string(),
                    enforced: true,
                    constraint: Constraint::SignedPacksOnly,
                },
            ],
            Self::Permissive => vec![],
            Self::Development => vec![],
        }
    }
}

/// A single policy rule with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    pub name: String,
    pub description: String,
    pub enforced: bool,
    pub constraint: Constraint,
}

/// Constraint type for a policy rule
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Constraint {
    /// Lockfile must exist
    LockfileRequired,
    /// Only signed packs allowed
    SignedPacksOnly,
    /// Custom constraint with message
    Custom { message: String },
}

/// Policy profile metadata for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyProfileInfo {
    pub name: String,
    pub description: String,
    pub strict: bool,
}

/// Result of policy validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyValidationResult {
    pub profile: String,
    pub valid: bool,
    pub violations: Vec<String>,
    pub warnings: Vec<String>,
}

// ============================================================================
// Resolver/Validator Struct with Business Logic
// ============================================================================

/// Policy validator for checking sync preconditions
pub struct PolicyValidator;

impl PolicyValidator {
    /// Create a new policy validator
    pub fn new() -> Self {
        Self
    }

    /// Validate policy profile with optional lockfile check
    ///
    /// # Arguments
    /// * `profile_name` - Profile name to validate against
    /// * `lockfile` - Optional lockfile path for validation
    pub fn validate(
        &self,
        profile_name: &str,
        lockfile: Option<&str>,
    ) -> Result<PolicyValidationResult, String> {
        let profile = PolicyProfile::from_str(profile_name)?;

        // Check lockfile if provided
        let mut violations = vec![];
        let warnings = vec![];

        if profile.requires_lockfile() {
            if let Some(lockfile_path) = lockfile {
                let path = Path::new(lockfile_path);
                if !path.exists() {
                    violations.push(format!(
                        "Lockfile required (profile='{}') but '{}' not found.",
                        profile.as_str(),
                        lockfile_path
                    ));
                }
            } else {
                violations.push(format!(
                    "Lockfile required (profile='{}') but no lockfile specified.",
                    profile.as_str()
                ));
            }
        }

        Ok(PolicyValidationResult {
            profile: profile.as_str().to_string(),
            valid: violations.is_empty(),
            violations,
            warnings,
        })
    }

    /// Validate policy profile with optional lockfile check
    ///
    /// # Arguments
    /// * `profile` - Optional profile name from `--profile <name>`
    /// * `locked` - Whether `--locked` was passed on the CLI
    /// * `workspace_root` - The working directory used to resolve `.ggen/packs.lock`
    pub fn validate_sync_preconditions(
        &self,
        profile: Option<&str>,
        locked: bool,
        workspace_root: &Path,
    ) -> Result<PolicyValidationResult, String> {
        let profile = match profile {
            Some(p) => PolicyProfile::from_str(p)?,
            None => {
                // No profile — only enforce --locked if it was explicitly requested
                if locked {
                    let lockfile = workspace_root.join(".ggen").join("packs.lock");
                    if !lockfile.exists() {
                        return Ok(PolicyValidationResult {
                            profile: "none".to_string(),
                            valid: false,
                            violations: vec![
                                "Lockfile required (--locked) but .ggen/packs.lock not found. \
                                 Run `ggen packs add <pack>` first."
                                    .to_string(),
                            ],
                            warnings: vec![],
                        });
                    }
                }
                return Ok(PolicyValidationResult {
                    profile: "none".to_string(),
                    valid: true,
                    violations: vec![],
                    warnings: vec![],
                });
            }
        };

        // --locked or profile.requires_lockfile() both mandate the lockfile
        let mut violations = vec![];
        let warnings = vec![];

        if locked || profile.requires_lockfile() {
            let lockfile = workspace_root.join(".ggen").join("packs.lock");
            if !lockfile.exists() {
                violations.push(format!(
                    "Lockfile required (profile='{}', --locked={}) but .ggen/packs.lock not found. \
                     Run `ggen packs add <pack>` first.",
                    profile.as_str(),
                    locked
                ));
            }
        }

        Ok(PolicyValidationResult {
            profile: profile.as_str().to_string(),
            valid: violations.is_empty(),
            violations,
            warnings,
        })
    }
}

impl Default for PolicyValidator {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // ── PolicyProfile::from_str ────────────────────────────────────────────────

    #[test]
    fn parse_known_profiles() {
        assert_eq!(
            PolicyProfile::from_str("enterprise-strict").unwrap(),
            PolicyProfile::EnterpriseStrict
        );
        assert_eq!(
            PolicyProfile::from_str("permissive").unwrap(),
            PolicyProfile::Permissive
        );
        assert_eq!(
            PolicyProfile::from_str("development").unwrap(),
            PolicyProfile::Development
        );
        assert_eq!(
            PolicyProfile::from_str("dev").unwrap(),
            PolicyProfile::Development
        );
    }

    #[test]
    fn parse_unknown_profile_returns_error() {
        let err = PolicyProfile::from_str("bogus").unwrap_err();
        assert!(err.contains("Unknown profile 'bogus'"), "error was: {err}");
        assert!(err.contains("enterprise-strict"), "error was: {err}");
    }

    // ── PolicyProfile methods ──────────────────────────────────────────────────

    #[test]
    fn enterprise_strict_requires_lockfile() {
        assert!(PolicyProfile::EnterpriseStrict.requires_lockfile());
        assert!(!PolicyProfile::Permissive.requires_lockfile());
        assert!(!PolicyProfile::Development.requires_lockfile());
    }

    #[test]
    fn enterprise_strict_forbids_unsigned_packs() {
        assert!(!PolicyProfile::EnterpriseStrict.allows_unsigned_packs());
        assert!(PolicyProfile::Permissive.allows_unsigned_packs());
    }

    #[test]
    fn as_str_round_trips() {
        for (input, expected) in [
            ("enterprise-strict", "enterprise-strict"),
            ("permissive", "permissive"),
            ("development", "development"),
            ("dev", "development"),
        ] {
            let p = PolicyProfile::from_str(input).unwrap();
            assert_eq!(p.as_str(), expected);
        }
    }

    // ── PolicyValidator::validate_sync_preconditions ───────────────────────────

    #[test]
    fn no_profile_no_locked_always_passes() {
        let dir = TempDir::new().unwrap();
        let validator = PolicyValidator::new();
        let result = validator
            .validate_sync_preconditions(None, false, dir.path())
            .unwrap();
        assert!(result.valid);
        assert!(result.violations.is_empty());
    }

    #[test]
    fn locked_flag_without_lockfile_fails() {
        let dir = TempDir::new().unwrap();
        let validator = PolicyValidator::new();
        let result = validator
            .validate_sync_preconditions(None, true, dir.path())
            .unwrap();
        assert!(!result.valid);
        assert!(result.violations.iter().any(|v| v.contains("--locked")));
        assert!(result.violations.iter().any(|v| v.contains("packs.lock")));
    }

    #[test]
    fn locked_flag_with_lockfile_passes() {
        let dir = TempDir::new().unwrap();
        let ggen_dir = dir.path().join(".ggen");
        fs::create_dir_all(&ggen_dir).unwrap();
        fs::write(ggen_dir.join("packs.lock"), "{}").unwrap();

        let validator = PolicyValidator::new();
        let result = validator
            .validate_sync_preconditions(None, true, dir.path())
            .unwrap();
        assert!(result.valid);
        assert!(result.violations.is_empty());
    }

    #[test]
    fn enterprise_strict_without_lockfile_fails() {
        let dir = TempDir::new().unwrap();
        let validator = PolicyValidator::new();
        let result = validator
            .validate_sync_preconditions(Some("enterprise-strict"), false, dir.path())
            .unwrap();
        assert!(!result.valid);
        assert!(result.violations.iter().any(|v| v.contains("enterprise-strict")));
        assert!(result.violations.iter().any(|v| v.contains("packs.lock")));
    }

    #[test]
    fn enterprise_strict_with_lockfile_passes() {
        let dir = TempDir::new().unwrap();
        let ggen_dir = dir.path().join(".ggen");
        fs::create_dir_all(&ggen_dir).unwrap();
        fs::write(ggen_dir.join("packs.lock"), "{}").unwrap();

        let validator = PolicyValidator::new();
        let result = validator
            .validate_sync_preconditions(Some("enterprise-strict"), false, dir.path())
            .unwrap();
        assert!(result.valid);
        assert!(result.violations.is_empty());
    }

    #[test]
    fn permissive_profile_passes_without_lockfile() {
        let dir = TempDir::new().unwrap();
        let validator = PolicyValidator::new();
        let result = validator
            .validate_sync_preconditions(Some("permissive"), false, dir.path())
            .unwrap();
        assert!(result.valid);
        assert!(result.violations.is_empty());
    }

    #[test]
    fn development_profile_passes_without_lockfile() {
        let dir = TempDir::new().unwrap();
        let validator = PolicyValidator::new();
        let result = validator
            .validate_sync_preconditions(Some("dev"), false, dir.path())
            .unwrap();
        assert!(result.valid);
        assert!(result.violations.is_empty());
    }

    #[test]
    fn unknown_profile_name_returns_error() {
        let dir = TempDir::new().unwrap();
        let validator = PolicyValidator::new();
        let result = validator.validate_sync_preconditions(Some("nonexistent"), false, dir.path());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Unknown profile"), "error was: {err}");
    }

    #[test]
    fn all_available_returns_expected_profiles() {
        let profiles = PolicyProfile::all_available();
        assert_eq!(profiles.len(), 3);
        assert!(profiles.iter().any(|p| p.name == "enterprise-strict"));
        assert!(profiles.iter().any(|p| p.name == "permissive"));
        assert!(profiles.iter().any(|p| p.name == "development"));
    }
}
