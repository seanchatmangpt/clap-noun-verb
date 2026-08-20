use clap_noun_verb_deploy::{
    AdmissionPolicy, CommandAllowList, Execution, Executor, Gateway, GatewayError, Invocation,
};
use std::convert::Infallible;

#[derive(Clone, Copy)]
struct Echo;

impl Executor for Echo {
    type Error = Infallible;

    fn execute(&self, invocation: &Invocation) -> Result<Execution, Self::Error> {
        Ok(Execution {
            exit_code: 0,
            stdout: invocation.args.join(" "),
            stderr: String::new(),
        })
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
    assert!(policy
        .admit(&Invocation::new(["safe", "read", "--format", "json"]))
        .is_admitted());
}
