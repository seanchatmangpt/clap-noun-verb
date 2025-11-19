//! Marketplace command implementations with enhanced error handling
//!
//! This module implements ggen's marketplace commands with user-friendly
//! error messages and comprehensive validation.

use clap_noun_verb::Result as CnvResult;
use clap_noun_verb_macros::verb;
use serde::Serialize;

use super::errors::{UserError, ErrorCategory};
use super::validators::validate_package_id;

// ============================================================================
// Data Types
// ============================================================================

#[derive(Serialize, Debug, Clone)]
pub struct PackageInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub downloads: u64,
    pub rating: f32,
}

#[derive(Serialize, Debug)]
pub struct SearchOutput {
    pub query: String,
    pub category: Option<String>,
    pub results: Vec<PackageInfo>,
    pub total_count: usize,
}

#[derive(Serialize, Debug)]
pub struct InstallOutput {
    pub package: PackageInfo,
    pub install_path: String,
    pub success: bool,
    pub message: String,
}

#[derive(Serialize, Debug)]
pub struct ListOutput {
    pub packages: Vec<PackageInfo>,
    pub total_count: usize,
    pub source: String,
}

#[derive(Serialize, Debug)]
pub struct PublishOutput {
    pub package_id: String,
    pub version: String,
    pub success: bool,
    pub url: String,
    pub message: String,
}

// ============================================================================
// Mock Data (In production, this would query a real marketplace)
// ============================================================================

fn get_mock_packages() -> Vec<PackageInfo> {
    vec![
        PackageInfo {
            id: "io.ggen.rust.axum".to_string(),
            name: "Axum Web Framework".to_string(),
            description: "Template for building web services with Axum".to_string(),
            version: "1.2.0".to_string(),
            author: "ggen-team".to_string(),
            downloads: 15420,
            rating: 4.8,
        },
        PackageInfo {
            id: "io.ggen.rust.cli".to_string(),
            name: "CLI Application".to_string(),
            description: "Template for command-line applications".to_string(),
            version: "1.0.5".to_string(),
            author: "ggen-team".to_string(),
            downloads: 8932,
            rating: 4.6,
        },
        PackageInfo {
            id: "io.ggen.rust.actix".to_string(),
            name: "Actix Web".to_string(),
            description: "High-performance web framework template".to_string(),
            version: "2.1.0".to_string(),
            author: "community".to_string(),
            downloads: 12100,
            rating: 4.7,
        },
        PackageInfo {
            id: "io.ggen.python.fastapi".to_string(),
            name: "FastAPI Service".to_string(),
            description: "Modern Python web API template".to_string(),
            version: "0.9.2".to_string(),
            author: "community".to_string(),
            downloads: 21000,
            rating: 4.9,
        },
    ]
}

// ============================================================================
// Business Logic (Pure Functions)
// ============================================================================

/// Search marketplace for packages
fn search_marketplace(
    query: &str,
    category: Option<&str>,
    limit: usize,
) -> Result<SearchOutput, UserError> {
    let all_packages = get_mock_packages();

    // Filter by query
    let query_lower = query.to_lowercase();
    let mut filtered: Vec<PackageInfo> = all_packages
        .into_iter()
        .filter(|pkg| {
            pkg.name.to_lowercase().contains(&query_lower)
                || pkg.description.to_lowercase().contains(&query_lower)
                || pkg.id.to_lowercase().contains(&query_lower)
        })
        .collect();

    // Filter by category if provided
    if let Some(cat) = category {
        let cat_lower = cat.to_lowercase();
        filtered.retain(|pkg| pkg.id.to_lowercase().contains(&cat_lower));
    }

    // Return error if no results
    if filtered.is_empty() {
        return Err(super::errors::no_search_results(query));
    }

    // Sort by downloads (most popular first)
    filtered.sort_by(|a, b| b.downloads.cmp(&a.downloads));

    // Limit results
    let total = filtered.len();
    filtered.truncate(limit);

    Ok(SearchOutput {
        query: query.to_string(),
        category: category.map(String::from),
        results: filtered,
        total_count: total,
    })
}

