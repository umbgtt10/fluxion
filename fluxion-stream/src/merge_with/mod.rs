// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! A stateful stream merger that combines multiple Timestamped streams while maintaining state.
//!
//! The `MergedStream` struct allows merging multiple streams into a single one, preserving
//! temporal order and allowing stateful processing of each item.
//!
//! # Example
//!
//! ```rust
//! use fluxion_stream::MergedStream;
//! use fluxion_test_utils::{Sequenced, helpers::unwrap_stream, unwrap_value, test_channel};
//! use futures::StreamExt;
//!
//! # async fn example() {
//! // Initial state is 0
//! let stream = MergedStream::seed::<Sequenced<i32>>(0);
//!
//! let (tx, rx) = test_channel::<Sequenced<i32>>();
//!
//! // Merge a stream that adds its value to the state and emits the new state
//! let mut stream = stream.merge_with(rx, |val, state| {
//!     *state += val;
//!     *state
//! });
//!
//! tx.unbounded_send((10, 1).into()).unwrap();
//! tx.unbounded_send((20, 2).into()).unwrap();
//!
//! let first = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
//! assert_eq!(first.value, 10); // 0 + 10
//!
//! let second = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
//! assert_eq!(second.value, 30); // 10 + 20
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
pub use multi_threaded::MergedStream;

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
pub use single_threaded::MergedStream;
