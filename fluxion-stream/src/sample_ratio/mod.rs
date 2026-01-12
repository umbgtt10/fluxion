// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Probabilistic downsampling operator for streams.
//!
//! This module provides the [`sample_ratio`](SampleRatioExt::sample_ratio) operator that randomly
//! samples a fraction of stream items based on a probability ratio.
//!
//! # Overview
//!
//! The `sample_ratio` operator emits each item with a given probability, allowing you to
//! downsample high-frequency streams for logging, monitoring, or load reduction.
//!
//! # Deterministic Testing
//!
//! The operator requires an explicit seed parameter, enabling deterministic testing:
//!
//! ```
//! use fluxion_stream::prelude::*;
//! use fluxion_test_utils::{Sequenced, test_channel};
//! use futures::StreamExt;
//!
//! # #[tokio::main]
//! # async fn main() {
//! let (tx, stream) = test_channel::<Sequenced<i32>>();
//!
//! // Use a fixed seed for deterministic testing
//! let sampled = stream.sample_ratio(0.5, 42);
//!
//! // Send items
//! tx.unbounded_send(Sequenced::new(1)).unwrap();
//! tx.unbounded_send(Sequenced::new(2)).unwrap();
//! tx.unbounded_send(Sequenced::new(3)).unwrap();
//! tx.unbounded_send(Sequenced::new(4)).unwrap();
//! drop(tx);
//!
//! // With seed=42, the sequence is deterministic
//! let results: Vec<_> = sampled
//!     .filter_map(|item| async move {
//!         match item {
//!             fluxion_core::StreamItem::Value(v) => Some(v.value),
//!             _ => None,
//!         }
//!     })
//!     .collect()
//!     .await;
//!
//! // Results will be consistent across runs with same seed
//! assert!(!results.is_empty());
//! # }
//! ```
//!
//! # Production Usage
//!
//! In production, use the global fastrand function for random sampling:
//!
//! ```no_run
//! use fluxion_stream::prelude::*;
//! use fluxion_test_utils::Sequenced;
//! use futures::StreamExt;
//!
//! # #[tokio::main]
//! # async fn main() {
//! let (_tx, rx) = async_channel::unbounded::<Sequenced<i32>>();
//! let stream = rx.into_fluxion_stream();
//!
//! // Random sampling in production - use fastrand's global RNG
//! let seed = std::time::SystemTime::now()
//!     .duration_since(std::time::UNIX_EPOCH)
//!     .unwrap()
//!     .as_secs();
//! let _sampled = stream.sample_ratio(0.1, seed); // Sample ~10%
//! # }
//! ```
//!
//! # Error Handling
//!
//! Errors always pass through unconditionally—they are never subject to sampling.
//! This ensures error observability is not affected by downsampling.

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
pub use multi_threaded::SampleRatioExt;

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
pub use single_threaded::SampleRatioExt;
