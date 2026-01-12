// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use core::fmt::Debug;
use event_listener::Event;
use fluxion_core::FluxionTask;
use futures::lock::Mutex as FutureMutex;

#[derive(Debug)]
pub(crate) struct Context<T> {
    pub(crate) state: FutureMutex<State<T>>,
    pub(crate) processing_complete: Event,
    pub(crate) task: FutureMutex<Option<FluxionTask>>,
}

#[derive(Debug)]
pub(crate) struct State<T> {
    pub(crate) item: Option<T>,
    pub(crate) is_processing: bool,
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
    pub async fn get_item(&self) -> Option<T>
    where
        T: Clone,
    {
        let mut state = self.state.lock().await;
        state.item.take().map_or_else(
            || {
                #[cfg(feature = "tracing")]
                tracing::error!(
                    "subscribe_latest: get_item called with no current item; marking idle"
                );
                #[cfg(not(feature = "tracing"))]
                eprintln!("subscribe_latest: get_item called with no current item; marking idle");
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
            true
        } else {
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
                    return;
                }
            }
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

macro_rules! define_subscribe_latest_impl {
    (@step #[$attr:meta], $($bounds:tt)*) => {
        use alloc::sync::Arc;
        use async_trait::async_trait;
        use core::fmt::Debug;
        use core::future::Future;
        use fluxion_core::{FluxionTask, CancellationToken, Result};
        use futures::{Stream, StreamExt};
        use crate::subscribe_latest::implementation::Context;

        /// Extension trait providing async subscription with automatic cancellation of outdated work.
        ///
        /// This trait enables processing stream items where newer items automatically cancel
        /// processing of older items, ensuring only the latest value is being processed.
        #[$attr]
        pub trait SubscribeLatestExt<T>: Stream<Item = T> + Sized {
            /// Subscribes to the stream, automatically cancelling processing of older items when new items arrive.
            ///
            /// This method is ideal for scenarios where you only care about processing the most recent
            /// value and want to abandon work on outdated values.
            ///
            /// # Behavior
            ///
            /// - Only one processing task runs at a time per stream
            /// - When a new item arrives during processing, it queues as "latest"
            /// - After current processing completes, the latest queued item is processed
            /// - Intermediate items between current and latest are discarded
            ///
            /// # Arguments
            ///
            /// * `on_next_func` - Async function called for each item
            /// * `on_error_callback` - Error handler for processing failures
            /// * `cancellation_token` - Optional token to stop all processing
            ///
            /// # See Also
            ///
            /// - [`subscribe`](crate::SubscribeExt::subscribe) - Sequential processing of all items
            async fn subscribe_latest<F, Fut, E, OnError>(
                self,
                on_next_func: F,
                on_error_callback: OnError,
                cancellation_token: Option<CancellationToken>,
            ) -> Result<()>
            where
                F: Fn(T, CancellationToken) -> Fut + Clone + $($bounds)* 'static,
                Fut: Future<Output = core::result::Result<(), E>> + $($bounds)* 'static,
                OnError: Fn(E) + Clone + $($bounds)* 'static,
                E: $($bounds)* 'static,
                T: Debug + Clone + $($bounds)* 'static;
        }

        #[$attr]
        impl<S, T> SubscribeLatestExt<T> for S
        where
            S: Stream<Item = T> + Unpin + $($bounds)* 'static,
            T: Debug + Clone + $($bounds)* 'static,
        {
            async fn subscribe_latest<F, Fut, E, OnError>(
                self,
                on_next_func: F,
                on_error_callback: OnError,
                cancellation_token: Option<CancellationToken>,
            ) -> Result<()>
            where
                F: Fn(T, CancellationToken) -> Fut + Clone + $($bounds)* 'static,
                Fut: Future<Output = core::result::Result<(), E>> + $($bounds)* 'static,
                OnError: Fn(E) + Clone + $($bounds)* 'static,
                E: $($bounds)* 'static,
                T: Debug + Clone + $($bounds)* 'static,
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
                                    if task_cancel.is_cancelled() || cancellation_token.is_cancelled() {
                                        break;
                                    }

                                    if let Err(error) =
                                        on_next_func(item.clone(), cancellation_token.clone()).await
                                    {
                                        on_error_callback(error);
                                    }

                                    if !state_for_task.finish_processing_and_check_for_next().await {
                                        break;
                                    }
                                }

                                state_for_task.notify_task_complete();
                            });

                            *state.task.lock().await = Some(task);
                        }
                    }
                })
                .await;

                state_for_wait.wait_for_processing_complete().await;

                Ok(())
            }
        }
    };

    // Single threaded (no bounds)
    () => {
        define_subscribe_latest_impl!(@step #[async_trait(?Send)], );
    };

    // Multi threaded (bounds provided)
    ($($bounds:tt)+) => {
        define_subscribe_latest_impl!(@step #[async_trait], $($bounds)+);
    };
}
