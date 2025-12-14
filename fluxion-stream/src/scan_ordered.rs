// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::fluxion_stream::FluxionStream;
use fluxion_core::{Fluxion, StreamItem};
use futures::{future::ready, Stream, StreamExt};
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

/// Extension trait providing the `scan_ordered` operator for streams.
///
/// This operator accumulates state across stream items, emitting intermediate
/// accumulated values. It's similar to `Iterator::fold` but emits a result
/// for each input item rather than just the final result.
///
/// # Key Characteristics
///
/// - **Stateful**: Maintains accumulator state across all stream items
/// - **Emits per item**: Produces output for each input (unlike fold which emits once)
/// - **Type transformation**: Can change output type from input type
/// - **Error preserving**: Errors propagate without resetting state
pub trait ScanOrderedExt<T>: Stream<Item = StreamItem<T>> + Sized
where
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
{
    /// Accumulates state across stream items, emitting intermediate results.
    ///
    /// The `scan_ordered` operator maintains an accumulator value that is updated for each
    /// input item. For each input, it calls the accumulator function with a mutable
    /// reference to the current state and the input value, producing an output value.
    ///
    /// # Type Parameters
    ///
    /// - `Acc`: The accumulator type (internal state)
    /// - `Out`: The output item type (must implement `Fluxion`)
    /// - `F`: The accumulator function
    ///
    /// # Behavior
    ///
    /// - **Timestamp Preservation**: Output items preserve the timestamp of their source items
    /// - **State Accumulation**: The accumulator state persists across all items
    /// - **Error Handling**: Errors are propagated immediately without affecting accumulator state
    /// - **Type Transformation**: Can transform input type to different output type
    ///
    /// # Arguments
    ///
    /// * `initial` - Initial accumulator value
    /// * `accumulator` - Function that updates state and produces output: `FnMut(&mut Acc, &T::Inner) -> Out::Inner`
    ///
    /// # Returns
    ///
    /// A `FluxionStream` of `Out` where each item is the result of accumulation at that point.
    ///
    /// # Errors
    ///
    /// This operator may produce `StreamItem::Error` in the following cases:
    ///
    /// - **Propagated Errors**: Any `StreamItem::Error` from the source stream is passed through unchanged
    /// - **Lock Errors**: When acquiring the accumulator lock fails (e.g., due to lock poisoning)
    ///
    /// Errors do NOT reset the accumulator state - processing continues with the existing state.
    /// See the [Error Handling Guide](../docs/ERROR-HANDLING.md) for patterns on handling these errors.
    ///
    /// # Examples
    ///
    /// ## Running Sum
    ///
    /// ```rust
    /// use fluxion_stream::{FluxionStream, ScanOrderedExt};
    /// use fluxion_test_utils::{Sequenced, test_channel, unwrap_stream, unwrap_value};
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let (tx, stream) = test_channel::<Sequenced<i32>>();
    ///
    /// // Accumulate sum
    /// let mut sums = stream.scan_ordered::<Sequenced<i32>, _, _>(0, |acc, val| {
    ///     *acc += val;
    ///     *acc
    /// });
    ///
    /// tx.send((10, 1).into())?;
    /// tx.send((20, 2).into())?;
    /// tx.send((30, 3).into())?;
    ///
    /// assert_eq!(unwrap_value(Some(unwrap_stream(&mut sums, 500).await)).value, 10);  // 0 + 10
    /// assert_eq!(unwrap_value(Some(unwrap_stream(&mut sums, 500).await)).value, 30);  // 10 + 20
    /// assert_eq!(unwrap_value(Some(unwrap_stream(&mut sums, 500).await)).value, 60);  // 30 + 30
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Type Transformation: Count to String
    ///
    /// ```rust
    /// use fluxion_stream::{FluxionStream, ScanOrderedExt};
    /// use fluxion_test_utils::{Sequenced, test_channel, unwrap_stream, unwrap_value};
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let (tx, stream) = test_channel::<Sequenced<String>>();
    ///
    /// // Count items and produce formatted strings
    /// let mut counts = stream.scan_ordered::<Sequenced<String>, _, _>(0i32, |count: &mut i32, _item: &String| {
    ///     *count += 1;
    ///     format!("Item #{}", count)
    /// });
    ///
    /// tx.send(("apple".to_string(), 1).into())?;
    /// tx.send(("banana".to_string(), 2).into())?;
    ///
    /// assert_eq!(unwrap_value(Some(unwrap_stream(&mut counts, 500).await)).value, "Item #1");
    /// assert_eq!(unwrap_value(Some(unwrap_stream(&mut counts, 500).await)).value, "Item #2");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Complex State: Building a List
    ///
    /// ```rust
    /// use fluxion_stream::{FluxionStream, ScanOrderedExt};
    /// use fluxion_test_utils::{Sequenced, test_channel, unwrap_stream, unwrap_value};
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let (tx, stream) = test_channel::<Sequenced<i32>>();
    ///
    /// // Collect all values seen so far
    /// let mut history = stream.scan_ordered::<Sequenced<Vec<i32>>, _, _>(Vec::<i32>::new(), |list: &mut Vec<i32>, val: &i32| {
    ///     list.push(*val);
    ///     list.clone() // Return snapshot
    /// });
    ///
    /// tx.send((10, 1).into())?;
    /// tx.send((20, 2).into())?;
    /// tx.send((30, 3).into())?;
    ///
    /// assert_eq!(unwrap_value(Some(unwrap_stream(&mut history, 500).await)).value, vec![10]);
    /// assert_eq!(unwrap_value(Some(unwrap_stream(&mut history, 500).await)).value, vec![10, 20]);
    /// assert_eq!(unwrap_value(Some(unwrap_stream(&mut history, 500).await)).value, vec![10, 20, 30]);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Use Cases
    ///
    /// - Running totals, averages, or statistics
    /// - Building collections or aggregations over time
    /// - Counting occurrences or tracking frequencies
    /// - State machines with accumulated context
    /// - Moving window calculations
    ///
    /// # Advanced Example: State Machine
    ///
    /// ```rust
    /// use fluxion_stream::{FluxionStream, ScanOrderedExt};
    /// use fluxion_test_utils::{Sequenced, test_channel, unwrap_stream};
    ///
    /// #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    /// enum Event { Login, Logout, Action(String) }
    ///
    /// #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    /// struct SessionState {
    ///     logged_in: bool,
    ///     action_count: usize,
    /// }
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let (tx, stream) = test_channel::<Sequenced<Event>>();
    ///
    /// let mut states = stream.scan_ordered::<Sequenced<SessionState>, _, _>(
    ///     SessionState { logged_in: false, action_count: 0 },
    ///     |state, event| {
    ///         match event {
    ///             Event::Login => state.logged_in = true,
    ///             Event::Logout => {
    ///                 state.logged_in = false;
    ///                 state.action_count = 0;
    ///             }
    ///             Event::Action(_) if state.logged_in => {
    ///                 state.action_count += 1;
    ///             }
    ///             _ => {}
    ///         }
    ///         state.clone()
    ///     }
    /// );
    ///
    /// tx.send(Sequenced::new(Event::Login))?;
    /// let s1 = unwrap_stream(&mut states, 500).await.unwrap().into_inner();
    /// assert!(s1.logged_in && s1.action_count == 0);
    ///
    /// tx.send(Sequenced::new(Event::Action("click".to_string())))?;
    /// let s2 = unwrap_stream(&mut states, 500).await.unwrap().into_inner();
    /// assert!(s2.logged_in && s2.action_count == 1);
    ///
    /// tx.send(Sequenced::new(Event::Logout))?;
    /// let s3 = unwrap_stream(&mut states, 500).await.unwrap().into_inner();
    /// assert!(!s3.logged_in && s3.action_count == 0);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Performance
    ///
    /// - O(1) time per item (depends on accumulator function)
    /// - Stores only the accumulator state (not all items)
    /// - Minimal overhead - just function calls and locking
    ///
    /// # See Also
    ///
    /// - [`map_ordered`](crate::FluxionStream::map_ordered) - Stateless transformation
    /// - [`combine_with_previous`](crate::CombineWithPreviousExt::combine_with_previous) - Simple stateful pairing
    /// - [`filter_ordered`](crate::FluxionStream::filter_ordered) - Conditional filtering
    fn scan_ordered<Out, Acc, F>(
        self,
        initial: Acc,
        accumulator: F,
    ) -> FluxionStream<impl Stream<Item = StreamItem<Out>>>
    where
        Acc: Send + 'static,
        Out: Fluxion,
        Out::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
        Out::Timestamp: From<T::Timestamp> + Debug + Ord + Send + Sync + Copy + 'static,
        F: FnMut(&mut Acc, &T::Inner) -> Out::Inner + Send + 'static;
}

