// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! OpenTelemetry instrumentation for the CLI dispatch path.
//!
//! This module is only compiled when the `otel` feature is enabled. It wires
//! `tracing` spans emitted by the registry dispatch path (`run`/`route`) into an
//! OpenTelemetry tracer provider via `tracing-opentelemetry`, following the
//! otel-weaver convention of a single named tracer per binary.
//!
//! ```no_run
//! # #[cfg(feature = "otel")]
//! # fn demo() {
//! let provider = clap_noun_verb::otel::init_tracer("my-cli");
//! // ... run the CLI; dispatch spans are now exported ...
//! clap_noun_verb::otel::shutdown(provider);
//! # }
//! ```

use opentelemetry::trace::TracerProvider as _;
use opentelemetry_sdk::trace::TracerProvider;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// The instrumentation scope name used for all dispatch spans.
pub const TRACER_NAME: &str = "clap-noun-verb";

/// Initialize an OpenTelemetry tracer provider and install a `tracing`
/// subscriber that forwards CLI dispatch spans to it.
///
/// The returned [`TracerProvider`] must be kept alive for the duration of
/// the CLI run and flushed via [`shutdown`] before the process exits so that
/// in-flight spans are exported.
pub fn init_tracer(service_name: &str) -> TracerProvider {
    let _ = service_name; // reserved for resource attributes wiring
    let provider = TracerProvider::builder().build();
    let tracer = provider.tracer(TRACER_NAME);

    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);
    // Best-effort install; ignore error if a global subscriber already exists.
    let _ = tracing_subscriber::registry().with(otel_layer).try_init();

    provider
}

/// Flush and shut down the tracer provider, exporting any buffered spans.
pub fn shutdown(provider: TracerProvider) {
    let _ = provider.shutdown();
}
