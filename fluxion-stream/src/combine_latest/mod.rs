// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Extension trait providing the `combine_latest` operator for `Timestamped` streams.
//!
//! This trait enables combining multiple streams where each emission waits for
//! at least one value from all streams, then emits the combination of the latest
//! values from each stream.
//!
//! # Behavior
//!
//! - Waits until all streams have emitted at least one value
//! - After initialization, emits whenever any stream produces a value
//! - Maintains temporal ordering using the `Ordered` trait
//! - Allows filtering emissions based on the combined state
//!
//! # Example
//!
//! ```rust
//! use fluxion_stream::CombineLatestExt;
//! use fluxion_test_utils::{Sequenced, helpers::unwrap_stream, unwrap_value, test_channel};
//! use fluxion_core::Timestamped as TimestampedTrait;
//!
//! # async fn example() {
//! // Create channels
//! let (tx1, stream1) = test_channel::<Sequenced<i32>>();
//! let (tx2, stream2) = test_channel::<Sequenced<i32>>();
//!
//! // Combine streams
//! let mut combined = stream1.combine_latest(
//!     vec![stream2],
//!     |state| state.values().len() == 2
//! );
//!
//! // Send values
//! tx1.unbounded_send((1, 1).into()).unwrap();
//! tx2.unbounded_send((2, 2).into()).unwrap();
//!
//! // Assert
//! let result = unwrap_value(Some(unwrap_stream(&mut combined, 500).await));
//! let values = result.values();
//! assert_eq!(values.len(), 2);
//! assert_eq!(values[0], 1);
//! assert_eq!(values[1], 2);
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
pub use multi_threaded::CombineLatestExt;

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
pub use single_threaded::CombineLatestExt;
