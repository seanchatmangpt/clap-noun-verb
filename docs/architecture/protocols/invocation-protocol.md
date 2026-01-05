# CLI Invocation Protocol

## Overview

After discovering capabilities via SPARQL, CLIs invoke each other using a type-safe gRPC-based protocol with cryptographic capability verification.

## Protocol Architecture

```
┌─────────────────────────────────────────────────────────┐
│ Invocation Flow                                         │
│                                                         │
│  CLI A (Invoker)              CLI B (Provider)          │
│  ┌──────────────┐             ┌──────────────┐         │
│  │              │             │              │         │
│  │ 1. Discover  │────SPARQL──►│ RDF Ontology │         │
│  │              │◄────────────┤              │         │
│  │              │             │              │         │
│  │ 2. Validate  │             │              │         │
│  │    Types     │             │              │         │
│  │              │             │              │         │
│  │ 3. Generate  │             │              │         │
│  │    gRPC      │             │              │         │
│  │    Client    │             │              │         │
│  │              │             │              │         │
│  │ 4. Sign      │             │              │         │
│  │    Request   │             │              │         │
│  │              │             │              │         │
│  │ 5. Invoke    │────gRPC────►│ 6. Verify    │         │
│  │              │   (HTTP/2)  │    Signature │         │
│  │              │             │              │         │
│  │              │             │ 7. Validate  │         │
│  │              │             │    Capability│         │
│  │              │             │              │         │
│  │              │             │ 8. Execute   │         │
│  │              │             │    Command   │         │
│  │              │             │              │         │
│  │ 10. Verify   │◄────────────│ 9. Sign      │         │
│  │     Result   │             │    Response  │         │
│  └──────────────┘             └──────────────┘         │
└─────────────────────────────────────────────────────────┘
```

## Message Format

### 1. RPC Request Structure

```protobuf
syntax = "proto3";

package clicap.v1;

// Auto-generated from RDF ontology
service CLIService {
  rpc InvokeCommand(CommandRequest) returns (CommandResponse);
  rpc StreamCommand(CommandRequest) returns (stream CommandResponse);
}

message CommandRequest {
  // Command metadata
  string command_id = 1;          // Unique request ID
  string command_name = 2;        // e.g., "convert"
  string cli_version = 3;         // Semantic version

  // Invocation metadata
  string invoker_uri = 4;         // URI of invoking CLI
  int64 timestamp = 5;            // Unix timestamp
  string trace_id = 6;            // Distributed tracing ID

  // Capability token (JWT)
  string capability_token = 7;

  // Command arguments (type-safe)
  map<string, ArgumentValue> arguments = 8;

  // Request signature
  Signature signature = 9;
}

message ArgumentValue {
  oneof value {
    string string_value = 1;
    int64 int_value = 2;
    double float_value = 3;
    bool bool_value = 4;
    bytes bytes_value = 5;
    ArgumentList list_value = 6;
    ArgumentMap map_value = 7;
  }
}

message ArgumentList {
  repeated ArgumentValue values = 1;
}

message ArgumentMap {
  map<string, ArgumentValue> values = 1;
}

message Signature {
  string algorithm = 1;           // e.g., "Ed25519"
  bytes public_key = 2;           // Invoker's public key
  bytes signature = 3;            // Signature over canonical request
}

message CommandResponse {
  // Response metadata
  string command_id = 1;          // Matches request
  ResponseStatus status = 2;
  int64 timestamp = 3;

  // Response data (type-safe)
  ArgumentValue result = 4;

  // Response signature
  Signature signature = 5;
}

enum ResponseStatus {
  SUCCESS = 0;
  INVALID_ARGUMENTS = 1;
  UNAUTHORIZED = 2;
  INTERNAL_ERROR = 3;
  TIMEOUT = 4;
}
```

### 2. RDF to Protobuf Code Generation

**Algorithm**:

```rust
fn generate_protobuf_from_rdf(ontology: &Graph) -> Result<String, Error> {
    let mut proto = String::from("syntax = \"proto3\";\n\n");

    // 1. Extract service definition
    let cli_uri = extract_cli_uri(ontology)?;
    let service_name = uri_to_service_name(&cli_uri);

    proto.push_str(&format!("service {} {{\n", service_name));

    // 2. For each command, generate RPC method
    for command in extract_commands(ontology) {
        let command_name = get_command_name(&command)?;
        let input_type = get_input_type(&command)?;
        let output_type = get_output_type(&command)?;

        proto.push_str(&format!(
            "  rpc {}({}) returns ({});\n",
            to_pascal_case(&command_name),
            rdf_type_to_proto(&input_type),
            rdf_type_to_proto(&output_type)
        ));
    }

    proto.push_str("}\n\n");

    // 3. Generate message types from RDF types
    for rdf_type in extract_types(ontology) {
        proto.push_str(&generate_message_type(&rdf_type)?);
    }

    Ok(proto)
}

fn rdf_type_to_proto(rdf_type: &str) -> String {
    match rdf_type {
        "xsd:string" => "string",
        "xsd:integer" => "int64",
        "xsd:float" => "double",
        "xsd:boolean" => "bool",
        "clicap:ImageFile" => "ImageFileMessage",
        custom => to_pascal_case(custom) + "Message",
    }.to_string()
}
```

