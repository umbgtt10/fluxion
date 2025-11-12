use futures::Stream;

use crate::timestamped::Timestamped;

/// Small trait representing streams that yield `Timestamped<T>` items.
///
/// This provides a single place to bind extension traits to, making it
/// easier to refactor or move timestamped-related utilities later.
pub trait TimestampedStreamExt<T>: Stream<Item = Timestamped<T>> {}

// Blanket impl: any Stream that yields `Timestamped<T>` automatically
// implements `TimestampedStreamExt<T>`.
impl<S, T> TimestampedStreamExt<T> for S where S: Stream<Item = Timestamped<T>> {}
