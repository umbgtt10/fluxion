// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use core::cmp::Ordering;
use core::fmt::Debug;
use core::ops::Deref;
use fluxion_core::{HasTimestamp, Timestamped};
use fluxion_runtime::runtime::Runtime;

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
/// use fluxion_stream_time::InstantTimestamped;
/// # #[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
/// use fluxion_runtime::impls::tokio::{TokioRuntime, TokioTimer};
/// # #[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
/// use fluxion_runtime::timer::Timer;
///
/// # #[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
/// # fn main() {
/// let timer = TokioTimer;
/// let item: InstantTimestamped<i32, TokioRuntime> = InstantTimestamped::new(42, timer.now());
/// # }
/// # #[cfg(not(all(feature = "runtime-tokio", not(target_arch = "wasm32"))))]
/// # fn main() {}
/// ```
#[derive(Debug)]
pub struct InstantTimestamped<T, R: Runtime> {
    pub value: T,
    pub timestamp: R::Instant,
}

impl<T, R: Runtime> Clone for InstantTimestamped<T, R>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            timestamp: self.timestamp,
        }
    }
}

impl<T, R: Runtime> InstantTimestamped<T, R> {
    /// Creates a new timestamped value with the given timestamp.
    pub fn new(value: T, timestamp: R::Instant) -> Self {
        Self { value, timestamp }
    }
}

impl<T, R: Runtime> HasTimestamp for InstantTimestamped<T, R> {
    type Timestamp = R::Instant;

    fn timestamp(&self) -> Self::Timestamp {
        self.timestamp
    }
}

impl<T, R: Runtime> Timestamped for InstantTimestamped<T, R>
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

impl<T, R: Runtime> PartialEq for InstantTimestamped<T, R>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.timestamp == other.timestamp && self.value == other.value
    }
}

impl<T, R: Runtime> Eq for InstantTimestamped<T, R> where T: Eq {}

impl<T, R: Runtime> PartialOrd for InstantTimestamped<T, R>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.timestamp.partial_cmp(&other.timestamp)
    }
}

impl<T, R: Runtime> Ord for InstantTimestamped<T, R>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.timestamp.cmp(&other.timestamp)
    }
}

impl<T, R: Runtime> Deref for InstantTimestamped<T, R> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
