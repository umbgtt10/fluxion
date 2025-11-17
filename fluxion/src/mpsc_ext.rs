// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Extension methods for tokio UnboundedReceiver to create FluxionStreams.

use crate::FluxionStream;
use fluxion_core::Ordered;
use futures::stream::Map;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

/// Type alias for the stream returned by `into_fluxion_stream`
type FluxionStreamFromReceiver<U> =
    FluxionStream<Map<UnboundedReceiverStream<U>, fn(U) -> fluxion_core::StreamItem<U>>>;

/// Extension trait for `UnboundedReceiver` to create FluxionStreams.
pub trait UnboundedReceiverExt<T> {
    /// Converts an `UnboundedReceiver<T>` into a `FluxionStream<U>` by applying a transformation.
    ///
    /// This method creates an intermediate channel internally and spawns a transformation task
    /// that maps items from type `T` to type `U`. This is useful for type erasure when combining
    /// streams that have different underlying types.
    ///
    /// The transformation task is spawned automatically, so you only get back the resulting stream.
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
    /// // Transform SensorReading to DataEvent - task spawned internally
    /// let stream = rx.into_fluxion_stream(|s| DataEvent::Sensor(s.clone()));
    ///
    /// // stream is FluxionStream<DataEvent>
    /// # drop(stream);
    /// # }
    /// ```
    fn into_fluxion_stream<U, F>(self, mapper: F) -> FluxionStreamFromReceiver<U>
    where
        F: FnMut(T) -> U + Send + 'static,
        U: Ordered<Inner = U> + Clone + std::fmt::Debug + Ord + Send + Sync + Unpin + 'static;
}

impl<T> UnboundedReceiverExt<T> for mpsc::UnboundedReceiver<T>
where
    T: Ordered<Inner = T> + Clone + std::fmt::Debug + Ord + Send + Sync + Unpin + 'static,
{
    fn into_fluxion_stream<U, F>(self, mut mapper: F) -> FluxionStreamFromReceiver<U>
    where
        F: FnMut(T) -> U + Send + 'static,
        U: Ordered<Inner = U> + Clone + std::fmt::Debug + Ord + Send + Sync + Unpin + 'static,
    {
        use futures::StreamExt;

        // Create intermediate channel for transformed items
        let (tx, rx) = mpsc::unbounded_channel();

        // Create and spawn the transformation task
        let task = FluxionStream::from_unbounded_receiver(self).for_each(move |item| {
            // Extract value from StreamItem
            if let fluxion_core::StreamItem::Value(value) = item {
                let transformed = mapper(value);
                let _ = tx.send(transformed);
            }
            // Note: Errors are silently dropped. Consider logging or propagating them.
            async {}
        });

        tokio::spawn(task);

        // Return the stream from the intermediate channel with concrete type
        FluxionStream::new(
            UnboundedReceiverStream::new(rx)
                .map(fluxion_core::StreamItem::Value as fn(U) -> fluxion_core::StreamItem<U>),
        )
    }
}
