//! Event Bus implementation for Nimbus
//!
//! This is the heart of our plugin system. Events flow through here
//! and plugins subscribe to what they care about.

use async_trait::async_trait;
use dashmap::DashMap;
use nimbus_types::events::{
    Event, EventBus as EventBusTrait, EventEnvelope, EventFilter, EventHandler, EventType,
};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

pub mod metrics;

/// In-memory event bus implementation
///
/// This is designed for single-instance deployments.
/// For multi-instance, we'd use Redis Pub/Sub or NATS.
pub struct InMemoryEventBus {
    /// Map of handler name to handler
    handlers: Arc<DashMap<String, Arc<Box<dyn EventHandler>>>>,
    /// Map of event type to interested handler names for quick lookup
    subscriptions: Arc<RwLock<DashMap<EventType, HashSet<String>>>>,
    /// Channel for event distribution
    event_sender: async_channel::Sender<EventEnvelope>,
    event_receiver: async_channel::Receiver<EventEnvelope>,
    /// Metrics collector
    metrics: Arc<metrics::EventBusMetrics>,
}

impl InMemoryEventBus {
    pub fn new(buffer_size: usize) -> Self {
        let (sender, receiver) = async_channel::bounded(buffer_size);
        
        Self {
            handlers: Arc::new(DashMap::new()),
            subscriptions: Arc::new(RwLock::new(DashMap::new())),
            event_sender: sender,
            event_receiver: receiver,
            metrics: Arc::new(metrics::EventBusMetrics::new()),
        }
    }

    /// Start the event bus processor
    pub fn start(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        let bus = self.clone();
        tokio::spawn(async move {
            info!("Event bus started");
            loop {
                match bus.event_receiver.recv().await {
                    Ok(envelope) => {
                        bus.process_event(envelope).await;
                    }
                    Err(_) => {
                        warn!("Event channel closed, shutting down event bus");
                        break;
                    }
                }
            }
        })
    }

    /// Process a single event
    async fn process_event(&self, envelope: EventEnvelope) {
        let event_type = Self::event_type(&envelope.event);
        debug!("Processing event: {:?}", event_type);
        
        // Record metrics
        self.metrics.event_received(event_type);
        let start = std::time::Instant::now();

        // Get handlers interested in this event
        let handler_names = {
            let subs = self.subscriptions.read().await;
            subs.get(&event_type)
                .map(|entry| entry.value().clone())
                .unwrap_or_default()
        };

        // Dispatch to all interested handlers
        let mut tasks = Vec::new();
        for name in handler_names {
            if let Some(handler_entry) = self.handlers.get(&name) {
                let handler = handler_entry.clone();
                let envelope_clone = envelope.clone();
                let metrics = self.metrics.clone();
                let handler_name = name.clone();
                
                // Check if event matches handler's filter
                if Self::matches_filter(&handler.filter(), &envelope_clone) {
                    tasks.push(tokio::spawn(async move {
                        debug!("Dispatching to handler: {}", handler_name);
                        let handler_start = std::time::Instant::now();
                        
                        match handler.handle(envelope_clone).await {
                            Ok(_) => {
                                metrics.handler_success(&handler_name);
                                debug!("Handler {} completed in {:?}", 
                                    handler_name, handler_start.elapsed());
                            }
                            Err(e) => {
                                metrics.handler_failure(&handler_name);
                                error!("Handler {} failed: {}", handler_name, e);
                            }
                        }
                    }));
                }
            }
        }

        // Wait for all handlers to complete (with timeout)
        let timeout = std::time::Duration::from_secs(30);
        let results = tokio::time::timeout(
            timeout,
            futures::future::join_all(tasks)
        ).await;

        match results {
            Ok(_) => {
                self.metrics.event_processed(event_type, start.elapsed());
                debug!("Event processing completed in {:?}", start.elapsed());
            }
            Err(_) => {
                self.metrics.event_timeout(event_type);
                error!("Event processing timed out after {:?}", timeout);
            }
        }
    }

    /// Determine event type from event
    fn event_type(event: &Event) -> EventType {
        match event {
            Event::Push { .. } => EventType::Push,
            Event::PullRequestOpened { .. } 
            | Event::PullRequestMerged { .. } 
            | Event::PullRequestClosed { .. } => EventType::PullRequest,
            Event::TagCreated { .. } => EventType::Tag,
            Event::RepositoryCreated { .. } 
            | Event::RepositoryDeleted { .. } => EventType::Repository,
            Event::ReviewRequested { .. } 
            | Event::ReviewSubmitted { .. } => EventType::Review,
            Event::CiRunStarted { .. } 
            | Event::CiRunCompleted { .. } => EventType::CiRun,
            _ => EventType::Push, // Default fallback
        }
    }

