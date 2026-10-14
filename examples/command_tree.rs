// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Executable witness for `CommandTree`, `CommandTreeBuilder`, and `TreeNode`.

use clap_noun_verb::tree::TreeNode;
use clap_noun_verb::{CommandTree, CommandTreeBuilder, Result};

fn main() -> Result<()> {
    let services = TreeNode::new("services", "Manage services")
        .add_child(TreeNode::new("status", "Show status").with_handler(|_| Ok(())))
        .add_child(TreeNode::new("restart", "Restart service").with_handler(|_| Ok(())));
    let config = TreeNode::new("config", "Manage configuration")
        .add_child(TreeNode::new("get", "Get value").with_handler(|_| Ok(())));

    let tree =
        CommandTree::from_builder(CommandTreeBuilder::new().add_root(services).add_root(config));

    assert_eq!(tree.roots().len(), 2);
    assert!(tree.root_names().contains(&"services"));
    assert_eq!(tree.find_command(&["services", "status"]).map(|node| node.name()), Some("status"));
    assert!(tree.find_command(&["services", "missing"]).is_none());

    let paths = tree.find_command(&["services"]).map(TreeNode::command_paths).unwrap_or_default();
    assert_eq!(paths.len(), 2);

    let command = tree.build_command();
    let subcommands: Vec<_> = command.get_subcommands().map(|item| item.get_name()).collect();
    assert!(subcommands.contains(&"services"));
    assert!(subcommands.contains(&"config"));

    println!("CommandTree roots={:?} paths={:?}", tree.root_names(), paths);
    Ok(())
}
