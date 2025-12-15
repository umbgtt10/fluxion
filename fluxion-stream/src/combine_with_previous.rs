// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::types::WithPrevious;
use fluxion_core::{Fluxion, StreamItem};
use futures::{future::ready, Stream, StreamExt};
use std::fmt::Debug;

/// Extension trait providing the `combine_with_previous` operator for timestamped streams.
///
/// This operator pairs each stream element with its predecessor, enabling
/// stateful processing and change detection.
pub trait CombineWithPreviousExt<T>: Stream<Item = StreamItem<T>> + Sized
where
    T: Fluxion,
    T::Inner: Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
{
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
    /// # Returns
    ///
    /// A stream of `WithPrevious<T>` where each item contains the current
    /// and previous values.
    ///
    /// # Errors
    ///
    /// This operator may produce `StreamItem::Error` in the following cases:
    ///
    /// - **Lock Errors**: When acquiring the previous value buffer lock fails (e.g., due to lock poisoning).
    ///   These are transient errors - the stream continues processing and may succeed on subsequent items.
    ///
    /// Lock errors are typically non-fatal and indicate temporary contention. The operator will continue
    /// processing subsequent items. See the [Error Handling Guide](../docs/ERROR-HANDLING.md) for patterns
    /// on handling these errors in your application.
    ///
    /// # See Also
    ///
    /// - [`combine_latest`](crate::CombineLatestExt::combine_latest) - Combines multiple streams
    /// - Useful for change detection and delta calculations
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fluxion_stream::CombineWithPreviousExt;
    /// use fluxion_test_utils::{Sequenced, helpers::unwrap_stream, unwrap_value, test_channel};
    /// use fluxion_core::Timestamped as TimestampedTrait;
    ///
    /// # async fn example() {
    /// // Create channel
    /// let (tx, stream) = test_channel::<Sequenced<i32>>();
    ///
    /// // Combine with previous
    /// let mut paired = stream.combine_with_previous();
    ///
    /// // Send values
    /// tx.send((1, 1).into()).unwrap();
    /// tx.send((2, 2).into()).unwrap();
    ///
    /// // Assert - first has no previous
    /// let first = unwrap_value(Some(unwrap_stream(&mut paired, 500).await));
    /// assert_eq!(first.previous, None);
    /// assert_eq!(first.current.value, 1);
    ///
    /// // Assert - second has previous
    /// let second = unwrap_value(Some(unwrap_stream(&mut paired, 500).await));
    /// assert_eq!(second.previous.as_ref().unwrap().value, 1);
    /// assert_eq!(second.current.value, 2);
    /// # }
    /// ```
    ///
    /// # Use Cases
    ///
    /// - Change detection (comparing consecutive values)
    /// - Delta calculation (computing differences)
    /// - State transitions (analyzing previous → current)
    /// - Duplicate filtering (skip if same as previous)
    fn combine_with_previous(self) -> impl Stream<Item = StreamItem<WithPrevious<T>>> + Send;
}

impl<T, S> CombineWithPreviousExt<T> for S
where
    S: Stream<Item = StreamItem<T>> + Send + Sized + 'static,
    T: Fluxion,
    T::Inner: Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
{
    fn combine_with_previous(self) -> impl Stream<Item = StreamItem<WithPrevious<T>>> + Send {
        Box::pin(
            self.scan(None, |state: &mut Option<T>, item: StreamItem<T>| {
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
}
