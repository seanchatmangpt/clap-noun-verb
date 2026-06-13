// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

use clap::ArgMatches;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;

/// Parser for KEY=VALUE argument format.
///
/// # Examples
///
/// ```
/// use clap_noun_verb_utils::adapters::parse_key_val;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let (key, val) = parse_key_val("foo=bar")?;
/// assert_eq!(key, "foo");
/// assert_eq!(val, "bar");
///
/// let invalid = parse_key_val("invalid_format");
/// assert!(invalid.is_err());
/// # Ok(())
/// # }
/// ```
pub fn parse_key_val(s: &str) -> Result<(String, String), String> {
    let pos = s.find('=').ok_or_else(|| format!("invalid KEY=value: no `=` found in `{}`", s))?;
    Ok((s[..pos].trim().to_string(), s[pos + 1..].trim().to_string()))
}

/// Extract key-value pairs from multiple arguments into a HashMap.
///
/// # Examples
///
/// ```
/// use clap::{Command, Arg};
/// use clap_noun_verb_utils::adapters::extract_key_value_pairs;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let cmd = Command::new("test").arg(Arg::new("pairs").action(clap::ArgAction::Append));
/// let matches = cmd.get_matches_from(vec!["test", "pairs=val1", "key2=val2"]);
/// let map = extract_key_value_pairs(&matches, "pairs")?;
/// assert_eq!(map.get("pairs").map(|s| s.as_str()), Some("val1"));
/// assert_eq!(map.get("key2").map(|s| s.as_str()), Some("val2"));
/// # Ok(())
/// # }
/// ```
pub fn extract_key_value_pairs(
    matches: &ArgMatches,
    arg_name: &str,
) -> Result<HashMap<String, String>, String> {
    let mut map = HashMap::new();
    if let Some(pairs) = matches.get_many::<String>(arg_name) {
        for pair in pairs {
            let (key, val) = parse_key_val(pair)?;
            map.insert(key, val);
        }
    }
    Ok(map)
}

/// Decoupling adapter trait to cleanly load domain models from ArgMatches.
///
/// # Examples
///
/// ```
/// use clap::{Command, Arg, ArgMatches};
/// use clap_noun_verb_utils::adapters::FromArgMatches;
///
/// struct MyConfig {
///     port: u16,
/// }
///
/// impl FromArgMatches for MyConfig {
///     type Error = String;
///     
///     fn from_arg_matches(matches: &ArgMatches) -> Result<Self, Self::Error> {
///         let port_str = matches.get_one::<String>("port")
///             .ok_or("port is required")?;
///         let port = port_str.parse::<u16>()
///             .map_err(|e| e.to_string())?;
///         Ok(MyConfig { port })
///     }
/// }
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let cmd = Command::new("test").arg(Arg::new("port").long("port").num_args(1));
/// let matches = cmd.get_matches_from(vec!["test", "--port", "8080"]);
/// let config = MyConfig::from_arg_matches(&matches)?;
/// assert_eq!(config.port, 8080);
/// # Ok(())
/// # }
/// ```
pub trait FromArgMatches: Sized {
    type Error;
    fn from_arg_matches(matches: &ArgMatches) -> Result<Self, Self::Error>;
}

/// Layered config resolver that merges config files, environment variables, and CLI overrides.
///
/// # Examples
///
/// ```
/// use serde::{Serialize, Deserialize};
/// use clap::{Command, Arg};
/// use clap_noun_verb_utils::adapters::LayeredConfigAdapter;
///
/// #[derive(Serialize, Deserialize, Debug, PartialEq)]
/// struct Config {
///     port: u16,
///     host: String,
/// }
///
/// impl Default for Config {
///     fn default() -> Self {
///         Self {
///             port: 8080,
///             host: "localhost".to_string(),
///         }
///     }
/// }
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let cmd = Command::new("test").arg(Arg::new("port").long("port").num_args(1));
/// let matches = cmd.get_matches_from(vec!["test", "--port", "9090"]);
///
/// let adapter = LayeredConfigAdapter::<Config>::new(None, None);
/// let resolved = adapter.resolve(&matches)?;
///
/// assert_eq!(resolved.port, 9090);
/// assert_eq!(resolved.host, "localhost");
/// # Ok(())
/// # }
/// ```
pub struct LayeredConfigAdapter<T> {
    pub file_path: Option<PathBuf>,
    pub env_prefix: Option<String>,
    _marker: std::marker::PhantomData<T>,
}

