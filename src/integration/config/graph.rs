//! Dependency graph resolution for plugins using topological sort.

use super::PluginConfig;
use std::collections::{HashMap, HashSet, VecDeque};

/// Plugin dependency graph resolver.
///
/// Uses Kahn's algorithm for topological sorting to determine plugin load order.
pub struct PluginDependencyGraph {
    /// Nodes: plugin names
    nodes: HashSet<String>,
    /// Edges: plugin -> dependencies
    edges: HashMap<String, Vec<String>>,
    /// Reverse edges for cycle detection
    reverse_edges: HashMap<String, Vec<String>>,
}

impl PluginDependencyGraph {
    /// Create a new empty dependency graph.
    pub fn new() -> Self {
        Self { nodes: HashSet::new(), edges: HashMap::new(), reverse_edges: HashMap::new() }
    }

    /// Add a plugin to the graph with its dependencies.
    pub fn add_plugin(&mut self, config: &PluginConfig) {
        self.nodes.insert(config.name.clone());
        self.edges.insert(config.name.clone(), config.dependencies.clone());

        for dep in &config.dependencies {
            self.reverse_edges
                .entry(dep.clone())
                .or_insert_with(Vec::new)
                .push(config.name.clone());
        }
    }

    /// Resolve the plugin load order using topological sort.
    ///
    /// Returns plugins in the order they should be loaded (dependencies first).
    ///
    /// # Errors
    ///
    /// Returns an error if there's a circular dependency.
    pub fn resolve(&self) -> crate::Result<Vec<String>> {
        // Calculate in-degrees
        let mut in_degree: HashMap<String, usize> =
            self.nodes.iter().map(|n| (n.clone(), 0)).collect();

        for (_, deps) in &self.edges {
            for dep in deps {
                *in_degree.entry(dep.clone()).or_insert(0) += 1;
            }
        }

        // Find all nodes with in-degree 0
        let mut queue: VecDeque<String> =
            in_degree.iter().filter(|(_, deg)| **deg == 0).map(|(name, _)| name.clone()).collect();

        let mut result = Vec::new();

        while let Some(node) = queue.pop_front() {
            result.push(node.clone());

            // Decrease in-degree for all dependents
            if let Some(dependents) = self.reverse_edges.get(&node) {
                for dependent in dependents {
                    if let Some(deg) = in_degree.get_mut(dependent) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push_back(dependent.clone());
                        }
                    }
                }
            }
        }

        // Check if all nodes were processed
        if result.len() != self.nodes.len() {
            return Err(crate::NounVerbError::PluginError(
                "Circular dependency detected in plugin configuration".to_string(),
            ));
        }

        Ok(result)
    }

    /// Check if the graph has any circular dependencies.
    pub fn has_cycle(&self) -> bool {
        self.resolve().is_err()
    }

    /// Get all plugins that a specific plugin depends on.
    pub fn get_dependencies(&self, name: &str) -> Option<&Vec<String>> {
        self.edges.get(name)
    }

    /// Get all plugins that depend on a specific plugin.
    pub fn get_dependents(&self, name: &str) -> Option<&Vec<String>> {
        self.reverse_edges.get(name)
    }

    /// Get the number of plugins in the graph.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Check if a plugin is in the graph.
    pub fn contains(&self, name: &str) -> bool {
        self.nodes.contains(name)
    }

    /// Get all plugin names.
    pub fn plugins(&self) -> Vec<&str> {
        self.nodes.iter().map(|s| s.as_str()).collect()
    }
}

impl Default for PluginDependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(all(test, feature = "experimental"))]
mod tests {
    use super::*;

    #[test]
    fn test_graph_creation() {
        let graph = PluginDependencyGraph::new();
        assert_eq!(graph.node_count(), 0);
    }

    #[test]
    fn test_add_plugin() {
        let mut graph = PluginDependencyGraph::new();
        let config = PluginConfig::new("test", "1.0.0", "lib.so");
        graph.add_plugin(&config);
        assert!(graph.contains("test"));
    }

    #[test]
    fn test_dependency_resolution() {
        let mut graph = PluginDependencyGraph::new();

        let plugin_a = PluginConfig::new("a", "1.0.0", "lib.so");
        let mut plugin_b = PluginConfig::new("b", "1.0.0", "lib.so");
        plugin_b.dependencies = vec!["a".to_string()];

        let mut plugin_c = PluginConfig::new("c", "1.0.0", "lib.so");
        plugin_c.dependencies = vec!["a".to_string(), "b".to_string()];

        graph.add_plugin(&plugin_a);
        graph.add_plugin(&plugin_b);
        graph.add_plugin(&plugin_c);

        let result = graph.resolve();
        assert!(result.is_ok());
        let order = result.unwrap();

        // A must come first
        assert_eq!(order[0], "a");
    }

    #[test]
    fn test_circular_dependency_detection() {
        let mut graph = PluginDependencyGraph::new();

        let mut plugin_a = PluginConfig::new("a", "1.0.0", "lib.so");
        let mut plugin_b = PluginConfig::new("b", "1.0.0", "lib.so");

        plugin_a.dependencies = vec!["b".to_string()];
        plugin_b.dependencies = vec!["a".to_string()];

        graph.add_plugin(&plugin_a);
        graph.add_plugin(&plugin_b);

        assert!(graph.has_cycle());
    }

    #[test]
    fn test_get_dependencies() {
        let mut graph = PluginDependencyGraph::new();
        let mut plugin = PluginConfig::new("test", "1.0.0", "lib.so");
        plugin.dependencies = vec!["dep1".to_string(), "dep2".to_string()];
        graph.add_plugin(&plugin);

        let deps = graph.get_dependencies("test");
        assert!(deps.is_some());
        assert_eq!(deps.unwrap().len(), 2);
    }
}
