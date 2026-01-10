// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use async_channel::Receiver;
use core::fmt::Debug;
use core::pin::Pin;
use fluxion_core::{StreamItem, Timestamped};
use futures::{Stream, StreamExt};
use std::boxed::Box;

/// Extension trait to convert async_channel receivers into fluxion streams.
///
/// This trait provides a simple way to wrap an `async_channel::Receiver` into
/// a stream that emits `StreamItem::Value` for each received item.
///
/// For single-threaded runtimes (Embassy, WASM).
pub trait IntoFluxionStream<T> {
    /// Converts this receiver into a fluxion stream.
    ///
    /// Each item received from the channel is wrapped in `StreamItem::Value`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fluxion_stream_single::IntoFluxionStream;
    /// use async_channel::unbounded;
    ///
    /// let (tx, rx) = unbounded::<i32>();
    /// let stream = rx.into_fluxion_stream();
    /// ```
    fn into_fluxion_stream(self) -> impl Stream<Item = StreamItem<T>>;

    /// Converts this receiver into a fluxion stream by applying a transformation.
    ///
    /// This method transforms items from type `T` to type `U` and returns a boxed stream.
    /// The boxed return type allows streams with different source types to be combined easily,
    /// which is useful when working with operators like `combine_latest`.
    ///
    /// # Type Erasure
    ///
    /// The returned stream is boxed (`Pin<Box<dyn Stream>>`), which erases the
    /// concrete type of the underlying channel. This allows you to combine multiple receivers
    /// of different types (e.g., `Receiver<SensorData>` and `Receiver<MetricData>`) into a
    /// single collection, as long as they all map to the same output type `U`.
    ///
    /// # Examples
    ///
    /// ```
    /// use fluxion_stream_single::IntoFluxionStream;
    /// use fluxion_core::{HasTimestamp, Timestamped};
    /// use async_channel::unbounded;
    ///
    /// #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    /// struct SensorReading {
    ///     timestamp: u64,
    ///     temperature: i32,
    /// }
    ///
    /// impl HasTimestamp for SensorReading {
    ///     type Timestamp = u64;
    ///     fn timestamp(&self) -> u64 { self.timestamp }
    /// }
    ///
    /// impl Timestamped for SensorReading {
    ///     type Inner = Self;
    ///     fn into_inner(self) -> Self { self }
    ///     fn with_timestamp(value: Self, _timestamp: u64) -> Self { value }
    /// }
    ///
    /// #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    /// enum DataEvent {
    ///     Sensor(SensorReading)
    /// }
    ///
    /// impl HasTimestamp for DataEvent {
    ///     type Timestamp = u64;
    ///     fn timestamp(&self) -> u64 {
    ///         match self {
    ///             DataEvent::Sensor(s) => s.timestamp
    ///         }
    ///     }
    /// }
    ///
    /// impl Timestamped for DataEvent {
    ///     type Inner = Self;
    ///     fn into_inner(self) -> Self { self }
    ///     fn with_timestamp(value: Self, _timestamp: u64) -> Self { value }
    /// }
    ///
    /// # async fn example() {
    /// let (tx, rx) = unbounded::<SensorReading>();
    ///
    /// // Transform SensorReading to DataEvent
    /// let stream = rx.into_fluxion_stream_map(|s| DataEvent::Sensor(s.clone()));
    ///
    /// // stream is a boxed stream
    /// # drop(stream);
    /// # }
    /// ```
    fn into_fluxion_stream_map<U, F>(
        self,
        mapper: F,
    ) -> Pin<Box<dyn Stream<Item = StreamItem<U>> + Send + Sync>>
    where
        F: FnMut(T) -> U + Send + Sync + 'static,
        U: Timestamped<Inner = U> + Clone + Debug + Ord + Unpin + Send + Sync + 'static;
}

impl<T: Send + 'static> IntoFluxionStream<T> for Receiver<T> {
    fn into_fluxion_stream(self) -> impl Stream<Item = StreamItem<T>> {
        Box::pin(self.map(StreamItem::Value))
    }

    fn into_fluxion_stream_map<U, F>(
        self,
        mut mapper: F,
    ) -> Pin<Box<dyn Stream<Item = StreamItem<U>> + Send + Sync>>
    where
        F: FnMut(T) -> U + Send + Sync + 'static,
        U: Timestamped<Inner = U> + Clone + Debug + Ord + Unpin + Send + Sync + 'static,
    {
        Box::pin(self.map(move |value| StreamItem::Value(mapper(value))))
    }
}
