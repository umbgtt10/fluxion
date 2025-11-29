// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Time-based operators for FluxionStream using chrono timestamps.
//!
//! This crate provides time-based operators for delaying and debouncing stream emissions,
//! along with the `ChronoTimestamped<T>` wrapper type that uses `DateTime<Utc>` for timestamps.
//!
//! # Overview
//!
//! - **`ChronoTimestamped<T>`** - Wraps a value with a UTC timestamp
//! - **`delay(duration)`** - Delays each emission by the specified duration
//! - **`debounce(duration)`** - Emits values only after a quiet period
//! - **`throttle(duration)`** - Emits a value then ignores subsequent values for a duration
//! - **`sample(duration)`** - Emits the most recent value at periodic intervals
//! - **`ChronoStreamOps`** - Extension trait for chainable time-based operations
//!
//! # Example
//!
//! ```rust
//! use fluxion_stream::FluxionStream;
//! use fluxion_stream_time::{ChronoTimestamped, ChronoStreamOps};
//! use fluxion_core::StreamItem;
//! use futures::stream;
//! use std::time::Duration;
//!
//! # async fn example() {
//! let source = stream::iter(vec![
//!     StreamItem::Value(ChronoTimestamped::now(42)),
//!     StreamItem::Value(ChronoTimestamped::now(100)),
//! ]);
//!
//! // Delay all emissions by 100ms
//! let delayed = FluxionStream::new(source)
//!     .delay(Duration::from_millis(100));
//!
//! // Or debounce to emit only after 100ms of quiet time
//! # let source = stream::iter(vec![
//! #     StreamItem::Value(ChronoTimestamped::now(42)),
//! # ]);
//! let debounced = FluxionStream::new(source)
//!     .debounce(Duration::from_millis(100));
//! # }
//! ```
//!
//! [`FluxionStream`]: fluxion_stream::FluxionStream

mod chrono_timestamped;
mod debounce;
mod delay;
mod fluxion_stream_time;
mod sample;
mod throttle;

pub use chrono_timestamped::ChronoTimestamped;
pub use debounce::debounce;
pub use delay::delay;
pub use fluxion_stream_time::FluxionStreamTimeOps;
pub use sample::sample;
pub use throttle::throttle;

use fluxion_stream::FluxionStream;
use futures::Stream;

