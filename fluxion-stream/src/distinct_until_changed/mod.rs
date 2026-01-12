// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

/// Extension trait providing the `distinct_until_changed` operator for streams.
///
/// This operator filters out consecutive duplicate values, emitting only when
/// the value changes from the previous emission.
///
/// Use [`DistinctUntilChangedExt::distinct_until_changed`] to use this operator.
///
/// # Behavior
///
/// - First value is always emitted (no previous value to compare)
/// - Subsequent values are compared to the last emitted value
/// - Only values where `current != previous` are emitted
/// - Timestamps are preserved from the original incoming values
/// - Errors are always propagated immediately
///
/// # Examples
///
/// ## Basic Deduplication
///
/// ```rust
/// use fluxion_stream::{DistinctUntilChangedExt, IntoFluxionStream};
/// use fluxion_test_utils::Sequenced;
/// use futures::StreamExt;
///
/// # async fn example() {
/// let (tx, rx) = async_channel::unbounded();
/// let stream = rx.into_fluxion_stream();
///
/// let mut distinct = stream.distinct_until_changed();
///
/// // Send: 1, 1, 2, 2, 2, 3, 2
/// tx.try_send(Sequenced::new(1)).unwrap();
/// tx.try_send(Sequenced::new(1)).unwrap(); // Filtered
/// tx.try_send(Sequenced::new(2)).unwrap();
/// tx.try_send(Sequenced::new(2)).unwrap(); // Filtered
/// tx.try_send(Sequenced::new(2)).unwrap(); // Filtered
/// tx.try_send(Sequenced::new(3)).unwrap();
/// tx.try_send(Sequenced::new(2)).unwrap(); // Emitted (different from 3)
///
/// // Output: 1, 2, 3, 2
/// assert_eq!(distinct.next().await.unwrap().unwrap().into_inner(), 1);
/// assert_eq!(distinct.next().await.unwrap().unwrap().into_inner(), 2);
/// assert_eq!(distinct.next().await.unwrap().unwrap().into_inner(), 3);
/// assert_eq!(distinct.next().await.unwrap().unwrap().into_inner(), 2);
/// # }
/// ```
///
/// ## Toggle/State Change Detection
///
/// Detect when a boolean state changes:
///
/// ```rust
/// use fluxion_stream::{DistinctUntilChangedExt, IntoFluxionStream};
/// use fluxion_test_utils::Sequenced;
/// use futures::StreamExt;
///
/// # async fn example() {
/// let (tx, rx) = async_channel::unbounded();
/// let stream = rx.into_fluxion_stream();
///
/// let mut changes = stream.distinct_until_changed();
///
/// // Simulate toggle events
/// tx.try_send(Sequenced::new(false)).unwrap(); // Initial state
/// tx.try_send(Sequenced::new(false)).unwrap(); // No change
/// tx.try_send(Sequenced::new(true)).unwrap();  // Changed!
/// tx.try_send(Sequenced::new(true)).unwrap();  // No change
/// tx.try_send(Sequenced::new(false)).unwrap(); // Changed!
///
/// // Only state transitions are emitted
/// assert!(!changes.next().await.unwrap().unwrap().into_inner());
/// assert!(changes.next().await.unwrap().unwrap().into_inner());
/// assert!(!changes.next().await.unwrap().unwrap().into_inner());
/// # }
/// ```
///
/// ## Timestamp Preservation
///
/// Timestamps are preserved from the original incoming values:
///
/// ```rust
/// use fluxion_stream::{DistinctUntilChangedExt, IntoFluxionStream};
/// use fluxion_test_utils::Sequenced;
/// use fluxion_core::HasTimestamp;
/// use futures::StreamExt;
///
/// # async fn example() {
/// let (tx, rx) = async_channel::unbounded();
/// let stream = rx.into_fluxion_stream();
///
/// let mut distinct = stream.distinct_until_changed();
///
/// tx.try_send(Sequenced::new(1)).unwrap();
/// let first = distinct.next().await.unwrap().unwrap();
/// let ts1 = first.timestamp();
///
/// tx.try_send(Sequenced::new(1)).unwrap(); // Filtered (duplicate)
/// tx.try_send(Sequenced::new(2)).unwrap(); // Emitted with its original timestamp
///
/// let second = distinct.next().await.unwrap().unwrap();
/// let ts2 = second.timestamp();
///
/// // Timestamps come from the original Sequenced values
/// assert!(ts2 > ts1);
/// # }
/// ```
///
/// # Use Cases
///
/// - Debouncing repeated sensor readings
/// - Detecting state transitions (on/off, connected/disconnected)
/// - Filtering redundant UI updates
/// - Change detection in data streams
/// - Rate limiting when values don't change
///
/// # Performance
///
/// - O(1) time complexity per item
/// - Stores only the last emitted value
/// - No buffering or lookahead required
///
/// # See Also
///
/// - [`filter_ordered`](crate::FilterOrderedExt::filter_ordered) - General filtering
/// - [`take_while_with`](crate::TakeWhileExt::take_while_with) - Conditional stream termination
#[macro_use]
mod implementation;

#[cfg(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std"
))]
mod multi_threaded;
#[cfg(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std"
))]
pub use multi_threaded::DistinctUntilChangedExt;

#[cfg(not(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std"
)))]
mod single_threaded;
#[cfg(not(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std"
)))]
pub use single_threaded::DistinctUntilChangedExt;
