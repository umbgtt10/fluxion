// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Extension methods for tokio UnboundedReceiver to create FluxionStreams.

use crate::FluxionStream;
use fluxion_core::{Ordered, StreamItem};
use futures::{Stream, StreamExt};
use std::fmt::Debug;
use std::pin::Pin;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio_stream::wrappers::UnboundedReceiverStream;

/// Extension trait for `UnboundedReceiver` to create FluxionStreams.
pub trait UnboundedReceiverExt<T> {
    /// Converts an `UnboundedReceiver<T>` into a `FluxionStream<U>` by applying a transformation.
    ///
    /// This method transforms items from type `T` to type `U` and returns a boxed stream.
    /// The boxed return type allows streams with different source types to be combined easily,
    /// which is useful when working with operators like `combine_latest`.
    ///
    /// # Type Erasure
    ///
    /// The returned stream is boxed (`Pin<Box<dyn Stream + Send + Sync>>`), which erases the
    /// concrete type of the underlying channel. This allows you to combine multiple receivers
    /// of different types (e.g., `UnboundedReceiver<SensorData>` and
    /// `UnboundedReceiver<MetricData>`) into a single collection, as long as they all map to
    /// the same output type `U`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fluxion_rx::prelude::*;
    /// use tokio::sync::mpsc;
    ///
    /// #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    /// struct SensorReading {
    ///     timestamp: u64,
    ///     temperature: i32,
    /// }
    ///
    /// impl Ordered for SensorReading {
    ///     type Inner = Self;
    ///     fn order(&self) -> u64 { self.timestamp }
    ///     fn get(&self) -> &Self::Inner { self }
    ///     fn with_order(value: Self, _order: u64) -> Self { value }
    /// }
    ///
    /// #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    /// enum DataEvent {
    ///     Sensor(SensorReading)
    /// }
    ///
    /// impl Ordered for DataEvent {
    ///     type Inner = Self;
    ///     fn order(&self) -> u64 {
    ///         match self {
    ///             DataEvent::Sensor(s) => s.timestamp
    ///         }
    ///     }
    ///     fn get(&self) -> &Self::Inner { self }
    ///     fn with_order(value: Self, _order: u64) -> Self { value }
    /// }
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let (tx, rx) = mpsc::unbounded_channel::<SensorReading>();
    ///
    /// // Transform SensorReading to DataEvent
    /// let stream = rx.into_fluxion_stream(|s| DataEvent::Sensor(s.clone()));
    ///
    /// // stream is FluxionStream with boxed inner stream
    /// # drop(stream);
    /// # }
    /// ```
    fn into_fluxion_stream<U, F>(
        self,
        mapper: F,
    ) -> FluxionStream<Pin<Box<dyn Stream<Item = StreamItem<U>> + Send + Sync>>>
    where
        F: FnMut(T) -> U + Send + Sync + 'static,
        U: Ordered<Inner = U> + Clone + Debug + Ord + Send + Sync + Unpin + 'static;
}

impl<T> UnboundedReceiverExt<T> for UnboundedReceiver<T>
where
    T: Ordered<Inner = T> + Clone + Debug + Ord + Send + Sync + Unpin + 'static,
{
    fn into_fluxion_stream<U, F>(
        self,
        mut mapper: F,
    ) -> FluxionStream<Pin<Box<dyn Stream<Item = StreamItem<U>> + Send + Sync>>>
    where
        F: FnMut(T) -> U + Send + Sync + 'static,
        U: Ordered<Inner = U> + Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    {
        // Transform the stream and box it to erase the concrete type
        let source = UnboundedReceiverStream::new(self);
        let mapped = source.map(move |value| StreamItem::Value(mapper(value)));
        let boxed = Box::pin(mapped);

        FluxionStream::new(boxed)
    }
}
