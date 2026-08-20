//! Small HTTP/1.1 serving adapter with no web-framework dependency.
//!
//! This surface is intentionally narrow: health, schema discovery, tool listing,
//! and admitted tool invocation. Put TLS/authentication/rate limiting at an
//! ingress, service mesh, or embedding application boundary.

use crate::{AdmissionPolicy, AdmitValidated, CliSchema, Executor, Gateway, GatewayError};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use thiserror::Error;

const MAX_REQUEST_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HttpConfig {
    pub bind: String,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self { bind: "0.0.0.0:8080".to_owned() }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    pub status: u16,
    pub content_type: &'static str,
    pub body: String,
}

pub struct HttpServer<E, P = AdmitValidated> {
    schema: CliSchema,
    gateway: Gateway<E, P>,
}

impl<E> HttpServer<E, AdmitValidated> {
    #[must_use]
    pub fn new(schema: CliSchema, executor: E) -> Self {
        let subject = schema.name.clone();
        Self { schema, gateway: Gateway::new(subject, executor, AdmitValidated) }
    }
}

impl<E, P> HttpServer<E, P> {
    #[must_use]
    pub fn with_policy(schema: CliSchema, executor: E, policy: P) -> Self {
        let subject = schema.name.clone();
        Self { schema, gateway: Gateway::new(subject, executor, policy) }
    }
}

#[derive(Debug, Error)]
pub enum HttpError {
    #[error("HTTP I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("executor failed: {0}")]
    Execution(String),
    #[error("request exceeded {MAX_REQUEST_BYTES} bytes")]
    RequestTooLarge,
}

impl<E, P> HttpServer<E, P>
where
    E: Executor,
    P: AdmissionPolicy,
{
    /// Handle a decoded HTTP request without opening a socket.
    pub fn handle(&self, method: &str, path: &str, body: &[u8]) -> Result<HttpResponse, HttpError> {
        let response = match (method, path) {
            ("GET", "/healthz") | ("GET", "/readyz") => response(200, json!({"status": "ok"})),
            ("GET", "/schema") => response(200, json!(self.schema)),
            ("GET", "/tools") => response(200, json!({"tools": self.schema.tools()})),
            ("POST", "/invoke") => self.invoke(body)?,
            _ => response(404, json!({"error": "not_found"})),
        };
        Ok(response)
    }

    /// Serve HTTP/1.1 until the listener is closed by the embedding process.
    pub fn serve(&self, listener: TcpListener) -> Result<(), HttpError> {
        for stream in listener.incoming() {
            self.serve_stream(stream?)?;
        }
        Ok(())
    }

    fn serve_stream(&self, mut stream: TcpStream) -> Result<(), HttpError> {
        let request = read_request(&mut stream)?;
        let response = self.handle(&request.method, &request.path, &request.body)?;
        write_response(&mut stream, &response)?;
        Ok(())
    }

    fn invoke(&self, body: &[u8]) -> Result<HttpResponse, HttpError> {
        let value: Value = match serde_json::from_slice(body) {
            Ok(value) => value,
            Err(parse_error) => {
                return Ok(response(
                    400,
                    json!({"error": "invalid_json", "message": parse_error.to_string()}),
                ))
            }
        };
        let Some(tool) = value.get("tool").and_then(Value::as_str) else {
            return Ok(response(400, json!({"error": "missing_tool"})));
        };
        let arguments =
            value.get("arguments").and_then(Value::as_object).cloned().unwrap_or_else(Map::new);
        let invocation = match self.schema.build_invocation(tool, &arguments) {
            Ok(invocation) => invocation,
            Err(build_error) => {
                return Ok(response(
                    422,
                    json!({"error": "refused", "message": build_error.to_string()}),
                ))
            }
        };
        let record = match self.gateway.execute(invocation) {
            Ok(record) => record,
            Err(GatewayError::Refused(reason)) => {
                return Ok(response(422, json!({"error": "refused", "message": reason})))
            }
            Err(gateway_error) => return Err(HttpError::Execution(gateway_error.to_string())),
        };
        let status = if record.execution.success() { 200 } else { 422 };
        Ok(response(status, json!(record)))
    }
}

#[derive(Debug)]
struct Request {
    method: String,
    path: String,
    body: Vec<u8>,
}

fn read_request(stream: &mut TcpStream) -> Result<Request, HttpError> {
    let mut bytes = Vec::new();
    let mut buffer = [0_u8; 4096];
    let header_end = loop {
        let read = stream.read(&mut buffer)?;
        if read == 0 {
            return Err(
                std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "request closed").into()
            );
        }
        bytes.extend_from_slice(&buffer[..read]);
        if bytes.len() > MAX_REQUEST_BYTES {
            return Err(HttpError::RequestTooLarge);
        }
        if let Some(position) = find_header_end(&bytes) {
            break position;
        }
    };

    let headers = String::from_utf8_lossy(&bytes[..header_end]);
    let mut lines = headers.split("\r\n");
    let request_line = lines.next().unwrap_or_default();
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default().to_owned();
    let path = parts.next().unwrap_or_default().to_owned();
    let content_length = lines
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("content-length")
                .then(|| value.trim().parse::<usize>().ok())
                .flatten()
        })
        .unwrap_or(0);
    if header_end + 4 + content_length > MAX_REQUEST_BYTES {
        return Err(HttpError::RequestTooLarge);
    }

    let body_start = header_end + 4;
    while bytes.len() < body_start + content_length {
        let read = stream.read(&mut buffer)?;
        if read == 0 {
            return Err(
                std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "request body truncated")
                    .into(),
            );
        }
        bytes.extend_from_slice(&buffer[..read]);
        if bytes.len() > MAX_REQUEST_BYTES {
            return Err(HttpError::RequestTooLarge);
        }
    }
    let body_end = body_start + content_length;
    Ok(Request { method, path, body: bytes[body_start..body_end].to_vec() })
}

fn find_header_end(bytes: &[u8]) -> Option<usize> {
    bytes.windows(4).position(|window| window == b"\r\n\r\n")
}

fn response(status: u16, body: Value) -> HttpResponse {
    HttpResponse { status, content_type: "application/json", body: body.to_string() }
}

fn write_response(stream: &mut TcpStream, response: &HttpResponse) -> Result<(), std::io::Error> {
    let reason = match response.status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        422 => "Unprocessable Entity",
        _ => "Error",
    };
    write!(
        stream,
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        response.status,
        reason,
        response.content_type,
        response.body.len(),
        response.body
    )?;
    stream.flush()
}
