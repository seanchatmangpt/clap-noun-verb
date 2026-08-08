use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
pub struct OutputPayload {
    pub noun: String,
    pub verb: String,
    pub payload: String,
    pub flags: Vec<String>,
}


pub mod users {
    use super::*;

    #[verb("create", "users")]
    pub fn create(name: String, force: bool) -> Result<OutputPayload> {
        Ok(OutputPayload {
            noun: "users".to_string(),
            verb: "create".to_string(),
            payload: name,
            flags: if force { vec!["force".to_string()] } else { vec![] },
        })
    }

    #[verb("read", "users")]
    pub fn read(id: String) -> Result<OutputPayload> {
        Ok(OutputPayload {
            noun: "users".to_string(),
            verb: "read".to_string(),
            payload: id,
            flags: vec![],
        })
    }

    #[verb("update", "users")]
    pub fn update(id: String, payload: String) -> Result<OutputPayload> {
        Ok(OutputPayload {
            noun: "users".to_string(),
            verb: "update".to_string(),
            payload: format!("{}:{}", id, payload),
            flags: vec![],
        })
    }

    #[verb("delete", "users")]
    pub fn delete(id: String, cascade: bool) -> Result<OutputPayload> {
        Ok(OutputPayload {
            noun: "users".to_string(),
            verb: "delete".to_string(),
            payload: id,
            flags: if cascade { vec!["cascade".to_string()] } else { vec![] },
        })
    }

    #[verb("list", "users")]
    pub fn list(limit: Option<usize>) -> Result<OutputPayload> {
        Ok(OutputPayload {
            noun: "users".to_string(),
            verb: "list".to_string(),
            payload: format!("limit={}", limit.unwrap_or(100)),
            flags: vec![],
        })
    }
}

pub mod roles {
    use super::*;

    #[verb("create", "roles")]
    pub fn create(name: String, force: bool) -> Result<OutputPayload> {
        Ok(OutputPayload {
            noun: "roles".to_string(),
            verb: "create".to_string(),
            payload: name,
            flags: if force { vec!["force".to_string()] } else { vec![] },
        })
    }

    #[verb("read", "roles")]
    pub fn read(id: String) -> Result<OutputPayload> {
        Ok(OutputPayload {
            noun: "roles".to_string(),
            verb: "read".to_string(),
            payload: id,
            flags: vec![],
        })
    }

    #[verb("update", "roles")]
    pub fn update(id: String, payload: String) -> Result<OutputPayload> {
        Ok(OutputPayload {
            noun: "roles".to_string(),
            verb: "update".to_string(),
            payload: format!("{}:{}", id, payload),
            flags: vec![],
        })
    }

    #[verb("delete", "roles")]
    pub fn delete(id: String, cascade: bool) -> Result<OutputPayload> {
        Ok(OutputPayload {
            noun: "roles".to_string(),
            verb: "delete".to_string(),
            payload: id,
            flags: if cascade { vec!["cascade".to_string()] } else { vec![] },
        })
    }

    #[verb("list", "roles")]
    pub fn list(limit: Option<usize>) -> Result<OutputPayload> {
        Ok(OutputPayload {
            noun: "roles".to_string(),
            verb: "list".to_string(),
            payload: format!("limit={}", limit.unwrap_or(100)),
            flags: vec![],
        })
    }
}

pub mod policies {
    use super::*;

    #[verb("create", "policies")]
    pub fn create(name: String, force: bool) -> Result<OutputPayload> {
        Ok(OutputPayload {
            noun: "policies".to_string(),
            verb: "create".to_string(),
            payload: name,
            flags: if force { vec!["force".to_string()] } else { vec![] },
        })
    }

    #[verb("read", "policies")]
    pub fn read(id: String) -> Result<OutputPayload> {
        Ok(OutputPayload {
            noun: "policies".to_string(),
            verb: "read".to_string(),
            payload: id,
            flags: vec![],
        })
    }

