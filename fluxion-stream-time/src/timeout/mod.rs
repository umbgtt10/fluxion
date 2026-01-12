// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Extension trait providing the `timeout` operator for streams.
//!
//! This trait allows any stream of `StreamItem<T>` where `T: Fluxion` to enforce a timeout
//! between emissions.
//!
//! The timeout operator monitors the time interval between emissions from the source stream.
//! If the source stream does not emit any value within the specified duration, the operator
//! emits a `FluxionError::TimeoutError` with "Timeout" context and terminates the stream.
//!
//! - If the source emits a value, the timer is reset.
//! - If the source completes, the timeout operator completes.
//! - If the source errors, the error is passed through.
//!
//! # Example
//!
//! ```rust,no_run
//! # #[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
//! use fluxion_stream_time::{TimeoutExt, TokioTimestamped};
//! # #[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
//! use fluxion_runtime::impls::tokio::TokioTimer;
//! # #[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
//! use fluxion_runtime::timer::Timer;
//! use fluxion_core::StreamItem;
//! # #[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
//! use fluxion_test_utils::test_data::person_alice;
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
//! let mut timed_out = source.timeout(Duration::from_millis(100));
//!
//! tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now())).unwrap();
//! // Timer auto-selected!
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
pub use multi_threaded::TimeoutExt;

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
pub use single_threaded::TimeoutExt;
