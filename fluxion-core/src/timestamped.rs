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
///     fn timestamp(&self) -> Self::Timestamp {
///         self.timestamp
///     }
///
///     fn with_timestamp(value: T, timestamp: Self::Timestamp) -> Self {
///         TimestampedEvent { value, timestamp }
///     }
///
///     fn with_fresh_timestamp(value: T) -> Self {
///         // In real implementation, get current timestamp
///         TimestampedEvent { value, timestamp: 0 }
///     }
///
///     fn into_inner(self) -> Self::Inner {
///         self.value
///     }
/// }
/// ```
pub trait Timestamped: Clone {
    /// The type of the inner value wrapped by this timestamped type
    type Inner: Clone;

    /// The type representing the timestamp
    type Timestamp: Ord + Copy + Send + Sync + std::fmt::Debug;

    /// Returns the timestamp value for this item.
    /// Stream operators use this to determine the order of items.
    fn timestamp(&self) -> Self::Timestamp;

    /// Creates a new instance wrapping the given value with the specified timestamp.
    fn with_timestamp(value: Self::Inner, timestamp: Self::Timestamp) -> Self;

    /// Creates a new instance wrapping the given value with a fresh timestamp.
    /// This is called by operators when emitting new values.
    /// Each implementation determines how to generate fresh timestamps
    /// (e.g., Utc::now() for chrono, atomic counter for u64, etc.)
    fn with_fresh_timestamp(value: Self::Inner) -> Self;

    /// Consumes self and returns the inner value.
    /// For wrapper types like `ChronoTimestamped<T>`, this extracts `T`.
    /// For domain types where `Inner = Self`, this typically returns `self`.
    fn into_inner(self) -> Self::Inner;
}
