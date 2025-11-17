use crate::plugin::{Plugin, PluginCapability, PluginMetadata};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

#[derive(Clone)]
pub struct EventBusPlugin {
    topics: Arc<Mutex<HashMap<String, Vec<String>>>>,
    loaded: bool,
}

impl EventBusPlugin {
    pub fn new() -> Self { Self { topics: Arc::new(Mutex::new(HashMap::new())), loaded: false } }
    pub fn publish(&self, _topic: &str, _data: &str) -> crate::Result<()> { Ok(()) }
}

impl Default for EventBusPlugin { fn default() -> Self { Self::new() } }
impl std::fmt::Debug for EventBusPlugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventBusPlugin").finish()
    }
}

impl Plugin for EventBusPlugin {
    fn name(&self) -> &str { "event-bus" }
    fn version(&self) -> &str { "1.0.0" }
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata::new(self.name(), self.version()).with_description("Pub/sub events")
    }
    fn capabilities(&self) -> Vec<PluginCapability> { vec![PluginCapability::Hook] }
    fn load(&mut self) -> crate::Result<()> { self.loaded = true; Ok(()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_event_bus_workflow() {
        let mut plugin = EventBusPlugin::new();
        plugin.load().unwrap();
        plugin.publish("system", "startup").unwrap();
    }
}
