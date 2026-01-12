// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Extension trait providing the `throttle` operator for streams.
//!
//! The throttle operator emits the first value, then ignores subsequent values
//! for the specified duration. After the duration expires, it accepts the next
//! value and repeats the process.
//!
//! This implements **leading throttle** semantics:
//! - When a value arrives and we are not throttling:
//!   - Emit the value immediately
//!   - Start the throttle timer
//!   - Ignore subsequent values until the timer expires
//! - When the timer expires:
//!   - We become ready to accept a new value
//!
//! Errors pass through immediately without throttling, to ensure timely
//! error propagation.
//!
//! # Example
//!
//! ```rust,no_run
//! # #[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
//! use fluxion_stream_time::{ThrottleExt, TokioTimestamped};
//! # #[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
//! use fluxion_runtime::impls::tokio::TokioTimer;
//! # #[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
//! use fluxion_runtime::timer::Timer;
//! use fluxion_core::StreamItem;
//! # #[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
//! use fluxion_test_utils::test_data::{person_alice, person_bob};
//! use futures::stream::StreamExt;
//! use std::time::Duration;
//! use futures::channel::mpsc;
//!
//! # #[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
//! # #[tokio::main]
//! # async fn main() {
//! let (mut tx, rx) = mpsc::unbounded();
//! let source = rx.map(StreamItem::Value);
//!
//! let timer = TokioTimer;
//! let mut throttled = source.throttle(Duration::from_millis(100));
//!
//! // Alice and Bob emitted immediately. Bob should be throttled (dropped).
//! tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now())).unwrap();
//! tx.unbounded_send(TokioTimestamped::new(person_bob(), timer.now())).unwrap();
//!
//! // Only Alice should remain. Timer auto-selected!
//! # }
//! # #[cfg(not(all(feature = "runtime-tokio", not(target_arch = "wasm32"))))]
//! # fn main() {}
//! ```

#[cfg(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std",
    feature = "runtime-embassy",
    feature = "runtime-wasm"
))]
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
pub use multi_threaded::ThrottleExt;

#[cfg(all(
    not(any(
        all(feature = "runtime-tokio", not(target_arch = "wasm32")),
        feature = "runtime-smol",
        feature = "runtime-async-std"
    )),
    any(
        all(feature = "runtime-tokio", not(target_arch = "wasm32")),
        feature = "runtime-smol",
        feature = "runtime-async-std",
        feature = "runtime-embassy",
        feature = "runtime-wasm"
    )
))]
mod single_threaded;

#[cfg(all(
    not(any(
        all(feature = "runtime-tokio", not(target_arch = "wasm32")),
        feature = "runtime-smol",
        feature = "runtime-async-std"
    )),
    any(
        all(feature = "runtime-tokio", not(target_arch = "wasm32")),
        feature = "runtime-smol",
        feature = "runtime-async-std",
        feature = "runtime-embassy",
        feature = "runtime-wasm"
    )
))]
pub use single_threaded::ThrottleExt;