impl<T> LayeredConfigAdapter<T>
where
    T: Serialize + DeserializeOwned + Default,
{
    pub fn new(file_path: Option<PathBuf>, env_prefix: Option<String>) -> Self {
        Self { file_path, env_prefix, _marker: std::marker::PhantomData }
    }

    pub fn resolve(&self, matches: &ArgMatches) -> Result<T, anyhow::Error> {
        // 1. Start with defaults
        let mut default_val = serde_json::to_value(T::default())?;
        let merged_map = default_val.as_object_mut().ok_or_else(|| {
            anyhow::anyhow!("Configuration model must serialize to a JSON Object")
        })?;

        // 2. Load from config file if present
        if let Some(ref path) = self.file_path {
            if path.exists() {
                let contents = std::fs::read_to_string(path)?;
                let file_val: Value = if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                    toml::from_str(&contents)?
                } else {
                    serde_json::from_str(&contents)?
                };

                if let Some(file_obj) = file_val.as_object() {
                    merge_json_maps(merged_map, file_obj.clone());
                }
            }
        }

        // 3. Load from environment variables
        if let Some(ref prefix) = self.env_prefix {
            let mut env_map = serde_json::Map::new();
            for (key, val) in std::env::vars() {
                if key.starts_with(prefix) {
                    let field_name = key[prefix.len()..].to_lowercase();
                    if !field_name.is_empty() {
                        env_map.insert(field_name, parse_env_val(&val));
                    }
                }
            }
            merge_json_maps(merged_map, env_map);
        }

        // 4. Override with CLI ArgMatches
        let cli_val = crate::display_json::arg_matches_to_json(matches);
        if let Some(cli_obj) = cli_val.as_object() {
            let mut filtered_cli_obj = serde_json::Map::new();
            for (key, val) in cli_obj {
                if matches.value_source(key) != Some(clap::parser::ValueSource::DefaultValue) {
                    filtered_cli_obj.insert(key.clone(), val.clone());
                }
            }
            merge_json_maps(merged_map, filtered_cli_obj);
        }

        // 5. Deserialize merged object back to configuration struct
        let resolved: T = serde_json::from_value(Value::Object(merged_map.clone()))?;
        Ok(resolved)
    }
}

fn parse_env_val(s: &str) -> Value {
    if let Ok(b) = s.parse::<bool>() {
        Value::Bool(b)
    } else if let Ok(i) = s.parse::<i64>() {
        Value::Number(i.into())
    } else if let Ok(f) = s.parse::<f64>() {
        if let Some(num) = serde_json::Number::from_f64(f) {
            Value::Number(num)
        } else {
            Value::String(s.to_string())
        }
    } else {
        Value::String(s.to_string())
    }
}

fn get_or_create_nested_map<'a>(
    target: &'a mut serde_json::Map<String, Value>,
    parts: &[&str],
) -> Option<&'a mut serde_json::Map<String, Value>> {
    if parts.is_empty() {
        Some(target)
    } else {
        let p = parts[0];
        let next_val =
            target.entry(p.to_string()).or_insert_with(|| Value::Object(serde_json::Map::new()));
        if !next_val.is_object() {
            *next_val = Value::Object(serde_json::Map::new());
        }
        let next_map = next_val.as_object_mut()?;
        get_or_create_nested_map(next_map, &parts[1..])
    }
}

fn merge_json_maps(
    target: &mut serde_json::Map<String, Value>,
    source: serde_json::Map<String, Value>,
) {
    for (k, v) in source {
        if v.is_null() {
            continue;
        }
        let normalized_k = k.replace("__", ".");
        let parts: Vec<&str> = normalized_k.split('.').filter(|s| !s.is_empty()).collect();
        if parts.is_empty() {
            continue;
        }

        let curr_map = match get_or_create_nested_map(target, &parts[..parts.len() - 1]) {
            Some(m) => m,
            None => continue,
        };

        let last_key = match parts.last() {
            Some(key) => key.to_string(),
            None => continue,
        };

        match curr_map.entry(last_key) {
            serde_json::map::Entry::Occupied(mut entry) => {
                if let (Some(target_obj), Some(source_obj)) =
                    (entry.get_mut().as_object_mut(), v.as_object())
                {
                    merge_json_maps(target_obj, source_obj.clone());
                } else {
                    entry.insert(v);
                }
            }
            serde_json::map::Entry::Vacant(entry) => {
                entry.insert(v);
            }
        }
    }
}
