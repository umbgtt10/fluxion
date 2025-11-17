// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Test utilities and fixtures for the Fluxion reactive streaming library.
//!
//! This crate provides helper types, test data structures, and utilities for testing
//! stream operators and async processing. It is designed for use in development and
//! testing only, not for production code.
//!
//! # Architecture
//!
//! Fluxion maintains a clean separation between production and test code:
//!
//! - **Production**: Use `FluxionStream` for pure, functional stream operations
//! - **Testing**: Use `tokio::sync::mpsc::unbounded_channel` for imperative test setup
//!
//! This separation solves the conflict between consuming operations (stream extensions
//! that take `self`) and mutation operations (sending values via channels).
//!
//! # Key Types
//!
//! ## `Sequenced<T>`
//!
//! A wrapper type that adds temporal ordering to test values:
//!
//! ```rust
//! use fluxion_test_utils::Sequenced;
//! use fluxion_core::Ordered;
//!
//! let item = Sequenced::new(42);  // Auto-sequenced
//! assert_eq!(item.value, 42);
//! // Sequence numbers are auto-incremented globally
//! ```
//!
//! ## TestData and Variants
//!
//! Enum types for creating diverse test scenarios:
//!
//! ```rust
//! use fluxion_test_utils::test_data::{TestData, person_alice, person_bob};
//!
//! // Use pre-defined fixtures
//! let alice = person_alice();
//! let bob = person_bob();
//!
//! // Create custom test data
//! match alice {
//!     TestData::Person(p) => assert_eq!(p.name, "Alice"),
//!     _ => panic!("Expected person"),
//! }
//! ```
//!
//! ## Test Fixtures
//!
//! Pre-defined types for common test scenarios:
//!
//! - `Person` - Represents a person with id and name
//! - `Animal` - Represents an animal with id and species
//! - `Plant` - Represents a plant with id and name
//!
//! # Examples
//!
//! ## Creating Ordered Test Values
//!
//! ```rust
//! use fluxion_test_utils::Sequenced;
//! use fluxion_core::Ordered;
//!
//! // Create sequenced values with explicit ordering
//! let first = Sequenced::with_sequence(100, 1);
//! let second = Sequenced::with_sequence(200, 2);
//! let third = Sequenced::with_sequence(300, 3);
//!
//! // Verify ordering
//! assert!(first.order() < second.order());
//! assert!(second.order() < third.order());
//!
//! // Access inner values
//! assert_eq!(first.value, 100);
//! assert_eq!(*second.get(), 200);
//! ```
//!
//! ## Using Assertion Helpers
//!
//! ```rust
//! use fluxion_test_utils::assert_no_element_emitted;
//! use futures::stream;
//!
//! # async fn example() {
//! let mut empty = stream::empty::<i32>();
//! assert_no_element_emitted(&mut empty, 10).await;
//! # }
//! ```
//!
//! # Module Organization
//!
//! - `sequenced` - `Sequenced<T>` wrapper and implementations
//! - `test_data` - Enum variants for diverse test scenarios
//! - `person`, `animal`, `plant` - Specific fixture types
//! - `helpers` - Assertion and utility functions

#![allow(clippy::multiple_crate_versions, clippy::doc_markdown)]
pub mod animal;
pub mod error_injection;
pub mod helpers;
pub mod person;
pub mod plant;
pub mod sequenced;
pub mod test_data;

use fluxion_core::StreamItem;
use futures::{Stream, StreamExt};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

// Re-export commonly used test utilities
pub use error_injection::ErrorInjectingStream;
pub use helpers::assert_no_element_emitted;
pub use sequenced::Sequenced;
pub use test_data::{DataVariant, TestData};

/// Creates a test channel that automatically wraps values in `StreamItem::Value`.
///
/// This helper simplifies test setup by handling the `StreamItem` wrapping automatically,
/// allowing tests to send plain values while the stream receives `StreamItem<T>`.
///
/// # Example
///
/// ```rust
/// use fluxion_test_utils::{test_channel, Sequenced};
/// use fluxion_test_utils::test_data::person_alice;
/// use futures::StreamExt;
///
/// # async fn example() {
/// let (tx, mut stream) = test_channel();
///
/// // Send plain values
/// tx.send(Sequenced::new(person_alice())).unwrap();
///
/// // Receive StreamItem-wrapped values
/// let item = stream.next().await.unwrap().unwrap(); // Option -> StreamItem -> Value
/// # }
/// ```
pub fn test_channel<T: Send + 'static>() -> (
    mpsc::UnboundedSender<T>,
    impl Stream<Item = StreamItem<T>> + Send,
) {
    let (tx, rx) = mpsc::unbounded_channel();
    let stream = UnboundedReceiverStream::new(rx).map(StreamItem::Value);
    (tx, stream)
}

/// Creates a test channel that accepts `StreamItem<T>` for testing error propagation.
///
/// This helper allows tests to explicitly send both values and errors through the stream,
/// enabling comprehensive error handling tests.
///
/// # Example
///
/// ```rust
/// use fluxion_test_utils::test_channel_with_errors;
/// use fluxion_core::{StreamItem, FluxionError};
/// use futures::StreamExt;
///
/// # async fn example() {
/// let (tx, mut stream) = test_channel_with_errors();
///
/// // Send values
/// tx.send(StreamItem::Value(42)).unwrap();
///
/// // Send errors
/// tx.send(StreamItem::Error(FluxionError::stream_error("test error"))).unwrap();
///
/// let value = stream.next().await.unwrap();
/// let error = stream.next().await.unwrap();
/// # }
/// ```
pub fn test_channel_with_errors<T: Send + 'static>() -> (
    mpsc::UnboundedSender<StreamItem<T>>,
    impl Stream<Item = StreamItem<T>> + Send,
) {
    let (tx, rx) = mpsc::unbounded_channel();
    let stream = UnboundedReceiverStream::new(rx);
    (tx, stream)
}