impl<T, S> ScanOrderedExt<T> for S
where
    S: Stream<Item = StreamItem<T>> + Send + Sized + 'static,
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
{
    fn scan_ordered<Out, Acc, F>(
        self,
        initial: Acc,
        accumulator: F,
    ) -> FluxionStream<impl Stream<Item = StreamItem<Out>>>
    where
        Acc: Send + 'static,
        Out: Fluxion,
        Out::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
        Out::Timestamp: From<T::Timestamp> + Debug + Ord + Send + Sync + Copy + 'static,
        F: FnMut(&mut Acc, &T::Inner) -> Out::Inner + Send + 'static,
    {
        let state = Arc::new(Mutex::new((initial, accumulator)));

        let result = self.then(move |item| {
            let state = Arc::clone(&state);
            ready(match item {
                StreamItem::Value(value) => {
                    let timestamp = value.timestamp();
                    let inner = value.into_inner();

                    // Lock state and apply accumulator function
                    match state.lock() {
                        Ok(mut guard) => {
                            let (acc, ref mut f) = &mut *guard;
                            let output = f(acc, &inner);
                            StreamItem::Value(Out::with_timestamp(output, timestamp.into()))
                        }
                        Err(poisoned) => {
                            // Recover from poisoned lock
                            let mut guard = poisoned.into_inner();
                            let (acc, ref mut f) = &mut *guard;
                            let output = f(acc, &inner);
                            StreamItem::Value(Out::with_timestamp(output, timestamp.into()))
                        }
                    }
                }
                StreamItem::Error(e) => {
                    // Propagate error without affecting accumulator state
                    StreamItem::Error(e)
                }
            })
        });

        FluxionStream::new(result)
    }
}
