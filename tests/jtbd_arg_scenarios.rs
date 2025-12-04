//! Jobs-To-Be-Done (JTBD) Scenario Tests for Argument Handling
//!
//! These tests validate complex, real-world argument handling scenarios that users
//! would encounter when building CLIs with clap-noun-verb.
//!
//! Each test represents a "job" that a user is trying to accomplish:
//! - JTBD-001: Process files passed as a comma-separated string
//! - JTBD-002: Use verbosity flags with counting (-v, -vv, -vvv)
//! - JTBD-003: Use numeric arguments with type inference
//! - JTBD-004: Use boolean flags with explicit values
//! - JTBD-005: Use environment variables as argument sources
//! - JTBD-006: Use mutually exclusive argument groups
//! - JTBD-007: Use required arguments with validation
//! - JTBD-008: Use path arguments for file operations
//! - JTBD-009: Use default values with fallback behavior
//! - JTBD-010: Mix positional and named arguments

use clap_noun_verb::error::Result;
use clap_noun_verb_macros::{noun, verb};
use serde::Serialize;

// =============================================================================
// JTBD-001: Multi-Value Arguments (as comma-separated string)
// Note: Vec<String> is not directly supported in #[verb] - use comma-sep string
// =============================================================================

#[derive(Serialize, Debug, PartialEq)]
struct MultiFileOutput {
    files: Vec<String>,
    count: usize,
}

/// Process multiple input files
///
/// # Arguments
/// * `files` - Input files to process (comma-separated list)
#[noun("batch", "Batch processing commands")]
#[verb("process")]
fn batch_process(files: String) -> Result<MultiFileOutput> {
    // Parse comma-separated string into Vec
    let file_list: Vec<String> = files.split(',').map(|s| s.trim().to_string()).collect();
    Ok(MultiFileOutput { count: file_list.len(), files: file_list })
}

#[test]
fn test_jtbd_001_multi_value_args_registered() {
    // Arrange: batch_process has a String argument for comma-sep files
    let registry = clap_noun_verb::cli::registry::CommandRegistry::get();
    let registry = registry.lock().unwrap_or_else(|e| e.into_inner());
    let cmd = registry.build_command();

    // Act: Find batch -> process command
    let batch_cmd = cmd.get_subcommands().find(|s| s.get_name() == "batch");
    assert!(batch_cmd.is_some(), "batch noun should be registered");

    let process_cmd = batch_cmd.unwrap().get_subcommands().find(|s| s.get_name() == "process");
    assert!(process_cmd.is_some(), "process verb should be registered");

    // Assert: files argument should exist
    let process_cmd = process_cmd.unwrap();
    let args: Vec<_> = process_cmd.get_arguments().collect();
    let files_arg = args.iter().find(|a| a.get_id().as_str() == "files");
    assert!(files_arg.is_some(), "files argument should exist");
}

// =============================================================================
// JTBD-002: Verbosity Flags with Counting (-v, -vv, -vvv)
// =============================================================================

#[derive(Serialize, Debug, PartialEq)]
struct VerbosityOutput {
    level: usize,
    description: String,
}

/// Run with configurable verbosity
///
/// # Arguments
/// * `verbose` - Verbosity level (use -v, -vv, or -vvv)
#[noun("runner", "Task runner commands")]
#[verb("execute")]
fn runner_execute(verbose: usize) -> Result<VerbosityOutput> {
    let description = match verbose {
        0 => "quiet",
        1 => "normal",
        2 => "verbose",
        _ => "debug",
    }
    .to_string();

    Ok(VerbosityOutput { level: verbose, description })
}

#[test]
fn test_jtbd_002_verbosity_counting_registered() {
    let registry = clap_noun_verb::cli::registry::CommandRegistry::get();
    let registry = registry.lock().unwrap_or_else(|e| e.into_inner());
    let cmd = registry.build_command();

    let runner_cmd = cmd.get_subcommands().find(|s| s.get_name() == "runner");
    assert!(runner_cmd.is_some(), "runner noun should be registered");

    let execute_cmd = runner_cmd.unwrap().get_subcommands().find(|s| s.get_name() == "execute");
    assert!(execute_cmd.is_some(), "execute verb should be registered");

    let execute_cmd = execute_cmd.unwrap();
    let args: Vec<_> = execute_cmd.get_arguments().collect();
    let verbose_arg = args.iter().find(|a| a.get_id().as_str() == "verbose");
    assert!(verbose_arg.is_some(), "verbose argument should exist");
}

