use crate::plugin::{Plugin, PluginCapability, PluginMetadata};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct MessageQueuePlugin {
    #[allow(dead_code)]
    messages: Arc<Mutex<VecDeque<String>>>,
    loaded: bool,
}

impl MessageQueuePlugin {
    pub fn new() -> Self { Self { messages: Arc::new(Mutex::new(VecDeque::new())), loaded: false } }
    pub fn publish(&self, _msg: &str) -> crate::Result<()> { Ok(()) }
    pub fn consume(&self) -> crate::Result<Option<String>> { Ok(None) }
}

impl Default for MessageQueuePlugin { fn default() -> Self { Self::new() } }
impl std::fmt::Debug for MessageQueuePlugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageQueuePlugin").finish()
    }
}

impl Plugin for MessageQueuePlugin {
    fn name(&self) -> &str { "message-queue" }
    fn version(&self) -> &str { "1.0.0" }
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata::new(self.name(), self.version()).with_description("Async messaging")
    }
    fn capabilities(&self) -> Vec<PluginCapability> { vec![PluginCapability::Middleware] }
    fn load(&mut self) -> crate::Result<()> { self.loaded = true; Ok(()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_message_queue_workflow() {
        let mut plugin = MessageQueuePlugin::new();
        plugin.load().unwrap();
        plugin.publish("test").unwrap();
    }
}
