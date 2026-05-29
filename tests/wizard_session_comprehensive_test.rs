use clap_noun_verb::{noun, verb, CliBuilder, VerbArgs};
use once_cell::sync::Lazy;
use std::sync::Mutex;

#[derive(Clone, Debug, PartialEq)]
struct WizardSession {
    step: usize,
    target: String,
    completed: bool,
}

static SESSION_STATE: Lazy<Mutex<Option<WizardSession>>> = Lazy::new(|| Mutex::new(None));

fn make_cli() -> CliBuilder {
    CliBuilder::new().name("wizard-session").noun(noun!(
        "wizard",
        "Wizard",
        [
            verb!("start", "Start session", |args: &VerbArgs| {
                let targets = args.get_many_opt_str("target");
                if let Some(target) = targets.first() {
                    if let Ok(mut guard) = SESSION_STATE.lock() {
                        if let Some(s) = guard.as_mut() {
                            s.step = 1;
                            s.target = target.to_string();
                        }
                    }
                }
                Ok(())
            }, args: [
                clap::Arg::new("target").long("target").required(true)
            ]),
            verb!("finish", "Finish session", |_args: &VerbArgs| {
                if let Ok(mut guard) = SESSION_STATE.lock() {
                    if let Some(s) = guard.as_mut() {
                        if s.step == 1 {
                            s.step = 2;
                            s.completed = true;
                        }
                    }
                }
                Ok(())
            })
        ]
    ))
}

#[test]
fn test_wizard_session_state_transitions() -> Result<(), Box<dyn std::error::Error>> {
    // Reset state safely
    if let Ok(mut guard) = SESSION_STATE.lock() {
        *guard = Some(WizardSession { step: 0, target: "".to_string(), completed: false });
    }

    let cmd = make_cli().build_command();

    // Step 1: Start
    let res1 = cmd
        .try_get_matches_from(vec![
            "wizard-session",
            "wizard",
            "start",
            "--target",
            "code-generator",
        ])
        .map_err(|e| e.to_string())?;

    // Execute start handler manually using start sub-matches
    let sub_w1 = res1.subcommand_matches("wizard").ok_or("missing subcommand wizard")?;
    let sub_s1 = sub_w1.subcommand_matches("start").ok_or("missing subcommand start")?;

    let cli = make_cli();
    let w_noun = cli.registry_ref().get_noun("wizard").ok_or("missing noun wizard")?;
    let start_verb = w_noun.verbs().into_iter().next().ok_or("missing verb start")?;
    let verb_args = VerbArgs::new(sub_s1.clone());
    start_verb.run(&verb_args)?;

    // Verify session updated to step 1
    let session_after_start = {
        let guard = SESSION_STATE.lock().map_err(|e| e.to_string())?;
        guard.clone().ok_or("session not initialized")?
    };
    assert_eq!(session_after_start.step, 1);
    assert_eq!(session_after_start.target, "code-generator");
    assert!(!session_after_start.completed);

    // Step 2: Finish
    let sub_f1 = sub_w1.subcommand_matches("finish");
    let finish_verb = w_noun.verbs().into_iter().nth(1).ok_or("missing verb finish")?;
    let verb_args_finish = VerbArgs::new(sub_f1.cloned().unwrap_or_default());
    finish_verb.run(&verb_args_finish)?;

    // Verify session updated to completed
    let session_after_finish = {
        let guard = SESSION_STATE.lock().map_err(|e| e.to_string())?;
        guard.clone().ok_or("session not initialized")?
    };
    assert_eq!(session_after_finish.step, 2);
    assert!(session_after_finish.completed);

    Ok(())
}
