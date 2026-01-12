// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use core::fmt::Debug;
use core::future::Future;
use fluxion_core::{CancellationToken, Result};
use futures::stream::{Stream, StreamExt};

// Shared implementation logic
pub async fn subscribe_impl<S, T, F, Fut, E, OnError>(
    mut stream: S,
    on_next_func: F,
    on_error_callback: OnError,
    cancellation_token: Option<CancellationToken>,
) -> Result<()>
where
    S: Stream<Item = T> + Unpin,
    F: Fn(T, CancellationToken) -> Fut + Clone,
    Fut: Future<Output = core::result::Result<(), E>>,
    OnError: Fn(E) + Clone,
    T: Debug + Clone,
{
    let cancellation_token = cancellation_token.unwrap_or_default();

    while let Some(item) = stream.next().await {
        if cancellation_token.is_cancelled() {
            break;
        }

        // Call handler directly (sequential processing)
        let result = on_next_func(item.clone(), cancellation_token.clone()).await;

        if let Err(error) = result {
            on_error_callback(error);
        }
    }

    Ok(())
}

macro_rules! define_subscribe_impl {
    (@step #[$attr:meta], $($bounds:tt)*) => {
        use alloc::boxed::Box;
        use async_trait::async_trait;
        use core::fmt::Debug;
        use core::future::Future;
        use fluxion_core::{CancellationToken, Result};
        use futures::stream::Stream;
        use crate::subscribe::implementation::subscribe_impl;

        /// Extension trait providing async subscription capabilities for streams.
        ///
        /// This trait enables processing stream items with async handlers in a sequential manner.
        #[$attr]
        pub trait SubscribeExt<T>: Stream<Item = T> + Sized {
            /// Subscribes to the stream with an async handler, processing items sequentially.
            ///
            /// This method consumes the stream and processes each item with the provided handler.
            /// Items are processed in the order they arrive, with each item's handler completing
            /// before the next item is processed.
            ///
            /// # Behavior
            ///
            /// - Processes each stream item with the provided async handler sequentially
            /// - Waits for handler completion before processing next item
            /// - Continues until stream ends or cancellation token is triggered
            /// - Errors from handlers are passed to the error callback
            ///
            /// # Arguments
            ///
            /// * `on_next_func` - Async function called for each stream item
            /// * `on_error_callback` - Error handler called when handler returns an error
            /// * `cancellation_token` - Optional token to stop processing
            ///
            /// # See Also
            ///
            /// - [`subscribe_latest`](crate::SubscribeLatestExt::subscribe_latest) - Cancels old work for new items
            async fn subscribe<F, Fut, E, OnError>(
                self,
                on_next_func: F,
                on_error_callback: OnError,
                cancellation_token: Option<CancellationToken>,
            ) -> Result<()>
            where
                F: Fn(T, CancellationToken) -> Fut + Clone + $($bounds)* 'static,
                Fut: Future<Output = core::result::Result<(), E>> + $($bounds)* 'static,
                OnError: Fn(E) + Clone + $($bounds)* 'static,
                T: Debug + Clone + $($bounds)* 'static,
                E: $($bounds)* 'static;
        }

        #[$attr]
        impl<S, T> SubscribeExt<T> for S
        where
            S: Stream<Item = T> + Unpin + $($bounds)* 'static,
            T: $($bounds)* 'static,
        {
            async fn subscribe<F, Fut, E, OnError>(
                self,
                on_next_func: F,
                on_error_callback: OnError,
                cancellation_token: Option<CancellationToken>,
            ) -> Result<()>
            where
                F: Fn(T, CancellationToken) -> Fut + Clone + $($bounds)* 'static,
                Fut: Future<Output = core::result::Result<(), E>> + $($bounds)* 'static,
                OnError: Fn(E) + Clone + $($bounds)* 'static,
                T: Debug + Clone + $($bounds)* 'static,
                E: $($bounds)* 'static,
            {
                subscribe_impl(self, on_next_func, on_error_callback, cancellation_token).await
            }
        }
    };

    // Single threaded (no bounds)
    () => {
        define_subscribe_impl!(@step #[async_trait(?Send)], );
    };

    // Multi threaded (bounds provided)
    ($($bounds:tt)+) => {
        define_subscribe_impl!(@step #[async_trait], $($bounds)+);
    };
}
