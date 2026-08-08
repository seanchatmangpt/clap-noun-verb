// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! # CommandTree Example
//!
//! Demonstrates `CommandTree` and `CommandTreeBuilder` — the lower-level
//! tree-construction API for CLIs that need explicit hierarchical control
//! rather than the auto-discovery path.
//!
//! ## Capabilities witnessed
//!
//! - `CommandTreeBuilder::new()` + `add_root()` + `build()`
//! - `TreeNode::new()` + `add_child()` + `with_handler()`
//! - `CommandTree::from_builder()` + `roots()` + `root_names()`
//! - `CommandTree::find_command()` — path-based lookup
//! - `TreeNode::command_paths()` — enumerate all leaf paths
//! - `CommandTree::build_command()` — produces a `clap::Command` tree
//!
//! ## Run
//!
//! ```sh
//! cargo run --example command_tree
//! ```
//!
//! ## Expected output
//!
//! ```text
//! roots: ["services", "config"]
//! find services/status: found="services status"
//! find services/missing: None
//! command_paths from services: [["services", "status"], ["services", "restart"]]
//! clap::Command name: myapp
//! ```
//!
//! **Doc**: docs/reference/api-catalog.md (CommandTree section)
//! **Reference**: docs/reference/api/types.md

use clap_noun_verb::tree::TreeNode;
use clap_noun_verb::{CommandTree, CommandTreeBuilder, Result};

fn main() -> Result<()> {
    // --- Build a tree with two nouns and leaf verbs with handlers ---
    let services = TreeNode::new("services", "Manage services")
        .add_child(TreeNode::new("status", "Show service status").with_handler(|_matches| {
            println!("[handler] services status");
            Ok(())
        }))
        .add_child(TreeNode::new("restart", "Restart a service").with_handler(|_matches| {
            println!("[handler] services restart");
            Ok(())
        }));

    let config = TreeNode::new("config", "Manage configuration").add_child(
        TreeNode::new("get", "Get a config value").with_handler(|_matches| {
            println!("[handler] config get");
            Ok(())
        }),
    );

    let tree =
        CommandTree::from_builder(CommandTreeBuilder::new().add_root(services).add_root(config));

    // --- Witness: roots() and root_names() ---
    let roots = tree.roots();
    assert_eq!(roots.len(), 2, "tree must have 2 roots");
    let names = tree.root_names();
    assert!(names.contains(&"services"), "root_names must include 'services'");
    assert!(names.contains(&"config"), "root_names must include 'config'");
    println!("roots: {:?}", names);

    // --- Witness: find_command() — path-based lookup ---
    let found = tree.find_command(&["services", "status"]);
    assert!(found.is_some(), "find_command must locate services/status");
    let node = found.expect("services/status node");
    assert_eq!(node.name(), "status", "found node name must be 'status'");
    assert_eq!(node.about(), "Show service status");
    println!("find services/status: found=\"services {}\"", node.name());

    let missing = tree.find_command(&["services", "missing"]);
    assert!(missing.is_none(), "find_command for unknown path must return None");
    println!("find services/missing: None");

    // --- Witness: command_paths() — enumerate leaf paths from a node ---
    let services_node = tree.find_command(&["services"]).expect("services node");
    let paths = services_node.command_paths();
    assert_eq!(paths.len(), 2, "services must have 2 leaf paths");
    assert!(
        paths.iter().any(|p| p == &["services".to_string(), "status".to_string()]),
        "paths must include services/status"
    );
    assert!(
        paths.iter().any(|p| p == &["services".to_string(), "restart".to_string()]),
        "paths must include services/restart"
    );
    println!("command_paths from services: {:?}", paths);

    // --- Witness: build_command() produces a clap::Command ---
    let cmd = tree.build_command();
    assert_eq!(cmd.get_name(), "cli", "build_command must produce a clap Command");
    let subnames: Vec<_> = cmd.get_subcommands().map(|s| s.get_name()).collect();
    assert!(subnames.contains(&"services"), "clap::Command must have 'services' subcommand");
    assert!(subnames.contains(&"config"), "clap::Command must have 'config' subcommand");
    println!("clap::Command name: {} subcommands: {:?}", cmd.get_name(), subnames);

    Ok(())
}