/// Install a package from marketplace
fn install_package(
    package_id: &str,
    version: Option<&str>,
    path: Option<&str>,
) -> Result<InstallOutput, UserError> {
    // Find package
    let packages = get_mock_packages();
    let package = packages
        .iter()
        .find(|p| p.id == package_id)
        .ok_or_else(|| super::errors::package_not_found(package_id))?;

    // Validate version if specified
    if let Some(ver) = version {
        if ver != package.version {
            return Err(UserError::new(
                ErrorCategory::NotFound,
                format!("Version '{}' not found for package '{}'", ver, package_id),
                format!(
                    "Available version: {}\n\n  \
                    Install latest version:\n  \
                    ggen marketplace install {}\n\n  \
                    Or specify correct version:\n  \
                    ggen marketplace install {} --version {}",
                    package.version, package_id, package_id, package.version
                ),
            ).with_docs("https://marketplace.ggen.io"));
        }
    }

    // Determine install path
    let install_path = path
        .map(String::from)
        .unwrap_or_else(|| format!(".ggen/packages/{}", package.name));

    // Check if already installed
    if std::path::Path::new(&install_path).exists() {
        return Err(UserError::new(
            ErrorCategory::Validation,
            format!("Package already installed at '{}'", install_path),
            format!(
                "Options:\n  \
                1. Use different path: --path ./custom/path\n  \
                2. Update existing: ggen marketplace update {}\n  \
                3. Remove and reinstall: ggen marketplace uninstall {} && ggen marketplace install {}",
                package_id, package_id, package_id
            ),
        ));
    }

    Ok(InstallOutput {
        package: package.clone(),
        install_path,
        success: true,
        message: format!("Successfully installed {} v{}", package.name, package.version),
    })
}

/// List installed or available packages
fn list_packages(
    source: &str,
    installed_only: bool,
) -> Result<ListOutput, UserError> {
    if installed_only {
        // In production, would read from .ggen/packages
        let packages = vec![];

        if packages.is_empty() {
            return Err(UserError::new(
                ErrorCategory::NotFound,
                "No packages installed",
                "Install packages from the marketplace:\n  \
                1. Search for packages: ggen marketplace search <query>\n  \
                2. Install a package: ggen marketplace install <package-id>\n\n  \
                Popular packages:\n  \
                - io.ggen.rust.axum (Web framework)\n  \
                - io.ggen.rust.cli (CLI template)\n  \
                - io.ggen.python.fastapi (Python API)".to_string(),
            ).with_docs("https://marketplace.ggen.io"));
        }

        Ok(ListOutput {
            packages,
            total_count: 0,
            source: "local".to_string(),
        })
    } else {
        // List all available packages
        let packages = get_mock_packages();
        let count = packages.len();

        Ok(ListOutput {
            packages,
            total_count: count,
            source: source.to_string(),
        })
    }
}

/// Publish a package to marketplace
fn publish_package(
    package_path: &str,
    version: Option<&str>,
    force: bool,
) -> Result<PublishOutput, UserError> {
    // Check package.toml exists
    let manifest_path = std::path::Path::new(package_path).join("pack.toml");
    if !manifest_path.exists() {
        return Err(UserError::new(
            ErrorCategory::NotFound,
            format!("No pack.toml found in '{}'", package_path),
            format!(
                "Create a valid package:\n  \
                1. Initialize package: ggen pack init {}\n  \
                2. Edit pack.toml with package metadata\n  \
                3. Publish: ggen marketplace publish {}\n\n  \
                Required pack.toml fields:\n  \
                - id (e.g., com.example.mypack)\n  \
                - name\n  \
                - version\n  \
                - description\n  \
                - author",
                package_path, package_path
            ),
        ).with_docs("https://docs.ggen.io/publishing"));
    }

    // Validate package ID from manifest
    // In production, would parse pack.toml
    let package_id = "io.ggen.rust.example";
    validate_package_id(package_id)
        .map_err(|e| clap_noun_verb::NounVerbError::ValidationFailed(e.to_string()))
        .map_err(|e| UserError::new(
            ErrorCategory::Validation,
            "Invalid package ID in pack.toml",
            e.to_string(),
        ))?;

    // Check if package already exists
    let existing = get_mock_packages()
        .iter()
        .any(|p| p.id == package_id);

    if existing && !force {
        return Err(UserError::new(
            ErrorCategory::Validation,
            format!("Package '{}' already exists", package_id),
            format!(
                "Options:\n  \
                1. Publish new version: ggen marketplace publish {} --version 1.1.0\n  \
                2. Force overwrite (not recommended): ggen marketplace publish {} --force\n  \
                3. Use different package ID in pack.toml",
                package_path, package_path
            ),
        ).with_docs("https://docs.ggen.io/publishing"));
    }

    let publish_version = version.unwrap_or("1.0.0");

    Ok(PublishOutput {
        package_id: package_id.to_string(),
        version: publish_version.to_string(),
        success: true,
        url: format!("https://marketplace.ggen.io/packages/{}", package_id),
        message: format!("Successfully published {} v{}", package_id, publish_version),
    })
}

// ============================================================================
// CLI Layer (Input Validation + Delegation)
// ============================================================================

