// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::{chrono_timestamped::ChronoTimestamped, debounce, delay, sample, throttle, timeout};
use fluxion_core::{ComparableUnpin, StreamItem};
use fluxion_stream::FluxionStream;
use futures::Stream;
use std::fmt::Debug;
use std::time::Duration;

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
    /// use fluxion_test_utils::test_data::person_alice;
    /// use futures::stream::StreamExt;
    /// use std::time::Duration;
    /// use tokio::sync::mpsc;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let (tx, rx) = mpsc::unbounded_channel();
    /// let mut stream = FluxionStream::from_unbounded_receiver(rx)
    ///     .delay(Duration::from_millis(10));
    ///
    /// tx.send(ChronoTimestamped::now(person_alice())).unwrap();
    ///
    /// let item = stream.next().await.unwrap().unwrap();
    /// assert_eq!(item.value, person_alice());
    /// # }
    /// ```
    fn delay(
        self,
        duration: Duration,
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
    /// use fluxion_test_utils::test_data::{person_alice, person_bob};
    /// use futures::stream::StreamExt;
    /// use std::time::Duration;
    /// use tokio::sync::mpsc;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let (tx, rx) = mpsc::unbounded_channel();
    /// let mut stream = FluxionStream::from_unbounded_receiver(rx)
    ///     .debounce(Duration::from_millis(100));
    ///
    /// // Alice and Bob emitted immediately. Alice should be debounced (dropped).
    /// tx.send(ChronoTimestamped::now(person_alice())).unwrap();
    /// tx.send(ChronoTimestamped::now(person_bob())).unwrap();
    ///
    /// // Only Bob should remain (trailing debounce)
    /// let item = stream.next().await.unwrap().unwrap();
    /// assert_eq!(item.value, person_bob());
    /// # }
    /// ```
    fn debounce(
        self,
        duration: Duration,
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
    /// use fluxion_test_utils::test_data::{person_alice, person_bob};
    /// use futures::stream::StreamExt;
    /// use std::time::Duration;
    /// use tokio::sync::mpsc;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let (tx, rx) = mpsc::unbounded_channel();
    /// let mut stream = FluxionStream::from_unbounded_receiver(rx)
    ///     .throttle(Duration::from_millis(100));
    ///
    /// // Alice and Bob emitted immediately. Bob should be throttled (dropped).
    /// tx.send(ChronoTimestamped::now(person_alice())).unwrap();
    /// tx.send(ChronoTimestamped::now(person_bob())).unwrap();
    ///
    /// // Only Alice should remain (leading throttle)
    /// let item = stream.next().await.unwrap().unwrap();
    /// assert_eq!(item.value, person_alice());
    /// # }
    /// ```
    fn throttle(
        self,
        duration: Duration,
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
    /// use fluxion_test_utils::test_data::{person_alice, person_bob};
    /// use futures::stream::StreamExt;
    /// use std::time::Duration;
    /// use tokio::sync::mpsc;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// // Use a channel to control emission timing relative to the sample interval
    /// let (tx, rx) = mpsc::unbounded_channel();
    /// let mut stream = FluxionStream::from_unbounded_receiver(rx)
    ///     .sample(Duration::from_millis(10));
    ///
    /// // Emit Alice and Bob immediately
    /// tx.send(ChronoTimestamped::now(person_alice())).unwrap();
    /// tx.send(ChronoTimestamped::now(person_bob())).unwrap();
    ///
    /// // Wait for sample duration
    /// tokio::time::sleep(Duration::from_millis(20)).await;
    ///
    /// // Sample should pick the latest one (Bob)
    /// let item = stream.next().await.unwrap().unwrap();
    /// assert_eq!(item.value, person_bob());
    /// # }
    /// ```
    fn sample(
        self,
        duration: Duration,
    ) -> FluxionStream<impl Stream<Item = StreamItem<ChronoTimestamped<T>>> + Send + Sync>;

    /// Errors if the stream does not emit any value within the specified duration.
    ///
    /// The timeout operator monitors the time interval between emissions from the source stream.
    /// If the source stream does not emit any value within the specified duration, the operator
    /// emits a `FluxionError::StreamProcessingError` with "Timeout" context and terminates the stream.
    ///
    /// - If the source emits a value, the timer is reset.
    /// - If the source completes, the timeout operator completes.
    /// - If the source errors, the error is passed through and the timer is reset.
    ///
    /// # Arguments
    ///
    /// * `duration` - The timeout duration
    ///
    /// # Returns
    ///
    /// A new `FluxionStream` that enforces the timeout policy.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fluxion_stream::FluxionStream;
    /// use fluxion_stream_time::{ChronoTimestamped, ChronoStreamOps};
    /// use fluxion_core::StreamItem;
    /// use fluxion_test_utils::test_data::person_alice;
    /// use futures::stream::StreamExt;
    /// use std::time::Duration;
    /// use tokio::sync::mpsc;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let (tx, rx) = mpsc::unbounded_channel();
    /// let mut stream = FluxionStream::from_unbounded_receiver(rx)
    ///     .timeout(Duration::from_millis(100));
    ///
    /// tx.send(ChronoTimestamped::now(person_alice())).unwrap();
    ///
    /// let item = stream.next().await.unwrap().unwrap();
    /// assert_eq!(item.value, person_alice());
    /// # }
    /// ```
    fn timeout(
        self,
        duration: Duration,
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
        duration: Duration,
    ) -> FluxionStream<impl Stream<Item = StreamItem<ChronoTimestamped<T>>> + Send + Sync> {
        let inner = self.into_inner();
        FluxionStream::new(delay(inner, duration))
    }

    fn debounce(
        self,
        duration: Duration,
    ) -> FluxionStream<impl Stream<Item = StreamItem<ChronoTimestamped<T>>> + Send + Sync> {
        let inner = self.into_inner();
        FluxionStream::new(debounce(inner, duration))
    }

    fn throttle(
        self,
        duration: Duration,
    ) -> FluxionStream<impl Stream<Item = StreamItem<ChronoTimestamped<T>>> + Send + Sync> {
        let inner = self.into_inner();
        FluxionStream::new(throttle(inner, duration))
    }

    fn sample(
        self,
        duration: Duration,
    ) -> FluxionStream<impl Stream<Item = StreamItem<ChronoTimestamped<T>>> + Send + Sync> {
        let inner = self.into_inner();
        FluxionStream::new(sample(inner, duration))
    }

    fn timeout(
        self,
        duration: Duration,
    ) -> FluxionStream<impl Stream<Item = StreamItem<ChronoTimestamped<T>>> + Send + Sync> {
        let inner = self.into_inner();
        FluxionStream::new(timeout(inner, duration))
    }
}
