// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Extension trait providing the `with_latest_from` operator for timestamped streams.
//!
//! This operator combines a primary stream with a secondary stream, emitting only
//! when the primary stream emits, using the latest value from the secondary stream.
//!
//! # Behavior
//!
//! - Emissions are triggered **only** by the primary stream (self)
//! - Secondary stream updates are stored but don't trigger emissions
//! - Waits until both streams have emitted at least once
//! - Preserves temporal ordering from the primary stream
//!
//! # Examples
//!
//! ```rust
//! use fluxion_stream::WithLatestFromExt;
//! use fluxion_test_utils::{Sequenced, helpers::unwrap_stream, unwrap_value, test_channel};
//! use fluxion_core::Timestamped as TimestampedTrait;
//!
//! # async fn example() {
//! // Create channels
//! let (tx_primary, primary) = test_channel::<Sequenced<i32>>();
//! let (tx_secondary, secondary) = test_channel::<Sequenced<i32>>();
//!
//! // Combine streams
//! let mut combined = primary.with_latest_from(
//!     secondary,
//!     |state| state.clone()
//! );
//!
//! // Send values
//! tx_secondary.unbounded_send((10, 1).into()).unwrap();
//! tx_primary.unbounded_send((1, 2).into()).unwrap();
//!
//! // Assert
//! let result = unwrap_value(Some(unwrap_stream(&mut combined, 500).await));
//! let values = result.values();
//! assert_eq!(values[0] + values[1], 11);
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
pub use multi_threaded::WithLatestFromExt;

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
pub use single_threaded::WithLatestFromExt;
