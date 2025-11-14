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
//! - **Test code**: Use `TestChannel` (from `fluxion-test-utils`) which adds push capabilities
//!
//! This architecture solves the fundamental conflict between:
//! - Consuming operations (stream extensions that take `self`)
//! - Mutation operations (push that needs `&self`)
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use fluxion::FluxionStream;
//! use tokio_stream::wrappers::UnboundedReceiverStream;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Create a stream from a tokio channel
//!     let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<String>();
//!     let stream = FluxionStream::from_unbounded_receiver(rx);
//!
//!     // Use stream operators
//!     // let result = stream.combine_latest(others, filter);
//! }
//! ```

// Re-export core types
pub use fluxion_core::into_stream::IntoStream;
pub use fluxion_core::{CompareByInner, Ordered, OrderedWrapper};

// Re-export the main FluxionStream type
pub use fluxion_stream::FluxionStream;

// Re-export commonly used types
pub use fluxion_stream::combine_latest::CombinedState;

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::FluxionStream;
    pub use fluxion_core::into_stream::IntoStream;
    pub use fluxion_core::Ordered;
    pub use fluxion_stream::combine_latest::CombinedState;
}
