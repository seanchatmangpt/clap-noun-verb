use clap_noun_verb::tree::{CommandHandler, CommandTreeBuilder, TreeNode};
use clap_noun_verb::VerbArgs;

#[test]
fn test_command_tree_builder_and_traversal() {
    let mut builder = CommandTreeBuilder::new();

    // Build a graph tree: root -> child -> leaf
    let leaf = TreeNode {
        name: "status".to_string(),
        about: "Show status".to_string(),
        children: vec![],
        handler: Some(CommandHandler { handler: Box::new(|_args: &VerbArgs| Ok(())) }),
    };

    let child = TreeNode {
        name: "services".to_string(),
        about: "Manage services".to_string(),
        children: vec![leaf],
        handler: None,
    };

    builder = builder.add_root(child);
    let tree = builder.build();

    assert_eq!(tree.root_names(), vec!["services"]);

    // Find node by path
    let found = tree.find_command(&["services", "status"]);
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "status");
    assert!(found.unwrap().handler.is_some());

    // Path that doesn't exist
    let not_found = tree.find_command(&["services", "nonexistent"]);
    assert!(not_found.is_none());
}

#[test]
fn test_command_tree_build_clap_commands() {
    let mut builder = CommandTreeBuilder::new();
    let leaf = TreeNode {
        name: "list".to_string(),
        about: "List items".to_string(),
        children: vec![],
        handler: Some(CommandHandler { handler: Box::new(|_args: &VerbArgs| Ok(())) }),
    };
    let root = TreeNode {
        name: "pack".to_string(),
        about: "Pack commands".to_string(),
        children: vec![leaf],
        handler: None,
    };

    builder = builder.add_root(root);
    let tree = builder.build();

    let clap_cmd = tree.build_command();
    // The build_command generates a command named "cli"
    assert_eq!(clap_cmd.get_name(), "cli");

    let pack_sub = clap_cmd.get_subcommands().find(|s| s.get_name() == "pack").unwrap();
    let list_sub = pack_sub.get_subcommands().find(|s| s.get_name() == "list").unwrap();
    assert_eq!(list_sub.get_name(), "list");
}
