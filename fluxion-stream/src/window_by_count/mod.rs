// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Window-by-count operator that batches stream items into fixed-size chunks.
//!
//! This module provides the [`window_by_count`](WindowByCountExt::window_by_count) operator
//! that groups consecutive stream items into vectors of a specified size.
//!
//! # Overview
//!
//! The `window_by_count` operator collects items into windows (batches) of a fixed size.
//! When the window is full, it emits a `Vec` containing all items in that window.
//! On stream completion, any partial window is also emitted.
//!
//! # Basic Usage
//!
//! ```
//! use fluxion_stream::prelude::*;
//! use fluxion_test_utils::Sequenced;
//! use futures::StreamExt;
//!
//! # #[tokio::main]
//! # async fn main() {
//! let (tx, rx) = async_channel::unbounded();
//! let stream = rx.into_fluxion_stream();
//!
//! let mut windowed = stream.window_by_count::<Sequenced<Vec<i32>>>(3);
//!
//! tx.try_send(Sequenced::new(1)).unwrap();
//! tx.try_send(Sequenced::new(2)).unwrap();
//! tx.try_send(Sequenced::new(3)).unwrap();  // Window complete!
//! tx.try_send(Sequenced::new(4)).unwrap();
//! tx.try_send(Sequenced::new(5)).unwrap();
//! drop(tx);  // Partial window [4, 5] emitted on completion
//!
//! // First window: [1, 2, 3]
//! let window1 = windowed.next().await.unwrap().unwrap().into_inner();
//! assert_eq!(window1, vec![1, 2, 3]);
//!
//! // Second window (partial): [4, 5]
//! let window2 = windowed.next().await.unwrap().unwrap().into_inner();
//! assert_eq!(window2, vec![4, 5]);
//! # }
//! ```
//!
//! # Use Cases
//!
//! - **Batch processing**: Process items in groups for efficiency
//! - **Micro-batching**: Balance latency and throughput in data pipelines
//! - **Aggregation windows**: Collect data for periodic analysis
//! - **Protocol framing**: Group bytes or messages into frames
//!
//! # Error Handling
//!
//! When an error occurs, the current partial window is discarded and the error
//! is propagated immediately. This ensures clean error boundaries without
//! emitting potentially incomplete data.

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
pub use multi_threaded::WindowByCountExt;

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
pub use single_threaded::WindowByCountExt;
