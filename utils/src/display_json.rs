use serde::Serialize;
use serde_json::Value;

/// A serializable schema of a clap CLI Command structure.
///
/// # Examples
///
/// ```
/// use clap_noun_verb_utils::display_json::CommandSchema;
///
/// let schema = CommandSchema {
///     name: "test".to_string(),
///     version: Some("1.0.0".to_string()),
///     author: None,
///     about: None,
///     subcommands: vec![],
///     arguments: vec![],
/// };
/// assert_eq!(schema.name, "test");
/// ```
#[derive(Debug, Serialize, Clone)]
pub struct CommandSchema {
    pub name: String,
    pub version: Option<String>,
    pub author: Option<String>,
    pub about: Option<String>,
    pub subcommands: Vec<CommandSchema>,
    pub arguments: Vec<ArgSchema>,
}

/// A serializable schema of a clap Argument structure.
///
/// # Examples
///
/// ```
/// use clap_noun_verb_utils::display_json::ArgSchema;
///
/// let schema = ArgSchema {
///     name: "port".to_string(),
///     short: Some('p'),
///     long: Some("port".to_string()),
///     help: None,
///     required: false,
///     multiple: false,
///     is_flag: false,
/// };
/// assert_eq!(schema.name, "port");
/// ```
#[derive(Debug, Serialize, Clone)]
pub struct ArgSchema {
    pub name: String,
    pub short: Option<char>,
    pub long: Option<String>,
    pub help: Option<String>,
    pub required: bool,
    pub multiple: bool,
    pub is_flag: bool,
}

/// Converts a `clap::Command` structure into its serializable schema representation.
///
/// # Examples
///
/// ```
/// use clap::{Command, Arg};
/// use clap_noun_verb_utils::display_json::extract_command_schema;
///
/// let cmd = Command::new("test")
///     .version("1.0")
///     .arg(Arg::new("port").short('p').action(clap::ArgAction::Set));
/// let schema = extract_command_schema(&cmd);
/// assert_eq!(schema.name, "test");
/// assert_eq!(schema.version, Some("1.0".to_string()));
/// assert_eq!(schema.arguments.len(), 1);
/// assert_eq!(schema.arguments[0].short, Some('p'));
/// ```
pub fn extract_command_schema(cmd: &clap::Command) -> CommandSchema {
    CommandSchema {
        name: cmd.get_name().to_string(),
        version: cmd.get_version().map(String::from),
        author: cmd.get_author().map(String::from),
        about: cmd.get_about().map(|s| s.to_string()),
        subcommands: cmd.get_subcommands().map(extract_command_schema).collect(),
        arguments: cmd.get_arguments()
            .map(|arg| ArgSchema {
                name: arg.get_id().as_str().to_string(),
                short: arg.get_short(),
                long: arg.get_long().map(String::from),
                help: arg.get_help().map(|s| s.to_string()),
                required: arg.is_required_set(),
                multiple: matches!(
                    arg.get_action(), 
                    clap::ArgAction::Append | clap::ArgAction::Count
                ),
                is_flag: matches!(
                    arg.get_action(), 
                    clap::ArgAction::SetTrue | clap::ArgAction::SetFalse | clap::ArgAction::Count
                ),
            })
            .collect(),
    }
}

/// Converts a parsed `clap::ArgMatches` into a JSON Value object map.
///
/// # Examples
///
/// ```
/// use clap::{Command, Arg};
/// use clap_noun_verb_utils::display_json::arg_matches_to_json;
///
/// let cmd = Command::new("test")
///     .arg(Arg::new("port").long("port").num_args(1))
///     .arg(Arg::new("flag").long("flag").action(clap::ArgAction::SetTrue));
/// let matches = cmd.get_matches_from(vec!["test", "--port", "8080", "--flag"]);
/// let json_val = arg_matches_to_json(&matches);
///
/// assert_eq!(json_val["port"], 8080);
/// assert_eq!(json_val["flag"], true);
/// ```
pub fn arg_matches_to_json(matches: &clap::ArgMatches) -> Value {
    let mut map = serde_json::Map::new();
    for id in matches.ids() {
        let name = id.as_str();
        
        if let Some(raw_vals) = matches.get_raw(name) {
            let list: Vec<String> = raw_vals
                .map(|os| os.to_string_lossy().to_string())
                .collect();
                
            if list.len() == 1 {
                map.insert(name.to_string(), parse_string_value(&list[0]));
            } else {
                map.insert(
                    name.to_string(),
                    Value::Array(list.into_iter().map(|s| parse_string_value(&s)).collect()),
                );
            }
        } else if let Ok(Some(&b)) = matches.try_get_one::<bool>(name) {
            map.insert(name.to_string(), Value::Bool(b));
        } else if let Ok(Some(s)) = matches.try_get_one::<String>(name) {
            map.insert(name.to_string(), parse_string_value(s));
        } else {
            let count = matches.get_count(name);
            map.insert(name.to_string(), Value::Number(count.into()));
        }
    }
    Value::Object(map)
}

fn parse_string_value(s: &str) -> Value {
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

/// Helper trait to print any serializable output struct as JSON.
///
/// # Examples
///
/// ```
/// use serde::Serialize;
/// use clap_noun_verb_utils::display_json::PrintJson;
///
/// #[derive(Serialize)]
/// struct User {
///     name: String,
/// }
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let user = User { name: "Alice".to_string() };
/// // This will print `{"name":"Alice"}` to stdout.
/// user.print_json()?;
/// # Ok(())
/// # }
/// ```
pub trait PrintJson {
    fn print_json(&self) -> Result<(), serde_json::Error>;
    fn print_json_pretty(&self) -> Result<(), serde_json::Error>;
}

impl<T: Serialize> PrintJson for T {
    fn print_json(&self) -> Result<(), serde_json::Error> {
        println!("{}", serde_json::to_string(self)?);
        Ok(())
    }
    
    fn print_json_pretty(&self) -> Result<(), serde_json::Error> {
        println!("{}", serde_json::to_string_pretty(self)?);
        Ok(())
    }
}
