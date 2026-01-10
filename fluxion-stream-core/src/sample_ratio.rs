// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Probabilistic downsampling operator.

use fluxion_core::StreamItem;
use futures::{Stream, StreamExt};

/// Randomly sample items from the stream with the given probability ratio.
///
/// # Arguments
///
/// * `stream` - The source stream
/// * `ratio` - Probability of emitting each item (0.0 to 1.0)
/// * `seed` - Seed for the RNG (for deterministic tests)
///
/// # Panics
///
/// Panics if `ratio` is not in range `0.0..=1.0`.
///
/// # Examples
///
/// ```rust
/// use fluxion_stream_core::sample_ratio::sample_ratio_impl;
/// use fluxion_core::StreamItem;
/// use fluxion_test_utils::Sequenced;
/// use futures::StreamExt;
///
/// # async fn example() {
/// let (tx, rx) = async_channel::unbounded();
/// let stream = rx.map(StreamItem::Value);
///
/// // Sample ~50% with deterministic seed
/// let mut sampled = sample_ratio_impl(stream, 0.5, 42);
///
/// tx.try_send(Sequenced::new(1)).unwrap();
/// // Results are deterministic with fixed seed
/// # }
/// ```
pub fn sample_ratio_impl<S, T>(
    stream: S,
    ratio: f64,
    seed: u64,
) -> impl Stream<Item = StreamItem<T>>
where
    S: Stream<Item = StreamItem<T>>,
{
    assert!(
        (0.0..=1.0).contains(&ratio),
        "sample_ratio: ratio must be between 0.0 and 1.0, got {ratio}"
    );

    let mut rng = fastrand::Rng::with_seed(seed);

    stream.filter_map(move |item| {
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
