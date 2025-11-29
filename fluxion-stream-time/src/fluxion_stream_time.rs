// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::{chrono_timestamped::ChronoTimestamped, debounce, delay, sample, throttle};
use fluxion_core::{ComparableUnpin, StreamItem};
use fluxion_stream::FluxionStream;
use futures::Stream;
use std::fmt::Debug;

/// Extension trait for time-based operators on `FluxionStream` with `ChronoTimestamped` items.
///
/// This trait provides time-based operators for streams where items are
/// `ChronoTimestamped<T>`, following the same pattern as `map_ordered` and
/// `filter_ordered` in `FluxionStream`.
pub trait FluxionStreamTimeOps<S, T>
where
    S: Stream<Item = StreamItem<ChronoTimestamped<T>>> + Send + Sync + Unpin + 'static,
    T: ComparableUnpin + Send + 'static,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin,
{
    /// Delays each emission by the specified duration while preserving temporal ordering.
    ///
    /// This operator shifts all emissions forward in time by the given duration.
    /// Each item is delayed independently, maintaining the stream's temporal ordering.
    /// Errors pass through immediately without delay for timely error propagation.
    ///
    /// # Arguments
    ///
    /// * `duration` - The duration by which to delay each emission
    ///
    /// # Returns
    ///
    /// A new `FluxionStream` where each value emission is delayed by the specified duration.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fluxion_stream::FluxionStream;
    /// use fluxion_stream_time::{ChronoTimestamped, ChronoStreamOps};
    /// use fluxion_core::StreamItem;
    /// use futures::stream;
    /// use chrono::Duration;
    ///
    /// # async fn example() {
    /// let source = stream::iter(vec![
    ///     StreamItem::Value(ChronoTimestamped::now(42)),
    /// ]);
    ///
    /// let delayed = FluxionStream::new(source)
    ///     .delay(std::time::Duration::from_millis(100));
    /// # }
    /// ```
    fn delay(
        self,
        duration: std::time::Duration,
    ) -> FluxionStream<impl Stream<Item = StreamItem<ChronoTimestamped<T>>> + Send + Sync>;

    /// Debounces the stream by the specified duration.
    ///
    /// The debounce operator emits a value only after a pause in emissions of at least
    /// the given duration. If a new value arrives before the timer expires, the timer
    /// resets and only the newest value is eventually emitted.
    ///
    /// This implements **trailing debounce** (Rx standard): values are emitted only after
    /// the specified quiet period. When the stream ends, any pending value is emitted immediately.
    ///
    /// Errors pass through immediately without debounce to ensure timely error propagation.
    ///
    /// # Arguments
    ///
    /// * `duration` - The duration of required inactivity before emitting a value
    ///
    /// # Returns
    ///
    /// A new `FluxionStream` where values are debounced by the specified duration.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fluxion_stream::FluxionStream;
    /// use fluxion_stream_time::{ChronoTimestamped, ChronoStreamOps};
    /// use fluxion_core::StreamItem;
    /// use futures::stream;
    /// use chrono::Duration;
    ///
    /// # async fn example() {
    /// let source = stream::iter(vec![
    ///     StreamItem::Value(ChronoTimestamped::now(42)),
    /// ]);
    ///
    /// let debounced = FluxionStream::new(source)
    ///     .debounce(std::time::Duration::from_millis(100));
    /// # }
    /// ```
    fn debounce(
        self,
        duration: std::time::Duration,
    ) -> FluxionStream<impl Stream<Item = StreamItem<ChronoTimestamped<T>>> + Send + Sync>;

    /// Throttles the stream by the specified duration.
    ///
    /// The throttle operator emits the first value, then ignores subsequent values
    /// for the specified duration. After the duration expires, it accepts the next
    /// value and repeats the process.
    ///
    /// This implements **leading throttle** semantics: values are emitted immediately,
    /// then a quiet period is enforced.
    ///
    /// Errors pass through immediately without throttling to ensure timely error propagation.
    ///
    /// # Arguments
    ///
    /// * `duration` - The duration to ignore values after an emission
    ///
    /// # Returns
    ///
    /// A new `FluxionStream` where values are throttled by the specified duration.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fluxion_stream::FluxionStream;
    /// use fluxion_stream_time::{ChronoTimestamped, ChronoStreamOps};
    /// use fluxion_core::StreamItem;
    /// use futures::stream;
    /// use chrono::Duration;
    ///
    /// # async fn example() {
    /// let source = stream::iter(vec![
    ///     StreamItem::Value(ChronoTimestamped::now(42)),
    /// ]);
    ///
    /// let throttled = FluxionStream::new(source)
    ///     .throttle(std::time::Duration::from_millis(100));
    /// # }
    /// ```
    fn throttle(
        self,
        duration: std::time::Duration,
    ) -> FluxionStream<impl Stream<Item = StreamItem<ChronoTimestamped<T>>> + Send + Sync>;

    /// Samples the stream at periodic intervals.
    ///
    /// The sample operator emits the most recently emitted value from the source
    /// stream within periodic time intervals.
    ///
    /// - If the source emits multiple values within the interval, only the last one is emitted.
    /// - If the source emits no values within the interval, nothing is emitted for that interval.
    /// - Errors are passed through immediately.
    ///
    /// # Arguments
    ///
    /// * `duration` - The sampling interval
    ///
    /// # Returns
    ///
    /// A new `FluxionStream` where values are sampled at the specified interval.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fluxion_stream::FluxionStream;
    /// use fluxion_stream_time::{ChronoTimestamped, ChronoStreamOps};
    /// use fluxion_core::StreamItem;
    /// use futures::stream;
    /// use chrono::Duration;
    ///
    /// # async fn example() {
    /// let source = stream::iter(vec![
    ///     StreamItem::Value(ChronoTimestamped::now(42)),
    /// ]);
    ///
    /// let sampled = FluxionStream::new(source)
    ///     .sample(std::time::Duration::from_millis(100));
    /// # }
    /// ```
    fn sample(
        self,
        duration: std::time::Duration,
    ) -> FluxionStream<impl Stream<Item = StreamItem<ChronoTimestamped<T>>> + Send + Sync>;
}

impl<S, T> FluxionStreamTimeOps<S, T> for FluxionStream<S>
where
    S: Stream<Item = StreamItem<ChronoTimestamped<T>>> + Send + Sync + Unpin + 'static,
    T: ComparableUnpin + Send + 'static,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin,
{
    fn delay(
        self,
        duration: std::time::Duration,
    ) -> FluxionStream<impl Stream<Item = StreamItem<ChronoTimestamped<T>>> + Send + Sync> {
        let inner = self.into_inner();
        FluxionStream::new(delay(inner, duration))
    }

    fn debounce(
        self,
        duration: std::time::Duration,
    ) -> FluxionStream<impl Stream<Item = StreamItem<ChronoTimestamped<T>>> + Send + Sync> {
        let inner = self.into_inner();
        FluxionStream::new(debounce(inner, duration))
    }

    fn throttle(
        self,
        duration: std::time::Duration,
    ) -> FluxionStream<impl Stream<Item = StreamItem<ChronoTimestamped<T>>> + Send + Sync> {
        let inner = self.into_inner();
        FluxionStream::new(throttle(inner, duration))
    }

    fn sample(
        self,
        duration: std::time::Duration,
    ) -> FluxionStream<impl Stream<Item = StreamItem<ChronoTimestamped<T>>> + Send + Sync> {
        let inner = self.into_inner();
        FluxionStream::new(sample(inner, duration))
    }
}
