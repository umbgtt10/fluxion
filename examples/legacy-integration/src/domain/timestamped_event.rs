// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use super::events::UnifiedEvent;
use fluxion_rx::{HasTimestamp, Timestamped};
use std::cmp::Ordering;
use std::sync::atomic::{AtomicU64, Ordering::SeqCst};

static EVENT_COUNTER: AtomicU64 = AtomicU64::new(0);

/// A wrapper that adds automatic timestamping to UnifiedEvent for temporal ordering.
///
/// Uses a monotonically increasing counter to establish event ordering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimestampedEvent {
    pub event: UnifiedEvent,
    timestamp: u64,
}

impl TimestampedEvent {
    /// Creates a new timestamped event with an automatically assigned counter value.
    pub fn new(event: UnifiedEvent) -> Self {
        Self {
            event,
            timestamp: EVENT_COUNTER.fetch_add(1, SeqCst),
        }
    }

    pub fn with_timestamp(event: UnifiedEvent, timestamp: u64) -> Self {
        Self { event, timestamp }
    }

    #[allow(dead_code)]
    pub fn into_inner(self) -> UnifiedEvent {
        self.event
    }
}

impl HasTimestamp for TimestampedEvent {
    type Timestamp = u64;

    fn timestamp(&self) -> Self::Timestamp {
        self.timestamp
    }
}

impl Timestamped for TimestampedEvent {
    type Inner = UnifiedEvent;

    fn into_inner(self) -> Self::Inner {
        self.event
    }

    fn with_timestamp(event: Self::Inner, timestamp: Self::Timestamp) -> Self {
        Self::with_timestamp(event, timestamp)
    }

    fn with_fresh_timestamp(event: Self::Inner) -> Self {
        Self::new(event)
    }
}

impl PartialOrd for TimestampedEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TimestampedEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        self.timestamp.cmp(&other.timestamp)
    }
}

impl std::fmt::Display for TimestampedEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.timestamp, self.event)
    }
}
