// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{HasTimestamp, Timestamped};
use std::cmp::Ordering;
use std::ops::Deref;
use std::time::Instant;

/// A timestamped value using std::time::Instant for monotonic time operations.
///
/// This type wraps a value with an `Instant` timestamp, implementing the `Timestamped` trait
/// with `std::time::Instant` as the timestamp type. This enables time-based
/// operators like `delay`, `debounce`, and `throttle`.
///
/// # Example
///
/// ```rust
/// use fluxion_stream_time::InstantTimestamped;
/// use std::time::Instant;
///
/// let item = InstantTimestamped::new(42, Instant::now());
/// let another = InstantTimestamped::now("hello");
/// ```
#[derive(Debug, Clone)]
pub struct InstantTimestamped<T> {
    pub value: T,
    pub timestamp: Instant,
}

impl<T> InstantTimestamped<T> {
    /// Creates a new timestamped value with the given timestamp.
    pub fn new(value: T, timestamp: Instant) -> Self {
        Self { value, timestamp }
    }

    /// Creates a new timestamped value with the current UTC time.
    pub fn now(value: T) -> Self {
        Self {
            value,
            timestamp: Instant::now(),
        }
    }
}

impl<T> HasTimestamp for InstantTimestamped<T> {
    type Timestamp = Instant;

    fn timestamp(&self) -> Self::Timestamp {
        self.timestamp
    }
}

impl<T> Timestamped for InstantTimestamped<T>
where
    T: Clone,
{
    type Inner = T;

    fn into_inner(self) -> Self::Inner {
        self.value
    }

    fn with_timestamp(inner: Self::Inner, timestamp: Self::Timestamp) -> Self {
        Self::new(inner, timestamp)
    }
}

impl<T> PartialEq for InstantTimestamped<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.timestamp == other.timestamp && self.value == other.value
    }
}

impl<T> Eq for InstantTimestamped<T> where T: Eq {}

impl<T> PartialOrd for InstantTimestamped<T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.timestamp.partial_cmp(&other.timestamp)
    }
}

impl<T> Ord for InstantTimestamped<T>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.timestamp.cmp(&other.timestamp)
    }
}

impl<T> Deref for InstantTimestamped<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
