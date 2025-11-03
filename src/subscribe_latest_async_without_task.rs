use crate::subscribe_async::SubscribeAsyncExt;
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

#[async_trait]
pub trait SubscribeLatestAsyncExt<T>: Stream<Item = T> + Unpin + Send + 'static
where
    T: Clone + Send + Sync + 'static,
{
    async fn subscribe_latest_async<F, Fut, E, OnError>(
        mut self,
        on_next_func: F,
        on_error_callback: Option<OnError>,
        cancellation_token: Option<CancellationToken>,
    ) where
        F: Fn(T, CancellationToken) -> Fut + Clone + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<(), E>> + Send + 'static,
        OnError: Fn(E) + Clone + Send + Sync + 'static,
        E: std::fmt::Debug + Send + 'static,
        T: Default + std::fmt::Debug + Clone + Send + Sync + 'static;
}

#[async_trait]
impl<S, T> SubscribeLatestAsyncExt<T> for S
where
    S: Stream<Item = T> + Unpin + Send + 'static,
    T: Clone + Send + Sync + 'static,
{
    async fn subscribe_latest_async<F, Fut, E, OnError>(
        mut self,
        on_next_func: F,
        on_error_callback: Option<OnError>,
        cancellation_token: Option<CancellationToken>,
    ) where
        F: Fn(T, CancellationToken) -> Fut + Clone + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<(), E>> + Send + 'static,
        OnError: Fn(E) + Clone + Send + Sync + 'static,
        E: std::fmt::Debug + Send + 'static,
        T: std::fmt::Debug + Clone + Send + Sync + 'static,
    {
        let state = Arc::new(Context::default());
        let cancellation_token = cancellation_token.unwrap_or_default();
        let on_next_func = on_next_func.clone();

        Box::pin(self.scan(state.clone(), |state, new_item| {
            let state = state.clone();
            async move {
                state.set_data(new_item).await;
                Some(state)
            }
        }))
        .subscribe_async(
            move |state, cancellation_token| {
                let on_next_func = on_next_func.clone();
                let state = state.clone();
                let cancellation_token = cancellation_token.clone();
                async move {
                    if let Some(data) = state.try_get_and_block().await {
                        /*

                          - calling on_nex_func here without awaiting leads to doing nothing
                          - awaiting it here leads to blocking the processing of new items until this one is done

                        */
                        on_next_func(data, cancellation_token);
                        state.unblock();
                    }
                    Ok(())
                }
            },
            Some(cancellation_token),
            on_error_callback,
        )
        .await;
    }
}

impl<T: std::fmt::Debug> Default for Context<T> {
    fn default() -> Self {
        Self {
            can_execute: AtomicBool::new(true),
            data: Mutex::new(None),
        }
    }
}

#[derive(Debug)]
pub struct Context<T: std::fmt::Debug> {
    can_execute: AtomicBool,
    data: Mutex<Option<T>>,
}

impl<T: std::fmt::Debug> Context<T> {
    pub async fn try_get_and_block(&self) -> Option<T> {
        if self
            .can_execute
            .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            let mut lock = self.data.lock().await;
            lock.take()
        } else {
            None
        }
    }

    pub async fn set_data(&self, data: T) {
        let mut lock = self.data.lock().await;
        *lock = Some(data);
    }

    pub fn unblock(&self) {
        self.can_execute.store(true, Ordering::SeqCst);
    }
}
