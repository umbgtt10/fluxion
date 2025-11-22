// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Extended trait bounds for ordered Fluxion operators.
//!
//! This module defines [`OrderedFluxionItem`], which extends [`FluxionItem`]
//! with additional bounds required by operators that need ordering and debugging.

use crate::FluxionItem;
use std::fmt::Debug;

/// Extended requirements for operators that need ordering and debugging.
///
/// This trait extends [`FluxionItem`] with additional bounds required by most
/// Fluxion operators: debugging support, total ordering, and unpin semantics.
///
/// # Automatic Implementation
///
/// This trait is automatically implemented for any type that satisfies the bounds:
///
/// ```
/// # use fluxion_core::{OrderedFluxionItem, Timestamped, HasTimestamp};
/// # #[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
/// # struct MyType(u64);
/// # impl HasTimestamp for MyType {
/// #     type Timestamp = u64;
/// #     fn timestamp(&self) -> u64 { self.0 }
/// # }
/// # impl Timestamped for MyType {
/// #     type Inner = MyType;
/// #     fn with_timestamp(value: Self::Inner, ts: u64) -> Self { MyType(ts) }
/// #     fn with_fresh_timestamp(value: Self::Inner) -> Self { value }
/// #     fn into_inner(self) -> Self::Inner { self }
/// # }
/// // MyType automatically implements OrderedFluxionItem
/// fn assert_ordered<T: OrderedFluxionItem>() {}
/// fn example() {
///     assert_ordered::<MyType>();
/// }
/// ```
///
/// # Usage
///
/// Use this trait bound for operators that require ordering or debugging:
///
/// ```
/// # use fluxion_core::{OrderedFluxionItem, StreamItem};
/// # use futures::Stream;
/// pub trait MyOrderedOperatorExt<T>: Stream<Item = StreamItem<T>>
/// where
///     T: OrderedFluxionItem,
/// {
///     // operator implementation that needs Ord, Debug, or Unpin
/// }
/// ```
///
/// # Bounds Breakdown
///
/// - `FluxionItem`: Base requirements (timestamping, cloning, thread safety)
/// - `Debug`: Enables debugging and error messages
/// - `Ord`: Provides total ordering for comparison-based operators
/// - `Unpin`: Allows safe movement in memory (required for most async contexts)
pub trait OrderedFluxionItem: FluxionItem + Debug + Ord + Unpin {}

/// Blanket implementation for all types that satisfy the extended requirements.
impl<T> OrderedFluxionItem for T where T: FluxionItem + Debug + Ord + Unpin {}
