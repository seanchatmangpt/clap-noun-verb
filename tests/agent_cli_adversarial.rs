use clap_noun_verb::{CliBuilder, noun, verb, VerbArgs, Arg};

#[test]
fn test_adversarial_null_bytes() {
    let cli = CliBuilder::new()
        .name("adversarial-app")
        .noun(noun!("user", "User operations", [
            verb!("create", "Create a user", |args: &VerbArgs| {
                let name = args.get_many_opt_str("name");
                assert!(!name.is_empty());
                Ok(())
            }, args: [Arg::new("name").long("name").required(true)])
        ]));

    let cmd = cli.build_command();
    // Passing inputs with null bytes should be handled gracefully by clap/builder
    let res = cmd.try_get_matches_from(vec!["adversarial-app", "user", "create", "--name", "john\0doe"]);
    assert!(res.is_ok());
}

#[test]
fn test_adversarial_overflow_args() {
    let cli = CliBuilder::new()
        .name("adversarial-app")
        .noun(noun!("calc", "Calc operations", [
            verb!("add", "Add values", |args: &VerbArgs| {
                let val = args.get_many_opt_str("val");
                assert!(!val.is_empty());
                Ok(())
            }, args: [Arg::new("val").long("val").required(true)])
        ]));

    let cmd = cli.build_command();
    // Pass massive integer or value to see if it causes panics
    let res = cmd.try_get_matches_from(vec![
        "adversarial-app",
        "calc",
        "add",
        "--val",
        "1844674407370955161599999999999999999999999999999999",
    ]);
    assert!(res.is_ok());
}

#[test]
fn test_adversarial_extremely_long_strings() {
    let cli = CliBuilder::new()
        .name("adversarial-app")
        .noun(noun!("data", "Data operations", [
            verb!("store", "Store data", |_args: &VerbArgs| {
                Ok(())
            }, args: [Arg::new("payload").long("payload").required(true)])
        ]));

    let cmd = cli.build_command();
    let long_str = "a".repeat(100_000); // 100KB string
    let res = cmd.try_get_matches_from(vec![
        "adversarial-app",
        "data",
        "store",
        "--payload",
        &long_str,
    ]);
    // The parser/builder should parse it fine without blowing up the stack
    assert!(res.is_ok());
}

#[test]
fn test_adversarial_invalid_command_suggestions() {
    let cli = CliBuilder::new()
        .name("adversarial-app")
        .noun(noun!("system", "System command", [
            verb!("status", "Get status", |_args: &VerbArgs| {
                Ok(())
            })
        ]));

    let cmd = cli.build_command();
    // Run an invalid subcommand and expect suggestion
    let res = cmd.try_get_matches_from(vec!["adversarial-app", "systm", "status"]);
    assert!(res.is_err());
    let err = res.unwrap_err();
    let err_str = err.to_string();
    println!("Clap error: {}", err_str);
    assert!(
        err_str.contains("system") || err_str.contains("subcommand") || err_str.contains("recognized"),
        "Unexpected error output: {}", err_str
    );
}

