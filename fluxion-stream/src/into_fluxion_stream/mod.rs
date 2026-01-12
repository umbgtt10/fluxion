// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Extension trait to convert futures channels into fluxion streams.
//!
//! This module provides [`IntoFluxionStream`] which wraps a futures `UnboundedReceiver`
//! (from `async-channel`) into a stream that emits [`StreamItem::Value`](crate::StreamItem::Value) for each received item.
//!
//! # Basic Usage
//!
//! ```rust
//! use fluxion_stream::IntoFluxionStream;
//! use async_channel::unbounded;
//!
//! # #[tokio::main]
//! # async fn main() {
//! let (tx, rx) = unbounded::<i32>();
//! let stream = rx.into_fluxion_stream();
//! # }
//! ```
//!
//! # Mapping
//!
//! You can also map values directly during conversion:
//!
//! ```rust
//! use fluxion_stream::IntoFluxionStream;
//! use fluxion_test_utils::Sequenced;
//! use async_channel::unbounded;
//!
//! # #[tokio::main]
//! # async fn main() {
//! let (tx, rx) = unbounded::<i32>();
//! // Convert i32 items to Sequenced<i32>
//! let stream = rx.into_fluxion_stream_map(|val| Sequenced::new(val));
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
pub use multi_threaded::IntoFluxionStream;

#[cfg(all(
    not(any(
        all(feature = "runtime-tokio", not(target_arch = "wasm32")),
        feature = "runtime-smol",
        feature = "runtime-async-std"
    )),
    any(
        feature = "runtime-wasm",
        feature = "runtime-embassy",
        target_arch = "wasm32",
        feature = "alloc"
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
        feature = "runtime-wasm",
        feature = "runtime-embassy",
        target_arch = "wasm32",
        feature = "alloc"
    )
))]
pub use single_threaded::IntoFluxionStream;
