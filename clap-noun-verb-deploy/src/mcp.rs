//! MCP stdio adapter for a projected CLI.
//!
//! The server implements the core JSON-RPC methods required for CLI tools:
//! `initialize`, `tools/list`, and `tools/call`. Requests are admitted through
//! [`CliSchema`](crate::CliSchema) and [`Gateway`](crate::Gateway) before an
//! executor can receive an invocation.

use crate::{AdmissionPolicy, AdmitValidated, CliSchema, Executor, Gateway, GatewayError};
use serde_json::{json, Value};
use std::io::{BufRead, Write};
use thiserror::Error;

pub const MCP_PROTOCOL_VERSION: &str = "2025-06-18";

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

    /// Handle one MCP JSON-RPC request. Notifications intentionally return `None`.
    pub fn handle(&self, request: &Value) -> Result<Option<Value>, McpError> {
        let Some(id) = request.get("id").cloned() else {
            return Ok(None);
        };
        if request.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
            return Ok(Some(error(id, -32600, "Invalid Request")));
        }
        let Some(method) = request.get("method").and_then(Value::as_str) else {
            return Ok(Some(error(id, -32600, "Invalid Request")));
        };

        let result = match method {
            "initialize" => json!({
                "protocolVersion": MCP_PROTOCOL_VERSION,
                "capabilities": {"tools": {"listChanged": false}},
                "serverInfo": {"name": self.name, "version": self.version}
            }),
            "ping" => json!({}),
            "tools/list" => json!({
                "tools": self.schema.tools().into_iter().map(|tool| json!({
                    "name": tool.name,
                    "description": tool.description.unwrap_or_default(),
                    "inputSchema": tool.input_schema
                })).collect::<Vec<_>>()
            }),
            "tools/call" => return self.call_tool(id, request),
            _ => return Ok(Some(error(id, -32601, "Method not found"))),
        };

        Ok(Some(json!({"jsonrpc": "2.0", "id": id, "result": result})))
    }

    fn call_tool(&self, id: Value, request: &Value) -> Result<Option<Value>, McpError> {
        let params = request.get("params").and_then(Value::as_object);
        let Some(params) = params else {
            return Ok(Some(error(id, -32602, "Invalid params")));
        };
        let Some(name) = params.get("name").and_then(Value::as_str) else {
            return Ok(Some(error(id, -32602, "Missing tool name")));
        };
        let arguments = params
            .get("arguments")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();

        let invocation = match self.schema.build_invocation(name, &arguments) {
            Ok(invocation) => invocation,
            Err(build_error) => return Ok(Some(error(id, -32602, &build_error.to_string()))),
        };
        let record = match self.gateway.execute(invocation) {
            Ok(record) => record,
            Err(GatewayError::Refused(reason)) => {
                return Ok(Some(error(id, -32602, &format!("refused: {reason}"))))
            }
            Err(gateway_error) => return Err(McpError::Execution(gateway_error.to_string())),
        };
        let is_error = !record.execution.success();
        let text = if record.execution.stdout.is_empty() {
            record.execution.stderr.clone()
        } else {
            record.execution.stdout.clone()
        };
        Ok(Some(json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "content": [{"type": "text", "text": text}],
                "isError": is_error,
                "_meta": {
                    "clapNounVerbDeploy": {
                        "subject": record.subject,
                        "fingerprint": record.fingerprint
                    }
                }
            }
        })))
    }
}

fn error(id: Value, code: i32, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {"code": code, "message": message}
    })
}
