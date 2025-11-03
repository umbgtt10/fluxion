use async_trait::async_trait;
use futures::{Stream, StreamExt};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tokio::sync::Mutex;
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
        E: std::fmt::Debug + Send + 'static,
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
        E: std::fmt::Debug + Send + 'static,
        T: std::fmt::Debug + Clone + Send + Sync + 'static,
    {
        let state = Arc::new(Context::default());
        let cancellation_token = cancellation_token.unwrap_or_default();

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
                        let item = state.get_item().await;
                        if let Err(error) = on_next_func(item.clone(), cancellation_token).await {
                            if let Some(on_error_callback) = on_error_callback {
                                on_error_callback(error);
                            } else {
                                panic!(
                                    "Unhandled error in subscribe_async while processing item: {:?}, error: {:?}",
                                    item, error
                                );
                            }
                        }

                        state.unblock();
                    });
                }
            }
        })
        .await;
    }
}

#[derive(Debug)]
struct Context<T> {
    item: Mutex<Option<T>>,
    is_processing: AtomicBool,
}

impl<T> Context<T> {
    pub async fn enqueue_and_try_start_processing(&self, value: T) -> bool {
        if self
            .is_processing
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            let mut guard = self.item.lock().await;
            *guard = Some(value);
            true
        } else {
            let mut guard = self.item.lock().await;
            *guard = Some(value);
            false
        }
    }

    pub async fn get_item(&self) -> T {
        let mut guard = self.item.lock().await;
        guard
            .take()
            .expect("Buffer should always contain data when `take` is called")
    }

    pub fn unblock(&self) {
        self.is_processing.store(false, Ordering::SeqCst);
    }
}

impl<T> Default for Context<T> {
    fn default() -> Self {
        Self {
            item: Mutex::new(None),
            is_processing: AtomicBool::new(false),
        }
    }
}
