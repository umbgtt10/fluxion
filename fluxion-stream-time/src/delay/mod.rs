// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Extension trait providing the `delay` operator for streams.
//!
//! This trait allows any stream of `StreamItem<T>` where `T: Fluxion` to delay emissions
//! by a specified duration.
//!
//! Each item is delayed independently - the delay is applied to each item
//! as it arrives. Errors are passed through without delay to ensure timely
//! error propagation.
//!
//! Timer is automatically selected based on runtime features.
//!
//! # Example
//!
//! ```rust,no_run
//! # #[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
//! use fluxion_stream_time::{DelayExt, TokioTimestamped};
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
//! # let timer = TokioTimer;
//! let (mut tx, rx) = mpsc::unbounded();
//! let source = rx.map(StreamItem::Value);
//!
//! let mut delayed = source.delay(Duration::from_millis(10));
//!
//! # tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now())).unwrap();
//! // Timer auto-selected based on timestamp type!
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
pub use multi_threaded::DelayExt;

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
pub use single_threaded::DelayExt;
