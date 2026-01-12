// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Side-effect operator for debugging and troubleshooting streams.
//!
//! This module provides the [`tap`](TapExt::tap) operator that allows observing
//! stream items without modifying them, useful for debugging complex pipelines.
//!
//! # Overview
//!
//! The `tap` operator invokes a side-effect function for each value without
//! affecting the stream. It's the reactive equivalent of inserting `println!`
//! statements for debugging.
//!
//! # Basic Usage
//!
//! ```
//! use fluxion_stream::prelude::*;
//! use fluxion_test_utils::Sequenced;
//! use futures::StreamExt;
//!
//! # #[tokio::main]
//! # async fn main() {
//! let (tx, rx) = async_channel::unbounded();
//! let stream = rx.into_fluxion_stream();
//!
//! let mut tapped = stream
//!     .tap(|value| println!("Observed: {:?}", value));
//!
//! tx.try_send(Sequenced::new(42)).unwrap();
//! drop(tx);
//!
//! // Value passes through unchanged
//! let item = tapped.next().await.unwrap();
//! assert_eq!(item.unwrap().into_inner(), 42);
//! # }
//! ```
//!
//! # Debugging Pipelines
//!
//! Insert `tap` at any point in a pipeline to observe intermediate values:
//!
//! ```
//! use fluxion_stream::prelude::*;
//! use fluxion_test_utils::Sequenced;
//! use futures::StreamExt;
//!
//! # #[tokio::main]
//! # async fn main() {
//! let (tx, rx) = async_channel::unbounded::<Sequenced<i32>>();
//! let stream = rx.into_fluxion_stream();
//!
//! let mut processed = stream
//!     .tap(|v| println!("Before filter: {:?}", v))
//!     .filter_ordered(|x| *x > 10)
//!     .tap(|v| println!("After filter: {:?}", v))
//!     .map_ordered(|x| Sequenced::new(x.into_inner() * 2))
//!     .tap(|v| println!("Final value: {:?}", v));
//!
//! tx.try_send(Sequenced::new(42)).unwrap();
//! drop(tx);
//!
//! let result = processed.next().await.unwrap().unwrap();
//! assert_eq!(result.into_inner(), 84);
//! # }
//! ```
//!
//! # Error Handling
//!
//! The tap function is only called for values, not errors. Errors pass through
//! unchanged without invoking the tap function.
//!
//! # See Also
//!
//! - [`MapOrderedExt::map_ordered`](crate::MapOrderedExt::map_ordered) - Transform values
//! - [`FilterOrderedExt::filter_ordered`](crate::FilterOrderedExt::filter_ordered) - Filter values
//! - [`OnErrorExt::on_error`](crate::OnErrorExt::on_error) - Handle errors with side effects

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
pub use multi_threaded::TapExt;

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
pub use single_threaded::TapExt;
