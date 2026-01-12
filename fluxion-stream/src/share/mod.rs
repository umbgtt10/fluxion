// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Shared stream subscription factory for Fluxion streams.
//!
//! A [`FluxionShared`] converts a cold stream into a hot, multi-subscriber source.
//! It consumes the original stream and broadcasts each item to all active subscribers.
//!
//! # Runtime Requirements
//!
//! This operator requires one of the following runtime features:
//! - `runtime-tokio` (default)
//! - `runtime-smol`
//! - `runtime-async-std`
//! - Or compiling for `wasm32` target
//!
//! It is not available when compiling without a runtime (no_std + alloc only).
//!
//! ## Alternatives for no_std
//!
//! - Use [`fluxion_core::FluxionSubject`] directly for manual multicast control
//!
//! ## Characteristics
//!
//! - **Hot**: Late subscribers do not receive past items—only items emitted after subscribing.
//! - **Shared execution**: The source stream is consumed once; results are broadcast to all.
//! - **Subscription factory**: Call `subscribe()` to create independent subscriber streams.
//! - **Owned lifecycle**: The forwarding task is owned and cancelled when dropped.
//!
//! ## Example
//!
//! ```rust
//! use fluxion_stream::{IntoFluxionStream, ShareExt, MapOrderedExt, FilterOrderedExt};
//! use fluxion_test_utils::Sequenced;
//!
//! # async fn example() {
//! let (tx, rx) = async_channel::unbounded();
//!
//! // Create a source stream
//! let source = rx.into_fluxion_stream()
//!     .map_ordered(|x: Sequenced<i32>| Sequenced::new(x.into_inner() * 2));
//!
//! // Share it among multiple subscribers
//! let shared = source.share();
//!
//! // Each subscriber gets broadcast values, can chain independently
//! let _sub1 = shared.subscribe().unwrap()
//!     .filter_ordered(|x| *x > 10);
//!
//! let _sub2 = shared.subscribe().unwrap()
//!     .map_ordered(|x: Sequenced<i32>| Sequenced::new(x.into_inner().to_string()));
//! # }
//! ```
//!
//! ## Comparison with FluxionSubject
//!
//! | Type | Source | Push API |
//! |------|--------|----------|
//! | [`fluxion_core::FluxionSubject`] | External (you call `next()`) | Yes |
//! | [`FluxionShared`] | Existing stream | No |
//!
//! Both are subscription factories with the same `subscribe()` pattern.

#[macro_use]
mod implementation;

#[cfg(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std"
))]
mod multi_threaded;

#[cfg(not(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std"
)))]
mod single_threaded;

#[cfg(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std"
))]
pub use multi_threaded::{FluxionShared, ShareExt, SharedBoxStream};

#[cfg(not(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std"
)))]
pub use single_threaded::{FluxionShared, ShareExt, SharedBoxStream};
