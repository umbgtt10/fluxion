// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

/// A trait for types that have a timestamp value.
///
/// This is the minimal trait for types that can participate in time-ordered
/// stream operations. Types only need to provide a timestamp - they don't
/// need to support construction or deconstruction.
///
/// Use this trait when you only need to read timestamps (e.g., for ordering).
/// Use [`Timestamped`] when you need to construct new instances from values.
///
/// # Type Parameters
/// * `Timestamp` - The type representing the timestamp (must be `Ord + Copy`)
///
/// # Examples
///
/// ```
/// use fluxion_core::HasTimestamp;
///
/// #[derive(Clone, Debug)]
/// struct Event {
///     data: String,
///     time: u64,
/// }
///
/// impl HasTimestamp for Event {
///     type Timestamp = u64;
///
///     fn timestamp(&self) -> u64 {
///         self.time
///     }
/// }
/// ```
pub trait HasTimestamp {
    /// The type representing the timestamp
    type Timestamp: Ord + Copy + Send + Sync + std::fmt::Debug;

    /// Returns the timestamp value for this item.
    /// Stream operators use this to determine the order of items.
    fn timestamp(&self) -> Self::Timestamp;
}

/// A trait for types that have an intrinsic timestamp for stream ordering.
///
/// This trait extends [`HasTimestamp`] with the ability to construct and
/// deconstruct timestamped wrapper types. Use this for wrapper types like
/// `Sequenced<T>` that wrap an inner value with a timestamp.
///
/// For types that only need to provide a timestamp value without wrapping,
/// implement [`HasTimestamp`] instead.
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
///
/// # Different Timestamp Types
///
/// The `Timestamp` type is generic and can represent various time sources:
///
/// **Monotonic counters** (u64, u128) - For test scenarios and event sourcing:
/// ```rust
/// use fluxion_core::Timestamped;
///
/// #[derive(Clone, Debug)]
/// struct SequenceNumbered<T> {
///     value: T,
///     seq: u64,
/// }
///
/// impl<T: Clone> Timestamped for SequenceNumbered<T> {
///     type Inner = T;
///     type Timestamp = u64;
///
///     fn timestamp(&self) -> u64 { self.seq }
///     fn with_timestamp(value: T, seq: u64) -> Self { Self { value, seq } }
///     fn with_fresh_timestamp(value: T) -> Self {
///         // Use atomic counter in production
///         Self { value, seq: 0 }
///     }
///     fn into_inner(self) -> T { self.value }
/// }
/// ```
///
/// **Wall-clock time** (Instant, SystemTime) - For real-time systems:
/// ```rust
/// use fluxion_core::Timestamped;
/// use std::time::Instant;
///
/// #[derive(Clone, Debug)]
/// struct TimedEvent<T> {
///     value: T,
///     time: Instant,
/// }
///
/// impl<T: Clone> Timestamped for TimedEvent<T> {
///     type Inner = T;
///     type Timestamp = Instant;
///
///     fn timestamp(&self) -> Instant { self.time }
///     fn with_timestamp(value: T, time: Instant) -> Self { Self { value, time } }
///     fn with_fresh_timestamp(value: T) -> Self {
///         Self { value, time: Instant::now() }
///     }
///     fn into_inner(self) -> T { self.value }
/// }
/// ```
pub trait Timestamped: HasTimestamp + Clone {
    /// The type of the inner value wrapped by this timestamped type
    type Inner: Clone;

    /// Creates a new instance wrapping the given value with the specified timestamp.
    fn with_timestamp(value: Self::Inner, timestamp: Self::Timestamp) -> Self;

    /// Creates a new instance wrapping the given value with a fresh timestamp.
    /// This is called by operators when emitting new values.
    /// Each implementation determines how to generate fresh timestamps
    /// (e.g., Utc::now() for chrono, atomic counter for u64, etc.)
    fn with_fresh_timestamp(value: Self::Inner) -> Self;

    /// Consumes self and returns the inner value.
    /// For wrapper types like `Sequenced<T>`, this extracts `T`.
    /// For domain types where `Inner = Self`, this typically returns `self`.
    fn into_inner(self) -> Self::Inner;
}
