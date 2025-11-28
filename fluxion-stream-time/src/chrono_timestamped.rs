use chrono::{DateTime, Utc};
use fluxion_core::{HasTimestamp, Timestamped};
use std::cmp::Ordering;

/// A timestamped value using chrono's DateTime for real-world time operations.
///
/// This type wraps a value with a UTC timestamp, implementing the `Timestamped` trait
/// with chrono's `DateTime<Utc>` as the timestamp type. This enables time-based
/// operators like `delay`.
///
/// # Example
///
/// ```rust
/// use fluxion_stream_time::ChronoTimestamped;
/// use chrono::Utc;
///
/// let item = ChronoTimestamped::new(42, Utc::now());
/// let another = ChronoTimestamped::now("hello");
/// ```
#[derive(Debug, Clone)]
pub struct ChronoTimestamped<T> {
    pub value: T,
    pub timestamp: DateTime<Utc>,
}

impl<T> ChronoTimestamped<T> {
    /// Creates a new timestamped value with the given timestamp.
    pub fn new(value: T, timestamp: DateTime<Utc>) -> Self {
        Self { value, timestamp }
    }

    /// Creates a new timestamped value with the current UTC time.
    pub fn now(value: T) -> Self {
        Self {
            value,
            timestamp: Utc::now(),
        }
    }
}

impl<T> HasTimestamp for ChronoTimestamped<T> {
    type Timestamp = DateTime<Utc>;

    fn timestamp(&self) -> Self::Timestamp {
        self.timestamp
    }
}

impl<T> Timestamped for ChronoTimestamped<T>
where
    T: Clone,
{
    type Inner = T;

    fn into_inner(self) -> Self::Inner {
        self.value
    }

    fn with_timestamp(inner: Self::Inner, timestamp: Self::Timestamp) -> Self {
        Self::new(inner, timestamp)
    }

    fn with_fresh_timestamp(inner: Self::Inner) -> Self {
        Self::now(inner)
    }
}

impl<T> PartialEq for ChronoTimestamped<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.timestamp == other.timestamp && self.value == other.value
    }
}

impl<T> Eq for ChronoTimestamped<T> where T: Eq {}

impl<T> PartialOrd for ChronoTimestamped<T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.timestamp.partial_cmp(&other.timestamp)
    }
}

impl<T> Ord for ChronoTimestamped<T>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.timestamp.cmp(&other.timestamp)
    }
}
