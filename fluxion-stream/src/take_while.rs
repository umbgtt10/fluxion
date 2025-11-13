use crate::ordered_merge::OrderedMergeExt;
use crate::sequenced::Sequenced;
use crate::sequenced_stream::SequencedStreamExt;
use futures::Stream;
use futures::stream::StreamExt;
use std::fmt::Debug;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

type PinnedItemStream<TI, TFI> = Pin<Box<dyn Stream<Item = Item<TI, TFI>> + Send>>;

/// Takes elements from the source stream while the condition on the filter stream is satisfied.
///
/// This operator combines the source stream with a filter stream, emitting source elements
/// only while the filter predicate applied to the latest filter stream element returns true.
/// Once the predicate returns false, the stream terminates.
///
/// Usage (instance-method form):
///
/// ```rust
/// // `source_stream` is a `Sequenced` stream of `T`
/// // `filter_stream` is a `Sequenced` stream of `TF`
/// // `filter` is a `Fn(&TF) -> bool`
///
/// let filtered = source_stream.take_while_with(filter_stream, |filter_val| {
///     // return true to continue emitting source items
///     *filter_val
/// });
/// ```
pub trait TakeWhileStreamExt<T, TF, S>: SequencedStreamExt<T> + Sized
where
    T: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    TF: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    S: Stream<Item = Sequenced<TF>> + Send + Sync + 'static,
{
    /// Takes elements from the source stream while the filter predicate returns true.
    ///
    /// This method is an instance-style operator: call it on the source stream.
    ///
    /// # Arguments
    /// * `filter_stream` - The stream providing filter values
    /// * `filter` - A function that takes the latest filter stream element and returns whether to continue
    ///
    /// # Returns
    /// A stream of unwrapped source elements that passes while the filter condition is true
    ///
    /// Example:
    ///
    /// ```rust
    /// let output = source_stream.take_while_with(filter_stream, |f| *f);
    /// ```
    fn take_while_with(
        self,
        filter_stream: S,
        filter: impl Fn(&TF) -> bool + Send + Sync + 'static,
    ) -> impl Stream<Item = T> + Send;
}

impl<T, TF, S, P> TakeWhileStreamExt<T, TF, S> for P
where
    P: SequencedStreamExt<T> + Send + Sync + Unpin + 'static,
    T: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    TF: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    S: Stream<Item = Sequenced<TF>> + Send + Sync + 'static,
{
    fn take_while_with(
        self,
        filter_stream: S,
        filter: impl Fn(&TF) -> bool + Send + Sync + 'static,
    ) -> impl Stream<Item = T> + Send {
        let filter = Arc::new(filter);

        // Tag each stream with its type (explicitly specify generic params so both streams
        // produce the same `Item<T, TF>` type)
        let source_stream = self.map(|value: Sequenced<T>| Item::<T, TF>::Source(value));
        let filter_stream = filter_stream.map(|value: Sequenced<TF>| Item::<T, TF>::Filter(value));

        // Box the streams to make them the same type
        let streams: Vec<PinnedItemStream<T, TF>> =
            vec![Box::pin(source_stream), Box::pin(filter_stream)];

        // State to track the latest filter value and termination
        let state = Arc::new(Mutex::new((None::<TF>, false)));

        // Use ordered_merge and process items in sequence order
        streams.ordered_merge().filter_map({
            let state = Arc::clone(&state);
            move |item| {
                let state = Arc::clone(&state);
                let filter = Arc::clone(&filter);

                async move {
                    let mut state_guard = state.lock().unwrap();
                    let (filter_state, terminated) = &mut *state_guard;

                    // If already terminated, signal stream end
                    if *terminated {
                        return None;
                    }

                    match item {
                        Item::Filter(filter_val) => {
                            // Update the filter state
                            *filter_state = Some(filter_val.value.clone());
                            None // Don't emit filter values
                        }
                        Item::Source(source_val) => {
                            // Check the current filter state
                            if let Some(fval) = filter_state {
                                if filter(fval) {
                                    Some(source_val.value.clone())
                                } else {
                                    // Filter condition failed, terminate stream
                                    *terminated = true;
                                    None
                                }
                            } else {
                                None // No filter value yet, don't emit
                            }
                        }
                    }
                }
            }
        })
    }
}

#[derive(Clone, Debug)]
enum Item<T, TF> {
    Source(Sequenced<T>),
    Filter(Sequenced<TF>),
}

impl<T, TF> Item<T, TF> {
    fn sequence(&self) -> u64 {
        match self {
            Item::Source(s) => s.sequence(),
            Item::Filter(f) => f.sequence(),
        }
    }
}

impl<T, TF> PartialEq for Item<T, TF> {
    fn eq(&self, other: &Self) -> bool {
        self.sequence() == other.sequence()
    }
}

impl<T, TF> Eq for Item<T, TF> {}

impl<T, TF> PartialOrd for Item<T, TF> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T, TF> Ord for Item<T, TF> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.sequence().cmp(&other.sequence())
    }
}