**Example Transformation**:

```turtle
# RDF Input
:ConvertCommand a clicap:Command ;
    clicap:commandName "convert" ;
    clicap:accepts clicap:ImageFile ;
    clicap:produces clicap:ImageFile ;
    clicap:hasArgument [
        a clicap:Option ;
        clicap:argumentName "format" ;
        clicap:argumentType clicap:ImageFormat
    ] .
```

```protobuf
// Generated Protobuf
service ImageConverterService {
  rpc Convert(ConvertRequest) returns (ConvertResponse);
}

message ConvertRequest {
  ImageFile input = 1;
  ImageFormat format = 2;
}

message ConvertResponse {
  ImageFile output = 1;
}

message ImageFile {
  bytes data = 1;
  string mime_type = 2;
  int64 size_bytes = 3;
}

enum ImageFormat {
  PNG = 0;
  JPEG = 1;
  WEBP = 2;
}
```

## Type Safety Verification

### Pre-Invocation Type Checking

```rust
async fn invoke_command_safe(
    target_cli: &str,
    command: &str,
    args: HashMap<String, Value>,
) -> Result<Value, InvocationError> {
    // 1. Fetch RDF ontology for target CLI
    let ontology = fetch_ontology(target_cli).await?;

    // 2. Extract command type signature
    let signature = extract_type_signature(&ontology, command)?;

    // 3. Validate argument types match signature
    validate_argument_types(&signature, &args)?;

    // 4. Generate gRPC client from RDF
    let client = generate_grpc_client(&ontology).await?;

    // 5. Invoke with validated arguments
    let response = client.invoke(command, args).await?;

    // 6. Validate response type matches signature
    validate_response_type(&signature, &response)?;

    Ok(response)
}

fn validate_argument_types(
    signature: &TypeSignature,
    args: &HashMap<String, Value>,
) -> Result<(), TypeError> {
    for (arg_name, expected_type) in &signature.inputs {
        let actual_value = args.get(arg_name)
            .ok_or(TypeError::MissingArgument(arg_name.clone()))?;

        let actual_type = infer_type(actual_value);

        if !is_compatible(&actual_type, expected_type) {
            return Err(TypeError::IncompatibleTypes {
                argument: arg_name.clone(),
                expected: expected_type.clone(),
                actual: actual_type,
            });
        }
    }

    Ok(())
}

fn is_compatible(actual: &Type, expected: &Type) -> bool {
    match (actual, expected) {
        // Exact match
        (a, b) if a == b => true,

        // Subtyping: PNG ⊆ ImageFile
        (Type::Named(a), Type::Named(b)) => is_subtype_of(a, b),

        // Sum types: A ∈ (A | B | C)
        (actual, Type::Sum(alternatives)) => {
            alternatives.iter().any(|alt| is_compatible(actual, alt))
        }

        // Collections: List<PNG> ⊆ List<ImageFile>
        (Type::List(a), Type::List(b)) => is_compatible(a, b),

        _ => false,
    }
}
```

### Runtime Type Validation

Even with compile-time checks, runtime validation is required for:
- Network deserialization errors
- Schema evolution mismatches
- Malicious payloads

```rust
fn validate_runtime_type(value: &Value, expected: &Type, ontology: &Graph) -> Result<()> {
    match (value, expected) {
        (Value::File(file), Type::Named(type_uri)) => {
            // Check MIME type against ontology
            let allowed_mimes = query_allowed_mime_types(ontology, type_uri)?;

            if !allowed_mimes.contains(&file.mime_type) {
                return Err(TypeError::InvalidMimeType {
                    expected: allowed_mimes,
                    actual: file.mime_type.clone(),
                });
            }

            // Check file size constraints (SHACL)
            let max_size = query_max_file_size(ontology, type_uri)?;
            if file.size_bytes > max_size {
                return Err(TypeError::FileTooLarge {
                    max: max_size,
                    actual: file.size_bytes,
                });
            }

            Ok(())
        }
        // ... other type validations
    }
}
```