/// Extension trait for time-based operators on `FluxionStream` with `ChronoTimestamped` items.
///
/// This trait provides the `delay` operator for streams containing `ChronoTimestamped<T>` values.
/// It follows the same pattern as `map_ordered` and `filter_ordered` in `FluxionStream`.
///
/// # Example
///
/// ```rust
/// use fluxion_stream::FluxionStream;
/// use fluxion_stream_time::{ChronoTimestamped, ChronoStreamOps};
/// use fluxion_core::StreamItem;
/// use futures::stream;
/// use std::time::Duration;
///
/// # async fn example() {
/// let source = stream::iter(vec![
///     StreamItem::Value(ChronoTimestamped::now(42)),
/// ]);
///
/// let delayed = FluxionStream::new(source)
///     .delay(Duration::from_millis(100));
/// # }
/// ```
pub trait ChronoStreamOps<S, T>
where
    S: Stream<Item = fluxion_core::StreamItem<ChronoTimestamped<T>>>
        + Send
        + Sync
        + Unpin
        + 'static,
    T: Clone + std::fmt::Debug + Ord + Send + Sync + Unpin + 'static,
{
    /// Delays each emission by the specified duration while preserving temporal ordering.
    ///
    /// Each item is delayed independently. Errors pass through immediately without delay
    /// to ensure timely error propagation.
    ///
    /// # Arguments
    ///
    /// * `duration` - The duration by which to delay each emission
    ///
    /// # Returns
    ///
    /// A new `FluxionStream` where each emission is delayed by the specified duration.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fluxion_stream::FluxionStream;
    /// use fluxion_stream_time::{ChronoTimestamped, ChronoStreamOps};
    /// use fluxion_core::StreamItem;
    /// use futures::stream;
    /// use std::time::Duration;
    ///
    /// # async fn example() {
    /// let source = stream::iter(vec![
    ///     StreamItem::Value(ChronoTimestamped::now(1)),
    ///     StreamItem::Value(ChronoTimestamped::now(2)),
    /// ]);
    ///
    /// let delayed = FluxionStream::new(source)
    ///     .delay(Duration::from_millis(100));
    /// # }
    /// ```
    fn delay(
        self,
        duration: std::time::Duration,
    ) -> FluxionStream<
        impl Stream<Item = fluxion_core::StreamItem<ChronoTimestamped<T>>> + Send + Sync,
    >;

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
    /// use std::time::Duration;
    ///
    /// # async fn example() {
    /// let source = stream::iter(vec![
    ///     StreamItem::Value(ChronoTimestamped::now(1)),
    ///     StreamItem::Value(ChronoTimestamped::now(2)),
    /// ]);
    ///
    /// let debounced = FluxionStream::new(source)
    ///     .debounce(Duration::from_millis(100));
    /// # }
    /// ```
    fn debounce(
        self,
        duration: std::time::Duration,
    ) -> FluxionStream<
        impl Stream<Item = fluxion_core::StreamItem<ChronoTimestamped<T>>> + Send + Sync,
    >;

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
    /// use std::time::Duration;
    ///
    /// # async fn example() {
    /// let source = stream::iter(vec![
    ///     StreamItem::Value(ChronoTimestamped::now(1)),
    ///     StreamItem::Value(ChronoTimestamped::now(2)),
    /// ]);
    ///
    /// let throttled = FluxionStream::new(source)
    ///     .throttle(Duration::from_millis(100));
    /// # }
    /// ```
    fn throttle(
        self,
        duration: std::time::Duration,
    ) -> FluxionStream<
        impl Stream<Item = fluxion_core::StreamItem<ChronoTimestamped<T>>> + Send + Sync,
    >;

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
    /// use std::time::Duration;
    ///
    /// # async fn example() {
    /// let source = stream::iter(vec![
    ///     StreamItem::Value(ChronoTimestamped::now(1)),
    ///     StreamItem::Value(ChronoTimestamped::now(2)),
    /// ]);
    ///
    /// let sampled = FluxionStream::new(source)
    ///     .sample(Duration::from_millis(100));
    /// # }
    /// ```
    fn sample(
        self,
        duration: std::time::Duration,
    ) -> FluxionStream<
        impl Stream<Item = fluxion_core::StreamItem<ChronoTimestamped<T>>> + Send + Sync,
    >;
}

impl<S, T> ChronoStreamOps<S, T> for FluxionStream<S>
where
    S: Stream<Item = fluxion_core::StreamItem<ChronoTimestamped<T>>>
        + Send
        + Sync
        + Unpin
        + 'static,
    T: Clone + std::fmt::Debug + Ord + Send + Sync + Unpin + 'static,
{
    fn delay(
        self,
        duration: std::time::Duration,
    ) -> FluxionStream<
        impl Stream<Item = fluxion_core::StreamItem<ChronoTimestamped<T>>> + Send + Sync,
    > {
        let inner = self.into_inner();
        FluxionStream::new(delay::delay(inner, duration))
    }

    fn debounce(
        self,
        duration: std::time::Duration,
    ) -> FluxionStream<
        impl Stream<Item = fluxion_core::StreamItem<ChronoTimestamped<T>>> + Send + Sync,
    > {
        let inner = self.into_inner();
        FluxionStream::new(debounce::debounce(inner, duration))
    }

    fn throttle(
        self,
        duration: std::time::Duration,
    ) -> FluxionStream<
        impl Stream<Item = fluxion_core::StreamItem<ChronoTimestamped<T>>> + Send + Sync,
    > {
        let inner = self.into_inner();
        FluxionStream::new(throttle::throttle(inner, duration))
    }

    fn sample(
        self,
        duration: std::time::Duration,
    ) -> FluxionStream<
        impl Stream<Item = fluxion_core::StreamItem<ChronoTimestamped<T>>> + Send + Sync,
    > {
        let inner = self.into_inner();
        FluxionStream::new(sample::sample(inner, duration))
    }
}
