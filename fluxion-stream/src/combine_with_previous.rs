// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::fluxion_stream::FluxionStream;
use crate::types::WithPrevious;
use fluxion_core::Ordered;
use fluxion_core::StreamItem;
use futures::{future::ready, Stream, StreamExt};

/// Extension trait providing the `combine_with_previous` operator for ordered streams.
///
/// This operator pairs each stream element with its predecessor, enabling
/// stateful processing and change detection.
pub trait CombineWithPreviousExt<T>: Stream<Item = T> + Sized
where
    T: Ordered + Clone + Send + Sync + 'static,
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
    /// A `FluxionStream` of `WithPrevious<T>` where each item contains the current
    /// and previous values.
    ///
    /// # See Also
    ///
    /// - [`combine_latest`](crate::CombineLatestExt::combine_latest) - Combines multiple streams
    /// - Useful for change detection and delta calculations
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fluxion_stream::{CombineWithPreviousExt, FluxionStream};
    /// use fluxion_test_utils::Sequenced;
    /// use futures::StreamExt;
    /// use fluxion_core::Ordered;
    ///
    /// # async fn example() {
    /// // Create channel
    /// let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    ///
    /// // Create stream
    /// let stream = FluxionStream::from_unbounded_receiver(rx);
    ///
    /// // Combine with previous
    /// let mut paired = stream.combine_with_previous();
    ///
    /// // Send values
    /// tx.send(Sequenced::with_sequence(1, 1)).unwrap();
    /// tx.send(Sequenced::with_sequence(2, 2)).unwrap();
    ///
    /// // Assert - first has no previous
    /// let first = paired.next().await.unwrap().unwrap();
    /// assert_eq!(first.previous, None);
    /// assert_eq!(*first.current.get(), 1);
    ///
    /// // Assert - second has previous
    /// let second = paired.next().await.unwrap().unwrap();
    /// assert_eq!(*second.previous.as_ref().unwrap().get(), 1);
    /// assert_eq!(*second.current.get(), 2);
    /// # }
    /// ```
    ///
    /// # Use Cases
    ///
    /// - Change detection (comparing consecutive values)
    /// - Delta calculation (computing differences)
    /// - State transitions (analyzing previous → current)
    /// - Duplicate filtering (skip if same as previous)
    fn combine_with_previous(
        self,
    ) -> FluxionStream<impl Stream<Item = fluxion_core::StreamItem<WithPrevious<T>>>>;
}

impl<T, S> CombineWithPreviousExt<T> for S
where
    S: Stream<Item = T> + Send + Sized + 'static,
    T: Ordered + Clone + Send + Sync + 'static,
{
    fn combine_with_previous(
        self,
    ) -> FluxionStream<impl Stream<Item = fluxion_core::StreamItem<WithPrevious<T>>>> {
        let result = self.scan(None, |state: &mut Option<T>, current: T| {
            let previous = state.take();
            *state = Some(current.clone());
            ready(Some(StreamItem::Value(WithPrevious::new(
                previous, current,
            ))))
        });
        FluxionStream::new(result)
    }
}
