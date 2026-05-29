//! Agent Sandbox features - in-memory synthetic command executor and mock registry database

use std::collections::HashMap;
use std::sync::Mutex;
use crate::integration::registry_client::{RegistryInfo, RegistrySource, RegistrySearchResult, RegistryHealth};
use clap_noun_verb::logic::HandlerOutput;
use clap_noun_verb::error::NounVerbError;

lazy_static::lazy_static! {
    pub static ref MOCK_REGISTRY: Mutex<MockRegistryDatabase> = Mutex::new(MockRegistryDatabase::new());
}

/// A programmatic mock registry database to configure mock behaviors for `RegistryClient`
pub struct MockRegistryDatabase {
    pub packages: HashMap<String, RegistryInfo>,
    pub sources: Vec<RegistrySource>,
    pub healthy: bool,
    pub active: bool,
}

impl MockRegistryDatabase {
    pub fn new() -> Self {
        Self {
            packages: HashMap::new(),
            sources: vec![
                RegistrySource {
                    name: "mock-default".to_string(),
                    url: "https://mock.registry.dev".to_string(),
                    priority: 100,
                }
            ],
            healthy: true,
            active: false,
        }
    }

    pub fn clear(&mut self) {
        self.packages.clear();
        self.sources.clear();
        self.sources.push(RegistrySource {
            name: "mock-default".to_string(),
            url: "https://mock.registry.dev".to_string(),
            priority: 100,
        });
        self.healthy = true;
        self.active = false;
    }

    pub fn register_package(&mut self, info: RegistryInfo) {
        self.packages.insert(info.name.clone(), info);
    }

    pub fn add_source(&mut self, source: RegistrySource) {
        self.sources.push(source);
    }

    pub fn set_healthy(&mut self, healthy: bool) {
        self.healthy = healthy;
    }

    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }
}


/// An in-memory synthetic command executor that runs command sequences without producing side effects on dry-runs
pub struct SyntheticCommandExecutor {
    dry_run: bool,
}

impl SyntheticCommandExecutor {
    pub fn new(dry_run: bool) -> Self {
        Self { dry_run }
    }

    pub fn execute(&self, args: Vec<String>) -> Result<HandlerOutput, NounVerbError> {
        // Parse noun and verb
        let mut args_iter = args.iter().peekable();
        if let Some(first) = args_iter.peek() {
            if *first == "mcpp" || *first == "cli" || first.ends_with("mcpp") {
                args_iter.next();
            }
        }
        let noun = args_iter.next().map(|s| s.as_str()).unwrap_or("");
        let verb = args_iter.next().map(|s| s.as_str()).unwrap_or("");

        // Check if it's a mutating capability in the ontology
        let capabilities = crate::domain::ontology::build_playground_ontology();
        let is_mutating = capabilities.iter().any(|cap| {
            cap.noun == noun && cap.verb == verb && cap.effects == crate::domain::ontology::EffectType::Mutating
        });

        if is_mutating && self.dry_run {
            return Ok(HandlerOutput {
                data: serde_json::json!({
                    "status": "dry_run_intercepted",
                    "noun": noun,
                    "verb": verb,
                    "message": format!("Dry-run intercepted mutating capability cnv:{}_{}", noun, verb)
                }),
                message: Some(format!("Dry-run intercepted mutating capability cnv:{}_{}", noun, verb)),
            });
        }

        // Retrieve command registry and execute single step
        let registry = clap_noun_verb::cli::registry::CommandRegistry::get();
        let reg = registry.lock().map_err(|e| {
            NounVerbError::execution_error(format!("Failed to lock registry: {}", e))
        })?;

        // Ensure arguments array starts with the binary name (for clap's routing expectations)
        let mut exec_args = args.clone();
        if let Some(first) = args.first() {
            if first != "mcpp" && first != "cli" && !first.ends_with("mcpp") {
                exec_args.insert(0, "mcpp".to_string());
            }
        } else {
            exec_args.insert(0, "mcpp".to_string());
        }

        reg.execute_single_step(exec_args)
    }
}
