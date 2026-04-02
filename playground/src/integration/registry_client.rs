//! Registry protocol client

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrySearchResult {
    pub name: String,
    pub version: String,
    pub description: String,
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryInfo {
    pub name: String,
    pub description: String,
    pub versions: Vec<String>,
    pub latest_version: String,
    pub dependencies: Vec<String>,
    pub homepage: Option<String>,
    pub repository: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrySource {
    pub name: String,
    pub url: String,
    pub priority: u32,
}

pub struct RegistryClient;

impl RegistryClient {
    pub fn default() -> Self {
        Self
    }

    pub fn search(
        &self,
        query: &str,
        category: Option<&str>,
        limit: usize,
    ) -> Result<Vec<RegistrySearchResult>, String> {
        // TODO: Implement registry search
        Ok(vec![])
    }

    pub fn get_info(&self, identifier: &str) -> Result<RegistryInfo, String> {
        // TODO: Implement info retrieval
        Ok(RegistryInfo {
            name: identifier.to_string(),
            description: "TODO".to_string(),
            versions: vec![],
            latest_version: "0.0.0".to_string(),
            dependencies: vec![],
            homepage: None,
            repository: None,
        })
    }

    pub fn list_sources(&self) -> Result<Vec<RegistrySource>, String> {
        Ok(vec![RegistrySource {
            name: "default".to_string(),
            url: "https://registry.ggen.dev".to_string(),
            priority: 100,
        }])
    }
}

impl Default for RegistryClient {
    fn default() -> Self {
        Self::default()
    }
}
