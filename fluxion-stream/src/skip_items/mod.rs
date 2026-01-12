// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Skip items operator - skips the first N items from a stream.
//!
//! The `skip_items` operator allows any stream of `StreamItem<T>` to skip its first n items.
//!
//! # Arguments
//!
//! * `n` - The number of items to skip.
//!
//! # Returns
//!
//! A new stream that skips the first `n` items from the source stream.
//!
//! # Error Handling
//!
//! **Important:** Errors count as items for the purpose of skipping.
//! If you want to skip 2 items and the stream emits `[Error, Error, Value]`,
//! both errors will be skipped and only the value will be emitted.
//!
//! Use `.on_error()` before `skip_items()` if you want to handle errors before they
//! are counted in the skip count.
//!
//! # Examples
//!
//! ```rust
//! use fluxion_stream::{SkipItemsExt, IntoFluxionStream};
//! use fluxion_test_utils::Sequenced;
//! use futures::StreamExt;
//!
//! # async fn example() {
//! let (tx, rx) = async_channel::unbounded();
//! let stream = rx.into_fluxion_stream();
//!
//! let mut after_first_two = stream.skip_items(2);
//!
//! tx.try_send(Sequenced::new(1)).unwrap();
//! tx.try_send(Sequenced::new(2)).unwrap();
//! tx.try_send(Sequenced::new(3)).unwrap();
//!
//! // First two are skipped
//! assert_eq!(after_first_two.next().await.unwrap().unwrap().into_inner(), 3);
//! # }
//! ```
//!
//! # See Also
//!
//! - [`TakeItemsExt::take_items`](crate::TakeItemsExt::take_items) - Take first n items
//! - [`StartWithExt::start_with`](crate::StartWithExt::start_with) - Prepend items

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
pub use multi_threaded::SkipItemsExt;

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
pub use single_threaded::SkipItemsExt;
