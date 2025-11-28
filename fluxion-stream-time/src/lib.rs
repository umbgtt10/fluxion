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
//! - **`ChronoStreamOps`** - Extension trait for chainable time-based operations
//!
//! # Example
//!
//! ```rust
//! use fluxion_stream::FluxionStream;
//! use fluxion_stream_time::{ChronoTimestamped, ChronoStreamOps};
//! use fluxion_core::StreamItem;
//! use futures::stream;
//! use chrono::Duration;
//!
//! # async fn example() {
//! let source = stream::iter(vec![
//!     StreamItem::Value(ChronoTimestamped::now(42)),
//!     StreamItem::Value(ChronoTimestamped::now(100)),
//! ]);
//!
//! // Delay all emissions by 100ms
//! let delayed = FluxionStream::new(source)
//!     .delay(Duration::milliseconds(100));
//!
//! // Or debounce to emit only after 100ms of quiet time
//! # let source = stream::iter(vec![
//! #     StreamItem::Value(ChronoTimestamped::now(42)),
//! # ]);
//! let debounced = FluxionStream::new(source)
//!     .debounce(Duration::milliseconds(100));
//! # }
//! ```
//!
//! [`FluxionStream`]: fluxion_stream::FluxionStream

mod chrono_timestamped;
mod debounce;
mod delay;
mod fluxion_stream_time;

pub use chrono_timestamped::ChronoTimestamped;
pub use debounce::debounce;
pub use delay::delay;
pub use fluxion_stream_time::FluxionStreamTimeOps;

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
/// use chrono::Duration;
///
/// # async fn example() {
/// let source = stream::iter(vec![
///     StreamItem::Value(ChronoTimestamped::now(42)),
/// ]);
///
/// let delayed = FluxionStream::new(source)
///     .delay(Duration::milliseconds(100));
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
    /// use chrono::Duration;
    ///
    /// # async fn example() {
    /// let source = stream::iter(vec![
    ///     StreamItem::Value(ChronoTimestamped::now(1)),
    ///     StreamItem::Value(ChronoTimestamped::now(2)),
    /// ]);
    ///
    /// let delayed = FluxionStream::new(source)
    ///     .delay(Duration::milliseconds(100));
    /// # }
    /// ```
    fn delay(
        self,
        duration: chrono::Duration,
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
    /// use chrono::Duration;
    ///
    /// # async fn example() {
    /// let source = stream::iter(vec![
    ///     StreamItem::Value(ChronoTimestamped::now(1)),
    ///     StreamItem::Value(ChronoTimestamped::now(2)),
    /// ]);
    ///
    /// let debounced = FluxionStream::new(source)
    ///     .debounce(Duration::milliseconds(100));
    /// # }
    /// ```
    fn debounce(
        self,
        duration: chrono::Duration,
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
        duration: chrono::Duration,
    ) -> FluxionStream<
        impl Stream<Item = fluxion_core::StreamItem<ChronoTimestamped<T>>> + Send + Sync,
    > {
        let inner = self.into_inner();
        FluxionStream::new(delay::delay(inner, duration))
    }

    fn debounce(
        self,
        duration: chrono::Duration,
    ) -> FluxionStream<
        impl Stream<Item = fluxion_core::StreamItem<ChronoTimestamped<T>>> + Send + Sync,
    > {
        let inner = self.into_inner();
        FluxionStream::new(debounce::debounce(inner, duration))
    }
}
