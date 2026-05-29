use clap_noun_verb::cli::discovery::CommandDiscovery;
use clap_noun_verb::cli::help::{CommandCategory, CommandInfo};

#[test]
fn test_command_discovery_registration_and_listing() {
    let mut discovery = CommandDiscovery::new();

    let cmd1 = CommandInfo::new("pack create", CommandCategory::Pack, "Create a new pack")
        .with_description("Allows creating a new code generation pack from scratch.")
        .with_example("ggen pack create my-pack")
        .with_popularity(95);

    let cmd2 = CommandInfo::new("ai query", CommandCategory::AI, "Query AI model")
        .with_description("Send a prompt to the configured AI provider.")
        .with_example("ggen ai query 'implement quicksort'")
        .with_popularity(85);

    discovery.register(cmd1);
    discovery.register(cmd2);

    let all = discovery.list_all();
    assert_eq!(all.len(), 2);
    assert_eq!(all[0].name, "pack create");
    assert_eq!(all[1].name, "ai query");

    let packs = discovery.list_by_category(&CommandCategory::Pack);
    assert_eq!(packs.len(), 1);
    assert_eq!(packs[0].name, "pack create");
}

#[test]
fn test_command_discovery_search_and_fuzzy_matching() {
    let mut discovery = CommandDiscovery::new();

    let cmd = CommandInfo::new("marketplace search", CommandCategory::Marketplace, "Search community packs")
        .with_description("Search for community packs available on the marketplace.")
        .with_popularity(70);

    discovery.register(cmd);

    // Exact name match
    let results = discovery.search("marketplace search");
    assert!(!results.is_empty());
    assert_eq!(results[0].name, "marketplace search");
    assert_eq!(results[0].score, 100.0);

    // Prefix match
    let results = discovery.search("marketplace");
    assert!(!results.is_empty());
    assert_eq!(results[0].name, "marketplace search");
    assert_eq!(results[0].score, 90.0);

    // Fuzzy match
    let results = discovery.search("mktplce");
    assert!(!results.is_empty());
    assert_eq!(results[0].name, "marketplace search");
    assert!(results[0].score > 0.0);
}

#[test]
fn test_command_discovery_suggestions() {
    let mut discovery = CommandDiscovery::new();

    let cmd = CommandInfo::new("config show", CommandCategory::Config, "Show current configuration")
        .with_popularity(50);
    discovery.register(cmd);

    let suggestions = discovery.suggest("cnfig");
    assert!(!suggestions.is_empty());
    assert_eq!(suggestions[0].command, "config show");
    assert!(suggestions[0].reason.contains("Similar to your input"));
}

#[test]
fn test_command_discovery_categories_summary() {
    let mut discovery = CommandDiscovery::new();

    discovery.register(CommandInfo::new("pack create", CommandCategory::Pack, "Create pack"));
    discovery.register(CommandInfo::new("pack delete", CommandCategory::Pack, "Delete pack"));
    discovery.register(CommandInfo::new("system status", CommandCategory::System, "System status"));

    let summaries = discovery.categories_summary();
    assert_eq!(summaries.len(), 6); // Display all 6 categories

    let pack_summary = summaries.iter().find(|s| s.name == "Pack").unwrap();
    assert_eq!(pack_summary.command_count, 2);

    let system_summary = summaries.iter().find(|s| s.name == "System").unwrap();
    assert_eq!(system_summary.command_count, 1);

    let ai_summary = summaries.iter().find(|s| s.name == "AI").unwrap();
    assert_eq!(ai_summary.command_count, 0);
}
