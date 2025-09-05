//! Metrics collection for the event bus
//!
//! Tracks event processing performance, handler success rates, etc.

use std::time::Duration;

use prometheus::{
    register_counter_vec, register_histogram_vec, CounterVec, HistogramVec,
};

use nimbus_types::events::EventType;

pub struct EventBusMetrics {
    events_received: CounterVec,
    events_processed: HistogramVec,
    events_timeout: CounterVec,
    handler_success: CounterVec,
    handler_failure: CounterVec,
}

impl EventBusMetrics {
    pub fn new() -> Self {
        Self {
            events_received: register_counter_vec!(
                "nimbus_events_received_total",
                "Total number of events received",
                &["event_type"]
            ).unwrap_or_else(|_| {
                // In tests, metrics might already be registered
                CounterVec::new(
                    prometheus::Opts::new(
                        "nimbus_events_received_total",
                        "Total number of events received",
                    ),
                    &["event_type"],
                )
                .unwrap()
            }),
            
            events_processed: register_histogram_vec!(
                "nimbus_events_processing_duration_seconds",
                "Time taken to process events",
                &["event_type"]
            ).unwrap_or_else(|_| {
                HistogramVec::new(
                    prometheus::HistogramOpts::new(
                        "nimbus_events_processing_duration_seconds",
                        "Time taken to process events",
                    ),
                    &["event_type"],
                )
                .unwrap()
            }),
            
            events_timeout: register_counter_vec!(
                "nimbus_events_timeout_total",
                "Total number of events that timed out",
                &["event_type"]
            ).unwrap_or_else(|_| {
                CounterVec::new(
                    prometheus::Opts::new(
                        "nimbus_events_timeout_total",
                        "Total number of events that timed out",
                    ),
                    &["event_type"],
                )
                .unwrap()
            }),
            
            handler_success: register_counter_vec!(
                "nimbus_handler_success_total",
                "Total number of successful handler executions",
                &["handler"]
            ).unwrap_or_else(|_| {
                CounterVec::new(
                    prometheus::Opts::new(
                        "nimbus_handler_success_total",
                        "Total number of successful handler executions",
                    ),
                    &["handler"],
                )
                .unwrap()
            }),
            
            handler_failure: register_counter_vec!(
                "nimbus_handler_failure_total",
                "Total number of failed handler executions",
                &["handler"]
            ).unwrap_or_else(|_| {
                CounterVec::new(
                    prometheus::Opts::new(
                        "nimbus_handler_failure_total",
                        "Total number of failed handler executions",
                    ),
                    &["handler"],
                )
                .unwrap()
            }),
        }
    }

    pub fn event_received(&self, event_type: EventType) {
        self.events_received
            .with_label_values(&[&format!("{:?}", event_type)])
            .inc();
    }

    pub fn event_processed(&self, event_type: EventType, duration: Duration) {
        self.events_processed
            .with_label_values(&[&format!("{:?}", event_type)])
            .observe(duration.as_secs_f64());
    }

    pub fn event_timeout(&self, event_type: EventType) {
        self.events_timeout
            .with_label_values(&[&format!("{:?}", event_type)])
            .inc();
    }

    pub fn handler_success(&self, handler: &str) {
        self.handler_success
            .with_label_values(&[handler])
            .inc();
    }

    pub fn handler_failure(&self, handler: &str) {
        self.handler_failure
            .with_label_values(&[handler])
            .inc();
    }
}