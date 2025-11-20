// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

/// A trait for types that have an intrinsic timestamp for stream ordering.
///
/// This trait allows stream operators like `combine_latest` and `ordered_merge`
/// to work with any type that can provide a timestamp value and wraps an inner value.
///
/// The timestamp type is generic, allowing implementations to use different time
/// representations (monotonic counters, wall-clock time, etc.).
///
/// # Type Parameters
/// * `Inner` - The type of the value wrapped by this timestamped type
/// * `Timestamp` - The type representing the timestamp (must be `Ord + Copy`)
///
/// # Examples
///
/// ```
/// use fluxion_core::Timestamped;
///
/// #[derive(Clone, Debug)]
/// struct TimestampedEvent<T> {
///     value: T,
///     timestamp: u64,
/// }
///
/// impl<T: Clone> Timestamped for TimestampedEvent<T> {
///     type Inner = T;
///     type Timestamp = u64;
///
///     fn timestamp(&self) -> u64 {
///         self.timestamp
///     }
///
///     fn inner(&self) -> &T {
///         &self.value
///     }
///
///     fn with_timestamp(value: T, timestamp: u64) -> Self {
///         TimestampedEvent { value, timestamp }
///     }
///
///     fn with_fresh_timestamp(value: T) -> Self {
///         // In real implementation, get current timestamp
///         TimestampedEvent { value, timestamp: 0 }
///     }
/// }
/// ```
pub trait Timestamped: Clone {
    /// The type of the inner value wrapped by this timestamped type
    type Inner: Clone;

    /// Returns the timestamp value for this item.
    /// Stream operators use this to determine the order of items.
    fn timestamp(&self) -> u64;

    /// Gets a reference to the inner value.
    fn inner(&self) -> &Self::Inner;

    /// Creates a new instance wrapping the given value with the specified timestamp.
    fn with_timestamp(value: Self::Inner, timestamp: u64) -> Self;

    /// Creates a new instance wrapping the given value with a fresh timestamp.
    /// This is called by operators when emitting new values.
    fn with_fresh_timestamp(value: Self::Inner) -> Self;

    /// Gets the inner value, consuming the wrapper.
    fn into_inner(self) -> Self::Inner {
        // Default implementation using clone, can be overridden for efficiency
        self.inner().clone()
    }

    /// Creates a new instance with a different inner type.
    /// This is used by operators like `combine_latest` that transform the inner value.
    fn map_inner<F, U>(self, f: F, timestamp: u64) -> impl Timestamped<Inner = U>
    where
        F: FnOnce(Self::Inner) -> U,
        U: Clone,
        Self: Sized,
    {
        TimestampedWrapper {
            value: f(self.into_inner()),
            timestamp,
        }
    }
}

use std::sync::atomic::{AtomicU64, Ordering};

static TIMESTAMP_COUNTER: AtomicU64 = AtomicU64::new(1);

/// A simple wrapper type for Timestamped values with transformed inner types
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimestampedWrapper<T> {
    value: T,
    timestamp: u64,
}

impl<T: Clone> Timestamped for TimestampedWrapper<T> {
    type Inner = T;

    fn timestamp(&self) -> u64 {
        self.timestamp
    }

    fn inner(&self) -> &T {
        &self.value
    }

    fn with_timestamp(value: T, timestamp: u64) -> Self {
        Self { value, timestamp }
    }

    fn with_fresh_timestamp(value: T) -> Self {
        let timestamp = TIMESTAMP_COUNTER.fetch_add(1, Ordering::SeqCst);
        Self { value, timestamp }
    }

    fn into_inner(self) -> T {
        self.value
    }
}

impl<T: Clone + PartialOrd> PartialOrd for TimestampedWrapper<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.timestamp.partial_cmp(&other.timestamp)
    }
}

impl<T: Clone + Ord> Ord for TimestampedWrapper<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.timestamp.cmp(&other.timestamp)
    }
}
