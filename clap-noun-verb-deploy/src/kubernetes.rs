//! Deterministic Kubernetes projection for a deployed CLI service.
//!
//! Rendering is CONSTRUCT-only: this module does not contact Kubernetes and
//! does not contain credentials, clients, discovery, apply, delete, or rollout
//! operations.

use std::collections::BTreeMap;
use thiserror::Error;

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

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum KubernetesRenderError {
    #[error("invalid Kubernetes field '{field}': {reason}")]
    InvalidField { field: &'static str, reason: &'static str },
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

    /// Render Deployment and ClusterIP Service YAML in stable key order after
    /// validating every scalar that can affect YAML structure or Kubernetes identity.
    pub fn render(&self) -> Result<String, KubernetesRenderError> {
        validate_dns_label(&self.name, "name")?;
        if let Some(namespace) = &self.namespace {
            validate_dns_label(namespace, "namespace")?;
        }
        if let Some(service_account_name) = &self.service_account_name {
            validate_dns_label(service_account_name, "service_account_name")?;
        }
        validate_image(&self.image)?;
        for name in self.env.keys() {
            validate_env_name(name)?;
        }

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
                    format!("\n        - name: {name}\n          value: {}", yaml_string(value))
                })
                .collect::<String>();
            format!("\n        env:{values}")
        };
        // A read-only root filesystem makes `/tmp` unwritable by default,
        // which breaks ephemeral scratch/log writes -- including the OCEL
        // 2.0 fallback path (`clap_noun_verb::ocel::fallback_path()`) -- for
        // any pod using this hardening default without hand-authoring a
        // volume. Mount a minimal `emptyDir` at `/tmp` automatically in that
        // case; leave the filesystem untouched otherwise so an operator who
        // already made their own filesystem writable is not overridden.
        let (tmp_volume_mount, tmp_volume) = if self.read_only_root_filesystem {
            (
                "\n        volumeMounts:\n        - name: tmp\n          mountPath: /tmp",
                "\n      volumes:\n      - name: tmp\n        emptyDir: {}",
            )
        } else {
            ("", "")
        };
        Ok(format!(
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
      automountServiceAccountToken: false
      securityContext:
        seccompProfile:
          type: RuntimeDefault
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
            port: http{tmp_volume_mount}{tmp_volume}
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
            tmp_volume_mount = tmp_volume_mount,
            tmp_volume = tmp_volume,
        ))
    }
}

/// Deterministic Kubernetes `CronJob` projection for a deployed CLI's
/// scheduled/batch verbs (e.g. a nightly `report generate` invocation) --
/// same CONSTRUCT-only discipline as [`KubernetesConfig`]: this renders
/// YAML text only, it never contacts a cluster.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CronJobConfig {
    pub name: String,
    pub namespace: Option<String>,
    pub image: String,
    /// A standard 5-field cron schedule (e.g. `"0 3 * * *"`).
    pub schedule: String,
    pub command: Vec<String>,
    pub args: Vec<String>,
    pub env: BTreeMap<String, String>,
    pub service_account_name: Option<String>,
    pub read_only_root_filesystem: bool,
    pub run_as_non_root: bool,
    /// `concurrencyPolicy`: one of `Allow`, `Forbid`, `Replace`.
    pub concurrency_policy: CronJobConcurrencyPolicy,
    /// `restartPolicy` for the job's pod template: `Never` or `OnFailure`
    /// (Kubernetes rejects `Always` for a `CronJob`'s pod template, so it
    /// is deliberately not offered here).
    pub restart_policy: CronJobRestartPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CronJobConcurrencyPolicy {
    Allow,
    Forbid,
    Replace,
}

impl CronJobConcurrencyPolicy {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Allow => "Allow",
            Self::Forbid => "Forbid",
            Self::Replace => "Replace",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CronJobRestartPolicy {
    Never,
    OnFailure,
}

impl CronJobRestartPolicy {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Never => "Never",
            Self::OnFailure => "OnFailure",
        }
    }
}

