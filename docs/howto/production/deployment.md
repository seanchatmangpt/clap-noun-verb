# How-To: Production Deployment

**Problem:** You need to deploy a clap-noun-verb CLI to production with reliability, monitoring, and rollback capabilities.

**Solution:** Use a multi-stage deployment pipeline with Docker, health checks, and gradual rollout.

---

## Prerequisites

- clap-noun-verb CLI built with release optimizations
- Docker installed
- Container registry access (Docker Hub, GHCR, ECR)
- Deployment target (Kubernetes, ECS, VMs)

---

## Quick Start (5 Minutes)

```bash
# 1. Build optimized binary
cargo build --release

# 2. Create Docker image
docker build -t my-cli:v1.0.0 .

# 3. Test locally
docker run --rm my-cli:v1.0.0 --version

# 4. Push to registry
docker push ghcr.io/user/my-cli:v1.0.0

# 5. Deploy to production
kubectl apply -f deployment.yaml
```

---

## Step 1: Optimize Release Build

### Cargo.toml Configuration

```toml
[profile.release]
opt-level = 3           # Maximum optimization
lto = true              # Link-time optimization
codegen-units = 1       # Better optimization, slower build
strip = true            # Strip debug symbols
panic = "abort"         # Smaller binary

[profile.release.package."*"]
opt-level = 3           # Optimize dependencies too
```

### Build Script

```bash
#!/bin/bash
# scripts/build-release.sh

set -euo pipefail

echo "Building release binary..."
cargo build --release

echo "Checking binary size..."
ls -lh target/release/my-cli

echo "Running tests..."
cargo test --release

echo "Running benchmarks..."
cargo bench

echo "✓ Release build complete"
```

---

## Step 2: Create Production Dockerfile

### Multi-Stage Dockerfile

```dockerfile
# Build stage
FROM rust:1.75-slim as builder

# Install build dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Cache dependencies (Docker layer caching)
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy source
COPY src ./src

# Build release binary
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates curl && \
    rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 appuser

# Copy binary from builder
COPY --from=builder /app/target/release/my-cli /usr/local/bin/my-cli

# Set ownership
RUN chown appuser:appuser /usr/local/bin/my-cli

# Switch to non-root user
USER appuser

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD /usr/local/bin/my-cli services health || exit 1

# Set entrypoint
ENTRYPOINT ["/usr/local/bin/my-cli"]
CMD ["--help"]
```

**Benefits:**
- ✅ Multi-stage build (small final image ~50MB)
- ✅ Layer caching for faster builds
- ✅ Non-root user for security
- ✅ Health check integration

---

## Step 3: Container Registry

### GitHub Container Registry (GHCR)

```bash
# Login
echo $GITHUB_TOKEN | docker login ghcr.io -u USERNAME --password-stdin

# Tag image
docker tag my-cli:v1.0.0 ghcr.io/user/my-cli:v1.0.0
docker tag my-cli:v1.0.0 ghcr.io/user/my-cli:latest

# Push
docker push ghcr.io/user/my-cli:v1.0.0
docker push ghcr.io/user/my-cli:latest
```

### AWS ECR

```bash
# Login
aws ecr get-login-password --region us-west-2 | \
  docker login --username AWS --password-stdin 123456789.dkr.ecr.us-west-2.amazonaws.com

# Tag
docker tag my-cli:v1.0.0 123456789.dkr.ecr.us-west-2.amazonaws.com/my-cli:v1.0.0

# Push
docker push 123456789.dkr.ecr.us-west-2.amazonaws.com/my-cli:v1.0.0
```

---

## Step 4: Kubernetes Deployment

### Deployment Manifest

```yaml
# deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: my-cli
  namespace: production
  labels:
    app: my-cli
    version: v1.0.0
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  selector:
    matchLabels:
      app: my-cli
  template:
    metadata:
      labels:
        app: my-cli
        version: v1.0.0
    spec:
      containers:
      - name: my-cli
        image: ghcr.io/user/my-cli:v1.0.0
        imagePullPolicy: IfNotPresent

        # Environment variables
        env:
        - name: LOG_LEVEL
          value: "info"
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: db-credentials
              key: url

        # Resource limits
        resources:
          requests:
            memory: "64Mi"
            cpu: "100m"
          limits:
            memory: "128Mi"
            cpu: "500m"

        # Readiness probe
        readinessProbe:
          exec:
            command:
            - /usr/local/bin/my-cli
            - services
            - health
          initialDelaySeconds: 5
          periodSeconds: 10
          timeoutSeconds: 3
          successThreshold: 1
          failureThreshold: 3

        # Liveness probe
        livenessProbe:
          exec:
            command:
            - /usr/local/bin/my-cli
            - services
            - health
          initialDelaySeconds: 15
          periodSeconds: 20
          timeoutSeconds: 3
          successThreshold: 1
          failureThreshold: 3

      # Security context
      securityContext:
        runAsNonRoot: true
        runAsUser: 1000
        fsGroup: 1000
```

### Deploy

```bash
# Apply deployment
kubectl apply -f deployment.yaml

# Check status
kubectl rollout status deployment/my-cli -n production

# View pods
kubectl get pods -n production -l app=my-cli

# View logs
kubectl logs -n production -l app=my-cli --tail=100 -f
```

---

## Step 5: Configuration Management

### ConfigMap

```yaml
# configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: my-cli-config
  namespace: production
data:
  LOG_LEVEL: "info"
  LOG_FORMAT: "json"
  ENVIRONMENT: "production"
  TIMEOUT_SECONDS: "30"
```

### Secret

```yaml
# secret.yaml
apiVersion: v1
kind: Secret
metadata:
  name: db-credentials
  namespace: production
type: Opaque
stringData:
  url: "postgres://user:password@db.prod:5432/mydb"
  api_key: "prod-api-key-xyz"
```

