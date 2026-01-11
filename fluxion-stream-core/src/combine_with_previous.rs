// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Combine-with-previous operator that pairs each value with its predecessor.

use alloc::boxed::Box;
use core::fmt::Debug;
use fluxion_core::{HasTimestamp, StreamItem, Timestamped};
use futures::{future::ready, Stream, StreamExt};

/// Represents a value paired with its previous value in the stream.
///
/// Used by `combine_with_previous` to provide both current and previous values.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct WithPrevious<T> {
    /// The previous value in the stream, if any
    pub previous: Option<T>,
    /// The current value in the stream
    pub current: T,
}

impl<T> WithPrevious<T> {
    /// Creates a new WithPrevious with the given previous and current values.
    pub fn new(previous: Option<T>, current: T) -> Self {
        Self { previous, current }
    }

    /// Returns true if there is a previous value.
    pub fn has_previous(&self) -> bool {
        self.previous.is_some()
    }

    /// Returns a tuple of references to (previous, current) if previous exists.
    pub fn as_pair(&self) -> Option<(&T, &T)> {
        self.previous.as_ref().map(|prev| (prev, &self.current))
    }
}

impl<T: Timestamped> HasTimestamp for WithPrevious<T> {
    type Timestamp = T::Timestamp;

    fn timestamp(&self) -> Self::Timestamp {
        self.current.timestamp()
    }
}

impl<T: Timestamped> Timestamped for WithPrevious<T> {
    type Inner = T::Inner;

    fn with_timestamp(value: Self::Inner, timestamp: Self::Timestamp) -> Self {
        Self {
            previous: None,
            current: T::with_timestamp(value, timestamp),
        }
    }

    fn into_inner(self) -> Self::Inner {
        self.current.into_inner()
    }
}

/// Pairs each stream element with its previous element.
///
/// This operator transforms a stream of `T` into a stream of `WithPrevious<T>`,
/// where each item contains both the current value and the previous value (if any).
/// The first element will have `previous = None`.
///
/// # Behavior
///
/// - First element: `WithPrevious { previous: None, current: first_value }`
/// - Subsequent elements: `WithPrevious { previous: Some(prev), current: curr }`
/// - Maintains state to track the previous value
/// - Preserves temporal ordering from the source stream
///
/// # Arguments
///
/// * `stream` - The source stream
///
/// # Returns
///
/// A stream of `WithPrevious<T>` where each item contains the current
/// and previous values.
///
/// # Error Handling
///
/// Errors from the source stream are propagated unchanged.
///
/// # Examples
///
/// ```rust
/// use fluxion_stream_core::combine_with_previous::{combine_with_previous_impl, WithPrevious};
/// use fluxion_core::StreamItem;
/// use fluxion_test_utils::Sequenced;
/// use futures::{StreamExt, pin_mut};
///
/// # async fn example() {
/// let (tx, rx) = async_channel::unbounded();
///
/// let paired = combine_with_previous_impl(rx.map(StreamItem::Value));
/// pin_mut!(paired);
///
/// tx.try_send(Sequenced::new(1)).unwrap();
/// tx.try_send(Sequenced::new(2)).unwrap();
///
/// // First has no previous
/// let first = paired.next().await.unwrap().unwrap();
/// assert_eq!(first.previous, None);
/// assert_eq!(first.current.into_inner(), 1);
///
/// // Second has previous
/// let second = paired.next().await.unwrap().unwrap();
/// assert_eq!(second.previous.as_ref().unwrap().value, 1);
/// assert_eq!(second.current.into_inner(), 2);
/// # }
/// ```
pub fn combine_with_previous_impl<S, T>(
    stream: S,
) -> impl Stream<Item = StreamItem<WithPrevious<T>>>
where
    S: Stream<Item = StreamItem<T>>,
    T: Clone + Debug,
{
    Box::pin(
        stream.scan(None, |state: &mut Option<T>, item: StreamItem<T>| {
            ready(Some(match item {
                StreamItem::Value(current) => {
                    let previous = state.take();
                    *state = Some(current.clone());
                    StreamItem::Value(WithPrevious::new(previous, current))
                }
                StreamItem::Error(e) => StreamItem::Error(e),
            }))
        }),
    )
}
