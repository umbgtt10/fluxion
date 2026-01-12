// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Extension trait providing high-level ordered merge for `Timestamped` streams.
//!
//! This trait merges multiple streams of timestamped items, emitting all values from
//! all streams in temporal order. Unlike `combine_latest`, this emits every value
//! from every stream (not just when all have emitted).
//!
//! # Characteristics
//!
//! - **Ordered**: Emits items with smallest timestamp first.
//! - **Fair**: Merges streams fairly assuming they are reasonably synchronized.
//! - **Buffered**: Buffers one item from each stream to determine the minimum timestamp.
//!
//! # Example
//!
//! ```rust
//! use fluxion_stream::OrderedStreamExt;
//! use fluxion_test_utils::{Sequenced, helpers::unwrap_stream, unwrap_value, test_channel};
//! use fluxion_core::Timestamped as TimestampedTrait;
//!
//! # async fn example() {
//! // Create channels
//! let (tx1, stream1) = test_channel::<Sequenced<i32>>();
//! let (tx2, stream2) = test_channel::<Sequenced<i32>>();
//!
//! // Merge streams
//! let mut merged = stream1.ordered_merge(vec![stream2]);
//!
//! // Send values with explicit ordering
//! tx1.unbounded_send((1, 100).into()).unwrap();
//! tx2.unbounded_send((2, 200).into()).unwrap();
//! tx1.unbounded_send((3, 300).into()).unwrap();
//!
//! // Assert - values emitted in temporal order
//! assert_eq!(unwrap_value(Some(unwrap_stream(&mut merged, 500).await)).value, 1);
//! assert_eq!(unwrap_value(Some(unwrap_stream(&mut merged, 500).await)).value, 2);
//! assert_eq!(unwrap_value(Some(unwrap_stream(&mut merged, 500).await)).value, 3);
//! # }
//! ```

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
pub use multi_threaded::{ordered_merge_with_index, OrderedStreamExt};

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
pub use single_threaded::{ordered_merge_with_index, OrderedStreamExt};
