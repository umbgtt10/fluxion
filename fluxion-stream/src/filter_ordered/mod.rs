// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

/// Extension trait providing the `filter_ordered` operator for streams.
///
/// This trait allows any stream of `StreamItem<T>` to filter items while
/// preserving temporal ordering semantics.
///
/// Use [`FilterOrderedExt::filter_ordered`] to use this operator.
///
/// # Behavior
///
/// - Predicate receives a reference to the **inner value** (`&T::Inner`), not the full wrapper
/// - Only items where predicate returns `true` are emitted
/// - Filtered items preserve their original temporal order
/// - Errors are passed through unchanged
///
/// # Arguments
///
/// * `predicate` - A function that takes `&T::Inner` and returns `true` to keep the item
///
/// # Returns
///
/// A stream of `StreamItem<T>` containing only items that passed the filter
///
/// # Examples
///
/// ```rust
/// use fluxion_stream::{FilterOrderedExt, IntoFluxionStream};
/// use fluxion_test_utils::Sequenced;
/// use futures::StreamExt;
///
/// # async fn example() {
/// let (tx, rx) = async_channel::unbounded();
/// let stream = rx.into_fluxion_stream();
///
/// // Filter for even numbers
/// let mut evens = stream.filter_ordered(|&n| n % 2 == 0);
///
/// tx.try_send(Sequenced::new(1)).unwrap();
/// tx.try_send(Sequenced::new(2)).unwrap();
/// tx.try_send(Sequenced::new(3)).unwrap();
/// tx.try_send(Sequenced::new(4)).unwrap();
///
/// assert_eq!(evens.next().await.unwrap().unwrap().into_inner(), 2);
/// assert_eq!(evens.next().await.unwrap().unwrap().into_inner(), 4);
/// # }
/// ```
///
/// # Use Cases
///
/// - **Threshold Filtering**: Remove values outside acceptable ranges
/// - **Type-Based Filtering**: Select specific enum variants
/// - **Conditional Processing**: Process only relevant events
/// - **Noise Reduction**: Remove unwanted data from streams
///
/// # Errors
///
/// This operator does not produce errors under normal operation. The predicate function receives
/// unwrapped values, and any errors from upstream are passed through unchanged.
///
/// # See Also
///
/// - [`MapOrderedExt::map_ordered`](crate::MapOrderedExt::map_ordered) - Transform items
/// - [`EmitWhenExt::emit_when`](crate::EmitWhenExt::emit_when) - Filter based on secondary stream
/// - [`TakeWhileExt::take_while_with`](crate::TakeWhileExt::take_while_with) - Filter until condition fails
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
pub use multi_threaded::FilterOrderedExt;

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
pub use single_threaded::FilterOrderedExt;