    /// Check if an event matches a handler's filter
    fn matches_filter(filter: &EventFilter, envelope: &EventEnvelope) -> bool {
        // Check event type filter
        if !filter.event_types.is_empty() {
            let event_type = Self::event_type(&envelope.event);
            if !filter.event_types.contains(&event_type) {
                return false;
            }
        }

        // Check repository filter
        if !filter.repositories.is_empty() {
            let repo_name = Self::extract_repository(&envelope.event);
            if let Some(repo) = repo_name {
                if !filter.repositories.contains(&repo) {
                    return false;
                }
            }
        }

        // Check branch filter (glob patterns)
        if !filter.branches.is_empty() {
            if let Some(branch) = Self::extract_branch(&envelope.event) {
                let matches = filter.branches.iter().any(|pattern| {
                    glob_match::glob_match(pattern, &branch)
                });
                if !matches {
                    return false;
                }
            }
        }

        true
    }

    /// Extract repository name from event
    fn extract_repository(event: &Event) -> Option<String> {
        match event {
            Event::Push { repository, .. } |
            Event::PullRequestOpened { repository, .. } |
            Event::PullRequestMerged { repository, .. } |
            Event::PullRequestClosed { repository, .. } |
            Event::TagCreated { repository, .. } |
            Event::RepositoryDeleted { repository, .. } |
            Event::CiRunStarted { repository, .. } |
            Event::CiRunCompleted { repository, .. } |
            Event::ReviewRequested { repository, .. } |
            Event::ReviewSubmitted { repository, .. } |
            Event::AiAnalysisRequested { repository, .. } |
            Event::AiAnalysisCompleted { repository, .. } => Some(repository.clone()),
            Event::RepositoryCreated { repository } => Some(repository.name.clone()),
        }
    }

    /// Extract branch from event
    fn extract_branch(event: &Event) -> Option<String> {
        match event {
            Event::Push { branch, .. } |
            Event::CiRunStarted { branch, .. } => Some(branch.clone()),
            Event::PullRequestOpened { from_branch, .. } => Some(from_branch.clone()),
            _ => None,
        }
    }
}

#[async_trait]
impl EventBusTrait for InMemoryEventBus {
    async fn publish(&self, event: EventEnvelope) -> Result<(), Box<dyn std::error::Error>> {
        self.event_sender.send(event).await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }

    async fn subscribe(
        &self,
        name: String,
        handler: Box<dyn EventHandler>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Registering handler: {}", name);
        
        // Store handler
        let handler = Arc::new(handler);
        self.handlers.insert(name.clone(), handler.clone());

        // Update subscription index for quick lookup
        let filter = handler.filter();
        let mut subs = self.subscriptions.write().await;
        
        if filter.event_types.is_empty() {
            // Subscribe to all event types
            for event_type in [
                EventType::Push,
                EventType::PullRequest,
                EventType::Tag,
                EventType::Repository,
                EventType::Review,
                EventType::CiRun,
            ] {
                subs.entry(event_type)
                    .or_insert_with(HashSet::new)
                    .insert(name.clone());
            }
        } else {
            // Subscribe to specific event types
            for event_type in &filter.event_types {
                subs.entry(*event_type)
                    .or_insert_with(HashSet::new)
                    .insert(name.clone());
            }
        }

        Ok(())
    }

    async fn unsubscribe(&self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        info!("Unregistering handler: {}", name);
        
        // Remove handler
        self.handlers.remove(name);

        // Remove from subscription index
        let mut subs = self.subscriptions.write().await;
        for (_, handlers) in subs.iter_mut() {
            handlers.remove(name);
        }

        Ok(())
    }

    async fn subscriber_count(&self) -> usize {
        self.handlers.len()
    }
}

// Re-export for convenience
pub use nimbus_types::events::{
    Event, EventEnvelope, EventFilter, EventHandler, EventMetadata, EventPriority, EventType,
};

// Add glob matching support
mod glob_match {
    pub fn glob_match(pattern: &str, text: &str) -> bool {
        // Simple glob implementation - in production use `glob` crate
        if pattern == "*" {
            return true;
        }
        if pattern.starts_with("*") && pattern.ends_with("*") {
            let inner = &pattern[1..pattern.len() - 1];
            return text.contains(inner);
        }
        if pattern.starts_with("*") {
            return text.ends_with(&pattern[1..]);
        }
        if pattern.ends_with("*") {
            return text.starts_with(&pattern[..pattern.len() - 1]);
        }
        pattern == text
    }
}

#[cfg(test)]
mod tests;

// Add missing dependencies
use futures;