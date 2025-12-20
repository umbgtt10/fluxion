// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! # Fluxion
//!
//! A reactive stream processing library with ordered semantics, friendly interface, and bullet-proof, state-of-the art test coverage and examples.
//!
//! ## Overview
//!
//! Fluxion provides a high-level API for working with ordered, reactive streams.
//! It builds on top of the Rust async ecosystem (tokio, futures) and adds
//! ordering guarantees and powerful stream composition operators.
//!
//! ## Design Philosophy
//!
//! Fluxion maintains a clean separation of concerns:
//!
//! - **Production code**: Use `FluxionStream` for composable, immutable stream transformations
//! - **Test code**: Use `futures::channel::mpsc` channels for imperative test setup
//!
//! This architecture solves the fundamental conflict between:
//! - Consuming operations (stream extensions that take `self`)
//! - Mutation operations (sending values via channels)
//!
//! ## Quick Start
//!
//! ```rust
//! use fluxion_rx::prelude::*;
//! use futures::StreamExt;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Create a stream from a futures channel
//!     let (tx, rx) = futures::channel::mpsc::unbounded::<i32>();
//!     let stream = rx.into_fluxion_stream();
//!
//!     // Send some values
//!     tx.unbounded_send(1).unwrap();
//!     tx.unbounded_send(2).unwrap();
//!     tx.unbounded_send(3).unwrap();
//!     drop(tx);
//!
//!     // Process the stream (unwrap StreamItem values)
//!     let sum: i32 = stream.fold(0, |acc, x| async move {
//!         acc + x.unwrap()
//!     }).await;
//!     println!("Sum: {}", sum);  // Prints: Sum: 6
//! }
//! ```
//!
//! ## Core Concepts
//!
//! ### Timestamped Trait
//!
//! All stream operators work with types implementing the [`Timestamped`] trait, which
//! provides temporal ordering:
//!
//! ```rust
//! use fluxion_rx::Timestamped;
//!
//! // Items must provide a timestamp
//! fn process_timestamped<T: Timestamped>(item: T) {
//!     let ts = item.timestamp();     // Get timestamp for ordering
//!     let value = item.into_inner();  // Extract inner value
//! }
//! ```
//!
//! ### Stream Operators
//!
//! Fluxion provides powerful stream composition operators:
//!
//! - **combine_latest** - Combine multiple streams, emitting when any emits
//! - **with_latest_from** - Sample one stream using another as trigger
//! - **ordered_merge** - Merge streams preserving temporal order
//! - **take_latest_when** - Sample on filter condition
//! - **emit_when** - Gate stream emissions based on conditions
//!
//! See [`fluxion_stream`] for the complete list.
//!
//! ## Workspace Structure
//!
//! - [`fluxion`](crate) - Main crate (this crate), re-exports core types
//! - [`fluxion_core`] - Core traits and utilities
//! - [`fluxion_stream`] - Stream operators and combinators
//! - `fluxion_exec` - Async execution and subscription utilities
// fluxion_error - Error types and handling

// Re-export core types
pub use fluxion_core::into_stream::IntoStream;
pub use fluxion_core::{HasTimestamp, Timestamped};

// Re-export commonly used types
pub use fluxion_stream::{CombinedState, WithPrevious};

// Re-export exec utilities
pub use fluxion_exec;

/// Prelude module for convenient imports.
///
/// Import this module to bring the most commonly used Fluxion types into scope:
///
/// ```rust
/// use fluxion_rx::prelude::*;
///
/// // Now you have access to:
/// // - Extension traits via fluxion_stream::prelude
/// // - Timestamped trait
/// // - IntoStream trait
/// // - CombinedState, WithPrevious
/// ```
///
/// This is the recommended way to use Fluxion in most applications.
pub mod prelude {
    pub use fluxion_core::into_stream::IntoStream;
    pub use fluxion_core::{HasTimestamp, Timestamped};
    pub use fluxion_stream::prelude::*;
    pub use fluxion_stream::{CombinedState, WithPrevious};
}
