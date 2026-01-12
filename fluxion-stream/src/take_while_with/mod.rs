// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Extension trait providing the `take_while_with` operator for timestamped streams.
//!
//! This operator conditionally emits elements from a source stream based on values
//! from a separate filter stream. The stream terminates when the filter condition
//! becomes false.
//!
//! # Behavior
//!
//! - Source values are emitted only when latest filter passes the predicate
//! - Filter stream updates change the gating condition
//! - Stream terminates immediately when filter predicate returns `false`
//! - Emitted values maintain their ordered wrapper
//!
//! # Example
//!
//! ```rust
//! use fluxion_stream::TakeWhileExt;
//! use fluxion_test_utils::{Sequenced, helpers::unwrap_stream, unwrap_value, test_channel};
//! use fluxion_core::Timestamped as TimestampedTrait;
//!
//! # async fn example() {
//! // Create channels
//! let (tx_data, data_stream) = test_channel::<Sequenced<i32>>();
//! let (tx_gate, gate_stream) = test_channel::<Sequenced<bool>>();
//!
//! // Combine streams
//! let mut gated = data_stream.take_while_with(
//!     gate_stream,
//!     |gate_value| *gate_value == true
//! );
//!
//! // Send values
//! tx_gate.unbounded_send((true, 1).into()).unwrap();
//! tx_data.unbounded_send((1, 2).into()).unwrap();
//!
//! // Assert
//! assert_eq!(&unwrap_value(Some(unwrap_stream(&mut gated, 500).await)).value, &1);
//! # }
//! ```
//!
//! # Use Cases
//!
//! - Emergency stop mechanism for data streams
//! - Time-bounded stream processing
//! - Conditional data forwarding with external control

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
pub use multi_threaded::TakeWhileExt;

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
pub use single_threaded::TakeWhileExt;
