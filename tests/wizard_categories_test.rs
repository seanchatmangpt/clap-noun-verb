use clap_noun_verb::cli::discovery::CommandDiscovery;
use clap_noun_verb::cli::help::{CommandCategory, CommandInfo};

#[test]
fn test_wizard_command_categorization() {
    let mut discovery = CommandDiscovery::new();

    discovery.register(CommandInfo::new("wizard generate", CommandCategory::AI, "AI Generation"));
    discovery.register(CommandInfo::new("wizard synthesize", CommandCategory::AI, "AI Synthesis"));
    discovery.register(CommandInfo::new("wizard deploy", CommandCategory::Pack, "Pack deploy"));
    discovery.register(CommandInfo::new("wizard config", CommandCategory::Config, "Wizard configuration"));

    // Verify AI category contains the commands
    let ai_cmds = discovery.list_by_category(&CommandCategory::AI);
    assert_eq!(ai_cmds.len(), 2);
    let names: Vec<String> = ai_cmds.into_iter().map(|c| c.name).collect();
    assert!(names.contains(&"wizard generate".to_string()));
    assert!(names.contains(&"wizard synthesize".to_string()));

    // Verify Pack category contains deploy
    let pack_cmds = discovery.list_by_category(&CommandCategory::Pack);
    assert_eq!(pack_cmds.len(), 1);
    assert_eq!(pack_cmds[0].name, "wizard deploy");

    // Verify summaries
    let summaries = discovery.categories_summary();
    let config_summary = summaries.iter().find(|s| s.name == "Config").unwrap();
    assert_eq!(config_summary.command_count, 1);
}
