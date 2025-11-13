use async_trait::async_trait;
use futures::{Stream, StreamExt};
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};
use tokio_util::sync::CancellationToken;

#[async_trait]
pub trait SubscribeLatestAsyncExt<T>: Stream<Item = T> + Send + 'static
where
    T: Send + 'static,
{
    async fn subscribe_latest_async<F, Fut, E, OnError>(
        self,
        on_next_func: F,
        on_error_callback: Option<OnError>,
        cancellation_token: Option<CancellationToken>,
    ) where
        F: Fn(T, CancellationToken) -> Fut + Clone + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<(), E>> + Send + 'static,
        OnError: Fn(E) + Clone + Send + Sync + 'static,
        E: std::error::Error + Send + 'static,
        T: std::fmt::Debug + Clone + Send + Sync + 'static;
}

#[async_trait]
impl<S, T> SubscribeLatestAsyncExt<T> for S
where
    S: futures::Stream<Item = T> + Send + Unpin + 'static,
    T: Clone + Send + Sync + 'static,
{
    async fn subscribe_latest_async<F, Fut, E, OnError>(
        self,
        on_next_func: F,
        on_error_callback: Option<OnError>,
        cancellation_token: Option<CancellationToken>,
    ) where
        F: Fn(T, CancellationToken) -> Fut + Clone + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<(), E>> + Send + 'static,
        OnError: Fn(E) + Clone + Send + Sync + 'static,
        E: std::error::Error + Send + 'static,
        T: std::fmt::Debug + Clone + Send + Sync + 'static,
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
                    let state = state.clone();
                    let on_next_func = on_next_func.clone();
                    let on_error_callback = on_error_callback.clone();
                    let cancellation_token = cancellation_token.clone();

                    tokio::spawn(async move {
                        loop {
                            // Get the latest item
                            let item = state.get_item().await;

                            // Process it
                            if let Err(error) = on_next_func(item.clone(), cancellation_token.clone()).await {
                                if let Some(on_error_callback) = on_error_callback.clone() {
                                    on_error_callback(error);
                                } else {
                                    crate::error!(
                                        "Unhandled error in subscribe_latest_async while processing item: {:?}, error: {}",
                                        item, error
                                    );
                                }
                            }

                            // Check if a new item arrived while we were processing
                            if !state.finish_processing_and_check_for_next().await {
                                // No new item, we're done
                                break;
                            }
                            // Loop to process the next item
                        }

                        // Notify that this processing task has completed
                        state.notify_task_complete();
                    });
                }
            }
        })
        .await;

        // Wait for any remaining processing tasks to complete
        state_for_wait.wait_for_processing_complete().await;
    }
}

#[derive(Debug)]
struct Context<T> {
    state: Mutex<State<T>>,
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

        if !state.is_processing {
            state.is_processing = true;
            true
        } else {
            false
        }
    }

    /// Gets the current item for processing.
    pub async fn get_item(&self) -> T
    where
        T: Clone,
    {
        let mut state = self.state.lock().await;
        if let Some(item) = state.item.take() {
            item
        } else {
            // This should never happen if called correctly, but we handle it gracefully
            // by returning a cloned value from the previous state if available
            panic!("get_item called when no item is available - this is a logic error")
        }
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
            state: Mutex::new(State {
                item: None,
                is_processing: false,
            }),
            processing_complete: Notify::new(),
        }
    }
}
