//! Registry commands for searching, info, and listing

use clap_noun_verb_macros::verb;
use clap_noun_verb::{Result, NounVerbError};

use crate::integration::registry_client::RegistryClient;
use crate::outputs::{RegistrySearchOutput, RegistrySearchResultItem, RegistryInfoOutput, RegistrySourcesOutput, RegistrySourceItem};

/// Search the registry for packages
#[verb("search")]
fn search_registry(
    query: String,
    category: Option<String>,
    limit: Option<usize>,
) -> Result<RegistrySearchOutput> {
    let client = RegistryClient::default();
    let results = client.search(&query, category.as_deref(), limit.unwrap_or(20))
        .map_err(|e| NounVerbError::ExecutionError { message: e })?;

    let count = results.len();
    Ok(RegistrySearchOutput {
        query: query.clone(),
        category: category.clone(),
        results: results.into_iter().map(|r| RegistrySearchResultItem {
            name: r.name,
            version: r.version,
            description: r.description,
            category: r.category,
        }).collect(),
        count,
    })
}

/// Get detailed information about a package
#[verb("info")]
fn registry_info(identifier: String) -> Result<RegistryInfoOutput> {
    let client = RegistryClient::default();
    let info = client.get_info(&identifier)
        .map_err(|e| NounVerbError::ExecutionError { message: e })?;

    Ok(RegistryInfoOutput {
        name: info.name,
        description: info.description,
        versions: info.versions,
        latest_version: info.latest_version,
        dependencies: info.dependencies,
        homepage: info.homepage,
        repository: info.repository,
    })
}

/// List all configured registry sources
#[verb("list")]
fn list_registries() -> Result<RegistrySourcesOutput> {
    let client = RegistryClient::default();
    let sources = client.list_sources()
        .map_err(|e| NounVerbError::ExecutionError { message: e })?;

    let count = sources.len();
    Ok(RegistrySourcesOutput {
        sources: sources.into_iter().map(|s| RegistrySourceItem {
            name: s.name,
            url: s.url,
            priority: s.priority,
        }).collect(),
        count,
    })
}
