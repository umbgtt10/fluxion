//! Time-based operators for FluxionStream using chrono timestamps.
//!
//! This crate provides the `delay` operator for delaying stream emissions, along with
//! the `ChronoTimestamped<T>` wrapper type that uses `DateTime<Utc>` for timestamps.
//!
//! # Overview
//!
//! - **`ChronoTimestamped<T>`** - Wraps a value with a UTC timestamp
//! - **`delay(duration)`** - Delays each emission by the specified duration
//! - **`ChronoStreamOps`** - Extension trait for chainable delay operations
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
//! let delayed = FluxionStream::new(source)
//!     .delay(Duration::milliseconds(100));
//! # }
//! ```
//!
//! [`FluxionStream`]: fluxion_stream::FluxionStream

mod chrono_timestamped;
mod delay;
mod fluxion_stream_time;

pub use chrono_timestamped::ChronoTimestamped;
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
}
