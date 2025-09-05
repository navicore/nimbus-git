//! Tests for the event bus

use std::sync::atomic::{AtomicUsize, Ordering};

use super::*;
use uuid::Uuid;

/// Test handler that counts events
struct CountingHandler {
    count: Arc<AtomicUsize>,
    filter: EventFilter,
}

impl CountingHandler {
    fn new(filter: EventFilter) -> Self {
        Self {
            count: Arc::new(AtomicUsize::new(0)),
            filter,
        }
    }

    fn get_count(&self) -> usize {
        self.count.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl EventHandler for CountingHandler {
    async fn handle(&self, _event: EventEnvelope) -> Result<(), Box<dyn std::error::Error>> {
        self.count.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    fn filter(&self) -> EventFilter {
        self.filter.clone()
    }
}

/// Test handler that fails
struct FailingHandler;

#[async_trait]
impl EventHandler for FailingHandler {
    async fn handle(&self, _event: EventEnvelope) -> Result<(), Box<dyn std::error::Error>> {
        Err("Test failure".into())
    }

    fn filter(&self) -> EventFilter {
        EventFilter {
            event_types: vec![],
            repositories: vec![],
            branches: vec![],
        }
    }
}

#[tokio::test]
async fn test_basic_publish_subscribe() {
    let bus = Arc::new(InMemoryEventBus::new(100));
    let _handle = bus.clone().start();

    // Create handler
    let handler = CountingHandler::new(EventFilter {
        event_types: vec![EventType::Push],
        repositories: vec![],
        branches: vec![],
    });
    let counter = handler.count.clone();

    // Subscribe
    bus.subscribe("test_handler".to_string(), Box::new(handler))
        .await
        .unwrap();

    // Publish push event
    let event = EventEnvelope {
        id: Uuid::new_v4(),
        timestamp: time::OffsetDateTime::now_utc(),
        event: Event::Push {
            repository: "test-repo".to_string(),
            branch: "main".to_string(),
            commits: vec![],
            pusher: "test-user".to_string(),
        },
        metadata: EventMetadata {
            target_plugins: vec![],
            priority: EventPriority::Normal,
            persistent: false,
        },
    };

    bus.publish(event).await.unwrap();

    // Wait for processing
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Check handler was called
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn test_multiple_subscribers() {
    let bus = Arc::new(InMemoryEventBus::new(100));
    let _handle = bus.clone().start();

    // Create multiple handlers
    let handler1 = CountingHandler::new(EventFilter {
        event_types: vec![EventType::Push],
        repositories: vec![],
        branches: vec![],
    });
    let counter1 = handler1.count.clone();

    let handler2 = CountingHandler::new(EventFilter {
        event_types: vec![EventType::Push],
        repositories: vec![],
        branches: vec![],
    });
    let counter2 = handler2.count.clone();

    // Subscribe both
    bus.subscribe("handler1".to_string(), Box::new(handler1))
        .await
        .unwrap();
    bus.subscribe("handler2".to_string(), Box::new(handler2))
        .await
        .unwrap();

    // Publish event
    let event = EventEnvelope {
        id: Uuid::new_v4(),
        timestamp: time::OffsetDateTime::now_utc(),
        event: Event::Push {
            repository: "test-repo".to_string(),
            branch: "main".to_string(),
            commits: vec![],
            pusher: "test-user".to_string(),
        },
        metadata: EventMetadata {
            target_plugins: vec![],
            priority: EventPriority::Normal,
            persistent: false,
        },
    };

    bus.publish(event).await.unwrap();

    // Wait for processing
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Both handlers should be called
    assert_eq!(counter1.load(Ordering::SeqCst), 1);
    assert_eq!(counter2.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn test_event_filtering() {
    let bus = Arc::new(InMemoryEventBus::new(100));
    let _handle = bus.clone().start();

    // Handler for push events only
    let push_handler = CountingHandler::new(EventFilter {
        event_types: vec![EventType::Push],
        repositories: vec![],
        branches: vec![],
    });
    let push_counter = push_handler.count.clone();

    // Handler for PR events only
    let pr_handler = CountingHandler::new(EventFilter {
        event_types: vec![EventType::PullRequest],
        repositories: vec![],
        branches: vec![],
    });
    let pr_counter = pr_handler.count.clone();

    // Subscribe
    bus.subscribe("push_handler".to_string(), Box::new(push_handler))
        .await
        .unwrap();
    bus.subscribe("pr_handler".to_string(), Box::new(pr_handler))
        .await
        .unwrap();

    // Publish push event
    let push_event = EventEnvelope {
        id: Uuid::new_v4(),
        timestamp: time::OffsetDateTime::now_utc(),
        event: Event::Push {
            repository: "test-repo".to_string(),
            branch: "main".to_string(),
            commits: vec![],
            pusher: "test-user".to_string(),
        },
        metadata: EventMetadata {
            target_plugins: vec![],
            priority: EventPriority::Normal,
            persistent: false,
        },
    };

    // Publish PR event
    let pr_event = EventEnvelope {
        id: Uuid::new_v4(),
        timestamp: time::OffsetDateTime::now_utc(),
        event: Event::PullRequestOpened {
            id: Uuid::new_v4(),
            repository: "test-repo".to_string(),
            from_branch: "feature".to_string(),
            to_branch: "main".to_string(),
            title: "Test PR".to_string(),
            author: "test-user".to_string(),
        },
        metadata: EventMetadata {
            target_plugins: vec![],
            priority: EventPriority::Normal,
            persistent: false,
        },
    };

    bus.publish(push_event).await.unwrap();
    bus.publish(pr_event).await.unwrap();

    // Wait for processing
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Check correct handlers were called
    assert_eq!(push_counter.load(Ordering::SeqCst), 1);
    assert_eq!(pr_counter.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn test_repository_filtering() {
    let bus = Arc::new(InMemoryEventBus::new(100));
    let _handle = bus.clone().start();

    // Handler for specific repository
    let handler = CountingHandler::new(EventFilter {
        event_types: vec![],
        repositories: vec!["important-repo".to_string()],
        branches: vec![],
    });
    let counter = handler.count.clone();

    bus.subscribe("repo_handler".to_string(), Box::new(handler))
        .await
        .unwrap();

    // Publish to matching repo
    let event1 = EventEnvelope {
        id: Uuid::new_v4(),
        timestamp: time::OffsetDateTime::now_utc(),
        event: Event::Push {
            repository: "important-repo".to_string(),
            branch: "main".to_string(),
            commits: vec![],
            pusher: "user".to_string(),
        },
        metadata: EventMetadata {
            target_plugins: vec![],
            priority: EventPriority::Normal,
            persistent: false,
        },
    };

    // Publish to non-matching repo
    let event2 = EventEnvelope {
        id: Uuid::new_v4(),
        timestamp: time::OffsetDateTime::now_utc(),
        event: Event::Push {
            repository: "other-repo".to_string(),
            branch: "main".to_string(),
            commits: vec![],
            pusher: "user".to_string(),
        },
        metadata: EventMetadata {
            target_plugins: vec![],
            priority: EventPriority::Normal,
            persistent: false,
        },
    };

    bus.publish(event1).await.unwrap();
    bus.publish(event2).await.unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Only one event should match
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn test_branch_filtering() {
    let bus = Arc::new(InMemoryEventBus::new(100));
    let _handle = bus.clone().start();

    // Handler for main branch only
    let handler = CountingHandler::new(EventFilter {
        event_types: vec![],
        repositories: vec![],
        branches: vec!["main".to_string()],
    });
    let counter = handler.count.clone();

    bus.subscribe("branch_handler".to_string(), Box::new(handler))
        .await
        .unwrap();

    // Push to main
    let main_event = EventEnvelope {
        id: Uuid::new_v4(),
        timestamp: time::OffsetDateTime::now_utc(),
        event: Event::Push {
            repository: "repo".to_string(),
            branch: "main".to_string(),
            commits: vec![],
            pusher: "user".to_string(),
        },
        metadata: EventMetadata {
            target_plugins: vec![],
            priority: EventPriority::Normal,
            persistent: false,
        },
    };

    // Push to feature branch
    let feature_event = EventEnvelope {
        id: Uuid::new_v4(),
        timestamp: time::OffsetDateTime::now_utc(),
        event: Event::Push {
            repository: "repo".to_string(),
            branch: "feature".to_string(),
            commits: vec![],
            pusher: "user".to_string(),
        },
        metadata: EventMetadata {
            target_plugins: vec![],
            priority: EventPriority::Normal,
            persistent: false,
        },
    };

    bus.publish(main_event).await.unwrap();
    bus.publish(feature_event).await.unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Only main branch event should match
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn test_glob_branch_filtering() {
    let bus = Arc::new(InMemoryEventBus::new(100));
    let _handle = bus.clone().start();

    // Handler for feature/* branches
    let handler = CountingHandler::new(EventFilter {
        event_types: vec![],
        repositories: vec![],
        branches: vec!["feature/*".to_string()],
    });
    let counter = handler.count.clone();

    bus.subscribe("glob_handler".to_string(), Box::new(handler))
        .await
        .unwrap();

    // Matching branches
    for branch in ["feature/auth", "feature/ui", "feature/api"] {
        let event = EventEnvelope {
            id: Uuid::new_v4(),
            timestamp: time::OffsetDateTime::now_utc(),
            event: Event::Push {
                repository: "repo".to_string(),
                branch: branch.to_string(),
                commits: vec![],
                pusher: "user".to_string(),
            },
            metadata: EventMetadata {
                target_plugins: vec![],
                priority: EventPriority::Normal,
                persistent: false,
            },
        };
        bus.publish(event).await.unwrap();
    }

    // Non-matching branch
    let main_event = EventEnvelope {
        id: Uuid::new_v4(),
        timestamp: time::OffsetDateTime::now_utc(),
        event: Event::Push {
            repository: "repo".to_string(),
            branch: "main".to_string(),
            commits: vec![],
            pusher: "user".to_string(),
        },
        metadata: EventMetadata {
            target_plugins: vec![],
            priority: EventPriority::Normal,
            persistent: false,
        },
    };
    bus.publish(main_event).await.unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Should match 3 feature branches
    assert_eq!(counter.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn test_handler_failure_doesnt_affect_others() {
    let bus = Arc::new(InMemoryEventBus::new(100));
    let _handle = bus.clone().start();

    // Good handler
    let good_handler = CountingHandler::new(EventFilter {
        event_types: vec![],
        repositories: vec![],
        branches: vec![],
    });
    let counter = good_handler.count.clone();

    // Subscribe both
    bus.subscribe("good".to_string(), Box::new(good_handler))
        .await
        .unwrap();
    bus.subscribe("bad".to_string(), Box::new(FailingHandler))
        .await
        .unwrap();

    // Publish event
    let event = EventEnvelope {
        id: Uuid::new_v4(),
        timestamp: time::OffsetDateTime::now_utc(),
        event: Event::Push {
            repository: "repo".to_string(),
            branch: "main".to_string(),
            commits: vec![],
            pusher: "user".to_string(),
        },
        metadata: EventMetadata {
            target_plugins: vec![],
            priority: EventPriority::Normal,
            persistent: false,
        },
    };

    bus.publish(event).await.unwrap();
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Good handler should still be called despite bad handler failing
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn test_unsubscribe() {
    let bus = Arc::new(InMemoryEventBus::new(100));
    let _handle = bus.clone().start();

    let handler = CountingHandler::new(EventFilter {
        event_types: vec![],
        repositories: vec![],
        branches: vec![],
    });
    let counter = handler.count.clone();

    // Subscribe
    bus.subscribe("test".to_string(), Box::new(handler))
        .await
        .unwrap();

    // Publish first event
    let event1 = EventEnvelope {
        id: Uuid::new_v4(),
        timestamp: time::OffsetDateTime::now_utc(),
        event: Event::Push {
            repository: "repo".to_string(),
            branch: "main".to_string(),
            commits: vec![],
            pusher: "user".to_string(),
        },
        metadata: EventMetadata {
            target_plugins: vec![],
            priority: EventPriority::Normal,
            persistent: false,
        },
    };
    bus.publish(event1).await.unwrap();
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    assert_eq!(counter.load(Ordering::SeqCst), 1);

    // Unsubscribe
    bus.unsubscribe("test").await.unwrap();

    // Publish second event
    let event2 = EventEnvelope {
        id: Uuid::new_v4(),
        timestamp: time::OffsetDateTime::now_utc(),
        event: Event::Push {
            repository: "repo".to_string(),
            branch: "main".to_string(),
            commits: vec![],
            pusher: "user".to_string(),
        },
        metadata: EventMetadata {
            target_plugins: vec![],
            priority: EventPriority::Normal,
            persistent: false,
        },
    };
    bus.publish(event2).await.unwrap();
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Count should still be 1
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}