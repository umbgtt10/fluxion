use futures::Stream;
use std::fmt::Debug;
use std::pin::Pin;

use crate::Ordered;

// Re-export low-level types from fluxion-ordered-merge
pub use fluxion_ordered_merge::{OrderedMerge, OrderedMergeExt};

/// High-level ordered merge for Ordered streams
/// This trait merges multiple Ordered streams and emits all values in order
pub trait OrderedStreamExt<T, S2>: Stream<Item = T> + Sized
where
    T: Clone + Debug + Ordered + Ord + Send + Sync + Unpin + 'static,
    S2: Stream<Item = T> + Send + Sync + 'static,
{
    /// Merges multiple Ordered streams, emitting all values in order.
    /// Unlike `combine_latest`, this doesn't wait for all streams - it emits every value
    /// from all streams in order.
    ///
    /// # Arguments
    /// * `others` - Vector of other streams to merge with this stream
    ///
    /// # Returns
    /// Stream of `T` where values are emitted in order
    fn ordered_merge(self, others: Vec<S2>) -> impl Stream<Item = T> + Send + Sync;
}

impl<T, S, S2> OrderedStreamExt<T, S2> for S
where
    T: Clone + Debug + Ordered + Ord + Send + Sync + Unpin + 'static,
    S: Stream<Item = T> + Send + Sync + 'static,
    S2: Stream<Item = T> + Send + Sync + 'static,
{
    fn ordered_merge(self, others: Vec<S2>) -> impl Stream<Item = T> + Send + Sync {
        let mut all_streams = vec![Box::pin(self) as Pin<Box<dyn Stream<Item = T> + Send + Sync>>];
        for stream in others {
            all_streams.push(Box::pin(stream));
        }

        OrderedMerge::new(all_streams)
    }
}
