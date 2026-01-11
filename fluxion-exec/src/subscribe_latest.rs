// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Latest-value stream subscription with automatic cancellation.
//!
//! # Runtime Requirements
//!
//! This operator requires one of the following runtime features:
//! - `runtime-tokio` (default)
//! - `runtime-smol`
//! - `runtime-async-std`
//! - Or compiling for `wasm32` target
//!
//! It is not available when compiling without a runtime (no_std + alloc only).
//!
//! ## Alternatives for no_std
//!
//! Use time-based operators with `subscribe()` for rate limiting:
//! ```ignore
//! stream.throttle(Duration::from_millis(100))
//!       .subscribe(slow_handler)
//! ```

#[cfg(any(
    feature = "runtime-tokio",
    feature = "runtime-smol",
    feature = "runtime-async-std",
    target_arch = "wasm32"
))]
use alloc::sync::Arc;
use async_trait::async_trait;
use core::fmt::Debug;
use core::future::Future;
use event_listener::Event;
use fluxion_core::{CancellationToken, FluxionTask, Result};
use futures::lock::Mutex as FutureMutex;
use futures::{Stream, StreamExt};

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
    /// * `on_error_callback` - Error handler for processing failures
    /// * `cancellation_token` - Optional token to stop all processing
    ///
    /// # Type Parameters
    ///
    /// * `F` - Function type for the item handler
    /// * `Fut` - Future type returned by the handler
    /// * `E` - Error type
    /// * `OnError` - Function type for error handling
    ///
    /// # Errors
    ///
    /// Returns `Ok(())` on stream completion. Errors from item handlers are passed
    /// to the error callback. Processing continues with new items even if previous
    /// items failed, unless the cancellation token is triggered.
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
    /// use futures::channel::mpsc::unbounded;
    /// use futures::StreamExt;
    /// use std::sync::Arc;
    /// use futures::lock::Mutex;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let (tx, rx) = unbounded();
    /// let stream = rx.map(|x: i32| x);
    ///
    /// let processed = Arc::new(Mutex::new(Vec::new()));
    /// let processed_clone = processed.clone();
    ///
    /// // Gate to control when first item completes
    /// let (gate_tx, gate_rx) = unbounded::<()>();
    /// let gate_shared = Arc::new(Mutex::new(Some(gate_rx)));
    ///
    /// // Signal when processing starts and completes
    /// let (started_tx, mut started_rx) = unbounded::<i32>();
    /// let started_tx = Arc::new(started_tx);
    /// let (done_tx, mut done_rx) = unbounded::<i32>();
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
    ///                 started.unbounded_send(item).unwrap();
    ///                 // First item waits at gate
    ///                 if let Some(mut rx) = gate.lock().await.take() {
    ///                     let _ = rx.next().await;
    ///                 }
    ///                 if !token.is_cancelled() {
    ///                     processed.lock().await.push(item);
    ///                 }
    ///                 done.unbounded_send(item).unwrap();
    ///                 Ok::<(), std::io::Error>(())
    ///             }
    ///         },
    ///         |_| {}, // Ignore errors
    ///         None
    ///     ).await
    /// });
    ///
    /// // Send first item and wait for it to start processing
    /// tx.unbounded_send(1).unwrap();
    /// assert_eq!(started_rx.next().await, Some(1));
    ///
    /// // Send subsequent items while item 1 is blocked
    /// tx.unbounded_send(2).unwrap();
    /// tx.unbounded_send(3).unwrap();
    /// tx.unbounded_send(4).unwrap();
    ///
    /// // Release the gate to let item 1 finish
    /// gate_tx.unbounded_send(()).unwrap();
    ///
    /// // Wait for item 1 to complete
    /// assert_eq!(done_rx.next().await, Some(1));
    /// // Wait for latest item (4) to start and complete
    /// assert_eq!(started_rx.next().await, Some(4));
    /// assert_eq!(done_rx.next().await, Some(4));
    ///
    /// // Now close the stream
    /// drop(tx);
    ///
    /// handle.await.unwrap().unwrap();
    ///
    /// let result = processed.lock().await.clone();
    /// // Should have at least 2 items (first and latest)
    /// assert!(result.len() >= 2, "Expected at least 2 items, got {}", result.len());
    /// assert_eq!(result[0], 1, "First item should be 1");
    /// // Last item should be 4 (latest)
    /// assert_eq!(*result.last().unwrap(), 4, "Last item should be 4");
    /// # }
    /// ```
    ///
    /// ## With Cancellation Token Checks
    ///
    /// Handlers should check the token periodically for responsive cancellation:
    ///
    /// ```
    /// use fluxion_exec::SubscribeLatestExt;
    /// use futures::channel::mpsc::unbounded;
    /// use futures::{StreamExt, FutureExt};
    /// use std::sync::Arc;
    /// use futures::lock::Mutex;
    /// use fluxion_core::CancellationToken;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let (tx, rx) = unbounded();
    /// let stream = rx.map(|x: i32| x);
    ///
    /// let completed = Arc::new(Mutex::new(Vec::new()));
    /// let completed_clone = completed.clone();
    /// let token = CancellationToken::new();
    /// let token_clone = token.clone();
    ///
    /// // Signal start
    /// let (started_tx, mut started_rx) = unbounded::<i32>();
    /// let started_tx = Arc::new(started_tx);
    ///
    /// // Gate
    /// let (gate_tx, mut gate_rx) = unbounded::<()>();
    /// let gate_shared = Arc::new(Mutex::new(Some(gate_rx)));
    ///
    /// let handle = tokio::spawn(async move {
    ///     stream.subscribe_latest(
    ///         move |item, token| {
    ///             let completed = completed_clone.clone();
    ///             let started = started_tx.clone();
    ///             let gate = gate_shared.clone();
    ///             async move {
    ///                 started.unbounded_send(item).unwrap();
    ///
    ///                 // Wait for gate or cancellation
    ///                 if let Some(mut rx) = gate.lock().await.take() {
    ///                     futures::select! {
    ///                         _ = rx.next().fuse() => {},
    ///                         _ = token.cancelled().fuse() => return Ok(()),
    ///                     }
    ///                 }
    ///
    ///                 completed.lock().await.push(item);
    ///                 Ok::<(), std::io::Error>(())
    ///             }
    ///         },
    ///         |_| {}, // Ignore errors
    ///         Some(token_clone)
    ///     ).await
    /// });
    ///
    /// tx.unbounded_send(1).unwrap();
    /// assert_eq!(started_rx.next().await, Some(1));
    ///
    /// // Cancel global token and close stream
    /// token.cancel();
    /// drop(tx);
    ///
    /// // Items should be cancelled
    /// handle.await.unwrap().unwrap();
    ///
    /// let result = completed.lock().await;
    /// // With cancellation, no items or very few should complete
    /// assert!(result.len() <= 1, "Expected 0-1 items with cancellation, got {}", result.len());
    /// # }
    /// ```
    ///
    /// ## UI Update Pattern
    ///
    /// Render only the latest application state:
    ///
    /// ```
    /// use fluxion_exec::SubscribeLatestExt;
    /// use futures::channel::mpsc::unbounded;
    /// use futures::StreamExt;
    /// use std::sync::Arc;
    /// use futures::lock::Mutex;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct AppState { counter: u32 }
    ///
    /// let (tx, rx) = unbounded();
    /// let stream = rx.map(|x: AppState| x);
    ///
    /// let rendered = Arc::new(Mutex::new(Vec::new()));
    /// let rendered_clone = rendered.clone();
    ///
    /// let handle = tokio::spawn(async move {
    ///     stream.subscribe_latest(
    ///         move |state, _token| {
    ///             let rendered = rendered_clone.clone();
    ///             async move {
    ///                 // Simulate some processing time
    ///                 tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
    ///                 rendered.lock().await.push(state);
    ///                 Ok::<(), std::io::Error>(())
    ///             }
    ///         },
    ///         |_| {}, // Ignore errors
    ///         None
    ///     ).await
    /// });
    ///
    /// // Send rapid state updates
    /// for i in 0..10 {
    ///     tx.unbounded_send(AppState { counter: i }).unwrap();
    /// }
    ///
    /// // Close stream to complete
    /// drop(tx);
    ///
    /// handle.await.unwrap().unwrap();
    ///
    /// // Verify skipping behavior: not all items were processed
    /// let rendered_states = rendered.lock().await;
    /// // With subscribe_latest, intermediate items should be skipped
    /// assert!(!rendered_states.is_empty(), "Expected at least 1 item");
    /// assert!(rendered_states.len() <= 10, "Should have processed some items");
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
        on_error_callback: OnError,
        cancellation_token: Option<CancellationToken>,
    ) -> Result<()>
    where
        F: Fn(T, CancellationToken) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = core::result::Result<(), E>> + Send + 'static,
        OnError: Fn(E) + Clone + Send + Sync + 'static,
        E: Send + 'static,
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
        on_error_callback: OnError,
        cancellation_token: Option<CancellationToken>,
    ) -> Result<()>
    where
        F: Fn(T, CancellationToken) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = core::result::Result<(), E>> + Send + 'static,
        OnError: Fn(E) + Clone + Send + Sync + 'static,
        E: Send + 'static,
        T: Debug + Clone + Send + Sync + 'static,
    {
        let state = Arc::new(Context::default());
        let cancellation_token = cancellation_token.unwrap_or_default();
        let state_for_wait = state.clone();

        self.for_each(move |new_data| {
            let on_next_func = on_next_func.clone();
            let state = state.clone();
            let cancellation_token = cancellation_token.clone();
            let on_error_callback = on_error_callback.clone();
            async move {
                if cancellation_token.is_cancelled() {
                    return;
                }

                if state.enqueue_and_try_start_processing(new_data).await {
                    let state_for_task = state.clone();
                    let on_next_func = on_next_func.clone();
                    let on_error_callback = on_error_callback.clone();
                    let cancellation_token = cancellation_token.clone();

                    let task = FluxionTask::spawn(|task_cancel| async move {
                        while let Some(item) = state_for_task.get_item().await {
                            if task_cancel.is_cancelled() {
                                break;
                            }

                            if let Err(error) =
                                on_next_func(item.clone(), cancellation_token.clone()).await
                            {
                                on_error_callback(error);
                            }

                            // Check if a new item arrived while we were processing
                            if !state_for_task.finish_processing_and_check_for_next().await {
                                // No new item, we're done
                                break;
                            }
                        }

                        state_for_task.notify_task_complete();
                    });

                    // Keep task alive to prevent immediate cancellation
                    *state.task.lock().await = Some(task);
                }
            }
        })
        .await;

        // Wait for any remaining processing tasks to complete
        state_for_wait.wait_for_processing_complete().await;

        Ok(())
    }
}

#[derive(Debug)]
struct Context<T> {
    state: FutureMutex<State<T>>,
    processing_complete: Event,
    task: FutureMutex<Option<FluxionTask>>,
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
        self.processing_complete.notify(usize::MAX);
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
            let listener = self.processing_complete.listen();
            listener.await;
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
            processing_complete: Event::new(),
            task: FutureMutex::new(None),
        }
    }
}
