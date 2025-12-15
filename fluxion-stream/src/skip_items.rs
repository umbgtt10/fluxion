// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Skip-items operator that skips the first n items from a stream.

use fluxion_core::StreamItem;
use futures::{Stream, StreamExt};

/// Extension trait providing the `skip_items` operator for streams.
///
/// This trait allows any stream of `StreamItem<T>` to skip its first n items.
pub trait SkipItemsExt<T>: Stream<Item = StreamItem<T>> + Sized {
    /// Skips the first `n` items from the stream.
    ///
    /// The first `n` items will be discarded, and only items after that will be emitted.
    /// If the stream has fewer than `n` items, no items will be emitted.
    ///
    /// # Arguments
    ///
    /// * `n` - The number of items to skip.
    ///
    /// # Returns
    ///
    /// A new stream that skips the first `n` items from the source stream.
    ///
    /// # Error Handling
    ///
    /// **Important:** Errors count as items for the purpose of skipping.
    /// If you want to skip 2 items and the stream emits `[Error, Error, Value]`,
    /// both errors will be skipped and only the value will be emitted.
    ///
    /// Use `.on_error()` before `skip_items()` if you want to handle errors before they
    /// are counted in the skip count.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fluxion_stream::{SkipItemsExt, IntoFluxionStream};
    /// use fluxion_test_utils::Sequenced;
    /// use futures::StreamExt;
    ///
    /// # async fn example() {
    /// let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    /// let stream = rx.into_fluxion_stream();
    ///
    /// let mut after_first_two = stream.skip_items(2);
    ///
    /// tx.send(Sequenced::new(1)).unwrap();
    /// tx.send(Sequenced::new(2)).unwrap();
    /// tx.send(Sequenced::new(3)).unwrap();
    ///
    /// // First two are skipped
    /// assert_eq!(after_first_two.next().await.unwrap().unwrap().into_inner(), 3);
    /// # }
    /// ```
    ///
    /// # See Also
    ///
    /// - [`TakeItemsExt::take_items`](crate::TakeItemsExt::take_items) - Take first n items
    /// - [`StartWithExt::start_with`](crate::StartWithExt::start_with) - Prepend items
    fn skip_items(self, n: usize) -> impl Stream<Item = StreamItem<T>>;
}

impl<S, T> SkipItemsExt<T> for S
where
    S: Stream<Item = StreamItem<T>>,
{
    fn skip_items(self, n: usize) -> impl Stream<Item = StreamItem<T>> {
        StreamExt::skip(self, n)
    }
}
