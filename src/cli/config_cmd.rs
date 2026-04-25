//! Built-in configuration management commands
//!
//! This module provides a pre-packaged suite of commands for managing
//! application configuration, including showing the effective config,
//! validating the structure, and saving current state.

use crate::config::ConfigLoader;
use crate::error::Result;
use clap::{Arg, ArgMatches, Command};

/// Generate the `config` subcommand suite
pub fn config_subcommand() -> Command {
    Command::new("config")
        .about("Manage application configuration")
        .subcommand(
            Command::new("show")
                .about("Display the effective configuration")
                .arg(Arg::new("profile").long("profile").help("The profile to show")),
        )
        .subcommand(Command::new("path").about("Display the path to the active configuration file"))
        .subcommand(
            Command::new("init").about("Initialize a default clap-nv.toml file").arg(
                Arg::new("force")
                    .long("force")
                    .action(clap::ArgAction::SetTrue)
                    .help("Overwrite existing file"),
            ),
        )
        .subcommand(
            Command::new("validate")
                .about("Validate the configuration file against registered commands"),
        )
        .subcommand(
            Command::new("save").about("Save the current CLI arguments to the configuration file"),
        )
}

/// Handle the config subcommand
pub fn handle_config_subcommand(
    root_matches: &ArgMatches,
    config_matches: &ArgMatches,
) -> Result<()> {
    match config_matches.subcommand() {
        Some(("show", sub_m)) => {
            let mut loader = ConfigLoader::new();
            if let Some(profile) = sub_m.get_one::<String>("profile") {
                loader = loader.with_profile(profile);
            }
            let config = loader.load_optional()?;
            println!("{:#}", config.to_flat_map_json()?);
            Ok(())
        }
        Some(("path", _)) => {
            let loader = ConfigLoader::new();
            let path = loader.find_config_path();
            if let Some(p) = path {
                let current = std::env::current_dir().map_err(|e| {
                    crate::error::NounVerbError::execution_error(format!(
                        "Failed to get current directory: {}",
                        e
                    ))
                })?;
                println!("{}", current.join(p).display());
            } else {
                println!("No configuration file found.");
            }
            Ok(())
        }
        Some(("init", sub_m)) => {
            let force = sub_m.get_flag("force");
            crate::cli::init::scaffold_config(force)
        }
        Some(("validate", _)) => {
            let loader = ConfigLoader::new();
            let config = loader.load_optional()?;
            let registry = crate::cli::registry::CommandRegistry::get();
            let registry = registry.lock().map_err(|e| {
                crate::error::NounVerbError::execution_error(format!(
                    "Failed to lock registry: {}",
                    e
                ))
            })?;
            registry.validate_config(&config);
            println!("Validation complete.");
            Ok(())
        }
        Some(("save", _)) => {
            let loader = ConfigLoader::new();
            let mut config = loader.load_optional()?;

            // Extract global arguments from root_matches
            let mut map = std::collections::HashMap::new();
            for arg in root_matches.ids() {
                let id = arg.as_str();
                // Exclude the subcommand itself
                if id == "config" || id.is_empty() {
                    continue;
                }

                if let Some(values) = root_matches.get_many::<String>(id) {
                    let vals: Vec<String> = values.cloned().collect();
                    if vals.len() == 1 {
                        map.insert(id.to_string(), vals[0].clone());
                    } else if vals.len() > 1 {
                        // For arrays, we'd need a different way to update, but let's do comma separated for now
                        // or just skip arrays for the basic save.
                        map.insert(id.to_string(), vals.join(","));
                    }
                } else if root_matches.get_flag(id) {
                    map.insert(id.to_string(), "true".to_string());
                }
            }

            if map.is_empty() {
                println!("No global arguments found to save.");
                return Ok(());
            }

            config.update_from_map(&map);

            let path = loader
                .find_config_path()
                .unwrap_or_else(|| std::path::PathBuf::from("clap-nv.toml"));
            config.save_to_file(&path)?;

            println!("Configuration saved to {}", path.display());
            Ok(())
        }
        _ => Ok(()),
    }
}

impl crate::config::Config {
    /// Return the flattened configuration as a JSON string
    pub fn to_flat_map_json(&self) -> Result<String> {
        let map = self.to_flat_map();
        serde_json::to_string_pretty(&map).map_err(|e| {
            crate::error::NounVerbError::execution_error(format!(
                "Failed to serialize config: {}",
                e
            ))
        })
    }
}
