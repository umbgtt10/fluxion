// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{CompareByInner, Timestamped};
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

// Allow converting () into any Sequenced<T> for seed streams
impl<T> From<()> for Sequenced<T>
where
    T: Default + Clone + Send + Sync + 'static,
{
    fn from(_: ()) -> Self {
        Self::new(T::default())
    }
}

impl<T> Timestamped for Sequenced<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Inner = T;
    type Timestamp = u64;

    fn into_inner(self) -> Self::Inner {
        Self::into_inner(self)
    }

    fn timestamp(&self) -> Self::Timestamp {
        self.timestamp
    }

    fn with_timestamp(value: Self::Inner, timestamp: Self::Timestamp) -> Self {
        Self::with_timestamp(value, timestamp)
    }

    fn with_fresh_timestamp(value: Self::Inner) -> Self {
        Self::new(value)
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

impl<T: Ord> CompareByInner for Sequenced<T> {
    fn cmp_inner(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }
}
