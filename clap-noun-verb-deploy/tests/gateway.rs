use clap_noun_verb_deploy::{
    AdmissionPolicy, AdmitValidated, CommandAllowList, EnvironmentAllowList, Execution, Executor,
    Gateway, GatewayError, Invocation, ProcessExecutionError, ProcessExecutor,
};
use std::convert::Infallible;

#[derive(Clone, Copy)]
struct Echo;

impl Executor for Echo {
    type Error = Infallible;

    fn execute(&self, invocation: &Invocation) -> Result<Execution, Self::Error> {
        Ok(Execution { exit_code: 0, stdout: invocation.args.join(" "), stderr: String::new() })
    }
}

#[test]
fn gateway_refuses_before_executor_authority() {
    let policy = CommandAllowList::default().allow(["safe", "read"]);
    let gateway = Gateway::new("demo", Echo, policy);
    let error = gateway
        .execute(Invocation::new(["danger", "delete"]))
        .expect_err("command must be refused");
    assert!(matches!(error, GatewayError::Refused(_)));
}

#[test]
fn execution_record_replays_exact_invocation() {
    let policy = CommandAllowList::default().allow(["safe", "read"]);
    let gateway = Gateway::new("demo", Echo, policy);
    let record = gateway
        .execute(Invocation::new(["safe", "read", "thing"]))
        .expect("admitted command executes");
    assert!(record.verify_integrity());
    let replay = record.replay(&Echo).expect("replay executes");
    assert!(replay.matches);
}

#[test]
fn allow_list_is_pure_admission() {
    let policy = CommandAllowList::default().allow(["safe", "read"]);
    assert!(policy.admit(&Invocation::new(["safe", "read", "--format", "json"])).is_admitted());
}

#[test]
fn default_policy_refuses_per_invocation_environment() {
    let invocation = Invocation::new(["safe", "read"]).with_env("LD_PRELOAD", "/tmp/inject.so");
    assert!(!AdmitValidated.admit(&invocation).is_admitted());
}

#[test]
fn environment_requires_explicit_policy_and_executor_authority() {
    let invocation = Invocation::new(["safe", "read"]).with_env("TENANT", "alpha");
    let policy = EnvironmentAllowList::new(AdmitValidated, ["TENANT"]);
    assert!(policy.admit(&invocation).is_admitted());

    let executor = ProcessExecutor::new("definitely-not-a-real-executable");
    let error = executor.execute(&invocation).expect_err("executor must independently refuse env");
    assert!(matches!(error, ProcessExecutionError::EnvironmentRefused(name) if name == "TENANT"));
}
