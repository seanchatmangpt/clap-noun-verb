import sys

out = """use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
pub struct OutputPayload {
    pub noun: String,
    pub verb: String,
    pub payload: String,
    pub flags: Vec<String>,
}

"""

nouns = [
    "users", "roles", "policies", "services", "deployments",
    "nodes", "clusters", "volumes", "networks", "firewalls"
]

for noun in nouns:
    out += f"""
pub mod {noun} {{
    use super::*;

    #[verb("create", "{noun}")]
    pub fn create(name: String, force: bool) -> Result<OutputPayload> {{
        Ok(OutputPayload {{
            noun: "{noun}".to_string(),
            verb: "create".to_string(),
            payload: name,
            flags: if force {{ vec!["force".to_string()] }} else {{ vec![] }},
        }})
    }}

    #[verb("read", "{noun}")]
    pub fn read(id: String) -> Result<OutputPayload> {{
        Ok(OutputPayload {{
            noun: "{noun}".to_string(),
            verb: "read".to_string(),
            payload: id,
            flags: vec![],
        }})
    }}

    #[verb("update", "{noun}")]
    pub fn update(id: String, payload: String) -> Result<OutputPayload> {{
        Ok(OutputPayload {{
            noun: "{noun}".to_string(),
            verb: "update".to_string(),
            payload: format!("{{}}:{{}}", id, payload),
            flags: vec![],
        }})
    }}

    #[verb("delete", "{noun}")]
    pub fn delete(id: String, cascade: bool) -> Result<OutputPayload> {{
        Ok(OutputPayload {{
            noun: "{noun}".to_string(),
            verb: "delete".to_string(),
            payload: id,
            flags: if cascade {{ vec!["cascade".to_string()] }} else {{ vec![] }},
        }})
    }}

    #[verb("list", "{noun}")]
    pub fn list(limit: Option<usize>) -> Result<OutputPayload> {{
        Ok(OutputPayload {{
            noun: "{noun}".to_string(),
            verb: "list".to_string(),
            payload: format!("limit={{}}", limit.unwrap_or(100)),
            flags: vec![],
        }})
    }}
}}
"""

out += """
#[verb("aggregate", "combinatorial")]
pub fn aggregate(
    step_input: String,
    stdin_input: Option<String>,
) -> Result<OutputPayload> {
    Ok(OutputPayload {
        noun: "combinatorial".to_string(),
        verb: "aggregate".to_string(),
        payload: format!("Merged Pipeline: step=[{}] stdin=[{:?}]", step_input, stdin_input),
        flags: vec!["aggregated".to_string()],
    })
}

fn main() -> Result<()> {
    clap_noun_verb::run()
}
"""

with open("examples/combinatorial_maximum.rs", "w") as f:
    f.write(out)

print("Generated examples/combinatorial_maximum.rs")
