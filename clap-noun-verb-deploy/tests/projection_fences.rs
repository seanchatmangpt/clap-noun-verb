#[cfg(feature = "container")]
#[test]
fn container_projection_refuses_grammar_injection() {
    use clap_noun_verb_deploy::container::{ContainerConfig, ContainerRenderError};

    let mut config = ContainerConfig::new("demo\nRUN touch /owned", "demo");
    let error = config.render_dockerfile().expect_err("package grammar injection must be refused");
    assert!(matches!(error, ContainerRenderError::InvalidField { field: "package", .. }));

    config.package = "demo".into();
    config.runtime_image = "debian:bookworm-slim\nRUN touch /owned".into();
    let error = config.render_dockerfile().expect_err("image grammar injection must be refused");
    assert!(matches!(error, ContainerRenderError::InvalidField { field: "runtime_image", .. }));
}

#[cfg(feature = "kubernetes")]
#[test]
fn kubernetes_projection_refuses_identity_injection_and_quotes_values() {
    use clap_noun_verb_deploy::kubernetes::{KubernetesConfig, KubernetesRenderError};

    let config = KubernetesConfig::new("demo\n---\nkind: Secret", "ghcr.io/example/demo:sha-123");
    let error = config.render().expect_err("name grammar injection must be refused");
    assert!(matches!(error, KubernetesRenderError::InvalidField { field: "name", .. }));

    let mut config = KubernetesConfig::new("demo", "ghcr.io/example/demo:sha-123");
    config.env.insert("MESSAGE".into(), "line1\nline2: value".into());
    config.args.push("hello\nworld".into());
    let yaml = config.render().expect("values are safely quoted");
    assert!(yaml.contains("value: \"line1\\nline2: value\""));
    assert!(yaml.contains("args: [\"hello\\nworld\"]"));
}

#[cfg(feature = "kubernetes")]
#[test]
fn cronjob_projection_refuses_identity_injection_and_quotes_values() {
    use clap_noun_verb_deploy::kubernetes::{CronJobConfig, KubernetesRenderError};

    let config =
        CronJobConfig::new("demo\n---\nkind: Secret", "ghcr.io/example/demo:sha-123", "0 3 * * *");
    let error = config.render().expect_err("name grammar injection must be refused");
    assert!(matches!(error, KubernetesRenderError::InvalidField { field: "name", .. }));

    let mut config = CronJobConfig::new("demo", "ghcr.io/example/demo:sha-123", "0 3 * * *");
    config.env.insert("MESSAGE".into(), "line1\nline2: value".into());
    config.args.push("hello\nworld".into());
    let yaml = config.render().expect("values are safely quoted");
    assert!(yaml.contains("value: \"line1\\nline2: value\""));
    assert!(yaml.contains("args: [\"hello\\nworld\"]"));
}