## Security: Request Signing & Verification

### 1. Request Signing (Invoker Side)

```rust
use ed25519_dalek::{Keypair, Signature, Signer};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct CanonicalRequest {
    command_id: String,
    command_name: String,
    cli_version: String,
    invoker_uri: String,
    timestamp: i64,
    arguments: BTreeMap<String, ArgumentValue>, // Sorted for canonical form
}

fn sign_request(request: &CommandRequest, keypair: &Keypair) -> Signature {
    // 1. Construct canonical form (deterministic serialization)
    let canonical = CanonicalRequest {
        command_id: request.command_id.clone(),
        command_name: request.command_name.clone(),
        cli_version: request.cli_version.clone(),
        invoker_uri: request.invoker_uri.clone(),
        timestamp: request.timestamp,
        arguments: request.arguments.clone().into_iter().collect(),
    };

    // 2. Serialize to canonical JSON (sorted keys)
    let json = serde_json::to_string(&canonical).unwrap();

    // 3. Sign with Ed25519
    keypair.sign(json.as_bytes())
}
```

### 2. Request Verification (Provider Side)

```rust
use ed25519_dalek::{PublicKey, Signature, Verifier};

fn verify_request_signature(
    request: &CommandRequest,
    public_key: &PublicKey,
) -> Result<(), SignatureError> {
    // 1. Reconstruct canonical form
    let canonical = CanonicalRequest {
        command_id: request.command_id.clone(),
        command_name: request.command_name.clone(),
        cli_version: request.cli_version.clone(),
        invoker_uri: request.invoker_uri.clone(),
        timestamp: request.timestamp,
        arguments: request.arguments.clone().into_iter().collect(),
    };

    let json = serde_json::to_string(&canonical).unwrap();

    // 2. Verify signature
    let signature = Signature::from_bytes(&request.signature.signature)?;
    public_key.verify(json.as_bytes(), &signature)?;

    // 3. Check timestamp freshness (prevent replay attacks)
    let now = current_timestamp();
    let age = now - request.timestamp;

    if age > 300 { // 5 minute window
        return Err(SignatureError::TimestampTooOld);
    }

    if age < -60 { // 1 minute clock skew tolerance
        return Err(SignatureError::TimestampTooNew);
    }

    Ok(())
}
```

### 3. Capability Token Verification

```rust
use jsonwebtoken::{decode, DecodingKey, Validation};

fn verify_capability_token(
    token: &str,
    required_capability: &str,
    public_key: &PublicKey,
) -> Result<(), CapabilityError> {
    // 1. Decode and verify JWT signature
    let token_data = decode::<CapabilityToken>(
        token,
        &DecodingKey::from_ed_pem(public_key.as_bytes())?,
        &Validation::new(jsonwebtoken::Algorithm::EdDSA),
    )?;

    let claims = token_data.claims;

    // 2. Check expiration
    let now = current_timestamp();
    if claims.exp < now {
        return Err(CapabilityError::TokenExpired);
    }

    if claims.nbf > now {
        return Err(CapabilityError::TokenNotYetValid);
    }

    // 3. Check capabilities
    if !claims.cap.contains(&required_capability.to_string()) {
        return Err(CapabilityError::InsufficientPermissions);
    }

    // 4. Check revocation list
    if is_revoked(&claims.jti).await? {
        return Err(CapabilityError::TokenRevoked);
    }

    // 5. Validate constraints (SHACL)
    if let Some(constraints) = claims.constraints {
        validate_constraints(&constraints)?;
    }

    Ok(())
}
```

## Streaming Protocol

For long-running commands, use server-side streaming:

```rust
// Client-side
let mut stream = client.stream_command(request).await?;

while let Some(response) = stream.message().await? {
    match response.status {
        ResponseStatus::Success => {
            println!("Progress: {:?}", response.result);
        }
        ResponseStatus::InternalError => {
            eprintln!("Error: {:?}", response.result);
            break;
        }
        _ => {}
    }
}

// Server-side
async fn stream_command(
    request: CommandRequest,
) -> Result<impl Stream<Item = CommandResponse>, Error> {
    let (tx, rx) = mpsc::channel(100);

    tokio::spawn(async move {
        for i in 0..100 {
            // Simulate long-running work
            tokio::time::sleep(Duration::from_millis(100)).await;

            tx.send(CommandResponse {
                command_id: request.command_id.clone(),
                status: ResponseStatus::Success,
                timestamp: current_timestamp(),
                result: ArgumentValue::from(format!("Progress: {}%", i)),
                signature: sign_response(...),
            }).await.ok();
        }
    });

    Ok(ReceiverStream::new(rx))
}
```

