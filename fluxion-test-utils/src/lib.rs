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
//! - **Testing**: Use `TestChannel` for imperative test setup with push capabilities
//!
//! This separation solves the conflict between consuming operations (stream extensions
//! that take `self`) and mutation operations (push that needs `&self`).
//!
//! # Key Types
//!
//! ## Sequenced<T>
//!
//! A wrapper type that adds temporal ordering to test values:
//!
//! ```rust,ignore
//! use fluxion_test_utils::Sequenced;
//!
//! let item = Sequenced::new(42, 1);  // value=42, order=1
//! assert_eq!(item.value, 42);
//! assert_eq!(item.order(), 1);
//! ```
//!
//! ## TestData and Variants
//!
//! Enum types for creating diverse test scenarios:
//!
//! ```rust,ignore
//! use fluxion_test_utils::{TestData, DataVariant};
//!
//! let person = TestData::person(1, "Alice");
//! let animal = TestData::animal(2, "Cat");
//! let plant = TestData::plant(3, "Oak");
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
//! ## Creating Ordered Test Streams
//!
//! ```rust,ignore
//! use fluxion_test_utils::Sequenced;
//! use tokio::sync::mpsc;
//! use fluxion_stream::FluxionStream;
//!
//! #[tokio::test]
//! async fn test_example() {
//!     let (tx, rx) = mpsc::unbounded_channel();
//!     let stream = FluxionStream::from_unbounded_receiver(rx);
//!     
//!     // Send ordered values
//!     tx.send(Sequenced::new(1, 100)).unwrap();
//!     tx.send(Sequenced::new(2, 200)).unwrap();
//!     tx.send(Sequenced::new(3, 300)).unwrap();
//!     
//!     // Test stream operations...
//! }
//! ```
//!
//! ## Using Assertion Helpers
//!
//! ```rust,ignore
//! use fluxion_test_utils::assert_no_element_emitted;
//! use futures::stream;
//!
//! #[tokio::test]
//! async fn test_empty_stream() {
//!     let empty = stream::empty::<i32>();
//!     assert_no_element_emitted(empty).await;
//! }
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
pub mod helpers;
pub mod person;
pub mod plant;
pub mod sequenced;
pub mod test_data;

// Re-export commonly used test utilities
pub use helpers::assert_no_element_emitted;
pub use sequenced::Sequenced;
pub use test_data::{DataVariant, TestData};
