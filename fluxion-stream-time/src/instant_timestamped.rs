// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::timer::Timer;
use fluxion_core::{HasTimestamp, Timestamped};
use std::cmp::Ordering;
use std::fmt::Debug;
use std::ops::Deref;

/// A timestamped value using a Timer's Instant type for monotonic time operations.
///
/// This type wraps a value with a timestamp from the provided Timer implementation,
/// implementing the `Timestamped` trait. This enables time-based
/// operators like `delay`, `debounce`, and `throttle`.
///
/// # Example
///
/// ```rust,no_run
/// # #[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
/// use fluxion_stream_time::{InstantTimestamped, TokioTimer};
/// use fluxion_stream_time::timer::Timer;
///
/// # #[cfg(not(target_arch = "wasm32"))]
/// # fn example() {
/// # let timer = TokioTimer;
/// let item: InstantTimestamped<i32, TokioTimer> = InstantTimestamped::new(42, timer.now());
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct InstantTimestamped<T, TM: Timer> {
    pub value: T,
    pub timestamp: TM::Instant,
}

impl<T, TM: Timer> InstantTimestamped<T, TM> {
    /// Creates a new timestamped value with the given timestamp.
    pub fn new(value: T, timestamp: TM::Instant) -> Self {
        Self { value, timestamp }
    }
}

impl<T, TM: Timer> HasTimestamp for InstantTimestamped<T, TM> {
    type Timestamp = TM::Instant;

    fn timestamp(&self) -> Self::Timestamp {
        self.timestamp
    }
}

impl<T, TM: Timer> Timestamped for InstantTimestamped<T, TM>
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

impl<T, TM: Timer> PartialEq for InstantTimestamped<T, TM>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.timestamp == other.timestamp && self.value == other.value
    }
}

impl<T, TM: Timer> Eq for InstantTimestamped<T, TM> where T: Eq {}

impl<T, TM: Timer> PartialOrd for InstantTimestamped<T, TM>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.timestamp.partial_cmp(&other.timestamp)
    }
}

impl<T, TM: Timer> Ord for InstantTimestamped<T, TM>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.timestamp.cmp(&other.timestamp)
    }
}

impl<T, TM: Timer> Deref for InstantTimestamped<T, TM> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
