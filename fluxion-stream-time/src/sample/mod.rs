// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Extension trait providing the `sample` operator for streams.
//!
//! The sample operator emits the most recently emitted value from the source
//! stream within periodic time intervals.
//!
//! - If the source emits multiple values within the interval, only the last one is emitted.
//! - If the source emits no values within the interval, nothing is emitted for that interval.
//! - Errors are passed through immediately.
//!
//! # Example
//!
//! ```rust,no_run
//! # #[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
//! use fluxion_stream_time::{SampleExt, TokioTimestamped};
//! # #[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
//! use fluxion_runtime::impls::tokio::TokioTimer;
//! # #[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
//! use fluxion_runtime::timer::Timer;
//! use fluxion_core::StreamItem;
//! # #[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
//! use fluxion_stream::prelude::*;
//! use futures::StreamExt;
//! use core::time::Duration;
//! use futures::channel::mpsc;
//!
//! # #[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
//! # #[tokio::main]
//! # async fn main() {
//! let (mut tx, rx) = mpsc::unbounded();
//! let source = rx.map(StreamItem::Value);
//!
//! let timer = TokioTimer;
//! let mut sampled = source.sample(Duration::from_millis(10));
//!
//! // Emit values
//! tx.unbounded_send(TokioTimestamped::new(1, timer.now())).unwrap();
//! tx.unbounded_send(TokioTimestamped::new(2, timer.now())).unwrap();
//!
//! // Sample should pick the latest one
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
pub use multi_threaded::SampleExt;

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
pub use single_threaded::SampleExt;
