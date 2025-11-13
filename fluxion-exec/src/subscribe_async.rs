use async_trait::async_trait;
use futures::stream::Stream;
use futures::stream::StreamExt;
use std::future::Future;
use tokio_util::sync::CancellationToken;

#[async_trait]
pub trait SubscribeAsyncExt<T>: Stream<Item = T> + Sized {
    async fn subscribe_async<F, Fut, E, OnError>(
        self,
        on_next_func: F,
        cancellation_token: Option<CancellationToken>,
        on_error_callback: Option<OnError>,
    ) where
        F: Fn(T, CancellationToken) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = Result<(), E>> + Send + 'static,
        OnError: Fn(E) + Clone + Send + Sync + 'static,
        T: std::fmt::Debug + Send + Clone + 'static,
        E: std::error::Error + Send + 'static;
}

#[async_trait]
impl<S, T> SubscribeAsyncExt<T> for S
where
    S: Stream<Item = T> + Send + Unpin + 'static,
    T: Send + 'static,
{
    async fn subscribe_async<F, Fut, E, OnError>(
        mut self,
        on_next_func: F,
        cancellation_token: Option<CancellationToken>,
        on_error_callback: Option<OnError>,
    ) where
        F: Fn(T, CancellationToken) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = Result<(), E>> + Send + 'static,
        OnError: Fn(E) + Clone + Send + Sync + 'static,
        T: std::fmt::Debug + Send + Clone + 'static,
        E: std::error::Error + Send + 'static,
    {
        let cancellation_token = cancellation_token.unwrap_or_default();

        while let Some(item) = self.next().await {
            if cancellation_token.is_cancelled() {
                break;
            }

            let on_next_func = on_next_func.clone();
            let cancellation_token = cancellation_token.clone();
            let on_error_callback = on_error_callback.clone();

            tokio::spawn(async move {
                let result = on_next_func(item.clone(), cancellation_token).await;

                if let Err(error) = result {
                    if let Some(on_error_callback) = on_error_callback {
                        on_error_callback(error);
                    } else {
                        crate::error!(
                            "Unhandled error in subscribe_async while processing item: {:?}, error: {}",
                            item,
                            error
                        );
                    }
                }
            });
        }
    }
}
