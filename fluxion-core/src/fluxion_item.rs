// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Base trait bounds for all Fluxion operators.
//!
//! This module defines [`FluxionItem`], which consolidates the minimal set
//! of trait bounds required by all Fluxion operators.

use crate::Timestamped;

/// Base requirements for all Fluxion operators.
///
/// This trait represents the minimal set of bounds required by Fluxion operators.
/// It combines timestamping, cloning, thread safety, and static lifetime requirements.
///
/// # Automatic Implementation
///
/// This trait is automatically implemented for any type that satisfies the bounds:
///
/// ```
/// # use fluxion_core::{FluxionItem, Timestamped, HasTimestamp};
/// # #[derive(Clone)]
/// # struct MyType;
/// # impl HasTimestamp for MyType {
/// #     type Inner = MyType;
/// #     type Timestamp = u64;
/// #     fn timestamp(&self) -> u64 { 0 }
/// # }
/// # impl Timestamped for MyType {
/// #     fn with_timestamp(value: Self::Inner, _: u64) -> Self { value }
/// #     fn with_fresh_timestamp(value: Self::Inner) -> Self { value }
/// #     fn into_inner(self) -> Self::Inner { self }
/// # }
/// // MyType automatically implements FluxionItem if it meets the bounds
/// fn assert_fluxion_item<T: FluxionItem>() {}
/// fn example() {
///     assert_fluxion_item::<MyType>();
/// }
/// ```
///
/// # Usage
///
/// Use this trait bound for operators that only require basic functionality:
///
/// ```
/// # use fluxion_core::{FluxionItem, StreamItem};
/// # use futures::Stream;
/// pub trait MyOperatorExt<T>: Stream<Item = StreamItem<T>>
/// where
///     T: FluxionItem,
/// {
///     // operator implementation
/// }
/// ```
pub trait FluxionItem: Timestamped + Clone + Send + Sync + 'static {}

/// Blanket implementation for all types that satisfy the base requirements.
impl<T> FluxionItem for T where T: Timestamped + Clone + Send + Sync + 'static {}
