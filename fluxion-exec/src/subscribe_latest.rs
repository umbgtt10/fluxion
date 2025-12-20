// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use async_trait::async_trait;
use fluxion_core::{FluxionError, Result};
use futures::lock::Mutex as FutureMutex;
use futures::{Stream, StreamExt};
use parking_lot::Mutex;
use std::fmt::Debug;
use std::future::Future;
use std::{error::Error, sync::Arc};
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

/// Extension trait providing async subscription with automatic cancellation of outdated work.
///
/// This trait enables processing stream items where newer items automatically cancel
/// processing of older items, ensuring only the latest value is being processed.
#[async_trait]
pub trait SubscribeLatestExt<T>: Stream<Item = T> + Send + 'static
where
    T: Send + 'static,
{
    /// Subscribes to the stream, automatically cancelling processing of older items when new items arrive.
    ///
    /// This method is ideal for scenarios where you only care about processing the most recent
    /// value and want to abandon work on outdated values. When a new item arrives while an
    /// existing item is being processed, a new task starts for the latest item.
    ///
    /// # Behavior
    ///
    /// - Only one processing task runs at a time per stream
    /// - When a new item arrives during processing, it queues as "latest"
    /// - After current processing completes, the latest queued item is processed
    /// - Intermediate items between current and latest are discarded
    /// - Waits for all processing to complete before returning
    ///
    /// # Arguments
    ///
    /// * `on_next_func` - Async function called for each item. Should check the cancellation
    ///                    token periodically to enable responsive cancellation.
    /// * `on_error_callback` - Optional error handler for processing failures
    /// * `cancellation_token` - Optional token to stop all processing
    ///
    /// # Type Parameters
    ///
    /// * `F` - Function type for the item handler
    /// * `Fut` - Future type returned by the handler
    /// * `E` - Error type that implements `std::error::Error`
    /// * `OnError` - Function type for error handling
    ///
    /// # Errors
    ///
    /// Returns `Err(FluxionError::MultipleErrors)` if any items failed to process and
    /// no error callback was provided. If an error callback is provided, errors are
    /// passed to it and the function returns `Ok(())` on stream completion.
    ///
    /// Processing continues with new items even if previous items failed, unless the
    /// cancellation token is triggered.
    ///
    /// # See Also
    ///
    /// - [`subscribe`](crate::SubscribeExt::subscribe) - Sequential processing of all items
    ///
    /// # Examples
    ///
    /// ## Basic Usage - Skipping Intermediate Values
    ///
    /// Only the first and latest items are processed:
    ///
    /// ```
    /// use fluxion_exec::SubscribeLatestExt;
    /// use tokio::sync::mpsc::unbounded_channel;
    /// use tokio_stream::wrappers::UnboundedReceiverStream;
    /// use futures::StreamExt;
    /// use std::sync::Arc;
    /// use tokio::sync::Mutex;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let (tx, rx) = unbounded_channel();
    /// let stream = UnboundedReceiverStream::new(rx).map(|x: i32| x);
    ///
    /// let processed = Arc::new(Mutex::new(Vec::new()));
    /// let processed_clone = processed.clone();
    ///
    /// // Gate to control when first item completes
    /// let (gate_tx, gate_rx) = unbounded_channel::<()>();
    /// let gate_shared = Arc::new(Mutex::new(Some(gate_rx)));
    ///
    /// // Signal when processing starts and completes
    /// let (started_tx, mut started_rx) = unbounded_channel::<i32>();
    /// let started_tx = Arc::new(started_tx);
    /// let (done_tx, mut done_rx) = unbounded_channel::<i32>();
    /// let done_tx = Arc::new(done_tx);
    ///
    /// let handle = tokio::spawn(async move {
    ///     stream.subscribe_latest(
    ///         move |item, token| {
    ///             let processed = processed_clone.clone();
    ///             let gate = gate_shared.clone();
    ///             let started = started_tx.clone();
    ///             let done = done_tx.clone();
    ///             async move {
    ///                 started.send(item).unwrap();
    ///                 // First item waits at gate
    ///                 if let Some(mut rx) = gate.lock().await.take() {
    ///                     let _ = rx.recv().await;
    ///                 }
    ///                 if !token.is_cancelled() {
    ///                     processed.lock().await.push(item);
    ///                 }
    ///                 done.send(item).unwrap();
    ///                 Ok::<(), std::io::Error>(())
    ///             }
    ///         },
    ///         None::<fn(std::io::Error)>,
    ///         None
    ///     ).await
    /// });
    ///
    /// // Send first item and wait for it to start processing
    /// tx.send(1).unwrap();
    /// assert_eq!(started_rx.recv().await, Some(1));
    ///
    /// // Send subsequent items while item 1 is blocked
    /// tx.send(2).unwrap();
    /// tx.send(3).unwrap();
    /// tx.send(4).unwrap();
    ///
    /// // Release the gate to let item 1 finish
    /// gate_tx.send(()).unwrap();
    ///
    /// // Wait for item 1 to complete
    /// assert_eq!(done_rx.recv().await, Some(1));
    /// // Wait for latest item (4) to start and complete
    /// assert_eq!(started_rx.recv().await, Some(4));
    /// assert_eq!(done_rx.recv().await, Some(4));
    ///
    /// // Now close the stream
    /// drop(tx);
    ///
    /// handle.await.unwrap().unwrap();
    ///
    /// let result = processed.lock().await.clone();
    /// assert_eq!(result.len(), 2); // Only first and latest
    /// assert_eq!(result[0], 1);
    /// assert_eq!(result[1], 4);
    /// # }
    /// ```
    ///
    /// ## With Cancellation Token Checks
    ///
    /// Handlers should check the token periodically for responsive cancellation:
    ///
    /// ```
    /// use fluxion_exec::SubscribeLatestExt;
    /// use tokio::sync::mpsc::unbounded_channel;
    /// use tokio_stream::wrappers::UnboundedReceiverStream;
    /// use futures::StreamExt;
    /// use std::sync::Arc;
    /// use tokio::sync::Mutex;
    /// use tokio_util::sync::CancellationToken;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let (tx, rx) = unbounded_channel();
    /// let stream = UnboundedReceiverStream::new(rx).map(|x: i32| x);
    ///
    /// let completed = Arc::new(Mutex::new(Vec::new()));
    /// let completed_clone = completed.clone();
    /// let token = CancellationToken::new();
    /// let token_clone = token.clone();
    ///
    /// // Signal start
    /// let (started_tx, mut started_rx) = unbounded_channel::<i32>();
    /// let started_tx = Arc::new(started_tx);
    ///
    /// // Gate
    /// let (gate_tx, mut gate_rx) = unbounded_channel::<()>();
    /// let gate_shared = Arc::new(Mutex::new(Some(gate_rx)));
    ///
    /// let handle = tokio::spawn(async move {
    ///     stream.subscribe_latest(
    ///         move |item, token| {
    ///             let completed = completed_clone.clone();
    ///             let started = started_tx.clone();
    ///             let gate = gate_shared.clone();
    ///             async move {
    ///                 started.send(item).unwrap();
    ///
    ///                 // Wait for gate or cancellation
    ///                 if let Some(mut rx) = gate.lock().await.take() {
    ///                     tokio::select! {
    ///                         _ = rx.recv() => {},
    ///                         _ = token.cancelled() => return Ok(()),
    ///                     }
    ///                 }
    ///
    ///                 completed.lock().await.push(item);
    ///                 Ok::<(), std::io::Error>(())
    ///             }
    ///         },
    ///         None::<fn(std::io::Error)>,
    ///         Some(token_clone)
    ///     ).await
    /// });
    ///
    /// tx.send(1).unwrap();
    /// assert_eq!(started_rx.recv().await, Some(1));
    ///
    /// // Cancel global token and close stream
    /// token.cancel();
    /// drop(tx);
    ///
    /// // 1 should be cancelled
    /// handle.await.unwrap().unwrap();
    ///
    /// let result = completed.lock().await;
    /// assert!(result.is_empty());
    /// # }
    /// ```
    ///
    /// ## Search-As-You-Type Pattern
    ///
    /// Only search for the latest query:
    ///
    /// ```
    /// use fluxion_exec::SubscribeLatestExt;
    /// use tokio::sync::mpsc::unbounded_channel;
    /// use tokio_stream::wrappers::UnboundedReceiverStream;
    /// use futures::StreamExt;
    /// use std::sync::Arc;
    /// use tokio::sync::Mutex;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// // Gate to control when first item completes
    /// let (gate_tx, mut gate_rx) = unbounded_channel::<()>();
    /// let gate_shared = Arc::new(Mutex::new(Some(gate_rx)));
    ///
    /// // Signal when processing starts
    /// let (started_tx, mut started_rx) = unbounded_channel::<String>();
    /// let started_tx = Arc::new(started_tx);
    ///
    /// // Signal when processing completes
    /// let (done_tx, mut done_rx) = unbounded_channel::<String>();
    /// let done_tx = Arc::new(done_tx);
    ///
    /// let (tx, rx) = unbounded_channel();
    /// let stream = UnboundedReceiverStream::new(rx);
    ///
    /// let results = Arc::new(Mutex::new(Vec::new()));
    /// let results_clone = results.clone();
    ///
    /// let handle = tokio::spawn(async move {
    ///     stream.subscribe_latest(
    ///         move |query: String, token| {
    ///             let results = results_clone.clone();
    ///             let gate = gate_shared.clone();
    ///             let started = started_tx.clone();
    ///             let done = done_tx.clone();
    ///             async move {
    ///                 started.send(query.clone()).unwrap();
    ///
    ///                 // First query waits at gate
    ///                 if let Some(mut rx) = gate.lock().await.take() {
    ///                     let _ = rx.recv().await;
    ///                 }
    ///
    ///                 if !token.is_cancelled() {
    ///                     results.lock().await.push(format!("result_for_{}", query));
    ///                 }
    ///                 done.send(query).unwrap();
    ///                 Ok::<(), std::io::Error>(())
    ///             }
    ///         },
    ///         None::<fn(std::io::Error)>,
    ///         None
    ///     ).await
    /// });
    ///
    /// // Send first item and wait for it to start processing
    /// tx.send("r".to_string()).unwrap();
    /// assert_eq!(started_rx.recv().await, Some("r".to_string()));
    ///
    /// // While "r" is blocked, send more items rapidly
    /// tx.send("ru".to_string()).unwrap();
    /// tx.send("rus".to_string()).unwrap();
    /// tx.send("rust".to_string()).unwrap();
    ///
    /// // Close the stream to ensure all items are delivered
    /// drop(tx);
    ///
    /// // Release "r" - this allows processing to continue
    /// gate_tx.send(()).unwrap();
    ///
    /// // Wait for "r" to complete
    /// assert_eq!(done_rx.recv().await, Some("r".to_string()));
    ///
    /// // Wait for "rust" (the latest) to complete
    /// assert_eq!(done_rx.recv().await, Some("rust".to_string()));
    ///
    /// handle.await.unwrap().unwrap();
    ///
    /// let results = results.lock().await;
    /// assert!(results.contains(&"result_for_r".to_string()));
    /// assert!(results.contains(&"result_for_rust".to_string()));
    /// // Intermediate items were skipped
    /// assert!(!results.contains(&"result_for_ru".to_string()));
    /// assert!(!results.contains(&"result_for_rus".to_string()));
    /// # }
    /// ```
    ///
    /// ## UI Update Pattern
    ///
    /// Render only the latest application state:
    ///
    /// ```
    /// use fluxion_exec::SubscribeLatestExt;
    /// use tokio::sync::mpsc::unbounded_channel;
    /// use tokio_stream::wrappers::UnboundedReceiverStream;
    /// use futures::StreamExt;
    /// use std::sync::Arc;
    /// use tokio::sync::Mutex;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct AppState { counter: u32 }
    ///
    /// let (tx, rx) = unbounded_channel();
    /// let stream = UnboundedReceiverStream::new(rx).map(|x: AppState| x);
    ///
    /// let rendered = Arc::new(Mutex::new(Vec::new()));
    /// let rendered_clone = rendered.clone();
    ///
    /// // Gate to control when renders complete (take pattern for first only)
    /// let (gate_tx, gate_rx) = unbounded_channel::<()>();
    /// let gate_rx_shared = Arc::new(Mutex::new(Some(gate_rx)));
    ///
    /// // Signal when processing starts
    /// let (started_tx, mut started_rx) = unbounded_channel::<u32>();
    /// let started_tx = Arc::new(started_tx);
    ///
    /// let handle = tokio::spawn(async move {
    ///     stream.subscribe_latest(
    ///         move |state, token| {
    ///             let rendered = rendered_clone.clone();
    ///             let gate_rx = gate_rx_shared.clone();
    ///             let started = started_tx.clone();
    ///             async move {
    ///                 started.send(state.counter).unwrap();
    ///
    ///                 // First item waits at gate
    ///                 if let Some(mut rx) = gate_rx.lock().await.take() {
    ///                     let _ = rx.recv().await;
    ///                 }
    ///
    ///                 if !token.is_cancelled() {
    ///                     rendered.lock().await.push(state);
    ///                 }
    ///                 Ok::<(), std::io::Error>(())
    ///             }
    ///         },
    ///         None::<fn(std::io::Error)>,
    ///         None
    ///     ).await
    /// });
    ///
    /// // Send first state and wait for it to start
    /// tx.send(AppState { counter: 0 }).unwrap();
    /// assert_eq!(started_rx.recv().await, Some(0));
    ///
    /// // Rapid state updates (intermediate ones will be skipped)
    /// for i in 1..10 {
    ///     tx.send(AppState { counter: i }).unwrap();
    /// }
    /// drop(tx);
    ///
    /// // Release first render - this unblocks item 0, then 9 runs immediately
    /// gate_tx.send(()).unwrap();
    ///
    /// handle.await.unwrap().unwrap();
    ///
    /// // Only first and latest states should be rendered (intermediate ones skipped)
    /// let rendered_states = rendered.lock().await;
    /// assert!(rendered_states.len() < 10); // Intermediate states are skipped
    /// assert_eq!(rendered_states.first().unwrap().counter, 0); // First state
    /// assert_eq!(rendered_states.last().unwrap().counter, 9); // Latest state
    /// # }
    /// ```
    ///
    /// # Use Cases
    ///
    /// - UI updates where only the latest state matters
    /// - Search-as-you-type with debouncing
    /// - Real-time data processing where stale data is irrelevant
    /// - API calls that should be cancelled when parameters change
    ///
    /// # Comparison with `subscribe`
    ///
    /// - `subscribe`: Processes every item, all handlers run to completion
    /// - `subscribe_latest`: Processes only latest, abandons outdated work
    ///
    /// # Thread Safety
    ///
    /// Uses tokio's async mutex for state coordination. Only one processing task
    /// is active at a time, but task spawning is non-blocking.
    async fn subscribe_latest<F, Fut, E, OnError>(
        self,
        on_next_func: F,
        on_error_callback: Option<OnError>,
        cancellation_token: Option<CancellationToken>,
    ) -> Result<()>
    where
        F: Fn(T, CancellationToken) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = std::result::Result<(), E>> + Send + 'static,
        OnError: Fn(E) + Clone + Send + Sync + 'static,
        E: Error + Send + Sync + 'static,
        T: Debug + Clone + Send + Sync + 'static;
}

