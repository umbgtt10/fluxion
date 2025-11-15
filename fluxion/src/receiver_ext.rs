// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Extension methods for tokio UnboundedReceiver to create FluxionStreams.

use crate::FluxionStream;
use fluxion_core::Ordered;
use tokio::sync::mpsc;

/// Extension trait for `UnboundedReceiver` to create FluxionStreams.
pub trait UnboundedReceiverExt<T> {
    /// Converts an `UnboundedReceiver` into a `FluxionStream` for ordered items.
    ///
    /// This is a convenience method for items that implement `Ordered<Inner = Self>`.
    /// It wraps the receiver in a FluxionStream, enabling ordered stream operations.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fluxion::prelude::*;
    /// use tokio::sync::mpsc;
    ///
    /// #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    /// struct SensorReading {
    ///     timestamp: u64,
    ///     temperature: i32,
    /// }
    ///
    /// impl Ordered for SensorReading {
    ///     type Inner = SensorReading;
    ///     fn order(&self) -> u64 { self.timestamp }
    ///     fn get(&self) -> &Self::Inner { self }
    ///     fn with_order(value: Self::Inner, _order: u64) -> Self { value }
    /// }
    ///
    /// # async fn example() {
    /// let (tx, rx) = mpsc::unbounded_channel::<SensorReading>();
    ///
    /// // Convert receiver to FluxionStream and use ordered operations
    /// let stream = rx.into_fluxion_stream()
    ///     .map_ordered(|reading| reading.temperature)
    ///     .filter_ordered(|temp| temp.get() > &200);
    /// # }
    /// ```
    /// Converts an `UnboundedReceiver<T>` into a `FluxionStream<U>` by applying a transformation.
    ///
    /// This method creates an intermediate channel internally and handles the transformation task.
    /// This is useful for type erasure when combining streams that have different underlying types.
    ///
    /// By default, spawns the transformation task automatically and returns just the stream.
    /// If you need control over task execution, pass `spawn: false` to get a tuple with the task.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use fluxion::prelude::*;
    /// use tokio::sync::mpsc;
    ///
    /// # #[derive(Clone)]
    /// # struct SensorReading { timestamp: u64 }
    /// # impl Ordered for SensorReading {
    /// #     type Inner = Self;
    /// #     fn order(&self) -> u64 { self.timestamp }
    /// #     fn get(&self) -> &Self::Inner { self }
    /// #     fn with_order(v: Self, _: u64) -> Self { v }
    /// # }
    /// #
    /// # #[derive(Clone)]
    /// # enum DataEvent { Sensor(SensorReading) }
    /// # impl Ordered for DataEvent {
    /// #     type Inner = Self;
    /// #     fn order(&self) -> u64 { match self { DataEvent::Sensor(s) => s.timestamp } }
    /// #     fn get(&self) -> &Self::Inner { self }
    /// #     fn with_order(v: Self, _: u64) -> Self { v }
    /// # }
    ///
    /// # async fn example() {
    /// let (tx, rx) = mpsc::unbounded_channel::<SensorReading>();
    ///
    /// // Transform with automatic task spawning (default)
    /// let stream = rx.into_fluxion_stream(|s| DataEvent::Sensor(s.clone()));
    ///
    /// // stream is FluxionStream<DataEvent>
    /// # drop(stream);
    /// # }
    /// ```
    fn into_fluxion_stream<U, F>(
        self,
        mapper: F,
    ) -> FluxionStream<tokio_stream::wrappers::UnboundedReceiverStream<U>>
    where
        F: FnMut(T) -> U + Send + 'static,
        U: Send + 'static;
}

impl<T> UnboundedReceiverExt<T> for mpsc::UnboundedReceiver<T>
where
    T: Ordered<Inner = T> + Clone + std::fmt::Debug + Ord + Send + Sync + Unpin + 'static,
{
    fn into_fluxion_stream<U, F>(
        self,
        mut mapper: F,
    ) -> FluxionStream<tokio_stream::wrappers::UnboundedReceiverStream<U>>
    where
        F: FnMut(T) -> U + Send + 'static,
        U: Send + 'static,
    {
        use futures::StreamExt;

        // Create intermediate channel for transformed items
        let (tx, rx) = mpsc::unbounded_channel();

        // Create and spawn the transformation task
        let task = FluxionStream::from_unbounded_receiver(self).for_each(move |item| {
            let transformed = mapper(item);
            let _ = tx.send(transformed);
            async {}
        });

        tokio::spawn(task);

        // Return the stream from the intermediate channel
        FluxionStream::from_unbounded_receiver(rx)
    }
}
