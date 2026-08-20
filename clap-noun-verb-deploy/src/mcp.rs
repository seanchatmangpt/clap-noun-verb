//! MCP 2026-07-28 stdio adapter for a projected CLI.
//!
//! The modern MCP era is stateless: there is no `initialize` handshake or
//! protocol session. Every request carries protocol metadata, optional discovery
//! uses `server/discover`, and all tool execution still crosses the same schema,
//! admission, gateway, executor, and execution-record boundary.

use crate::{AdmissionPolicy, AdmitValidated, CliSchema, Executor, Gateway, GatewayError};
use serde_json::{json, Map, Value};
use std::io::{BufRead, Write};
use thiserror::Error;

pub const MCP_PROTOCOL_VERSION: &str = "2026-07-28";
const PROTOCOL_VERSION_META: &str = "io.modelcontextprotocol/protocolVersion";
const CLIENT_INFO_META: &str = "io.modelcontextprotocol/clientInfo";
const CLIENT_CAPABILITIES_META: &str = "io.modelcontextprotocol/clientCapabilities";
const SERVER_INFO_META: &str = "io.modelcontextprotocol/serverInfo";

pub struct McpServer<E, P = AdmitValidated> {
    name: String,
    version: String,
    schema: CliSchema,
    gateway: Gateway<E, P>,
}

impl<E> McpServer<E, AdmitValidated> {
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        version: impl Into<String>,
        schema: CliSchema,
        executor: E,
    ) -> Self {
        let name = name.into();
        Self {
            version: version.into(),
            schema,
            gateway: Gateway::new(name.clone(), executor, AdmitValidated),
            name,
        }
    }
}

impl<E, P> McpServer<E, P> {
    #[must_use]
    pub fn with_policy(
        name: impl Into<String>,
        version: impl Into<String>,
        schema: CliSchema,
        executor: E,
        policy: P,
    ) -> Self {
        let name = name.into();
        Self {
            version: version.into(),
            schema,
            gateway: Gateway::new(name.clone(), executor, policy),
            name,
        }
    }

    #[must_use]
    pub const fn schema(&self) -> &CliSchema {
        &self.schema
    }
}

#[derive(Debug, Error)]
pub enum McpError {
    #[error("stdio transport failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid JSON-RPC request: {0}")]
    Json(#[from] serde_json::Error),
    #[error("executor failed: {0}")]
    Execution(String),
}

impl<E, P> McpServer<E, P>
where
    E: Executor,
    P: AdmissionPolicy,
{
    /// Serve newline-delimited JSON-RPC over an arbitrary buffered reader/writer.
    pub fn serve_stdio<R: BufRead, W: Write>(
        &self,
        reader: R,
        mut writer: W,
    ) -> Result<(), McpError> {
        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let request: Value = serde_json::from_str(&line)?;
            if let Some(response) = self.handle(&request)? {
                serde_json::to_writer(&mut writer, &response)?;
                writeln!(writer)?;
                writer.flush()?;
            }
        }
        Ok(())
    }

    /// Handle one modern MCP JSON-RPC request. Notifications return `None`.
    pub fn handle(&self, request: &Value) -> Result<Option<Value>, McpError> {
        let Some(id) = request.get("id").cloned() else {
            return Ok(None);
        };
        if request.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
            return Ok(Some(self.error(id, -32600, "Invalid Request")));
        }
        let Some(method) = request.get("method").and_then(Value::as_str) else {
            return Ok(Some(self.error(id, -32600, "Invalid Request")));
        };
        if let Err(message) = validate_request_meta(request) {
            return Ok(Some(self.error(id, -32600, message)));
        }