#[async_trait]
impl<S, T> SubscribeLatestExt<T> for S
where
    S: futures::Stream<Item = T> + Send + Unpin + 'static,
    T: Clone + Send + Sync + 'static,
{
    async fn subscribe_latest<F, Fut, E, OnError>(
        self,
        on_next_func: F,
        on_error_callback: Option<OnError>,
        cancellation_token: Option<CancellationToken>,
    ) -> Result<()>
    where
        F: Fn(T, CancellationToken) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = std::result::Result<(), E>> + Send + 'static,
        OnError: Fn(E) + Clone + Send + Sync + 'static,
        E: Error + Send + Sync + 'static,
        T: Debug + Clone + Send + Sync + 'static,
    {
        let state = Arc::new(Context::default());
        let cancellation_token = cancellation_token.unwrap_or_default();
        let state_for_wait = state.clone();
        let errors: Arc<Mutex<Vec<E>>> = Arc::new(Mutex::new(Vec::new()));
        let errors_clone = errors.clone();

        self.for_each(move |new_data| {
            let on_next_func = on_next_func.clone();
            let state = state.clone();
            let cancellation_token = cancellation_token.clone();
            let on_error_callback = on_error_callback.clone();
            let errors = errors.clone();
            async move {
                if cancellation_token.is_cancelled() {
                    return;
                }

                if state.enqueue_and_try_start_processing(new_data).await {
                    let state = state.clone();
                    let on_next_func = on_next_func.clone();
                    let on_error_callback = on_error_callback.clone();
                    let cancellation_token = cancellation_token.clone();

                    tokio::spawn(async move {
                        while let Some(item) = state.get_item().await {
                            if let Err(error) =
                                on_next_func(item.clone(), cancellation_token.clone()).await
                            {
                                if let Some(on_error_callback) = on_error_callback.clone() {
                                    on_error_callback(error);
                                } else {
                                    // Collect error for later aggregation
                                    errors.lock().push(error);
                                }
                            }

                            // Check if a new item arrived while we were processing
                            if !state.finish_processing_and_check_for_next().await {
                                // No new item, we're done
                                break;
                            }
                        }

                        state.notify_task_complete();
                    });
                }
            }
        })
        .await;

        // Wait for any remaining processing tasks to complete
        state_for_wait.wait_for_processing_complete().await;

        // Check if any errors were collected
        let collected_errors = errors_clone.lock().drain(..).collect::<Vec<_>>();

        if collected_errors.is_empty() {
            Ok(())
        } else {
            Err(FluxionError::from_user_errors(collected_errors))
        }
    }
}

