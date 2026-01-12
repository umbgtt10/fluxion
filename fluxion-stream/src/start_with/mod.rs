// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Start-with operator - prepends initial values to a stream.
//!
//! The `start_with` operator allows any stream of `StreamItem<T>` to have initial values
//! prepended before the source stream items.
//!
//! # Arguments
//!
//! * `initial_values` - Vector of `StreamItem<T>` to emit before the source stream.
//!
//! # Returns
//!
//! A new stream that emits the initial values followed by all values from the source stream.
//!
//! # Error Handling
//!
//! Errors in both the initial values and the source stream are propagated as-is.
//! This operator does not consume or transform errors - they flow through unchanged.
//! Use `.on_error()` before or after `start_with()` to handle errors if needed.
//!
//! # Examples
//!
//! ```rust
//! use fluxion_stream::{StartWithExt, IntoFluxionStream};
//! use fluxion_core::StreamItem;
//! use fluxion_test_utils::Sequenced;
//! use futures::StreamExt;
//!
//! # async fn example() {
//! let (tx, rx) = async_channel::unbounded();
//! let stream = rx.into_fluxion_stream();
//!
//! let initial = vec![
//!     StreamItem::Value(Sequenced::new(1)),
//!     StreamItem::Value(Sequenced::new(2)),
//! ];
//!
//! let mut stream_with_prefix = stream.start_with(initial);
//!
//! // Initial values come first
//! assert_eq!(stream_with_prefix.next().await.unwrap().unwrap().into_inner(), 1);
//! assert_eq!(stream_with_prefix.next().await.unwrap().unwrap().into_inner(), 2);
//!
//! // Then stream values
//! tx.try_send(Sequenced::new(3)).unwrap();
//! assert_eq!(stream_with_prefix.next().await.unwrap().unwrap().into_inner(), 3);
//! # }
//! ```
//!
//! # See Also
//!
//! - [`SkipItemsExt::skip_items`](crate::SkipItemsExt::skip_items) - Skip initial items
//! - [`TakeItemsExt::take_items`](crate::TakeItemsExt::take_items) - Take first n items

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
pub use multi_threaded::StartWithExt;

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
pub use single_threaded::StartWithExt;
