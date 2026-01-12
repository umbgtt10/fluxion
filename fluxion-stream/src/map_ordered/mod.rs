// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Map ordered operator - transforms stream items while preserving temporal ordering.
//!
//! The [`map_ordered`](MapOrderedExt::map_ordered) operator applies a transformation function
//! to each item in the stream while maintaining the original temporal ordering.
//!
//! ## Characteristics
//!
//! - **Chainable**: Returns a stream that can be further chained
//! - **Timestamp-preserving**: Original timestamps are maintained
//! - **Lazy**: Transformations are applied only when items are polled
//! - **Type-flexible**: Can transform between different types
//! - **Error-passthrough**: Errors pass through unchanged
//!
//! ## Example
//!
//! ```rust
//! use fluxion_stream::{IntoFluxionStream, MapOrderedExt};
//! use fluxion_test_utils::Sequenced;
//! use futures::StreamExt;
//!
//! # async fn example() {
//! let (tx, rx) = async_channel::unbounded();
//!
//! // Transform integers to their doubles
//! let mut stream = rx.into_fluxion_stream()
//!     .map_ordered(|n: Sequenced<i32>| Sequenced::new(n.into_inner() * 2));
//!
//! tx.try_send(Sequenced::new(1)).unwrap();
//! tx.try_send(Sequenced::new(2)).unwrap();
//! tx.try_send(Sequenced::new(3)).unwrap();
//! drop(tx);
//!
//! assert_eq!(stream.next().await.unwrap().unwrap().into_inner(), 2);
//! assert_eq!(stream.next().await.unwrap().unwrap().into_inner(), 4);
//! assert_eq!(stream.next().await.unwrap().unwrap().into_inner(), 6);
//! # }
//! ```
//!
//! ## Use Cases
//!
//! - **Data transformation**: Convert between types or modify values
//! - **Normalization**: Scale or adjust values consistently
//! - **Extraction**: Pull specific fields from complex types
//! - **Enrichment**: Add computed properties to data

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
pub use multi_threaded::MapOrderedExt;

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
pub use single_threaded::MapOrderedExt;
