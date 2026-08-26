use clap_noun_verb::{Arg, ArgAction, Command};
use clap_noun_verb_deploy::{ArgumentKind, CliSchema, InvocationBuildError};
use serde_json::{json, Map};

fn schema() -> CliSchema {
    CliSchema::from_command(&Command::new("demo").subcommand(
        Command::new("run")
            .arg(Arg::new("feature").long("feature").action(ArgAction::SetTrue))
            .arg(Arg::new("cache").long("cache").action(ArgAction::SetFalse))
            .arg(Arg::new("verbose").short('v').long("verbose").action(ArgAction::Count)),
    ))
}

#[test]
fn preserves_boolean_and_count_kinds() {
    let schema = schema();
    let command = schema.commands.iter().find(|command| command.callable).expect("callable");
    assert_eq!(command.arguments[0].kind, ArgumentKind::Boolean);
    assert_eq!(command.arguments[1].kind, ArgumentKind::Boolean);
    assert_eq!(command.arguments[2].kind, ArgumentKind::Integer);
}

#[test]
fn manufactures_set_true_set_false_and_count_argv() {
    let mut arguments = Map::new();
    arguments.insert("feature".into(), json!(true));
    arguments.insert("cache".into(), json!(false));
    arguments.insert("verbose".into(), json!(2));
    let invocation = schema().build_invocation("run", &arguments).expect("valid invocation");
    assert_eq!(invocation.args, ["run", "--feature", "--cache", "--verbose", "--verbose"]);
}

#[test]
fn omits_switch_when_requested_value_matches_default_polarity() {
    let mut arguments = Map::new();
    arguments.insert("feature".into(), json!(false));
    arguments.insert("cache".into(), json!(true));
    arguments.insert("verbose".into(), json!(0));
    let invocation = schema().build_invocation("run", &arguments).expect("valid invocation");
    assert_eq!(invocation.args, ["run"]);
}

#[test]
fn refuses_count_above_clap_u8_range() {
    let mut arguments = Map::new();
    arguments.insert("verbose".into(), json!(256));
    let error = schema().build_invocation("run", &arguments).expect_err("count must be bounded");
    assert!(matches!(error, InvocationBuildError::OutOfRange { .. }));
}
