use clap_noun_verb::{CliBuilder, NounCommand, NounVerbError, VerbArgs, VerbCommand};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

struct SimpleVerb {
    name: &'static str,
    about: &'static str,
    handler: Arc<dyn Fn(&VerbArgs) -> clap_noun_verb::Result<()> + Send + Sync>,
}

impl VerbCommand for SimpleVerb {
    fn name(&self) -> &'static str {
        self.name
    }
    fn about(&self) -> &'static str {
        self.about
    }
    fn run(&self, args: &VerbArgs) -> clap_noun_verb::Result<()> {
        (self.handler)(args)
    }
}

struct RateLimitNoun {
    handler: Arc<dyn Fn(&VerbArgs) -> clap_noun_verb::Result<()> + Send + Sync>,
}

impl NounCommand for RateLimitNoun {
    fn name(&self) -> &'static str {
        "wizard"
    }
    fn about(&self) -> &'static str {
        "Wizard"
    }
    fn verbs(&self) -> Vec<Box<dyn VerbCommand>> {
        vec![Box::new(SimpleVerb {
            name: "generate",
            about: "Generate",
            handler: self.handler.clone(),
        })]
    }
}

#[test]
fn test_wizard_handler_rate_limiting() -> Result<(), Box<dyn std::error::Error>> {
    let simulated_time = Arc::new(AtomicU64::new(100));
    let simulated_time_clone = simulated_time.clone();
    let last_called = Arc::new(AtomicU64::new(0));
    let last_called_clone = last_called.clone();

    let handler1 = Arc::new(move |_args: &VerbArgs| {
        let now = simulated_time_clone.load(Ordering::SeqCst);
        let prev = last_called_clone.swap(now, Ordering::SeqCst);

        // If called within 10 milliseconds, rate limit it
        if prev != 0 && now - prev < 10 {
            return Err(NounVerbError::execution_error("Rate limit exceeded"));
        }
        Ok(())
    });

    let noun1 = RateLimitNoun { handler: handler1 };

    let cli = CliBuilder::new().name("wizard-rate-limit").noun(noun1);

    let cmd = cli.build_command();

    // First call: should succeed
    let res = cmd
        .try_get_matches_from(vec!["wizard-rate-limit", "wizard", "generate"])
        .map_err(|e| e.to_string())?;

    // Execute the handler manually to simulate rate-limiting trigger
    let sub1 = res.subcommand_matches("wizard").ok_or("missing wizard subcommand")?.clone();
    let sub2 = sub1.subcommand_matches("generate").ok_or("missing generate subcommand")?.clone();
    let args = VerbArgs::new(sub2);

    // Call 1: success
    let simulated_time_inner = Arc::new(AtomicU64::new(100));
    let simulated_time_inner_clone = simulated_time_inner.clone();
    let last_called_inner = Arc::new(AtomicU64::new(0));
    let last_called_inner_clone = last_called_inner.clone();
    let handler = move |_: &VerbArgs| {
        let now = simulated_time_inner_clone.load(Ordering::SeqCst);
        let prev = last_called_inner_clone.swap(now, Ordering::SeqCst);
        if prev != 0 && now - prev < 50 {
            return Err(NounVerbError::execution_error("Rate limit exceeded"));
        }
        Ok(())
    };

    assert!(handler(&args).is_ok());

    // Call 2 (immediate, no time advancement): should fail with rate limit error
    let res2 = handler(&args);
    assert!(res2.is_err());
    assert_eq!(
        res2.err().ok_or("expected err")?.to_string(),
        "Command execution failed: Rate limit exceeded"
    );

    Ok(())
}
