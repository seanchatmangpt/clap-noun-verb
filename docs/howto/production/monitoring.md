# How-To: Production Monitoring

**Problem:** You need visibility into your deployed CLI's performance, errors, and behavior in production.

**Solution:** Implement comprehensive observability with structured logging, metrics, distributed tracing, and alerting.

---

## Prerequisites

- clap-noun-verb CLI deployed to production
- Access to observability stack (Prometheus, Grafana, Jaeger)
- OpenTelemetry dependencies installed

---

## Quick Start (10 Minutes)

```bash
# 1. Add telemetry dependencies
cargo add tracing tracing-subscriber opentelemetry opentelemetry-otlp

# 2. Initialize telemetry
# See Step 1 below

# 3. Add instrumentation to commands
# See Step 2 below

# 4. Deploy with OTEL collector
# See Step 3 below

# 5. Create Grafana dashboards
# See Step 6 below
```

---

## Step 1: Structured Logging

### Initialize Tracing Subscriber

```rust
// src/main.rs
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize structured logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .json() // JSON output for log aggregation
                .with_target(true)
                .with_file(true)
                .with_line_number(true)
        )
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into())
        )
        .init();

    clap_noun_verb::run()
}
```

### Add Logging to Commands

```rust
use clap_noun_verb_macros::verb;
use tracing::{info, warn, error, instrument};
use serde::Serialize;

#[derive(Serialize)]
pub struct DeploymentResult {
    deployment_id: String,
    status: String,
    duration_ms: u64,
}

#[instrument(
    name = "deploy_command",
    fields(
        environment = %environment,
        region = %region
    )
)]
#[verb(help = "Deploy application")]
pub fn deploy(
    #[arg(help = "Target environment")] environment: String,
    #[arg(help = "AWS region")] region: String,
) -> Result<DeploymentResult, Box<dyn std::error::Error>> {
    info!("Starting deployment");

    let start = std::time::Instant::now();

    // Deploy
    let deployment_id = match crate::domain::deployments::deploy(&environment, &region) {
        Ok(id) => {
            info!(deployment_id = %id, "Deployment successful");
            id
        }
        Err(e) => {
            error!(error = %e, "Deployment failed");
            return Err(e.into());
        }
    };

    let duration_ms = start.elapsed().as_millis() as u64;

    if duration_ms > 5000 {
        warn!(duration_ms, "Deployment took longer than expected");
    }

    Ok(DeploymentResult {
        deployment_id,
        status: "success".to_string(),
        duration_ms,
    })
}
```

**Log output:**
```json
{
  "timestamp": "2025-12-03T18:00:00.123Z",
  "level": "INFO",
  "target": "my_cli::commands",
  "fields": {
    "message": "Starting deployment",
    "environment": "production",
    "region": "us-west-2"
  },
  "span": {
    "name": "deploy_command",
    "environment": "production",
    "region": "us-west-2"
  }
}
```

---

## Step 2: Metrics with OpenTelemetry

### Initialize Metrics

```rust
use opentelemetry::{global, KeyValue};
use opentelemetry_sdk::metrics::{reader::DefaultTemporalitySelector, PeriodicReader};
use opentelemetry_otlp::WithExportConfig;

fn init_metrics() -> Result<(), Box<dyn std::error::Error>> {
    let export_config = opentelemetry_otlp::ExportConfig {
        endpoint: "http://localhost:4317".to_string(),
        ..Default::default()
    };

    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_export_config(export_config);

    let reader = PeriodicReader::builder(exporter, opentelemetry_sdk::runtime::Tokio)
        .with_interval(std::time::Duration::from_secs(30))
        .build();

    let meter_provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
        .with_reader(reader)
        .build();

    global::set_meter_provider(meter_provider);

    Ok(())
}
```

### Add Metrics to Commands

```rust
use opentelemetry::{global, KeyValue};

#[verb(help = "Process order")]
pub fn process_order(
    #[arg] order_id: String,
) -> Result<OrderResult, Box<dyn std::error::Error>> {
    let meter = global::meter("my_cli");

    // Counter: Total orders processed
    let counter = meter
        .u64_counter("orders_processed_total")
        .with_description("Total orders processed")
        .init();

    counter.add(1, &[KeyValue::new("status", "started")]);

    // Histogram: Order processing duration
    let histogram = meter
        .f64_histogram("order_processing_duration_seconds")
        .with_description("Order processing duration in seconds")
        .init();

    let start = std::time::Instant::now();

    // Process order
    let result = match crate::domain::orders::process(&order_id) {
        Ok(order) => {
            counter.add(1, &[KeyValue::new("status", "success")]);
            order
        }
        Err(e) => {
            counter.add(1, &[KeyValue::new("status", "failure")]);
            return Err(e.into());
        }
    };

    histogram.record(start.elapsed().as_secs_f64(), &[]);

    Ok(OrderResult::from(result))
}
```

