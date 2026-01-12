// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

macro_rules! define_sample_ratio_impl {
    ($($bounds:tt)*) => {
        use core::fmt::Debug;
        use fluxion_core::{Fluxion, StreamItem};
        use futures::{Stream, StreamExt};

        /// Extension trait providing the [`sample_ratio`](Self::sample_ratio) operator.
        ///
        /// This trait is implemented for all streams of [`StreamItem<T>`] where `T` implements [`Fluxion`].
        pub trait SampleRatioExt<T>: Stream<Item = StreamItem<T>> + Sized
        where
            T: Fluxion,
            T::Inner: Clone + Debug + Ord + Unpin + $($bounds)* 'static,
            T::Timestamp: Debug + Ord + Copy + $($bounds)* 'static,
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
            ///   deterministic tests, or `fastrand::u64(..)` for production randomness.
            ///
            /// # Panics
            ///
            /// Panics if `ratio` is not in the range `0.0..=1.0`.
            ///
            /// # Error Handling
            ///
            /// Errors always pass through—they are never filtered by sampling.
            ///
            fn sample_ratio(self, ratio: f64, seed: u64) -> impl Stream<Item = StreamItem<T>> + $($bounds)*
            where
                Self: Unpin + $($bounds)* 'static;
        }

        impl<S, T> SampleRatioExt<T> for S
        where
            S: Stream<Item = StreamItem<T>>,
            T: Fluxion,
            T::Inner: Clone + Debug + Ord + Unpin + $($bounds)* 'static,
            T::Timestamp: Debug + Ord + Copy + $($bounds)* 'static,
        {
            fn sample_ratio(self, ratio: f64, seed: u64) -> impl Stream<Item = StreamItem<T>> + $($bounds)*
            where
                Self: Unpin + $($bounds)* 'static,
            {
                assert!(
                    (0.0..=1.0).contains(&ratio),
                    "sample_ratio: ratio must be between 0.0 and 1.0, got {ratio}"
                );

                let mut rng = fastrand::Rng::with_seed(seed);

                self.filter_map(move |item| {
                    futures::future::ready(match item {
                        StreamItem::Value(value) => {
                            if rng.f64() < ratio {
                                Some(StreamItem::Value(value))
                            } else {
                                None
                            }
                        }
                        // Errors always pass through
                        StreamItem::Error(e) => Some(StreamItem::Error(e)),
                    })
                })
            }
        }
    };
}
