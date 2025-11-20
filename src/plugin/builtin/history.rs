//! Command execution history tracking plugin.

use crate::plugin::{Plugin, PluginCapability, PluginMetadata};
use std::collections::VecDeque;

/// Plugin for tracking command execution history.
#[derive(Debug)]
pub struct HistoryPlugin {
    loaded: bool,
    max_history: usize,
    history: VecDeque<String>,
}

impl HistoryPlugin {
    /// Create a new history plugin.
    pub fn new() -> Self {
        Self { loaded: false, max_history: 1000, history: VecDeque::new() }
    }

    /// Set maximum history size.
    pub fn with_max_history(mut self, max: usize) -> Self {
        self.max_history = max;
        self
    }

    /// Add a command to history.
    pub fn add_command(&mut self, command: String) {
        if self.history.len() >= self.max_history {
            self.history.pop_front();
        }
        self.history.push_back(command);
    }

    /// Get command history.
    pub fn get_history(&self) -> Vec<&str> {
        self.history.iter().map(|s| s.as_str()).collect()
    }

    /// Clear history.
    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    /// Get the number of commands in history.
    pub fn history_len(&self) -> usize {
        self.history.len()
    }
}

impl Default for HistoryPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for HistoryPlugin {
    fn name(&self) -> &str {
        "history"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata::new(self.name(), self.version())
            .with_author("clap-noun-verb")
            .with_description("Command execution history tracking")
    }

    fn capabilities(&self) -> Vec<PluginCapability> {
        vec![PluginCapability::Hook, PluginCapability::Middleware]
    }

    fn load(&mut self) -> crate::Result<()> {
        self.loaded = true;
        Ok(())
    }

    fn unload(&mut self) -> crate::Result<()> {
        self.loaded = false;
        self.clear_history();
        Ok(())
    }

    fn status(&self) -> String {
        format!(
            "HistoryPlugin v{} ({} commands, {})",
            self.version(),
            self.history_len(),
            if self.loaded { "loaded" } else { "unloaded" }
        )
    }
}
