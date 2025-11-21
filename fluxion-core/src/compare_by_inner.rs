use std::cmp::Ordering;

/// Trait for comparing timestamped values by their inner content, ignoring timestamps.
///
/// This trait enables comparing wrapped timestamped values based solely on their
/// inner data, without considering their temporal ordering. It's primarily used
/// in test assertions and validation scenarios where you want to verify the data
/// content matches, regardless of when the events occurred.
///
/// # Purpose
///
/// When working with timestamped types like `ChronoTimestamped<T>` or domain types
/// implementing [`Timestamped`](crate::Timestamped), the default `Ord` implementation
/// compares by timestamp first. `CompareByInner` provides an alternative comparison
/// that only considers the inner value.
///
/// # Relationship to `Ord`
///
/// - **`Ord`**: Compares by timestamp, then by inner value (for temporal ordering)
/// - **`CompareByInner`**: Compares only by inner value (for data validation)
///
/// # When to Implement
///
/// Implement this trait when:
/// - Your type wraps a timestamped value and you need content-based comparison
/// - You're writing tests that verify data content independent of timing
/// - You need to check if two events have the same data but different timestamps
///
/// # Examples
///
/// ```
/// use fluxion_core::CompareByInner;
/// use std::cmp::Ordering;
///
/// struct Wrapper<T> {
///     value: T,
///     timestamp: u64,
/// }
///
/// impl<T: Ord> CompareByInner for Wrapper<T> {
///     fn cmp_inner(&self, other: &Self) -> Ordering {
///         self.value.cmp(&other.value)
///     }
/// }
///
/// let a = Wrapper { value: 10, timestamp: 1 };
/// let b = Wrapper { value: 10, timestamp: 2 };
///
/// // Different timestamps, but same inner value
/// assert_eq!(a.cmp_inner(&b), Ordering::Equal);
/// ```
pub trait CompareByInner {
    /// Compares `self` with `other` based solely on inner content.
    ///
    /// Returns `Ordering::Equal` if the inner values are equal, regardless of
    /// any wrapper metadata like timestamps.
    fn cmp_inner(&self, other: &Self) -> Ordering;
}