**Metrics exposed:**
- `orders_processed_total{status="success"}` - Counter
- `orders_processed_total{status="failure"}` - Counter
- `order_processing_duration_seconds` - Histogram

---

## Step 3: Distributed Tracing

### Initialize Tracing

```rust
use opentelemetry::{global, trace::TracerProvider};
use opentelemetry_sdk::trace::TracerProvider as SdkTracerProvider;
use opentelemetry_otlp::WithExportConfig;
use tracing_opentelemetry::OpenTelemetryLayer;

fn init_tracing() -> Result<(), Box<dyn std::error::Error>> {
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:4317")
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)?;

    let provider = tracer.provider().unwrap();

    global::set_tracer_provider(provider);

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing()?;

    let tracer = global::tracer("my_cli");

    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(telemetry)
        .with(tracing_subscriber::fmt::layer())
        .init();

    clap_noun_verb::run()?;

    global::shutdown_tracer_provider();

    Ok(())
}
```

### Add Spans to Commands

```rust
use tracing::{instrument, Span};
use opentelemetry::trace::{TraceContextExt, Tracer};

#[instrument(name = "deploy")]
#[verb(help = "Deploy with distributed tracing")]
pub fn deploy(
    #[arg] environment: String,
) -> Result<DeploymentResult, Box<dyn std::error::Error>> {
    let current_span = Span::current();
    current_span.record("environment", &environment.as_str());

    // Nested span for database operation
    let db_result = {
        let _db_span = tracing::info_span!("database_query").entered();
        crate::domain::db::get_deployment_config(&environment)?
    };

    // Nested span for API call
    let api_result = {
        let _api_span = tracing::info_span!("api_call", api = "deployment-service").entered();
        crate::domain::api::trigger_deployment(&environment)?
    };

    Ok(DeploymentResult {
        deployment_id: api_result.id,
        status: "success".to_string(),
    })
}
```

**Trace visualization (Jaeger):**
```
deploy [100ms]
├── database_query [20ms]
└── api_call [75ms]
```

---

## Step 4: OpenTelemetry Collector

### Collector Configuration

```yaml
# otel-collector-config.yaml
receivers:
  otlp:
    protocols:
      grpc:
        endpoint: 0.0.0.0:4317
      http:
        endpoint: 0.0.0.0:4318

processors:
  batch:
    timeout: 10s
    send_batch_size: 1024

exporters:
  prometheus:
    endpoint: "0.0.0.0:8889"

  jaeger:
    endpoint: jaeger:14250
    tls:
      insecure: true

  logging:
    loglevel: debug

service:
  pipelines:
    traces:
      receivers: [otlp]
      processors: [batch]
      exporters: [jaeger, logging]

    metrics:
      receivers: [otlp]
      processors: [batch]
      exporters: [prometheus, logging]
```

### Deploy Collector (Kubernetes)

```yaml
# otel-collector.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: otel-collector
  namespace: observability
spec:
  replicas: 1
  selector:
    matchLabels:
      app: otel-collector
  template:
    metadata:
      labels:
        app: otel-collector
    spec:
      containers:
      - name: otel-collector
        image: otel/opentelemetry-collector:latest
        args:
        - --config=/conf/otel-collector-config.yaml
        volumeMounts:
        - name: config
          mountPath: /conf
        ports:
        - containerPort: 4317  # OTLP gRPC
        - containerPort: 4318  # OTLP HTTP
        - containerPort: 8889  # Prometheus metrics
      volumes:
      - name: config
        configMap:
          name: otel-collector-config
---
apiVersion: v1
kind: Service
metadata:
  name: otel-collector
  namespace: observability
spec:
  selector:
    app: otel-collector
  ports:
  - name: otlp-grpc
    port: 4317
  - name: otlp-http
    port: 4318
  - name: metrics
    port: 8889
```

---

## Step 5: Prometheus Configuration

### Scrape Config

```yaml
# prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'otel-collector'
    static_configs:
    - targets: ['otel-collector.observability:8889']

  - job_name: 'kubernetes-pods'
    kubernetes_sd_configs:
    - role: pod
      namespaces:
        names:
        - production

    relabel_configs:
    - source_labels: [__meta_kubernetes_pod_label_app]
      action: keep
      regex: my-cli
```

---

## Step 6: Grafana Dashboards

### CLI Performance Dashboard

