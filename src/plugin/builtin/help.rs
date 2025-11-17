//! Enhanced help generation plugin.

use crate::plugin::{Plugin, PluginCapability, PluginMetadata};

/// Plugin for enhanced help generation.
#[derive(Debug)]
pub struct HelpPlugin {
    loaded: bool,
}

impl HelpPlugin {
    /// Create a new help plugin.
    pub fn new() -> Self {
        Self { loaded: false }
    }
}

impl Default for HelpPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for HelpPlugin {
    fn name(&self) -> &str {
        "help"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata::new(self.name(), self.version())
            .with_author("clap-noun-verb")
            .with_description("Enhanced help text generation")
    }

    fn capabilities(&self) -> Vec<PluginCapability> {
        vec![PluginCapability::Hook, PluginCapability::Completion]
    }

    fn load(&mut self) -> crate::Result<()> {
        self.loaded = true;
        Ok(())
    }

    fn unload(&mut self) -> crate::Result<()> {
        self.loaded = false;
        Ok(())
    }

    fn status(&self) -> String {
        format!(
            "HelpPlugin v{} ({})",
            self.version(),
            if self.loaded { "loaded" } else { "unloaded" }
        )
    }
}
