// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Probabilistic downsampling operator for single-threaded runtimes.

use core::fmt::Debug;
use fluxion_core::{Fluxion, StreamItem};
use fluxion_stream_core::sample_ratio::sample_ratio_impl;
use futures::Stream;

/// Extension trait providing the `sample_ratio` operator.
///
/// This trait is implemented for all streams of `StreamItem<T>` where `T` implements `Fluxion`.
pub trait SampleRatioExt<T>: Stream<Item = StreamItem<T>> + Sized
where
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Unpin + Copy + 'static,
{
    /// Randomly samples items from the stream with the given probability ratio.
    ///
    /// Each item has a `ratio` probability of being emitted. The `seed` parameter
    /// controls the random number generator for reproducibility.
    ///
    /// # Arguments
    ///
    /// * `ratio` - Probability of emitting each item (0.0 to 1.0 inclusive)
    ///   - `0.0` - Never emit any items
    ///   - `0.5` - Emit approximately half of items
    ///   - `1.0` - Emit all items
    /// * `seed` - Seed for the random number generator. Use a fixed value for
    ///   deterministic tests, or generate randomly for production.
    ///
    /// # Panics
    ///
    /// Panics if `ratio` is not in the range `0.0..=1.0`.
    ///
    /// # Error Handling
    ///
    /// Errors always pass through—they are never filtered by sampling.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fluxion_stream_single::{SampleRatioExt, IntoFluxionStream};
    /// use fluxion_test_utils::Sequenced;
    /// use futures::StreamExt;
    ///
    /// # async fn example() {
    /// let (tx, rx) = async_channel::unbounded();
    /// let stream = rx.into_fluxion_stream();
    ///
    /// // Sample ~50% with deterministic seed
    /// let mut sampled = stream.sample_ratio(0.5, 42);
    ///
    /// tx.try_send(Sequenced::new(1)).unwrap();
    /// tx.try_send(Sequenced::new(2)).unwrap();
    /// // Results are deterministic with fixed seed
    /// # }
    /// ```
    ///
    /// # See Also
    ///
    /// - [`FilterOrderedExt::filter_ordered`](crate::FilterOrderedExt::filter_ordered) - Deterministic filtering
    fn sample_ratio(self, ratio: f64, seed: u64) -> impl Stream<Item = StreamItem<T>>
    where
        Self: 'static;
}

impl<S, T> SampleRatioExt<T> for S
where
    S: Stream<Item = StreamItem<T>>,
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Unpin + Copy + 'static,
{
    fn sample_ratio(self, ratio: f64, seed: u64) -> impl Stream<Item = StreamItem<T>>
    where
        Self: 'static,
    {
        sample_ratio_impl(self, ratio, seed)
    }
}