// =============================================================================
// JTBD-003: Numeric Arguments with Type Inference
// =============================================================================

#[derive(Serialize, Debug, PartialEq)]
struct ServerConfig {
    port: u16,
    timeout: u64,
    workers: usize,
}

/// Configure server settings
///
/// # Arguments
/// * `port` - Server port number [default: 8080]
/// * `timeout` - Request timeout in seconds [default: 30]
/// * `workers` - Number of worker threads [default: 4]
#[noun("server", "Server management commands")]
#[verb("configure")]
fn server_configure(
    port: Option<u16>,
    timeout: Option<u64>,
    workers: Option<usize>,
) -> Result<ServerConfig> {
    Ok(ServerConfig {
        port: port.unwrap_or(8080),
        timeout: timeout.unwrap_or(30),
        workers: workers.unwrap_or(4),
    })
}

#[test]
fn test_jtbd_003_numeric_args_registered() {
    let registry = clap_noun_verb::cli::registry::CommandRegistry::get();
    let registry = registry.lock().unwrap_or_else(|e| e.into_inner());
    let cmd = registry.build_command();

    let server_cmd = cmd.get_subcommands().find(|s| s.get_name() == "server");
    assert!(server_cmd.is_some(), "server noun should be registered");

    let configure_cmd = server_cmd.unwrap().get_subcommands().find(|s| s.get_name() == "configure");
    assert!(configure_cmd.is_some(), "configure verb should be registered");

    let configure_cmd = configure_cmd.unwrap();
    let args: Vec<_> = configure_cmd.get_arguments().collect();

    // Verify all numeric arguments are registered
    let port_arg = args.iter().find(|a| a.get_id().as_str() == "port");
    let timeout_arg = args.iter().find(|a| a.get_id().as_str() == "timeout");
    let workers_arg = args.iter().find(|a| a.get_id().as_str() == "workers");

    assert!(port_arg.is_some(), "port argument should exist");
    assert!(timeout_arg.is_some(), "timeout argument should exist");
    assert!(workers_arg.is_some(), "workers argument should exist");
}

// =============================================================================
// JTBD-004: Boolean Flags with Explicit Values
// =============================================================================

#[derive(Serialize, Debug, PartialEq)]
struct FeatureFlags {
    debug: bool,
    optimize: bool,
    cache: bool,
}

/// Set feature flags
///
/// # Arguments
/// * `debug` - Enable debug mode
/// * `optimize` - Enable optimizations
/// * `cache` - Enable caching
#[noun("feature", "Feature flag commands")]
#[verb("toggle")]
fn feature_toggle(debug: bool, optimize: bool, cache: bool) -> Result<FeatureFlags> {
    Ok(FeatureFlags { debug, optimize, cache })
}

#[test]
fn test_jtbd_004_boolean_flags_registered() {
    let registry = clap_noun_verb::cli::registry::CommandRegistry::get();
    let registry = registry.lock().unwrap_or_else(|e| e.into_inner());
    let cmd = registry.build_command();

    let feature_cmd = cmd.get_subcommands().find(|s| s.get_name() == "feature");
    assert!(feature_cmd.is_some(), "feature noun should be registered");

    let toggle_cmd = feature_cmd.unwrap().get_subcommands().find(|s| s.get_name() == "toggle");
    assert!(toggle_cmd.is_some(), "toggle verb should be registered");

    let toggle_cmd = toggle_cmd.unwrap();
    let args: Vec<_> = toggle_cmd.get_arguments().collect();

    let debug_arg = args.iter().find(|a| a.get_id().as_str() == "debug");
    let optimize_arg = args.iter().find(|a| a.get_id().as_str() == "optimize");
    let cache_arg = args.iter().find(|a| a.get_id().as_str() == "cache");

    assert!(debug_arg.is_some(), "debug argument should exist");
    assert!(optimize_arg.is_some(), "optimize argument should exist");
    assert!(cache_arg.is_some(), "cache argument should exist");
}

