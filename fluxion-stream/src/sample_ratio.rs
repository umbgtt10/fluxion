// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Probabilistic downsampling operator for streams.
//!
//! This module provides the [`sample_ratio`](SampleRatioExt::sample_ratio) operator that randomly
//! samples a fraction of stream items based on a probability ratio.
//!
//! # Overview
//!
//! The `sample_ratio` operator emits each item with a given probability, allowing you to
//! downsample high-frequency streams for logging, monitoring, or load reduction.
//!
//! # Deterministic Testing
//!
//! The operator requires an explicit seed parameter, enabling deterministic testing:
//!
//! ```
//! use fluxion_stream::prelude::*;
//! use fluxion_test_utils::{Sequenced, test_channel};
//! use futures::StreamExt;
//!
//! # #[tokio::main]
//! # async fn main() {
//! let (tx, stream) = test_channel::<Sequenced<i32>>();
//!
//! // Use a fixed seed for deterministic testing
//! let sampled = stream.sample_ratio(0.5, 42);
//!
//! // Send items
//! tx.unbounded_send(Sequenced::new(1)).unwrap();
//! tx.unbounded_send(Sequenced::new(2)).unwrap();
//! tx.unbounded_send(Sequenced::new(3)).unwrap();
//! tx.unbounded_send(Sequenced::new(4)).unwrap();
//! drop(tx);
//!
//! // With seed=42, the sequence is deterministic
//! let results: Vec<_> = sampled
//!     .filter_map(|item| async move {
//!         match item {
//!             fluxion_core::StreamItem::Value(v) => Some(v.value),
//!             _ => None,
//!         }
//!     })
//!     .collect()
//!     .await;
//!
//! // Results will be consistent across runs with same seed
//! assert!(!results.is_empty());
//! # }
//! ```
//!
//! # Production Usage
//!
//! In production, use the global fastrand function for random sampling:
//!
//! ```no_run
//! use fluxion_stream::prelude::*;
//! use fluxion_test_utils::Sequenced;
//!
//! # #[tokio::main]
//! # async fn main() {
//! let (_tx, rx) = async_channel::unbounded::<Sequenced<i32>>();
//! let stream = rx.into_fluxion_stream();
//!
//! // Random sampling in production - use fastrand's global RNG
//! let seed = std::time::SystemTime::now()
//!     .duration_since(std::time::UNIX_EPOCH)
//!     .unwrap()
//!     .as_secs();
//! let _sampled = stream.sample_ratio(0.1, seed); // Sample ~10%
//! # }
//! ```
//!
//! # Error Handling
//!
//! Errors always pass through unconditionally—they are never subject to sampling.
//! This ensures error observability is not affected by downsampling.

use core::fmt::Debug;
use fluxion_core::{Fluxion, StreamItem};
use futures::{Stream, StreamExt};

/// Extension trait providing the [`sample_ratio`](Self::sample_ratio) operator.
///
/// This trait is implemented for all streams of [`StreamItem<T>`] where `T` implements [`Fluxion`].
#[cfg(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std"
))]
pub trait SampleRatioExt<T>: Stream<Item = StreamItem<T>> + Sized
where
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
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
    /// # Example
    ///
    /// ```
    /// use fluxion_stream::prelude::*;
    /// use fluxion_test_utils::{Sequenced, test_channel};
    /// use futures::StreamExt;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let (tx, stream) = test_channel::<Sequenced<i32>>();
    ///
    /// // Deterministic test with fixed seed
    /// let mut sampled = stream.sample_ratio(0.5, 12345);
    ///
    /// tx.unbounded_send(Sequenced::new(42)).unwrap();
    /// drop(tx);
    ///
    /// // Item may or may not appear depending on seed
    /// let _results: Vec<_> = sampled.collect().await;
    /// # }
    /// ```
    ///
    /// # See Also
    ///
    /// - [`FilterOrderedExt::filter_ordered`](crate::FilterOrderedExt::filter_ordered) - Deterministic filtering
    fn sample_ratio(self, ratio: f64, seed: u64) -> impl Stream<Item = StreamItem<T>> + Send + Sync
    where
        Self: Send + Sync + Unpin + 'static;
}

#[cfg(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std"
))]
impl<S, T> SampleRatioExt<T> for S
where
    S: Stream<Item = StreamItem<T>>,
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
{
    fn sample_ratio(self, ratio: f64, seed: u64) -> impl Stream<Item = StreamItem<T>> + Send + Sync
    where
        Self: Send + Sync + Unpin + 'static,
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

// Single-threaded version
#[cfg(not(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std"
)))]
pub trait SampleRatioExt<T>: Stream<Item = StreamItem<T>> + Sized
where
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Unpin + 'static,
    T::Timestamp: Debug + Ord + Copy + 'static,
{
    fn sample_ratio(self, ratio: f64, seed: u64) -> impl Stream<Item = StreamItem<T>>
    where
        Self: Unpin + 'static;
}

#[cfg(not(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std"
)))]
impl<S, T> SampleRatioExt<T> for S
where
    S: Stream<Item = StreamItem<T>>,
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Unpin + 'static,
    T::Timestamp: Debug + Ord + Copy + 'static,
{
    fn sample_ratio(self, ratio: f64, seed: u64) -> impl Stream<Item = StreamItem<T>>
    where
        Self: Unpin + 'static,
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
