// Exact macro output test - manually written to debug

use clap_noun_verb::error::Result;
use clap_noun_verb::logic::{HandlerInput, HandlerOutput};
use clap_noun_verb::cli::registry::{ArgMetadata, CommandRegistry};

fn test_opt(opt: Option<String>) -> Result<()> {
    println!("Got: {:?}", opt);
    Ok(())
}

fn __test_opt_wrapper(
    input: HandlerInput,
) -> Result<HandlerOutput> {
    let opt = input.args.get("opt").map(|v| v.clone());
    let result = test_opt(opt)?;
    HandlerOutput::from_data(result)
}

#[linkme::distributed_slice(::clap_noun_verb::cli::registry::__VERB_REGISTRY)]
static __init_test_opt: fn() = || {
    let (noun_name_static, noun_about_static, verb_name_final) = {
        let inferred_name = "exact_macro_output".to_string();
        let final_verb_name = "test".to_string();
        let noun_about = String::new();
        let name_static: &'static str = Box::leak(inferred_name.into_boxed_str());
        let about_static: &'static str = Box::leak(noun_about.into_boxed_str());
        let verb_static: &'static str = Box::leak(final_verb_name.into_boxed_str());
        CommandRegistry::register_noun(name_static, about_static);
        (name_static, about_static, verb_static)
    };

    let args = vec![
        ArgMetadata {
            name: "opt".to_string(),
            required: false,
            is_flag: false,
            help: None,
            min_value: None,
            max_value: None,
            min_length: None,
            max_length: None,
            short: None,
            default_value: None,
            env: None,
            multiple: false,
            value_name: None,
            aliases: vec![],
            positional: None,
            action: None,
            group: None,
            requires: vec![],
            conflicts_with: vec![],
            value_parser: None,
            hide: false,
            next_help_heading: None,
            long_help: None,
            next_line_help: false,
            display_order: None,
            exclusive: None,
            trailing_vararg: false,
            allow_negative_numbers: false,
        },
    ];
    CommandRegistry::register_verb_with_args::<_>(
        noun_name_static,
        verb_name_final,
        "",
        args,
        __test_opt_wrapper,
    );
};

#[test]
fn it_compiles() {
    // If this compiles, the macro output is correct
    test_opt(Some("test".to_string()));
    test_opt(None);
}
