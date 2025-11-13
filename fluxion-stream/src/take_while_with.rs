use crate::fluxion_stream::FluxionStream;
use fluxion_core::Ordered;
use fluxion_ordered_merge::OrderedMergeExt;
use futures::Stream;
use futures::stream::StreamExt;
use std::fmt::Debug;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

type PinnedItemStream<TItem, TFilter> =
    Pin<Box<dyn Stream<Item = Item<TItem, TFilter>> + Send + Sync>>;

/// Takes elements from the source stream while the condition on the filter stream is satisfied.
///
/// This operator combines the source stream with a filter stream, emitting source elements
/// only while the filter predicate applied to the latest filter stream element returns true.
/// Once the predicate returns false, the stream terminates.
pub trait TakeWhileExt<TItem, TFilter, S>: Stream<Item = TItem> + Sized
where
    TItem: Ordered + Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    TItem::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    TFilter: Ordered + Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    TFilter::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    S: Stream<Item = TFilter> + Send + Sync + 'static,
{
    /// Takes elements from the source stream while the filter predicate returns true.
    ///
    /// # Arguments
    /// * `filter_stream` - The stream providing filter values
    /// * `filter` - A function that takes the latest filter stream element and returns whether to continue
    ///
    /// # Returns
    /// A stream of unwrapped source elements that passes while the filter condition is true
    fn take_while_with(
        self,
        filter_stream: S,
        filter: impl Fn(&TFilter::Inner) -> bool + Send + Sync + 'static,
    ) -> FluxionStream<impl Stream<Item = TItem::Inner> + Send>;
}

impl<TItem, TFilter, S, P> TakeWhileExt<TItem, TFilter, S> for P
where
    P: Stream<Item = TItem> + Send + Sync + Unpin + 'static,
    TItem: Ordered + Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    TItem::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    TFilter: Ordered + Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    TFilter::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    S: Stream<Item = TFilter> + Send + Sync + 'static,
{
    fn take_while_with(
        self,
        filter_stream: S,
        filter: impl Fn(&TFilter::Inner) -> bool + Send + Sync + 'static,
    ) -> FluxionStream<impl Stream<Item = TItem::Inner> + Send> {
        let filter = Arc::new(filter);

        // Tag each stream with its type
        let source_stream = self.map(|value: TItem| Item::<TItem, TFilter>::Source(value));
        let filter_stream =
            filter_stream.map(|value: TFilter| Item::<TItem, TFilter>::Filter(value));

        // Box the streams to make them the same type
        let streams: Vec<PinnedItemStream<TItem, TFilter>> =
            vec![Box::pin(source_stream), Box::pin(filter_stream)];

        // State to track the latest filter value and termination
        let state = Arc::new(Mutex::new((None::<TFilter::Inner>, false)));

        // Use ordered_merge and process items in order
        let result = streams.ordered_merge().filter_map({
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
                            *filter_state = Some(filter_val.get().clone());
                            None // Don't emit filter values
                        }
                        Item::Source(source_val) => {
                            // Check the current filter state
                            if let Some(fval) = filter_state {
                                if filter(fval) {
                                    Some(source_val.get().clone())
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
        });

        FluxionStream::new(result)
    }
}

#[derive(Clone, Debug)]
enum Item<TItem, TFilter> {
    Source(TItem),
    Filter(TFilter),
}

impl<TItem, TFilter> Item<TItem, TFilter>
where
    TItem: Ordered,
    TFilter: Ordered,
{
    fn order(&self) -> u64 {
        match self {
            Item::Source(s) => s.order(),
            Item::Filter(f) => f.order(),
        }
    }
}

impl<TItem, TFilter> PartialEq for Item<TItem, TFilter>
where
    TItem: Ordered,
    TFilter: Ordered,
{
    fn eq(&self, other: &Self) -> bool {
        self.order() == other.order()
    }
}

impl<TItem, TFilter> Eq for Item<TItem, TFilter>
where
    TItem: Ordered,
    TFilter: Ordered,
{
}

impl<TItem, TFilter> PartialOrd for Item<TItem, TFilter>
where
    TItem: Ordered,
    TFilter: Ordered,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<TItem, TFilter> Ord for Item<TItem, TFilter>
where
    TItem: Ordered,
    TFilter: Ordered,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.order().cmp(&other.order())
    }
}
