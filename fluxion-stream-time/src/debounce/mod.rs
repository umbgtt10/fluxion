// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Extension trait providing the `debounce` operator for streams.
//!
//! The debounce operator waits for a pause in the input stream of at least
//! the given duration before emitting the latest value. If a new value
//! arrives before the duration elapses, the timer is reset and only the
//! newest value is eventually emitted.
//!
//! This implements **trailing debounce** semantics (Rx standard):
//! - When a value arrives, start/restart the timer
//! - If no new value arrives before the timer expires, emit the latest value
//! - If a new value arrives, discard the pending value and restart the timer
//! - When the stream ends, emit any pending value immediately
//!
//! Errors pass through immediately without debounce, to ensure timely
//! error propagation.
//!
//! # Example
//!
//! ```rust,no_run
//! # #[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
//! use fluxion_stream_time::{DebounceExt, TokioTimestamped};
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
//! let mut debounced = source.debounce(Duration::from_millis(100));
//!
//! // Alice and Bob emitted immediately. Alice should be debounced (dropped).
//! tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now())).unwrap();
//! tx.unbounded_send(TokioTimestamped::new(person_bob(), timer.now())).unwrap();
//!
//! // Only Bob should remain (trailing debounce). Timer auto-selected!
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
pub use multi_threaded::DebounceExt;

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
pub use single_threaded::DebounceExt;