impl CronJobConfig {
    #[must_use]
    pub fn new(name: impl Into<String>, image: impl Into<String>, schedule: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            namespace: None,
            image: image.into(),
            schedule: schedule.into(),
            command: Vec::new(),
            args: Vec::new(),
            env: BTreeMap::new(),
            service_account_name: None,
            read_only_root_filesystem: true,
            run_as_non_root: true,
            concurrency_policy: CronJobConcurrencyPolicy::Forbid,
            restart_policy: CronJobRestartPolicy::Never,
        }
    }

    /// Render `CronJob` YAML in stable key order after validating every
    /// scalar that can affect YAML structure or Kubernetes identity,
    /// mirroring [`KubernetesConfig::render`]'s discipline exactly.
    pub fn render(&self) -> Result<String, KubernetesRenderError> {
        validate_dns_label(&self.name, "name")?;
        if let Some(namespace) = &self.namespace {
            validate_dns_label(namespace, "namespace")?;
        }
        if let Some(service_account_name) = &self.service_account_name {
            validate_dns_label(service_account_name, "service_account_name")?;
        }
        validate_image(&self.image)?;
        validate_cron_schedule(&self.schedule)?;
        for name in self.env.keys() {
            validate_env_name(name)?;
        }

        let metadata_namespace = self
            .namespace
            .as_ref()
            .map(|namespace| format!("\n  namespace: {namespace}"))
            .unwrap_or_default();
        let service_account = self
            .service_account_name
            .as_ref()
            .map(|name| format!("\n              serviceAccountName: {name}"))
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
                    format!("\n                - name: {name}\n                  value: {}", yaml_string(value))
                })
                .collect::<String>();
            format!("\n                env:{values}")
        };
        let (tmp_volume_mount, tmp_volume) = if self.read_only_root_filesystem {
            (
                "\n                volumeMounts:\n                - name: tmp\n                  mountPath: /tmp",
                "\n              volumes:\n              - name: tmp\n                emptyDir: {}",
            )
        } else {
            ("", "")
        };
        Ok(format!(
            "apiVersion: batch/v1
kind: CronJob
metadata:
  name: {name}{metadata_namespace}
  labels:
    app.kubernetes.io/name: {name}
spec:
  schedule: {schedule}
  concurrencyPolicy: {concurrency_policy}
  jobTemplate:
    spec:
      template:
        metadata:
          labels:
            app.kubernetes.io/name: {name}
        spec:{service_account}
          automountServiceAccountToken: false
          restartPolicy: {restart_policy}
          securityContext:
            seccompProfile:
              type: RuntimeDefault
          containers:
          - name: {name}
            image: {image}
            imagePullPolicy: IfNotPresent{command}{args}{env}
            securityContext:
              allowPrivilegeEscalation: false
              readOnlyRootFilesystem: {read_only}
              runAsNonRoot: {non_root}
              capabilities:
                drop: [\"ALL\"]{tmp_volume_mount}{tmp_volume}
",
            name = self.name,
            schedule = yaml_string(&self.schedule),
            concurrency_policy = self.concurrency_policy.as_str(),
            restart_policy = self.restart_policy.as_str(),
            image = self.image,
            read_only = self.read_only_root_filesystem,
            non_root = self.run_as_non_root,
            tmp_volume_mount = tmp_volume_mount,
            tmp_volume = tmp_volume,
        ))
    }
}

fn validate_cron_schedule(value: &str) -> Result<(), KubernetesRenderError> {
    let fields: Vec<&str> = value.split_whitespace().collect();
    let valid = fields.len() == 5 && fields.iter().all(|field| !field.is_empty());
    if valid {
        Ok(())
    } else {
        Err(KubernetesRenderError::InvalidField {
            field: "schedule",
            reason: "must be a standard 5-field cron expression (minute hour day-of-month month day-of-week)",
        })
    }
}

fn validate_dns_label(value: &str, field: &'static str) -> Result<(), KubernetesRenderError> {
    let length_valid = !value.is_empty() && value.len() <= 63;
    let edge_valid =
        value.as_bytes().first().zip(value.as_bytes().last()).is_some_and(|(first, last)| {
            first.is_ascii_alphanumeric() && last.is_ascii_alphanumeric()
        });
    let body_valid = value
        .bytes()
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');
    if length_valid && edge_valid && body_valid {
        Ok(())
    } else {
        Err(KubernetesRenderError::InvalidField {
            field,
            reason: "must be a lowercase DNS-1123 label of at most 63 characters",
        })
    }
}

fn validate_image(value: &str) -> Result<(), KubernetesRenderError> {
    if value.is_empty() {
        return Err(KubernetesRenderError::InvalidField {
            field: "image",
            reason: "must not be empty",
        });
    }
    if value.chars().any(char::is_whitespace) || value.chars().any(char::is_control) {
        return Err(KubernetesRenderError::InvalidField {
            field: "image",
            reason: "must not contain whitespace or control characters",
        });
    }
    Ok(())
}

fn validate_env_name(value: &str) -> Result<(), KubernetesRenderError> {
    let mut characters = value.chars();
    let first_valid = characters
        .next()
        .is_some_and(|character| character == '_' || character.is_ascii_alphabetic());
    let rest_valid =
        characters.all(|character| character == '_' || character.is_ascii_alphanumeric());
    if first_valid && rest_valid {
        Ok(())
    } else {
        Err(KubernetesRenderError::InvalidField {
            field: "env",
            reason: "environment names must use ASCII letters, digits and '_' and not start with a digit",
        })
    }
}

fn yaml_inline_array(name: &str, values: &[String]) -> String {
    if values.is_empty() {
        return String::new();
    }
    let values = values.iter().map(|value| yaml_string(value)).collect::<Vec<_>>().join(", ");
    format!("\n        {name}: [{values}]")
}

fn yaml_string(value: &str) -> String {
    let mut output = String::with_capacity(value.len() + 2);
    output.push('"');
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            character if character.is_control() => {
                use std::fmt::Write as _;
                let _ = write!(output, "\\u{:04x}", u32::from(character));
            }
            character => output.push(character),
        }
    }
    output.push('"');
    output
}
