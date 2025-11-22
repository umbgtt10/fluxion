// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

/// A minimal trait for types that have an intrinsic timestamp for stream ordering.
///
/// This trait provides read-only access to a timestamp value and the wrapped value type,
/// allowing stream operators to order and compare items based on their temporal position.
///
/// # Relationship with `Timestamped`
///
/// `HasTimestamp` is a minimal trait for reading timestamps and knowing the inner type,
/// while `Timestamped` extends it with construction methods (`with_timestamp`, `with_fresh_timestamp`, `into_inner`).
///
/// Use `HasTimestamp` when you only need to read timestamps (e.g., ordering, comparison).
/// Use `Timestamped` when you need to construct new timestamped values.
///
/// # Type Parameters
/// * `Inner` - The type of the value wrapped by this timestamped type
/// * `Timestamp` - The type representing the timestamp (must be `Ord + Copy`)
///
/// # Examples
///
/// ```
/// use fluxion_core::HasTimestamp;
///
/// #[derive(Clone, Debug)]
/// struct TimestampedEvent<T> {
///     value: T,
///     timestamp: u64,
/// }
///
/// impl<T: Clone> HasTimestamp for TimestampedEvent<T> {
///     type Inner = T;
///     type Timestamp = u64;
///
///     fn timestamp(&self) -> Self::Timestamp {
///         self.timestamp
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
/// use fluxion_core::HasTimestamp;
///
/// #[derive(Clone, Debug)]
/// struct SequenceNumbered<T> {
///     value: T,
///     seq: u64,
/// }
///
/// impl<T: Clone> HasTimestamp for SequenceNumbered<T> {
///     type Inner = T;
///     type Timestamp = u64;
///     fn timestamp(&self) -> u64 { self.seq }
/// }
/// ```
///
/// **Wall-clock time** (Instant, SystemTime) - For real-time systems:
/// ```rust
/// use fluxion_core::HasTimestamp;
/// use std::time::Instant;
///
/// #[derive(Clone, Debug)]
/// struct TimedEvent<T> {
///     value: T,
///     time: Instant,
/// }
///
/// impl<T: Clone> HasTimestamp for TimedEvent<T> {
///     type Inner = T;
///     type Timestamp = Instant;
///     fn timestamp(&self) -> Instant { self.time }
/// }
/// ```
pub trait HasTimestamp {
    /// The type of the inner value wrapped by this timestamped type
    type Inner: Clone;

    /// The type representing the timestamp
    type Timestamp: Ord + Copy + Send + Sync + std::fmt::Debug;

    /// Returns the timestamp value for this item.
    /// Stream operators use this to determine the order of items.
    fn timestamp(&self) -> Self::Timestamp;
}
