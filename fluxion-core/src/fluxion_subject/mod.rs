// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Hot, multi-subscriber subject for Fluxion streams.
//!
//! A [`FluxionSubject`] broadcasts each [`StreamItem<T>`](crate::StreamItem) to all active subscribers.
//!
//! ## Characteristics
//!
//! - **Hot**: Late subscribers do not receive past items—only items sent after subscribing.
//! - **Unbounded**: Uses unbounded mpsc channels internally (no backpressure).
//! - **Thread-safe**: Cheap to clone; all clones share the same internal state.
//! - **std-only**: Requires the `std` feature (uses `parking_lot::Mutex`).
//! - **Error/close**: Errors are propagated to all subscribers and terminate the subject.
//!
//! ## Example
//!
//! ```
//! use fluxion_core::{FluxionSubject, StreamItem};
//! use futures::StreamExt;
//!
//! # #[tokio::main]
//! # async fn main() {
//! let subject = FluxionSubject::<i32>::new();
//!
//! // Subscribe before sending
//! let mut stream = subject.subscribe().unwrap();
//!
//! // Send values to all subscribers
//! subject.send(StreamItem::Value(1)).unwrap();
//! subject.send(StreamItem::Value(2)).unwrap();
//! subject.close();
//!
//! // Receive values
//! assert_eq!(stream.next().await, Some(StreamItem::Value(1)));
//! assert_eq!(stream.next().await, Some(StreamItem::Value(2)));
//! assert_eq!(stream.next().await, None); // Subject closed
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
pub use multi_threaded::FluxionSubject;

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
pub use single_threaded::FluxionSubject;