```json
{
  "dashboard": {
    "title": "CLI Performance",
    "panels": [
      {
        "title": "Command Success Rate",
        "targets": [
          {
            "expr": "rate(orders_processed_total{status=\"success\"}[5m]) / rate(orders_processed_total[5m])"
          }
        ]
      },
      {
        "title": "Command Duration (p95)",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, rate(order_processing_duration_seconds_bucket[5m]))"
          }
        ]
      },
      {
        "title": "Error Rate",
        "targets": [
          {
            "expr": "rate(orders_processed_total{status=\"failure\"}[5m])"
          }
        ]
      },
      {
        "title": "Pod Restarts",
        "targets": [
          {
            "expr": "kube_pod_container_status_restarts_total{pod=~\"my-cli.*\"}"
          }
        ]
      }
    ]
  }
}
```

---

## Step 7: Alerting

### Prometheus Alerts

```yaml
# alerts.yml
groups:
- name: cli_alerts
  interval: 30s
  rules:
  - alert: HighErrorRate
    expr: rate(orders_processed_total{status="failure"}[5m]) > 0.05
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "High error rate detected"
      description: "Error rate is {{ $value }} errors/sec"

  - alert: SlowCommands
    expr: histogram_quantile(0.95, rate(order_processing_duration_seconds_bucket[5m])) > 5
    for: 10m
    labels:
      severity: warning
    annotations:
      summary: "Commands are slow"
      description: "P95 latency is {{ $value }} seconds"

  - alert: PodRestarts
    expr: rate(kube_pod_container_status_restarts_total{pod=~"my-cli.*"}[15m]) > 0
    for: 5m
    labels:
      severity: critical
    annotations:
      summary: "Pod is restarting"
      description: "Pod {{ $labels.pod }} is restarting"
```

### Alertmanager Configuration

```yaml
# alertmanager.yml
global:
  resolve_timeout: 5m

route:
  group_by: ['alertname', 'cluster']
  group_wait: 10s
  group_interval: 10s
  repeat_interval: 12h
  receiver: 'slack'

receivers:
- name: 'slack'
  slack_configs:
  - api_url: 'https://hooks.slack.com/services/...'
    channel: '#alerts'
    title: 'CLI Alert'
    text: '{{ range .Alerts }}{{ .Annotations.description }}{{ end }}'
```

---

## Step 8: Log Aggregation

### Fluentd Configuration

```yaml
# fluentd-configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: fluentd-config
  namespace: observability
data:
  fluent.conf: |
    <source>
      @type tail
      path /var/log/containers/my-cli-*.log
      pos_file /var/log/fluentd-my-cli.log.pos
      tag kubernetes.*
      <parse>
        @type json
        time_key timestamp
        time_format %Y-%m-%dT%H:%M:%S.%NZ
      </parse>
    </source>

    <filter kubernetes.**>
      @type kubernetes_metadata
    </filter>

    <match kubernetes.**>
      @type elasticsearch
      host elasticsearch.observability
      port 9200
      logstash_format true
      logstash_prefix my-cli
    </match>
```

---

## Step 9: Error Tracking

### Sentry Integration

```rust
use sentry;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = sentry::init((
        "https://your-dsn@sentry.io/project-id",
        sentry::ClientOptions {
            release: Some(env!("CARGO_PKG_VERSION").into()),
            environment: Some(std::env::var("ENVIRONMENT").unwrap_or("production".to_string()).into()),
            ..Default::default()
        },
    ));

    clap_noun_verb::run()
}

#[verb]
pub fn risky_operation() -> Result<Output, Box<dyn std::error::Error>> {
    match do_risky_thing() {
        Ok(result) => Ok(result),
        Err(e) => {
            sentry::capture_error(&e);
            Err(e.into())
        }
    }
}
```

---

## Best Practices

✅ **Use structured logging (JSON)** - Easy to parse and aggregate
✅ **Add context to logs** - Include request IDs, user IDs, etc.
✅ **Instrument critical paths** - Measure what matters
✅ **Set up alerting** - Know when things break
✅ **Monitor error rates** - Track failures over time
✅ **Track latency percentiles** - P50, P95, P99
✅ **Use distributed tracing** - Understand request flow
✅ **Keep metrics cardinality low** - Avoid label explosion

---

## Troubleshooting

### Missing Metrics

```bash
# Check if OTEL collector is receiving data
kubectl logs -n observability -l app=otel-collector

# Check Prometheus targets
kubectl port-forward -n observability svc/prometheus 9090:9090
# Visit http://localhost:9090/targets
```

### High Cardinality

```bash
# Check metric cardinality
curl http://prometheus:9090/api/v1/status/tsdb | jq '.data.seriesCountByMetricName'
```

---

## Next Steps

- **[How-To: Production Configuration](configuration.md)** - Configuration management
- **[How-To: Production Security](security.md)** - Security hardening
- **[Tutorial 06: Autonomic Features](../../tutorial/06-autonomic-features.md)** - Execution receipts

---

*Part of the [clap-noun-verb How-To Guides](../README.md) - Problem-solving documentation*