## Error Handling

### Error Response Structure

```protobuf
message ErrorResponse {
  ErrorCode code = 1;
  string message = 2;
  repeated ErrorDetail details = 3;
}

enum ErrorCode {
  INVALID_ARGUMENTS = 0;
  UNAUTHORIZED = 1;
  CAPABILITY_DENIED = 2;
  TYPE_MISMATCH = 3;
  SIGNATURE_INVALID = 4;
  TOKEN_EXPIRED = 5;
  COMMAND_NOT_FOUND = 6;
  INTERNAL_ERROR = 7;
}

message ErrorDetail {
  string field = 1;
  string problem = 2;
  string suggestion = 3;
}
```

### Retry Logic

```rust
async fn invoke_with_retry(
    client: &mut CLIClient,
    request: CommandRequest,
) -> Result<CommandResponse, Error> {
    let mut backoff = Duration::from_millis(100);

    for attempt in 0..5 {
        match client.invoke_command(request.clone()).await {
            Ok(response) => return Ok(response),

            Err(e) if e.is_retriable() => {
                tokio::time::sleep(backoff).await;
                backoff *= 2; // Exponential backoff
            }

            Err(e) => return Err(e), // Non-retriable error
        }
    }

    Err(Error::MaxRetriesExceeded)
}

trait Retriable {
    fn is_retriable(&self) -> bool;
}

impl Retriable for tonic::Status {
    fn is_retriable(&self) -> bool {
        matches!(
            self.code(),
            tonic::Code::Unavailable
                | tonic::Code::DeadlineExceeded
                | tonic::Code::ResourceExhausted
        )
    }
}
```

## Performance Optimizations

### 1. Connection Pooling

```rust
use tonic::transport::Channel;
use std::sync::Arc;
use dashmap::DashMap;

struct ConnectionPool {
    connections: Arc<DashMap<String, Channel>>,
}

impl ConnectionPool {
    async fn get_or_create(&self, endpoint: &str) -> Result<Channel, Error> {
        if let Some(conn) = self.connections.get(endpoint) {
            return Ok(conn.clone());
        }

        let channel = Channel::from_shared(endpoint.to_string())?
            .connect()
            .await?;

        self.connections.insert(endpoint.to_string(), channel.clone());

        Ok(channel)
    }
}
```

### 2. Request Batching

```rust
async fn batch_invoke(
    requests: Vec<CommandRequest>,
) -> Vec<Result<CommandResponse, Error>> {
    futures::future::join_all(
        requests.into_iter().map(|req| invoke_command_safe(req))
    ).await
}
```

### 3. Compression

```protobuf
// Enable gRPC compression
service CLIService {
  option (grpc.compression) = gzip;

  rpc InvokeCommand(CommandRequest) returns (CommandResponse);
}
```

## Audit Trail

Every invocation is logged for security audits:

```rust
#[derive(Serialize)]
struct AuditLogEntry {
    timestamp: i64,
    invoker_uri: String,
    provider_uri: String,
    command_name: String,
    capability_token_id: String,
    request_signature: String,
    response_signature: String,
    duration_ms: u64,
    success: bool,
}

async fn log_invocation(
    request: &CommandRequest,
    response: &CommandResponse,
    duration: Duration,
) {
    let entry = AuditLogEntry {
        timestamp: current_timestamp(),
        invoker_uri: request.invoker_uri.clone(),
        provider_uri: extract_provider_uri(request),
        command_name: request.command_name.clone(),
        capability_token_id: extract_token_id(&request.capability_token),
        request_signature: hex::encode(&request.signature.signature),
        response_signature: hex::encode(&response.signature.signature),
        duration_ms: duration.as_millis() as u64,
        success: response.status == ResponseStatus::Success,
    };

    // Write to immutable append-only log
    append_to_audit_log(&entry).await;
}
```

## Implementation Checklist

- [ ] Implement RDF to Protobuf code generator
- [ ] Implement Ed25519 request signing
- [ ] Implement Ed25519 signature verification
- [ ] Implement JWT capability token validation
- [ ] Implement SHACL constraint validation
- [ ] Implement gRPC client with connection pooling
- [ ] Implement gRPC server with streaming support
- [ ] Implement type safety verification (pre + runtime)
- [ ] Implement exponential backoff retry logic
- [ ] Implement audit logging (append-only)
- [ ] Add integration tests for cross-CLI invocation
- [ ] Add property tests for type compatibility
- [ ] Add performance benchmarks (latency, throughput)
