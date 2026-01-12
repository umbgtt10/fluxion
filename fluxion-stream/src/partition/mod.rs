// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Partition operator that splits a stream into two based on a predicate.
//!
//! The [`partition`](PartitionExt::partition) operator routes each item to one of two
//! output streams based on a predicate function. Items satisfying the predicate go to
//! the "true" stream, while others go to the "false" stream.
//!
//! # Runtime Requirements
//!
//! This operator requires one of the following runtime features:
//! - `runtime-tokio` (default)
//! - `runtime-smol`
//! - `runtime-async-std`
//! - Or compiling for `wasm32` target
//!
//! It is not available when compiling without a runtime (no_std + alloc only).
//!
//! ## Characteristics
//!
//! - **Chain-breaking**: Returns two streams, cannot chain further on the original
//! - **Spawns task**: Routing runs in a background Tokio task
//! - **Timestamp-preserving**: Original timestamps are preserved in both output streams
//! - **Routing**: Every item goes to exactly one output stream
//! - **Non-blocking**: Both streams can be consumed independently
//! - **Hot**: Uses internal subjects for broadcasting (late consumers miss items)
//! - **Error propagation**: Errors are sent to both output streams
//! - **Unbounded buffers**: Items are buffered in memory until consumed
//!
//! ## Buffer Behavior
//!
//! The partition operator uses unbounded internal channels. If one partition stream
//! is consumed slowly (or not at all), items destined for that stream will accumulate
//! in memory. This is typically fine for balanced workloads, but be aware:
//!
//! - If you only consume one partition, items for the other still buffer
//! - For high-throughput streams with imbalanced consumption, consider adding
//!   backpressure mechanisms downstream
//! - Dropping one partition stream is safe; items for it are simply discarded
//!
//! ## Example
//!
//! ```rust
//! use fluxion_stream::{IntoFluxionStream, PartitionExt};
//! use fluxion_test_utils::Sequenced;
//! use futures::StreamExt;
//!
//! # async fn example() {
//! let (tx, rx) = async_channel::unbounded();
//!
//! // Partition numbers into even and odd
//! let (mut evens, mut odds) = rx.into_fluxion_stream()
//!     .partition(|n: &i32| n % 2 == 0);
//!
//! tx.try_send(Sequenced::new(1)).unwrap();
//! tx.try_send(Sequenced::new(2)).unwrap();
//! tx.try_send(Sequenced::new(3)).unwrap();
//! tx.try_send(Sequenced::new(4)).unwrap();
//! drop(tx);
//!
//! // Evens: 2, 4
//! assert_eq!(evens.next().await.unwrap().unwrap().into_inner(), 2);
//! assert_eq!(evens.next().await.unwrap().unwrap().into_inner(), 4);
//!
//! // Odds: 1, 3
//! assert_eq!(odds.next().await.unwrap().unwrap().into_inner(), 1);
//! assert_eq!(odds.next().await.unwrap().unwrap().into_inner(), 3);
//! # }
//! ```
//!
//! ## Use Cases
//!
//! - **Error routing**: Separate successful values from validation failures
//! - **Priority queues**: Split high-priority and low-priority items
//! - **Type routing**: Route different enum variants to specialized handlers
//! - **Threshold filtering**: Split values above/below a threshold

#[macro_use]
mod implementation;

// Multi-threaded runtime (tokio, smol, async-std)
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
pub use multi_threaded::{PartitionExt, PartitionedStream};

// Single-threaded runtime (wasm32, embassy)
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
pub use single_threaded::{PartitionExt, PartitionedStream};
