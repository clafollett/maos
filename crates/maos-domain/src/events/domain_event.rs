use crate::value_objects::{AgentId, SessionId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use uuid::Uuid;

/// Macro to implement the as_json method for DomainEvent implementations
macro_rules! impl_domain_event_as_json {
    () => {
        fn as_json(&self) -> Result<String, EventError> {
            serde_json::to_string(self)
                .map_err(|e| EventError::SerializationError(e.to_string()))
        }
    };
}

pub(crate) use impl_domain_event_as_json;

/// Core domain event trait for event sourcing and pub/sub patterns
pub trait DomainEvent: Debug + Clone + Send + Sync {
    /// Unique identifier for this event instance
    fn event_id(&self) -> Uuid;
    
    /// Type identifier for the event (e.g., "SessionCreated", "AgentSpawned")
    fn event_type(&self) -> &'static str;
    
    /// Aggregate identifier that generated this event
    fn aggregate_id(&self) -> String;
    
    /// Version of the aggregate when this event was created
    fn aggregate_version(&self) -> u64;
    
    /// Timestamp when the event occurred
    fn occurred_at(&self) -> DateTime<Utc>;
    
    /// Optional metadata for correlation and debugging
    fn metadata(&self) -> &HashMap<String, String>;
    
    /// Serialize the event to JSON for persistence/transmission
    fn as_json(&self) -> Result<String, EventError>;
}

/// Event dispatcher for handling domain events
/// Future-ready for pub/sub integration (RabbitMQ, Redis, etc.)
#[derive(Debug, Clone)]
pub struct EventDispatcher {
    handlers: Vec<Box<dyn EventHandler>>,
}

impl EventDispatcher {
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    /// Register an event handler
    pub fn register_handler(&mut self, handler: Box<dyn EventHandler>) {
        self.handlers.push(handler);
    }

    /// Dispatch event to all registered handlers
    pub async fn dispatch<T: DomainEvent>(&self, event: T) -> Result<(), EventError> {
        for handler in &self.handlers {
            handler.handle(&event).await?;
        }
        Ok(())
    }

    /// Batch dispatch multiple events in order
    pub async fn dispatch_batch(&self, events: Vec<Box<dyn DomainEvent>>) -> Result<(), EventError> {
        for event in events {
            for handler in &self.handlers {
                handler.handle(event.as_ref()).await?;
            }
        }
        Ok(())
    }
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Event handler trait for processing domain events
#[async_trait::async_trait]
pub trait EventHandler: Send + Sync {
    async fn handle(&self, event: &dyn DomainEvent) -> Result<(), EventError>;
    fn can_handle(&self, event_type: &str) -> bool;
}

/// Errors that can occur during event processing
#[derive(Debug, thiserror::Error)]
pub enum EventError {
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Handler error: {0}")]
    HandlerError(String),
    
    #[error("Event validation error: {0}")]
    ValidationError(String),
    
    #[error("Dispatch error: {0}")]
    DispatchError(String),
}

/// Base event structure with common fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseEvent {
    pub event_id: Uuid,
    pub aggregate_id: String,
    pub aggregate_version: u64,
    pub occurred_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

impl BaseEvent {
    pub fn new(aggregate_id: String, aggregate_version: u64) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            aggregate_id,
            aggregate_version,
            occurred_at: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Event store trait for persistence (future implementation)
#[async_trait::async_trait]
pub trait EventStore: Send + Sync {
    async fn append_event(&self, event: &dyn DomainEvent) -> Result<(), EventError>;
    async fn append_events(&self, events: Vec<&dyn DomainEvent>) -> Result<(), EventError>;
    async fn get_events(&self, aggregate_id: &str) -> Result<Vec<Box<dyn DomainEvent>>, EventError>;
    async fn get_events_from_version(
        &self,
        aggregate_id: &str,
        from_version: u64,
    ) -> Result<Vec<Box<dyn DomainEvent>>, EventError>;
}

/// In-memory event store for testing and development
#[derive(Debug, Default)]
pub struct InMemoryEventStore {
    events: std::sync::RwLock<Vec<String>>, // JSON serialized events
}

#[async_trait::async_trait]
impl EventStore for InMemoryEventStore {
    async fn append_event(&self, event: &dyn DomainEvent) -> Result<(), EventError> {
        let json = serde_json::to_string(&event)
            .map_err(|e| EventError::SerializationError(e.to_string()))?;
        
        let mut events = self.events.write().unwrap();
        events.push(json);
        Ok(())
    }

    async fn append_events(&self, events: Vec<&dyn DomainEvent>) -> Result<(), EventError> {
        for event in events {
            self.append_event(event).await?;
        }
        Ok(())
    }

    async fn get_events(&self, aggregate_id: &str) -> Result<Vec<Box<dyn DomainEvent>>, EventError> {
        // TODO: Implement event deserialization and filtering
        // This is a placeholder for the interface
        let _events = self.events.read().unwrap();
        // Filter by aggregate_id and deserialize
        Ok(Vec::new())
    }

    async fn get_events_from_version(
        &self,
        aggregate_id: &str,
        _from_version: u64,
    ) -> Result<Vec<Box<dyn DomainEvent>>, EventError> {
        // TODO: Implement event deserialization and filtering
        self.get_events(aggregate_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestEvent {
        #[serde(flatten)]
        base: BaseEvent,
        test_data: String,
    }

    impl DomainEvent for TestEvent {
        fn event_id(&self) -> Uuid {
            self.base.event_id
        }

        fn event_type(&self) -> &'static str {
            "TestEvent"
        }

        fn aggregate_id(&self) -> String {
            self.base.aggregate_id.clone()
        }

        fn aggregate_version(&self) -> u64 {
            self.base.aggregate_version
        }

        fn occurred_at(&self) -> DateTime<Utc> {
            self.base.occurred_at
        }

        fn metadata(&self) -> &HashMap<String, String> {
            &self.base.metadata
        }

        impl_domain_event_as_json!();
    }

    #[tokio::test]
    async fn test_event_dispatcher_creation() {
        let dispatcher = EventDispatcher::new();
        assert_eq!(dispatcher.handlers.len(), 0);
    }

    #[tokio::test]
    async fn test_base_event_creation() {
        let event = BaseEvent::new("test-aggregate-1".to_string(), 1);
        
        assert_eq!(event.aggregate_id, "test-aggregate-1");
        assert_eq!(event.aggregate_version, 1);
        assert!(event.metadata.is_empty());
    }

    #[tokio::test]
    async fn test_base_event_with_metadata() {
        let event = BaseEvent::new("test-aggregate-1".to_string(), 1)
            .with_metadata("correlation_id".to_string(), "12345".to_string());
        
        assert_eq!(event.metadata.get("correlation_id"), Some(&"12345".to_string()));
    }

    #[tokio::test]
    async fn test_test_event_domain_event_trait() {
        let test_event = TestEvent {
            base: BaseEvent::new("test-aggregate-1".to_string(), 1),
            test_data: "test value".to_string(),
        };

        assert_eq!(test_event.event_type(), "TestEvent");
        assert_eq!(test_event.aggregate_id(), "test-aggregate-1");
        assert_eq!(test_event.aggregate_version(), 1);
    }

    #[tokio::test]
    async fn test_in_memory_event_store() {
        let store = InMemoryEventStore::default();
        let test_event = TestEvent {
            base: BaseEvent::new("test-aggregate-1".to_string(), 1),
            test_data: "test value".to_string(),
        };

        // Test append_event
        let result = store.append_event(&test_event).await;
        assert!(result.is_ok());

        // Check that event was stored
        let events = store.events.read().unwrap();
        assert_eq!(events.len(), 1);
    }
}