// =============================================================================
// JTBD-005: Environment Variable Sources
// =============================================================================

#[derive(Serialize, Debug, PartialEq)]
struct EnvConfig {
    api_key: Option<String>,
    base_url: String,
}

/// Configure from environment
///
/// # Arguments
/// * `api_key` - API key [env: API_KEY]
/// * `base_url` - Base URL [env: BASE_URL] [default: https://api.example.com]
#[noun("env", "Environment configuration")]
#[verb("setup")]
fn env_setup(api_key: Option<String>, base_url: Option<String>) -> Result<EnvConfig> {
    Ok(EnvConfig {
        api_key,
        base_url: base_url.unwrap_or_else(|| "https://api.example.com".to_string()),
    })
}

#[test]
fn test_jtbd_005_env_var_args_registered() {
    let registry = clap_noun_verb::cli::registry::CommandRegistry::get();
    let registry = registry.lock().unwrap_or_else(|e| e.into_inner());
    let cmd = registry.build_command();

    let env_cmd = cmd.get_subcommands().find(|s| s.get_name() == "env");
    assert!(env_cmd.is_some(), "env noun should be registered");

    let setup_cmd = env_cmd.unwrap().get_subcommands().find(|s| s.get_name() == "setup");
    assert!(setup_cmd.is_some(), "setup verb should be registered");

    let setup_cmd = setup_cmd.unwrap();
    let args: Vec<_> = setup_cmd.get_arguments().collect();

    let api_key_arg = args.iter().find(|a| a.get_id().as_str() == "api_key");
    let base_url_arg = args.iter().find(|a| a.get_id().as_str() == "base_url");

    assert!(api_key_arg.is_some(), "api_key argument should exist");
    assert!(base_url_arg.is_some(), "base_url argument should exist");
}

// =============================================================================
// JTBD-006: Mutually Exclusive Argument Groups
// =============================================================================

#[derive(Serialize, Debug, PartialEq)]
struct OutputFormat {
    format: String,
}

/// Export data in different formats
///
/// # Arguments
/// * `json` - Export as JSON [group: format]
/// * `yaml` - Export as YAML [group: format]
/// * `csv` - Export as CSV [group: format]
#[noun("export", "Data export commands")]
#[verb("data")]
fn export_data(json: bool, yaml: bool, csv: bool) -> Result<OutputFormat> {
    let format = if json {
        "json"
    } else if yaml {
        "yaml"
    } else if csv {
        "csv"
    } else {
        "json" // default
    }
    .to_string();

    Ok(OutputFormat { format })
}

#[test]
fn test_jtbd_006_exclusive_groups_registered() {
    let registry = clap_noun_verb::cli::registry::CommandRegistry::get();
    let registry = registry.lock().unwrap_or_else(|e| e.into_inner());
    let cmd = registry.build_command();

    let export_cmd = cmd.get_subcommands().find(|s| s.get_name() == "export");
    assert!(export_cmd.is_some(), "export noun should be registered");

    let data_cmd = export_cmd.unwrap().get_subcommands().find(|s| s.get_name() == "data");
    assert!(data_cmd.is_some(), "data verb should be registered");

    let data_cmd = data_cmd.unwrap();
    let args: Vec<_> = data_cmd.get_arguments().collect();

    // All three format options should be registered
    let json_arg = args.iter().find(|a| a.get_id().as_str() == "json");
    let yaml_arg = args.iter().find(|a| a.get_id().as_str() == "yaml");
    let csv_arg = args.iter().find(|a| a.get_id().as_str() == "csv");

    assert!(json_arg.is_some(), "json argument should exist");
    assert!(yaml_arg.is_some(), "yaml argument should exist");
    assert!(csv_arg.is_some(), "csv argument should exist");
}

// =============================================================================
// JTBD-007: Required Arguments with Validation
// =============================================================================

