use futures::Stream;
use std::fmt::Debug;
use std::pin::Pin;

use crate::sequenced::Sequenced;
use crate::sequenced_stream::SequencedStreamExt;

// Re-export low-level types from fluxion-ordered-merge
pub use fluxion_ordered_merge::{
    OrderedMerge, OrderedMergeExt, OrderedMergeSync, OrderedMergeSyncExt,
};

/// High-level ordered merge for Sequenced streams with Sync support
/// This variant returns Sync streams for composition with operators requiring Sync
pub trait OrderedMergeSequencedSyncExt<T, S2>: SequencedStreamExt<T> + Sized
where
    T: Clone + Debug + Send + Sync + Unpin + 'static,
    Sequenced<T>: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    S2: Stream<Item = Sequenced<T>> + Send + Sync + 'static,
{
    /// Merges multiple Sequenced streams, emitting all values in sequence order.
    /// This version returns a Sync stream for composition with other Sync-requiring operators.
    fn ordered_merge_sync(self, others: Vec<S2>) -> impl Stream<Item = Sequenced<T>> + Send + Sync;
}

impl<T, S, S2> OrderedMergeSequencedSyncExt<T, S2> for S
where
    T: Clone + Debug + Send + Sync + Unpin + 'static,
    Sequenced<T>: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    S: SequencedStreamExt<T> + Send + Sync + 'static,
    S2: Stream<Item = Sequenced<T>> + Send + Sync + 'static,
{
    fn ordered_merge_sync(self, others: Vec<S2>) -> impl Stream<Item = Sequenced<T>> + Send + Sync {
        let mut all_streams =
            vec![Box::pin(self) as Pin<Box<dyn Stream<Item = Sequenced<T>> + Send + Sync>>];
        for stream in others {
            all_streams.push(Box::pin(stream));
        }

        OrderedMergeSync::new(all_streams)
    }
}