/// Search the marketplace for packages
///
/// # Arguments
/// * `query` - Search query (keywords, package name, etc.)
/// * `category` - Filter by category (optional)
/// * `limit` - Maximum results to return (default: 10)
///
/// # Examples
/// ```bash
/// # Search for web framework templates
/// ggen marketplace search "web framework"
///
/// # Search within Rust category
/// ggen marketplace search api --category rust
///
/// # Get more results
/// ggen marketplace search rust --limit 20
/// ```
#[verb("search", "marketplace")]
pub fn marketplace_search(
    query: String,
    #[arg(short, long)] category: Option<String>,
    #[arg(short, long, default_value = "10")] limit: usize,
) -> CnvResult<SearchOutput> {
    // Validate query
    if query.trim().is_empty() {
        return Err(clap_noun_verb::NounVerbError::ValidationFailed(
            "Search query cannot be empty. Try: ggen marketplace search rust".to_string()
        ));
    }

    if limit == 0 || limit > 100 {
        return Err(clap_noun_verb::NounVerbError::ValidationFailed(
            "Limit must be between 1 and 100".to_string()
        ));
    }

    // Delegate to business logic
    search_marketplace(&query, category.as_deref(), limit)
        .map_err(|e| clap_noun_verb::NounVerbError::ExecutionError { message: e.to_string() })
}

/// Install a package from the marketplace
///
/// # Arguments
/// * `package` - Package identifier (e.g., io.ggen.rust.axum)
/// * `version` - Specific version to install (optional)
/// * `path` - Custom installation path (optional)
///
/// # Examples
/// ```bash
/// # Install latest version
/// ggen marketplace install io.ggen.rust.axum
///
/// # Install specific version
/// ggen marketplace install io.ggen.rust.axum --version 1.2.0
///
/// # Install to custom path
/// ggen marketplace install io.ggen.rust.cli --path ./my-templates
/// ```
#[verb("install", "marketplace")]
pub fn marketplace_install(
    package: String,
    #[arg(short, long)] version: Option<String>,
    #[arg(short, long)] path: Option<String>,
) -> CnvResult<InstallOutput> {
    // Validate package ID
    validate_package_id(&package)
        .map_err(|e| clap_noun_verb::NounVerbError::ValidationFailed(e.to_string()))?;

    // Delegate to business logic
    install_package(&package, version.as_deref(), path.as_deref())
        .map_err(|e| clap_noun_verb::NounVerbError::ExecutionError { message: e.to_string() })
}

/// List available or installed packages
///
/// # Arguments
/// * `source` - Package source (marketplace or local)
/// * `installed` - Show only installed packages
///
/// # Examples
/// ```bash
/// # List all marketplace packages
/// ggen marketplace list
///
/// # List installed packages
/// ggen marketplace list --installed
/// ```
#[verb("list", "marketplace")]
pub fn marketplace_list(
    #[arg(short, long, default_value = "marketplace")] source: String,
    #[arg(short, long, default_value = "false")] installed: bool,
) -> CnvResult<ListOutput> {
    // Delegate to business logic
    list_packages(&source, installed)
        .map_err(|e| clap_noun_verb::NounVerbError::ExecutionError { message: e.to_string() })
}

/// Publish a package to the marketplace
///
/// # Arguments
/// * `path` - Path to package directory (must contain pack.toml)
/// * `version` - Version to publish (optional, uses pack.toml if not specified)
/// * `force` - Force overwrite if package exists
///
/// # Examples
/// ```bash
/// # Publish package
/// ggen marketplace publish ./my-package
///
/// # Publish specific version
/// ggen marketplace publish ./my-package --version 2.0.0
///
/// # Force overwrite
/// ggen marketplace publish ./my-package --force
/// ```
#[verb("publish", "marketplace")]
pub fn marketplace_publish(
    path: String,
    #[arg(short, long)] version: Option<String>,
    #[arg(short, long, default_value = "false")] force: bool,
) -> CnvResult<PublishOutput> {
    // Validate path exists
    if !std::path::Path::new(&path).exists() {
        return Err(clap_noun_verb::NounVerbError::ValidationFailed(
            format!("Path '{}' does not exist", path)
        ));
    }

    // Delegate to business logic
    publish_package(&path, version.as_deref(), force)
        .map_err(|e| clap_noun_verb::NounVerbError::ExecutionError { message: e.to_string() })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_marketplace_success() {
        let result = search_marketplace("rust", None, 10);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(!output.results.is_empty());
    }

    #[test]
    fn test_search_marketplace_no_results() {
        let result = search_marketplace("nonexistent-xyz-123", None, 10);
        assert!(result.is_err());
    }

    #[test]
    fn test_search_marketplace_with_category() {
        let result = search_marketplace("api", Some("rust"), 10);
        assert!(result.is_ok());
    }

    #[test]
    fn test_install_package_success() {
        let result = install_package("io.ggen.rust.axum", None, Some("/tmp/test"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_install_package_not_found() {
        let result = install_package("io.ggen.nonexistent", None, None);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(err.problem.contains("not found"));
    }

    #[test]
    fn test_list_packages_marketplace() {
        let result = list_packages("marketplace", false);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(!output.packages.is_empty());
    }

    #[test]
    fn test_list_packages_installed_empty() {
        let result = list_packages("local", true);
        assert!(result.is_err()); // No packages installed in test
    }

    #[test]
    fn test_publish_package_no_manifest() {
        let result = publish_package("/nonexistent", None, false);
        assert!(result.is_err());
    }
}