    #[verb("update", "policies")]
    pub fn update(id: String, payload: String) -> Result<OutputPayload> {
        Ok(OutputPayload {
            noun: "policies".to_string(),
            verb: "update".to_string(),
            payload: format!("{}:{}", id, payload),
            flags: vec![],
        })
    }

    #[verb("delete", "policies")]
    pub fn delete(id: String, cascade: bool) -> Result<OutputPayload> {
        Ok(OutputPayload {
            noun: "policies".to_string(),
            verb: "delete".to_string(),
            payload: id,
            flags: if cascade { vec!["cascade".to_string()] } else { vec![] },
        })
    }

    #[verb("list", "policies")]
    pub fn list(limit: Option<usize>) -> Result<OutputPayload> {
        Ok(OutputPayload {
            noun: "policies".to_string(),
            verb: "list".to_string(),
            payload: format!("limit={}", limit.unwrap_or(100)),
            flags: vec![],
        })
    }
}

pub mod services {
    use super::*;

    #[verb("create", "services")]
    pub fn create(name: String, force: bool) -> Result<OutputPayload> {
        Ok(OutputPayload {
            noun: "services".to_string(),
            verb: "create".to_string(),
            payload: name,
            flags: if force { vec!["force".to_string()] } else { vec![] },
        })
    }

    #[verb("read", "services")]
    pub fn read(id: String) -> Result<OutputPayload> {
        Ok(OutputPayload {
            noun: "services".to_string(),
            verb: "read".to_string(),
            payload: id,
            flags: vec![],
        })
    }

    #[verb("update", "services")]
    pub fn update(id: String, payload: String) -> Result<OutputPayload> {
        Ok(OutputPayload {
            noun: "services".to_string(),
            verb: "update".to_string(),
            payload: format!("{}:{}", id, payload),
            flags: vec![],
        })
    }

    #[verb("delete", "services")]
    pub fn delete(id: String, cascade: bool) -> Result<OutputPayload> {
        Ok(OutputPayload {
            noun: "services".to_string(),
            verb: "delete".to_string(),
            payload: id,
            flags: if cascade { vec!["cascade".to_string()] } else { vec![] },
        })
    }

    #[verb("list", "services")]
    pub fn list(limit: Option<usize>) -> Result<OutputPayload> {
        Ok(OutputPayload {
            noun: "services".to_string(),
            verb: "list".to_string(),
            payload: format!("limit={}", limit.unwrap_or(100)),
            flags: vec![],
        })
    }
}

pub mod deployments {
    use super::*;

    #[verb("create", "deployments")]
    pub fn create(name: String, force: bool) -> Result<OutputPayload> {
        Ok(OutputPayload {
            noun: "deployments".to_string(),
            verb: "create".to_string(),
            payload: name,
            flags: if force { vec!["force".to_string()] } else { vec![] },
        })
    }

    #[verb("read", "deployments")]
    pub fn read(id: String) -> Result<OutputPayload> {
        Ok(OutputPayload {
            noun: "deployments".to_string(),
            verb: "read".to_string(),
            payload: id,
            flags: vec![],
        })
    }

    #[verb("update", "deployments")]
    pub fn update(id: String, payload: String) -> Result<OutputPayload> {
        Ok(OutputPayload {
            noun: "deployments".to_string(),
            verb: "update".to_string(),
            payload: format!("{}:{}", id, payload),
            flags: vec![],
        })
    }

    #[verb("delete", "deployments")]
    pub fn delete(id: String, cascade: bool) -> Result<OutputPayload> {
        Ok(OutputPayload {
            noun: "deployments".to_string(),
            verb: "delete".to_string(),
            payload: id,
            flags: if cascade { vec!["cascade".to_string()] } else { vec![] },
        })
    }

    #[verb("list", "deployments")]
    pub fn list(limit: Option<usize>) -> Result<OutputPayload> {
        Ok(OutputPayload {
            noun: "deployments".to_string(),
            verb: "list".to_string(),
            payload: format!("limit={}", limit.unwrap_or(100)),
            flags: vec![],
        })
    }
}