**Create secrets:**
```bash
# From file
kubectl create secret generic db-credentials \
  --from-file=url=./db-url.txt \
  --from-file=api_key=./api-key.txt \
  -n production

# From literal
kubectl create secret generic db-credentials \
  --from-literal=url='postgres://...' \
  --from-literal=api_key='prod-key' \
  -n production
```

---

## Step 6: Rolling Updates

### Update Deployment

```bash
# Update image
kubectl set image deployment/my-cli \
  my-cli=ghcr.io/user/my-cli:v1.1.0 \
  -n production

# Watch rollout
kubectl rollout status deployment/my-cli -n production

# Check history
kubectl rollout history deployment/my-cli -n production
```

### Rollback

```bash
# Rollback to previous version
kubectl rollout undo deployment/my-cli -n production

# Rollback to specific revision
kubectl rollout undo deployment/my-cli --to-revision=2 -n production
```

---

## Step 7: Health Checks

### Implement Health Command

```rust
use clap_noun_verb_macros::verb;
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthStatus {
    status: String,
    checks: Vec<HealthCheck>,
}

#[derive(Serialize)]
pub struct HealthCheck {
    name: String,
    healthy: bool,
    latency_ms: u64,
}

#[verb(
    help = "Health check for orchestration",
    effects = ["reads_state"]
)]
pub fn health() -> Result<HealthStatus, Box<dyn std::error::Error>> {
    let checks = vec![
        check_database()?,
        check_cache()?,
        check_external_api()?,
    ];

    let all_healthy = checks.iter().all(|c| c.healthy);

    Ok(HealthStatus {
        status: if all_healthy { "healthy".to_string() } else { "unhealthy".to_string() },
        checks,
    })
}

fn check_database() -> Result<HealthCheck, Box<dyn std::error::Error>> {
    let start = std::time::Instant::now();
    let healthy = crate::domain::db::ping().is_ok();
    let latency_ms = start.elapsed().as_millis() as u64;

    Ok(HealthCheck {
        name: "database".to_string(),
        healthy,
        latency_ms,
    })
}
```

---

## Step 8: CI/CD Pipeline

### GitHub Actions

```yaml
# .github/workflows/deploy.yml
name: Deploy to Production

on:
  push:
    tags:
      - 'v*'

jobs:
  build-and-deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Rust
        uses: actions-rust-lang/setup-rust-toolchain@v1

      - name: Run tests
        run: cargo test --release

      - name: Build release binary
        run: cargo build --release

      - name: Login to GHCR
        uses: docker/login-action@v3
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Build and push Docker image
        uses: docker/build-push-action@v5
        with:
          context: .
          push: true
          tags: |
            ghcr.io/${{ github.repository }}:${{ github.ref_name }}
            ghcr.io/${{ github.repository }}:latest

      - name: Deploy to Kubernetes
        uses: azure/k8s-deploy@v4
        with:
          manifests: |
            deployment.yaml
          images: |
            ghcr.io/${{ github.repository }}:${{ github.ref_name }}
          kubectl-version: 'latest'
```

---

## Step 9: Monitoring and Alerts

### Prometheus Metrics

```yaml
# servicemonitor.yaml
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: my-cli
  namespace: production
spec:
  selector:
    matchLabels:
      app: my-cli
  endpoints:
  - port: metrics
    interval: 30s
```

### Grafana Dashboard

Create dashboard with:
- Pod restart count
- Health check success rate
- Command execution latency
- Error rate

---

## Step 10: Disaster Recovery

### Backup Strategy

```bash
# Backup configuration
kubectl get configmap my-cli-config -n production -o yaml > backup/configmap.yaml
kubectl get secret db-credentials -n production -o yaml > backup/secret.yaml
kubectl get deployment my-cli -n production -o yaml > backup/deployment.yaml
```

### Restore

```bash
# Restore from backup
kubectl apply -f backup/configmap.yaml
kubectl apply -f backup/secret.yaml
kubectl apply -f backup/deployment.yaml
```

---

## Troubleshooting

### Pod Won't Start

```bash
# Check pod events
kubectl describe pod <pod-name> -n production

# Check logs
kubectl logs <pod-name> -n production --previous

# Check node resources
kubectl top nodes
```

### Health Check Failing

```bash
# Test health check manually
kubectl exec -it <pod-name> -n production -- /usr/local/bin/my-cli services health

# Check dependencies
kubectl exec -it <pod-name> -n production -- curl -v https://database:5432
```

### Image Pull Errors

```bash
# Check image pull secret
kubectl get secrets -n production

# Create image pull secret
kubectl create secret docker-registry ghcr-secret \
  --docker-server=ghcr.io \
  --docker-username=<username> \
  --docker-password=<token> \
  -n production

# Add to deployment
spec:
  imagePullSecrets:
  - name: ghcr-secret
```

---

## Best Practices

✅ **Use multi-stage Docker builds** - Smaller images, faster deploys
✅ **Implement health checks** - Enable zero-downtime rolling updates
✅ **Set resource limits** - Prevent resource exhaustion
✅ **Use non-root users** - Security best practice
✅ **Version images** - Never use `:latest` in production
✅ **Automate deployments** - CI/CD pipeline for consistency
✅ **Monitor everything** - Metrics, logs, traces
✅ **Test rollbacks** - Practice disaster recovery

---

## Next Steps

- **[How-To: Production Monitoring](monitoring.md)** - Observability and alerting
- **[How-To: Production Configuration](configuration.md)** - Configuration management
- **[How-To: Production Security](security.md)** - Security hardening

---

*Part of the [clap-noun-verb How-To Guides](../README.md) - Problem-solving documentation*
