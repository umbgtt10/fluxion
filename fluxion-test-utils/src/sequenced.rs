// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

use fluxion_core::{HasTimestamp, Timestamped};
use std::cmp::Ordering;
use std::sync::atomic::{AtomicU64, Ordering::SeqCst};

static GLOBAL_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// A wrapper that adds automatic sequencing to any value for temporal ordering.
///
/// Uses a monotonically increasing sequence counter to establish a total ordering of events.
/// The sequence is assigned when the value is created.
#[derive(Debug, Clone)]
pub struct Sequenced<T> {
    pub value: T,
    timestamp: u64,
}

impl<T: PartialEq> PartialEq for Sequenced<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.timestamp == other.timestamp
    }
}

impl<T: Eq> Eq for Sequenced<T> {}

impl<T> Sequenced<T> {
    /// Creates a new timestamped value with an automatically assigned sequence number.
    pub fn new(value: T) -> Self {
        Self {
            value,
            timestamp: GLOBAL_SEQUENCE.fetch_add(1, SeqCst),
        }
    }

    pub fn with_timestamp(value: T, timestamp: u64) -> Self {
        Self { value, timestamp }
    }

    pub fn into_inner(self) -> T {
        self.value
    }
}

impl<T> From<(T, u64)> for Sequenced<T> {
    fn from((value, timestamp): (T, u64)) -> Self {
        Self { value, timestamp }
    }
}

impl<T> HasTimestamp for Sequenced<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Timestamp = u64;

    fn timestamp(&self) -> Self::Timestamp {
        self.timestamp
    }
}

impl<T> Timestamped for Sequenced<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Inner = T;

    fn into_inner(self) -> Self::Inner {
        Self::into_inner(self)
    }

    fn with_timestamp(value: Self::Inner, timestamp: Self::Timestamp) -> Self {
        Self::with_timestamp(value, timestamp)
    }
}

impl<T> PartialOrd for Sequenced<T>
where
    T: Eq,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Sequenced<T>
where
    T: Eq,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.timestamp.cmp(&other.timestamp)
    }
}