pub mod nodes {
    use super::*;

    #[verb("create", "nodes")]
    pub fn create(name: String, force: bool) -> Result<OutputPayload> {
        Ok(OutputPayload {
            noun: "nodes".to_string(),
            verb: "create".to_string(),
            payload: name,
            flags: if force { vec!["force".to_string()] } else { vec![] },
        })
    }

    #[verb("read", "nodes")]
    pub fn read(id: String) -> Result<OutputPayload> {
        Ok(OutputPayload {
            noun: "nodes".to_string(),
            verb: "read".to_string(),
            payload: id,
            flags: vec![],
        })
    }

    #[verb("update", "nodes")]
    pub fn update(id: String, payload: String) -> Result<OutputPayload> {
        Ok(OutputPayload {
            noun: "nodes".to_string(),
            verb: "update".to_string(),
            payload: format!("{}:{}", id, payload),
            flags: vec![],
        })
    }

    #[verb("delete", "nodes")]
    pub fn delete(id: String, cascade: bool) -> Result<OutputPayload> {
        Ok(OutputPayload {
            noun: "nodes".to_string(),
            verb: "delete".to_string(),
            payload: id,
            flags: if cascade { vec!["cascade".to_string()] } else { vec![] },
        })
    }

    #[verb("list", "nodes")]
    pub fn list(limit: Option<usize>) -> Result<OutputPayload> {
        Ok(OutputPayload {
            noun: "nodes".to_string(),
            verb: "list".to_string(),
            payload: format!("limit={}", limit.unwrap_or(100)),
            flags: vec![],
        })
    }
}

pub mod clusters {
    use super::*;

    #[verb("create", "clusters")]
    pub fn create(name: String, force: bool) -> Result<OutputPayload> {
        Ok(OutputPayload {
            noun: "clusters".to_string(),
            verb: "create".to_string(),
            payload: name,
            flags: if force { vec!["force".to_string()] } else { vec![] },
        })
    }

    #[verb("read", "clusters")]
    pub fn read(id: String) -> Result<OutputPayload> {
        Ok(OutputPayload {
            noun: "clusters".to_string(),
            verb: "read".to_string(),
            payload: id,
            flags: vec![],
        })
    }

    #[verb("update", "clusters")]
    pub fn update(id: String, payload: String) -> Result<OutputPayload> {
        Ok(OutputPayload {
            noun: "clusters".to_string(),
            verb: "update".to_string(),
            payload: format!("{}:{}", id, payload),
            flags: vec![],
        })
    }

    #[verb("delete", "clusters")]
    pub fn delete(id: String, cascade: bool) -> Result<OutputPayload> {
        Ok(OutputPayload {
            noun: "clusters".to_string(),
            verb: "delete".to_string(),
            payload: id,
            flags: if cascade { vec!["cascade".to_string()] } else { vec![] },
        })
    }

    #[verb("list", "clusters")]
    pub fn list(limit: Option<usize>) -> Result<OutputPayload> {
        Ok(OutputPayload {
            noun: "clusters".to_string(),
            verb: "list".to_string(),
            payload: format!("limit={}", limit.unwrap_or(100)),
            flags: vec![],
        })
    }
}

pub mod volumes {
    use super::*;

    #[verb("create", "volumes")]
    pub fn create(name: String, force: bool) -> Result<OutputPayload> {
        Ok(OutputPayload {
            noun: "volumes".to_string(),
            verb: "create".to_string(),
            payload: name,
            flags: if force { vec!["force".to_string()] } else { vec![] },
        })
    }

    #[verb("read", "volumes")]
    pub fn read(id: String) -> Result<OutputPayload> {
        Ok(OutputPayload {
            noun: "volumes".to_string(),
            verb: "read".to_string(),
            payload: id,
            flags: vec![],
        })
    }

    #[verb("update", "volumes")]
    pub fn update(id: String, payload: String) -> Result<OutputPayload> {
        Ok(OutputPayload {
            noun: "volumes".to_string(),
            verb: "update".to_string(),
            payload: format!("{}:{}", id, payload),
            flags: vec![],
        })
    }