#[derive(Serialize, Debug, PartialEq)]
struct UserInfo {
    username: String,
    email: String,
    age: Option<u8>,
}

/// Create a new user
///
/// # Arguments
/// * `username` - Username (required)
/// * `email` - Email address (required)
/// * `age` - User age (optional)
#[noun("user", "User management commands")]
#[verb("create")]
fn user_create(username: String, email: String, age: Option<u8>) -> Result<UserInfo> {
    Ok(UserInfo { username, email, age })
}

#[test]
fn test_jtbd_007_required_args_registered() {
    let registry = clap_noun_verb::cli::registry::CommandRegistry::get();
    let registry = registry.lock().unwrap_or_else(|e| e.into_inner());
    let cmd = registry.build_command();

    let user_cmd = cmd.get_subcommands().find(|s| s.get_name() == "user");
    assert!(user_cmd.is_some(), "user noun should be registered");

    let create_cmd = user_cmd.unwrap().get_subcommands().find(|s| s.get_name() == "create");
    assert!(create_cmd.is_some(), "create verb should be registered");

    let create_cmd = create_cmd.unwrap();
    let args: Vec<_> = create_cmd.get_arguments().collect();

    let username_arg = args.iter().find(|a| a.get_id().as_str() == "username");
    let email_arg = args.iter().find(|a| a.get_id().as_str() == "email");
    let age_arg = args.iter().find(|a| a.get_id().as_str() == "age");

    assert!(username_arg.is_some(), "username argument should exist");
    assert!(email_arg.is_some(), "email argument should exist");
    assert!(age_arg.is_some(), "age argument should exist");

    // Username and email should be required (non-optional String)
    // Age should be optional (Option<u8>)
}

// =============================================================================
// JTBD-008: Path Arguments for File Operations
// =============================================================================

#[derive(Serialize, Debug)]
struct FileOp {
    source: String,
    destination: Option<String>,
}

/// Copy a file
///
/// # Arguments
/// * `source` - Source file path
/// * `destination` - Destination file path
#[noun("file", "File operations")]
#[verb("copy")]
fn file_copy(source: String, destination: Option<String>) -> Result<FileOp> {
    Ok(FileOp { source, destination })
}

#[test]
fn test_jtbd_008_path_args_registered() {
    let registry = clap_noun_verb::cli::registry::CommandRegistry::get();
    let registry = registry.lock().unwrap_or_else(|e| e.into_inner());
    let cmd = registry.build_command();

    let file_cmd = cmd.get_subcommands().find(|s| s.get_name() == "file");
    assert!(file_cmd.is_some(), "file noun should be registered");

    let copy_cmd = file_cmd.unwrap().get_subcommands().find(|s| s.get_name() == "copy");
    assert!(copy_cmd.is_some(), "copy verb should be registered");

    let copy_cmd = copy_cmd.unwrap();
    let args: Vec<_> = copy_cmd.get_arguments().collect();

    let source_arg = args.iter().find(|a| a.get_id().as_str() == "source");
    let dest_arg = args.iter().find(|a| a.get_id().as_str() == "destination");

    assert!(source_arg.is_some(), "source argument should exist");
    assert!(dest_arg.is_some(), "destination argument should exist");
}

// =============================================================================
// JTBD-009: Default Values with Fallback Behavior
// =============================================================================

#[derive(Serialize, Debug, PartialEq)]
struct PageConfig {
    page: usize,
    limit: usize,
    sort: String,
}

/// List items with pagination
///
/// # Arguments
/// * `page` - Page number [default: 1]
/// * `limit` - Items per page [default: 10]
/// * `sort` - Sort order [default: asc]
#[noun("items", "Item listing commands")]
#[verb("list")]
fn items_list(
    page: Option<usize>,
    limit: Option<usize>,
    sort: Option<String>,
) -> Result<PageConfig> {
    Ok(PageConfig {
        page: page.unwrap_or(1),
        limit: limit.unwrap_or(10),
        sort: sort.unwrap_or_else(|| "asc".to_string()),
    })
}