#[derive(Debug)]
struct Context<T> {
    state: FutureMutex<State<T>>,
    processing_complete: Notify,
}

#[derive(Debug)]
struct State<T> {
    item: Option<T>,
    is_processing: bool,
}

impl<T> Context<T> {
    /// Enqueues an item. Returns true if we should spawn a processing task.
    pub async fn enqueue_and_try_start_processing(&self, value: T) -> bool {
        let mut state = self.state.lock().await;
        state.item = Some(value);

        if state.is_processing {
            false
        } else {
            state.is_processing = true;
            true
        }
    }

    /// Gets the current item for processing.
    /// Returns `None` if no item is available (should be unreachable in normal flow).
    pub async fn get_item(&self) -> Option<T>
    where
        T: Clone,
    {
        let mut state = self.state.lock().await;
        state.item.take().map_or_else(
            || {
                // Defensive: log invalid state and mark as not processing to avoid deadlock
                error!("subscribe_latest: get_item called with no current item; marking idle");
                state.is_processing = false;
                None
            },
            |item| Some(item),
        )
    }

    /// Called when processing finishes. Returns true if there's another item to process.
    pub async fn finish_processing_and_check_for_next(&self) -> bool {
        let mut state = self.state.lock().await;

        if state.item.is_some() {
            // New item arrived during processing, continue
            true
        } else {
            // No new item, mark as idle
            state.is_processing = false;
            false
        }
    }

    /// Notify that a processing task has completed
    pub fn notify_task_complete(&self) {
        self.processing_complete.notify_waiters();
    }

    /// Wait for all processing to complete
    pub async fn wait_for_processing_complete(&self) {
        loop {
            {
                let state = self.state.lock().await;
                if !state.is_processing {
                    // No active processing
                    return;
                }
            }
            // Wait for notification
            self.processing_complete.notified().await;
        }
    }
}

impl<T> Default for Context<T> {
    fn default() -> Self {
        Self {
            state: FutureMutex::new(State {
                item: None,
                is_processing: false,
            }),
            processing_complete: Notify::new(),
        }
    }
}