        let result = match method {
            "server/discover" => json!({
                "supportedVersions": [MCP_PROTOCOL_VERSION],
                "capabilities": {"tools": {"listChanged": false}},
                "instructions": "CLI tools are schema-validated and policy-admitted before execution.",
                "ttlMs": 60_000,
                "cacheScope": "public"
            }),
            "ping" => json!({}),
            "tools/list" => {
                let mut tools = self.schema.tools();
                tools.sort_by(|left, right| left.name.cmp(&right.name));
                json!({
                    "tools": tools.into_iter().map(|tool| json!({
                        "name": tool.name,
                        "description": tool.description.unwrap_or_default(),
                        "inputSchema": tool.input_schema
                    })).collect::<Vec<_>>(),
                    "ttlMs": 60_000,
                    "cacheScope": "public"
                })
            }
            "tools/call" => return self.call_tool(id, request),
            _ => return Ok(Some(self.error(id, -32601, "Method not found"))),
        };

        Ok(Some(self.success(id, result)))
    }

    fn call_tool(&self, id: Value, request: &Value) -> Result<Option<Value>, McpError> {
        let params = request.get("params").and_then(Value::as_object);
        let Some(params) = params else {
            return Ok(Some(self.error(id, -32602, "Invalid params")));
        };
        let Some(name) = params.get("name").and_then(Value::as_str) else {
            return Ok(Some(self.error(id, -32602, "Missing tool name")));
        };
        let arguments =
            params.get("arguments").and_then(Value::as_object).cloned().unwrap_or_default();

        let invocation = match self.schema.build_invocation(name, &arguments) {
            Ok(invocation) => invocation,
            Err(build_error) => {
                return Ok(Some(self.error(id, -32602, &build_error.to_string())))
            }
        };
        let record = match self.gateway.execute(invocation) {
            Ok(record) => record,
            Err(GatewayError::Refused(reason)) => {
                return Ok(Some(self.error(id, -32602, &format!("refused: {reason}"))))
            }
            Err(gateway_error) => return Err(McpError::Execution(gateway_error.to_string())),
        };
        let is_error = !record.execution.success();
        let text = if record.execution.stdout.is_empty() {
            record.execution.stderr.clone()
        } else {
            record.execution.stdout.clone()
        };
        let result = json!({
            "content": [{"type": "text", "text": text}],
            "isError": is_error,
            "_meta": {
                "clapNounVerbDeploy": {
                    "subject": record.subject,
                    "fingerprint": record.fingerprint
                }
            }
        });
        Ok(Some(self.success(id, result)))
    }

    fn success(&self, id: Value, mut result: Value) -> Value {
        if let Some(result_object) = result.as_object_mut() {
            result_object.insert("resultType".to_owned(), Value::String("complete".to_owned()));
        }
        stamp_server_info(&mut result, &self.name, &self.version);
        json!({"jsonrpc": "2.0", "id": id, "result": result})
    }

    fn error(&self, id: Value, code: i32, message: &str) -> Value {
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {"code": code, "message": message},
            "_meta": {
                SERVER_INFO_META: {"name": self.name, "version": self.version}
            }
        })
    }
}

fn validate_request_meta(request: &Value) -> Result<(), &'static str> {
    let Some(meta) = request.get("_meta").and_then(Value::as_object) else {
        return Err("missing MCP 2026-07-28 request metadata");
    };
    if meta.get(PROTOCOL_VERSION_META).and_then(Value::as_str) != Some(MCP_PROTOCOL_VERSION) {
        return Err("unsupported or missing MCP protocol version");
    }
    if !meta.get(CLIENT_CAPABILITIES_META).is_some_and(Value::is_object) {
        return Err("missing or malformed MCP client capabilities");
    }
    if let Some(client_info) = meta.get(CLIENT_INFO_META) {
        let Some(client_info) = client_info.as_object() else {
            return Err("malformed MCP client info");
        };
        if client_info.get("name").and_then(Value::as_str).is_none()
            || client_info.get("version").and_then(Value::as_str).is_none()
        {
            return Err("malformed MCP client info");
        }
    }
    Ok(())
}

fn stamp_server_info(result: &mut Value, name: &str, version: &str) {
    let Some(result) = result.as_object_mut() else {
        return;
    };
    let meta = result.entry("_meta").or_insert_with(|| Value::Object(Map::new()));
    let Some(meta) = meta.as_object_mut() else {
        return;
    };
    meta.insert(SERVER_INFO_META.to_owned(), json!({"name": name, "version": version}));
}
