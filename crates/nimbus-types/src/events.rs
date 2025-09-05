//! Event bus types and traits for plugin communication
//!
//! The event system is the heart of our plugin architecture.
//! Core emits events, plugins subscribe and react.

use std::hash::Hash;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use async_trait::async_trait;

use crate::{Commit, Repository};

/// Event subscription filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventFilter {
    /// Event types to receive (empty = all)
    pub event_types: Vec<EventType>,
    /// Repository names to filter (empty = all)
    pub repositories: Vec<String>,
    /// Branch patterns to match (glob patterns)
    pub branches: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EventType {
    Push,
    PullRequest,
    Tag,
    Repository,
    Review,
    CiRun,
}

/// Extended event with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub id: Uuid,
    pub timestamp: time::OffsetDateTime,
    pub event: Event,
    pub metadata: EventMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    /// Which plugin should handle this (if specific)
    pub target_plugins: Vec<String>,
    /// Priority for ordering
    pub priority: EventPriority,
    /// Should this event be persisted?
    pub persistent: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Ord, PartialOrd, Eq)]
pub enum EventPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Events that flow through the system
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    // Core Git Events
    Push {
        repository: String,
        branch: String,
        commits: Vec<Commit>,
        pusher: String,
    },

    PullRequestOpened {
        id: Uuid,
        repository: String,
        from_branch: String,
        to_branch: String,
        title: String,
        author: String,
    },

    PullRequestMerged {
        id: Uuid,
        repository: String,
        merge_commit: String,
    },

    PullRequestClosed {
        id: Uuid,
        repository: String,
    },

    TagCreated {
        repository: String,
        tag: String,
        target: String,
        tagger: String,
    },

    // Repository Events
    RepositoryCreated {
        repository: Repository,
    },

    RepositoryDeleted {
        repository: String,
    },

    // CI/CD Events (from plugins)
    CiRunStarted {
        id: Uuid,
        repository: String,
        branch: String,
        plugin: String,
    },

    CiRunCompleted {
        id: Uuid,
        repository: String,
        status: CiStatus,
        plugin: String,
    },

    // Review Events (from plugins)
    ReviewRequested {
        pull_request_id: Uuid,
        repository: String,
        reviewer: String,
        plugin: String,
    },

    ReviewSubmitted {
        pull_request_id: Uuid,
        repository: String,
        reviewer: String,
        status: ReviewStatus,
        plugin: String,
    },

    // AI Events (from plugins)
    AiAnalysisRequested {
        id: Uuid,
        repository: String,
        context: AnalysisContext,
        plugin: String,
    },

    AiAnalysisCompleted {
        id: Uuid,
        repository: String,
        suggestions: Vec<AiSuggestion>,
        plugin: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CiStatus {
    Success,
    Failure,
    Cancelled,
    Timeout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReviewStatus {
    Approved,
    RequestedChanges,
    Commented,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisContext {
    PullRequest { id: Uuid },
    File { path: String, commit: String },
    Repository,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSuggestion {
    pub file: String,
    pub line: Option<u32>,
    pub suggestion: String,
    pub severity: SuggestionSeverity,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SuggestionSeverity {
    Info,
    Warning,
    Error,
}

/// Trait for event handlers (implemented by plugins)
#[async_trait]
pub trait EventHandler: Send + Sync {
    /// Handle an event
    async fn handle(&self, event: EventEnvelope) -> Result<(), Box<dyn std::error::Error>>;

    /// Get the filter for events this handler wants
    fn filter(&self) -> EventFilter;

    /// Health check
    async fn health_check(&self) -> bool {
        true
    }
}

/// Trait for the event bus itself
#[async_trait]
pub trait EventBus: Send + Sync {
    /// Publish an event to all interested subscribers
    async fn publish(&self, event: EventEnvelope) -> Result<(), Box<dyn std::error::Error>>;

    /// Subscribe a handler to events
    async fn subscribe(
        &self,
        name: String,
        handler: Box<dyn EventHandler>,
    ) -> Result<(), Box<dyn std::error::Error>>;

    /// Unsubscribe a handler
    async fn unsubscribe(&self, name: &str) -> Result<(), Box<dyn std::error::Error>>;

    /// Get subscriber count
    async fn subscriber_count(&self) -> usize;
}
