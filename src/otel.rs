// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! OpenTelemetry instrumentation for the CLI dispatch path.
//!
//! This module is compiled only when the `otel` feature is enabled. It installs
//! a tracing layer backed by an SDK tracer provider. Export configuration remains
//! the responsibility of the consuming binary.

use opentelemetry::trace::TracerProvider as _;
use opentelemetry_sdk::trace::TracerProvider;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// Instrumentation scope used for dispatch spans.
pub const TRACER_NAME: &str = "clap-noun-verb";

/// Initialize a tracer provider and best-effort global tracing subscriber.
#[must_use]
pub fn init_tracer(service_name: &str) -> TracerProvider {
    let _service_name = service_name;
    let provider = TracerProvider::builder().build();
    let tracer = provider.tracer(TRACER_NAME);
    let layer = tracing_opentelemetry::layer().with_tracer(tracer);
    let _installed = tracing_subscriber::registry().with(layer).try_init();
    provider
}

/// Flush and shut down the tracer provider.
pub fn shutdown(provider: TracerProvider) {
    let _result = provider.shutdown();
}
