// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Extension trait providing the `emit_when` operator for timestamped streams.
//!
//! This operator gates a source stream based on conditions from a filter stream,
//! emitting source values only when the combined state passes a predicate.
//!
//! # Behavior
//!
//! - Maintains latest value from both source and filter streams
//! - Evaluates predicate on `CombinedState` containing both values
//! - Emits source value when predicate returns `true`
//! - Both streams must emit at least once before any emission occurs
//! - Preserves temporal ordering of source stream
//!
//! # Examples
//!
//! ```rust
//! use fluxion_stream::EmitWhenExt;
//! use fluxion_test_utils::{Sequenced, helpers::unwrap_stream, unwrap_value, test_channel};
//! use fluxion_core::Timestamped as TimestampedTrait;
//!
//! # async fn example() {
//! // Create channels
//! let (tx_data, data_stream) = test_channel::<Sequenced<i32>>();
//! let (tx_enable, enable_stream) = test_channel::<Sequenced<i32>>();
//!
//! // Combine streams
//! let mut gated = data_stream.emit_when(
//!     enable_stream,
//!     |state| {
//!         let values = state.values();
//!         values[1] > 0  // Enable when value > 0
//!     }
//! );
//!
//! // Send values
//! tx_enable.unbounded_send((1, 1).into()).unwrap();  // Enabled
//! tx_data.unbounded_send((42, 2).into()).unwrap();
//!
//! // Assert - data emits when enabled
//! let result = unwrap_value(Some(unwrap_stream(&mut gated, 500).await));
//! assert_eq!(result.value, 42);
//! # }
//! ```
//!
//! # Use Cases
//!
//! - Conditional forwarding based on external signals
//! - State-dependent filtering
//! - Complex gating logic involving multiple stream values

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
pub use multi_threaded::EmitWhenExt;

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
pub use single_threaded::EmitWhenExt;
