//! Deterministic Kubernetes projection for a deployed CLI service.
//!
//! Rendering is CONSTRUCT-only: this module does not contact Kubernetes and
//! does not contain credentials, clients, discovery, apply, delete, or rollout
//! operations.

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KubernetesConfig {
    pub name: String,
    pub namespace: Option<String>,
    pub image: String,
    pub replicas: u32,
    pub port: u16,
    pub command: Vec<String>,
    pub args: Vec<String>,
    pub env: BTreeMap<String, String>,
    pub service_account_name: Option<String>,
    pub read_only_root_filesystem: bool,
    pub run_as_non_root: bool,
}

impl KubernetesConfig {
    #[must_use]
    pub fn new(name: impl Into<String>, image: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            namespace: None,
            image: image.into(),
            replicas: 1,
            port: 8080,
            command: Vec::new(),
            args: Vec::new(),
            env: BTreeMap::new(),
            service_account_name: None,
            read_only_root_filesystem: true,
            run_as_non_root: true,
        }
    }

    /// Render Deployment and ClusterIP Service YAML in stable key order.
    #[must_use]
    pub fn render(&self) -> String {
        let metadata_namespace = self
            .namespace
            .as_ref()
            .map(|namespace| format!("\n  namespace: {namespace}"))
            .unwrap_or_default();
        let pod_namespace = self
            .namespace
            .as_ref()
            .map(|namespace| format!("\n  namespace: {namespace}"))
            .unwrap_or_default();
        let service_account = self
            .service_account_name
            .as_ref()
            .map(|name| format!("\n      serviceAccountName: {name}"))
            .unwrap_or_default();
        let command = yaml_inline_array("command", &self.command);
        let args = yaml_inline_array("args", &self.args);
        let env = if self.env.is_empty() {
            String::new()
        } else {
            let values = self
                .env
                .iter()
                .map(|(name, value)| {
                    format!("\n        - name: {}\n          value: \"{}\"", name, escape(value))
                })
                .collect::<String>();
            format!("\n        env:{values}")
        };
        format!(
            "apiVersion: apps/v1
kind: Deployment
metadata:
  name: {name}{metadata_namespace}
  labels:
    app.kubernetes.io/name: {name}
spec:
  replicas: {replicas}
  selector:
    matchLabels:
      app.kubernetes.io/name: {name}
  template:
    metadata:
      labels:
        app.kubernetes.io/name: {name}
    spec:{service_account}
      containers:
      - name: {name}
        image: {image}
        imagePullPolicy: IfNotPresent
        ports:
        - name: http
          containerPort: {port}{command}{args}{env}
        securityContext:
          allowPrivilegeEscalation: false
          readOnlyRootFilesystem: {read_only}
          runAsNonRoot: {non_root}
          capabilities:
            drop: [\"ALL\"]
        readinessProbe:
          httpGet:
            path: /readyz
            port: http
        livenessProbe:
          httpGet:
            path: /healthz
            port: http
---
apiVersion: v1
kind: Service
metadata:
  name: {name}{pod_namespace}
spec:
  type: ClusterIP
  selector:
    app.kubernetes.io/name: {name}
  ports:
  - name: http
    port: {port}
    targetPort: http
",
            name = self.name,
            replicas = self.replicas,
            image = self.image,
            port = self.port,
            read_only = self.read_only_root_filesystem,
            non_root = self.run_as_non_root,
        )
    }
}

fn yaml_inline_array(name: &str, values: &[String]) -> String {
    if values.is_empty() {
        return String::new();
    }
    let values =
        values.iter().map(|value| format!("\"{}\"", escape(value))).collect::<Vec<_>>().join(", ");
    format!("\n        {name}: [{values}]")
}

fn escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
