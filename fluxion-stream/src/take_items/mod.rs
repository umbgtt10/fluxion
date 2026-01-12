// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Take-items operator - limits stream to first n items.
//!
//! The `take_items` operator allows any stream of `StreamItem<T>` to be limited to
//! the first n items.
//!
//! # Arguments
//!
//! * `n` - The maximum number of items to emit.
//!
//! # Returns
//!
//! A new stream that emits at most `n` items from the source stream.
//!
//! # Error Handling
//!
//! **Important:** Errors count as items for the purpose of the limit.
//! If you want to take 3 values and the stream emits `[Value, Error, Value, Value, Value]`,
//! only the first 3 items will be emitted: `[Value, Error, Value]`.
//!
//! Errors are propagated unchanged. Use `.on_error()` before `take_items()` if you want
//! to filter errors before counting.
//!
//! # Examples
//!
//! ```rust
//! use fluxion_stream::{TakeItemsExt, IntoFluxionStream};
//! use fluxion_test_utils::Sequenced;
//! use futures::StreamExt;
//!
//! # async fn example() {
//! let (tx, rx) = async_channel::unbounded();
//! let stream = rx.into_fluxion_stream();
//!
//! let mut first_two = stream.take_items(2);
//!
//! tx.try_send(Sequenced::new(1)).unwrap();
//! tx.try_send(Sequenced::new(2)).unwrap();
//! tx.try_send(Sequenced::new(3)).unwrap();
//! drop(tx);
//!
//! assert_eq!(first_two.next().await.unwrap().unwrap().into_inner(), 1);
//! assert_eq!(first_two.next().await.unwrap().unwrap().into_inner(), 2);
//! assert!(first_two.next().await.is_none());
//! # }
//! ```
//!
//! # See Also
//!
//! - [`SkipItemsExt::skip_items`](crate::SkipItemsExt::skip_items) - Skip first n items
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
pub use multi_threaded::TakeItemsExt;

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
pub use single_threaded::TakeItemsExt;