#[test]
fn test_jtbd_009_default_values_registered() {
    let registry = clap_noun_verb::cli::registry::CommandRegistry::get();
    let registry = registry.lock().unwrap_or_else(|e| e.into_inner());
    let cmd = registry.build_command();

    let items_cmd = cmd.get_subcommands().find(|s| s.get_name() == "items");
    assert!(items_cmd.is_some(), "items noun should be registered");

    let list_cmd = items_cmd.unwrap().get_subcommands().find(|s| s.get_name() == "list");
    assert!(list_cmd.is_some(), "list verb should be registered");

    let list_cmd = list_cmd.unwrap();
    let args: Vec<_> = list_cmd.get_arguments().collect();

    let page_arg = args.iter().find(|a| a.get_id().as_str() == "page");
    let limit_arg = args.iter().find(|a| a.get_id().as_str() == "limit");
    let sort_arg = args.iter().find(|a| a.get_id().as_str() == "sort");

    assert!(page_arg.is_some(), "page argument should exist");
    assert!(limit_arg.is_some(), "limit argument should exist");
    assert!(sort_arg.is_some(), "sort argument should exist");
}

// =============================================================================
// JTBD-010: Mixed Positional and Named Arguments
// =============================================================================

#[derive(Serialize, Debug)]
struct DeployConfig {
    environment: String,
    version: Option<String>,
    dry_run: bool,
    force: bool,
}

/// Deploy to an environment
///
/// # Arguments
/// * `environment` - Target environment (positional)
/// * `version` - Version to deploy
/// * `dry_run` - Perform a dry run without actual changes
/// * `force` - Force deployment even with warnings
#[noun("deploy", "Deployment commands")]
#[verb("to")]
fn deploy_to(
    environment: String,
    version: Option<String>,
    dry_run: bool,
    force: bool,
) -> Result<DeployConfig> {
    Ok(DeployConfig { environment, version, dry_run, force })
}

#[test]
fn test_jtbd_010_mixed_args_registered() {
    let registry = clap_noun_verb::cli::registry::CommandRegistry::get();
    let registry = registry.lock().unwrap_or_else(|e| e.into_inner());
    let cmd = registry.build_command();

    let deploy_cmd = cmd.get_subcommands().find(|s| s.get_name() == "deploy");
    assert!(deploy_cmd.is_some(), "deploy noun should be registered");

    let to_cmd = deploy_cmd.unwrap().get_subcommands().find(|s| s.get_name() == "to");
    assert!(to_cmd.is_some(), "to verb should be registered");

    let to_cmd = to_cmd.unwrap();
    let args: Vec<_> = to_cmd.get_arguments().collect();

    let env_arg = args.iter().find(|a| a.get_id().as_str() == "environment");
    let version_arg = args.iter().find(|a| a.get_id().as_str() == "version");
    let dry_run_arg = args.iter().find(|a| a.get_id().as_str() == "dry_run");
    let force_arg = args.iter().find(|a| a.get_id().as_str() == "force");

    assert!(env_arg.is_some(), "environment argument should exist");
    assert!(version_arg.is_some(), "version argument should exist");
    assert!(dry_run_arg.is_some(), "dry_run argument should exist");
    assert!(force_arg.is_some(), "force argument should exist");
}

// =============================================================================
// Integration Test: Verify All JTBD Scenarios in Single Registry
// =============================================================================

#[test]
fn test_all_jtbd_nouns_registered() {
    let registry = clap_noun_verb::cli::registry::CommandRegistry::get();
    let registry = registry.lock().unwrap_or_else(|e| e.into_inner());

    let nouns = registry.get_nouns();
    let noun_names: Vec<&str> = nouns.iter().map(|(name, _)| *name).collect();

    // Verify all 10 JTBD scenario nouns are registered
    let expected_nouns = vec![
        "batch", "runner", "server", "feature", "env", "export", "user", "file", "items", "deploy",
    ];

    for noun in expected_nouns {
        assert!(
            noun_names.contains(&noun),
            "Expected '{}' noun to be registered. Found: {:?}",
            noun,
            noun_names
        );
    }
}
