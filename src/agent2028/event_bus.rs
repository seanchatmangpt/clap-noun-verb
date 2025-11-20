/// Cross-Tier Event Bus
///
/// Pub/Sub system enabling communication between 2028 and 2029+ systems,
/// coordinating distributed events across the trillion-agent ecosystem.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

/// Event type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    /// Agent lifecycle events
    AgentStarted,
    AgentFailed,
    AgentRecovered,
    /// Coordination events
    ConsensusRequired,
    VotingCompleted,
    /// Swarm events
    SwarmFormed,
    SwarmDisbanded,
    SwarmDecision,
    /// Resource events
    ResourceExhausted,
    ResourceAvailable,
    /// Failure recovery
    FailoverInitiated,
    FailoverCompleted,
}

/// Cross-tier event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub event_id: String,
    pub event_type: EventType,
    pub source_agent: String,
    pub timestamp: u64,
    pub data: String,
    pub priority: u32, // 1-10
    pub requires_response: bool,
}

impl Event {
    pub fn new(event_type: EventType, source_agent: String, data: String) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            event_id: Uuid::new_v4().to_string(),
            event_type,
            source_agent,
            timestamp,
            data,
            priority: 5,
            requires_response: false,
        }
    }
}

/// Event handler callback
pub type EventHandler = Arc<dyn Fn(&Event) + Send + Sync>;

/// Event subscription
#[derive(Clone)]
pub struct Subscription {
    pub subscription_id: String,
    pub agent_id: String,
    pub event_types: Vec<EventType>,
    pub handler_count: usize,
}

/// Central Event Bus for cross-tier communication
pub struct EventBus {
    tx: broadcast::Sender<Event>,
    subscriptions: Arc<RwLock<HashMap<String, Subscription>>>,
    event_history: Arc<RwLock<Vec<Event>>>,
    stats: Arc<RwLock<EventBusStats>>,
}

/// Event bus statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventBusStats {
    pub total_events: u64,
    pub total_subscriptions: u64,
    pub active_subscribers: usize,
    pub events_by_type: HashMap<String, u64>,
}

impl EventBus {
    pub fn new(buffer_size: usize) -> Self {
        let (tx, _rx) = broadcast::channel(buffer_size);

        Self {
            tx,
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            event_history: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(EventBusStats {
                total_events: 0,
                total_subscriptions: 0,
                active_subscribers: 0,
                events_by_type: HashMap::new(),
            })),
        }
    }

    /// Publish event to all subscribers
    pub async fn publish(&self, event: Event) -> Result<(), String> {
        // Record event
        let mut history = self.event_history.write().await;
        history.push(event.clone());

        // Update stats
        let mut stats = self.stats.write().await;
        stats.total_events += 1;
        *stats.events_by_type.entry(format!("{:?}", event.event_type)).or_insert(0) += 1;
        drop(stats);

        // Publish to subscribers
        if self.tx.receiver_count() > 0 {
            self.tx.send(event).map_err(|_| "Failed to publish event".to_string())?;
        }

        Ok(())
    }

    /// Subscribe to specific event types
    pub async fn subscribe(
        &self,
        agent_id: String,
        event_types: Vec<EventType>,
    ) -> (String, broadcast::Receiver<Event>) {
        let subscription_id = Uuid::new_v4().to_string();
        let handler_count = event_types.len();

        let subscription = Subscription {
            subscription_id: subscription_id.clone(),
            agent_id,
            event_types,
            handler_count,
        };

        let mut subs = self.subscriptions.write().await;
        subs.insert(subscription_id.clone(), subscription);

        let mut stats = self.stats.write().await;
        stats.total_subscriptions += 1;
        stats.active_subscribers = subs.len();
        drop(stats);

        let rx = self.tx.subscribe();
        (subscription_id, rx)
    }

    /// Unsubscribe from events
    pub async fn unsubscribe(&self, subscription_id: &str) {
        let mut subs = self.subscriptions.write().await;
        subs.remove(subscription_id);

        let mut stats = self.stats.write().await;
        stats.active_subscribers = subs.len();
    }

    /// Get event history
    pub async fn get_history(&self, limit: usize) -> Vec<Event> {
        let history = self.event_history.read().await;
        history.iter().rev().take(limit).cloned().collect()
    }

    /// Get statistics
    pub async fn stats(&self) -> EventBusStats {
        self.stats.read().await.clone()
    }

    /// Get subscriptions for agent
    pub async fn agent_subscriptions(&self, agent_id: &str) -> Vec<Subscription> {
        let subs = self.subscriptions.read().await;
        subs.values().filter(|s| s.agent_id == agent_id).cloned().collect()
    }

    /// Filter events by type
    pub async fn get_events_by_type(&self, event_type: EventType, limit: usize) -> Vec<Event> {
        let history = self.event_history.read().await;
        history.iter().filter(|e| e.event_type == event_type).rev().take(limit).cloned().collect()
    }
}

/// Event Handler Registry
pub struct EventHandlerRegistry {
    handlers: Arc<RwLock<HashMap<String, Vec<EventHandler>>>>, // event_type -> handlers
}

impl EventHandlerRegistry {
    pub fn new() -> Self {
        Self { handlers: Arc::new(RwLock::new(HashMap::new())) }
    }

    /// Register handler for event type
    pub async fn register(&self, event_type_name: String, handler: EventHandler) {
        let mut handlers = self.handlers.write().await;
        handlers.entry(event_type_name).or_insert_with(Vec::new).push(handler);
    }

    /// Get handlers for event type
    pub async fn get_handlers(&self, event_type_name: &str) -> Vec<EventHandler> {
        self.handlers.read().await.get(event_type_name).cloned().unwrap_or_default()
    }

    /// Execute all handlers for event
    pub async fn execute_handlers(&self, event: &Event) {
        let handlers = self.get_handlers(&format!("{:?}", event.event_type)).await;

        for handler in handlers {
            handler(event);
        }
    }
}

impl Default for EventHandlerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(1000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_event_creation() {
        let event = Event::new(
            EventType::AgentStarted,
            "agent-1".to_string(),
            "started successfully".to_string(),
        );

        assert_eq!(event.source_agent, "agent-1");
        assert_eq!(event.event_type, EventType::AgentStarted);
    }

    #[tokio::test]
    async fn test_event_bus_creation() {
        let bus = EventBus::new(100);
        let stats = bus.stats().await;

        assert_eq!(stats.total_events, 0);
        assert_eq!(stats.active_subscribers, 0);
    }

    #[tokio::test]
    async fn test_publish_event() {
        let bus = EventBus::new(100);

        let event = Event::new(EventType::AgentStarted, "agent-1".to_string(), "data".to_string());

        let result = bus.publish(event.clone()).await;
        assert!(result.is_ok());

        let history = bus.get_history(10).await;
        assert_eq!(history.len(), 1);
    }

    #[tokio::test]
    async fn test_subscription() {
        let bus = EventBus::new(100);

        let (_sub_id, _rx) = bus
            .subscribe("agent-1".to_string(), vec![EventType::AgentStarted, EventType::AgentFailed])
            .await;

        let stats = bus.stats().await;
        assert_eq!(stats.active_subscribers, 1);
    }

    #[tokio::test]
    async fn test_event_handler_registry() {
        let registry = EventHandlerRegistry::new();

        let handler: EventHandler = Arc::new(|_event| {
            // Handler logic
        });

        registry.register("AgentStarted".to_string(), handler).await;

        let handlers = registry.get_handlers("AgentStarted").await;
        assert_eq!(handlers.len(), 1);
    }
}
