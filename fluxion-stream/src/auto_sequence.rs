// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Automatic ordering for streams containing Ordered items.

use crate::FluxionStream;
use fluxion_core::Ordered;
use futures::Stream;

/// Extension trait for streams of Ordered items.
///
/// This trait provides the `auto_ordered` method which wraps a stream of
/// items that already implement `Ordered` with `Inner = Self` into a FluxionStream.
/// This enables ordered stream operations like `map_ordered`, `filter_ordered`, etc.
///
/// # Examples
///
/// ```
/// use fluxion_stream::{FluxionStream, AutoOrderedExt};
/// use fluxion_core::Ordered;
///
/// #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
/// struct Event {
///     timestamp: u64,
///     data: String,
/// }
///
/// impl Ordered for Event {
///     type Inner = Event;
///     fn order(&self) -> u64 { self.timestamp }
///     fn get(&self) -> &Self::Inner { self }
///     fn with_order(value: Self::Inner, _order: u64) -> Self { value }
/// }
///
/// # #[tokio::main]
/// # async fn main() {
/// let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Event>();
///
/// // auto_ordered() wraps in FluxionStream for ordered operations
/// let stream = FluxionStream::from_unbounded_receiver(rx)
///     .auto_ordered(); // Now can use map_ordered, filter_ordered, etc.
/// # drop(stream);
/// # }
/// ```
pub trait AutoOrderedExt<T>: Stream<Item = T> + Sized {
    /// Wraps the stream in a FluxionStream for Ordered items where Inner = Self.
    ///
    /// This is useful for enabling ordered stream operations on items that
    /// already implement `Ordered` with `Inner = Self`.
    ///
    /// # Type Parameters
    /// * `T` - Must implement `Ordered` with `Inner = T`
    fn auto_ordered(self) -> FluxionStream<Self>
    where
        T: Ordered<Inner = T>,
    {
        FluxionStream::new(self)
    }
}

impl<T, S> AutoOrderedExt<T> for S where S: Stream<Item = T> {}