    #[verb("delete", "volumes")]
    pub fn delete(id: String, cascade: bool) -> Result<OutputPayload> {
        Ok(OutputPayload {
            noun: "volumes".to_string(),
            verb: "delete".to_string(),
            payload: id,
            flags: if cascade { vec!["cascade".to_string()] } else { vec![] },
        })
    }

    #[verb("list", "volumes")]
    pub fn list(limit: Option<usize>) -> Result<OutputPayload> {
        Ok(OutputPayload {
            noun: "volumes".to_string(),
            verb: "list".to_string(),
            payload: format!("limit={}", limit.unwrap_or(100)),
            flags: vec![],
        })
    }
}

pub mod networks {
    use super::*;

    #[verb("create", "networks")]
    pub fn create(name: String, force: bool) -> Result<OutputPayload> {
        Ok(OutputPayload {
            noun: "networks".to_string(),
            verb: "create".to_string(),
            payload: name,
            flags: if force { vec!["force".to_string()] } else { vec![] },
        })
    }

    #[verb("read", "networks")]
    pub fn read(id: String) -> Result<OutputPayload> {
        Ok(OutputPayload {
            noun: "networks".to_string(),
            verb: "read".to_string(),
            payload: id,
            flags: vec![],
        })
    }

    #[verb("update", "networks")]
    pub fn update(id: String, payload: String) -> Result<OutputPayload> {
        Ok(OutputPayload {
            noun: "networks".to_string(),
            verb: "update".to_string(),
            payload: format!("{}:{}", id, payload),
            flags: vec![],
        })
    }

    #[verb("delete", "networks")]
    pub fn delete(id: String, cascade: bool) -> Result<OutputPayload> {
        Ok(OutputPayload {
            noun: "networks".to_string(),
            verb: "delete".to_string(),
            payload: id,
            flags: if cascade { vec!["cascade".to_string()] } else { vec![] },
        })
    }

    #[verb("list", "networks")]
    pub fn list(limit: Option<usize>) -> Result<OutputPayload> {
        Ok(OutputPayload {
            noun: "networks".to_string(),
            verb: "list".to_string(),
            payload: format!("limit={}", limit.unwrap_or(100)),
            flags: vec![],
        })
    }
}

pub mod firewalls {
    use super::*;

    #[verb("create", "firewalls")]
    pub fn create(name: String, force: bool) -> Result<OutputPayload> {
        Ok(OutputPayload {
            noun: "firewalls".to_string(),
            verb: "create".to_string(),
            payload: name,
            flags: if force { vec!["force".to_string()] } else { vec![] },
        })
    }

    #[verb("read", "firewalls")]
    pub fn read(id: String) -> Result<OutputPayload> {
        Ok(OutputPayload {
            noun: "firewalls".to_string(),
            verb: "read".to_string(),
            payload: id,
            flags: vec![],
        })
    }

    #[verb("update", "firewalls")]
    pub fn update(id: String, payload: String) -> Result<OutputPayload> {
        Ok(OutputPayload {
            noun: "firewalls".to_string(),
            verb: "update".to_string(),
            payload: format!("{}:{}", id, payload),
            flags: vec![],
        })
    }

    #[verb("delete", "firewalls")]
    pub fn delete(id: String, cascade: bool) -> Result<OutputPayload> {
        Ok(OutputPayload {
            noun: "firewalls".to_string(),
            verb: "delete".to_string(),
            payload: id,
            flags: if cascade { vec!["cascade".to_string()] } else { vec![] },
        })
    }

    #[verb("list", "firewalls")]
    pub fn list(limit: Option<usize>) -> Result<OutputPayload> {
        Ok(OutputPayload {
            noun: "firewalls".to_string(),
            verb: "list".to_string(),
            payload: format!("limit={}", limit.unwrap_or(100)),
            flags: vec![],
        })
    }
}

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
