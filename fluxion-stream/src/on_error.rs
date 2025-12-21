// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, StreamItem};
use futures::future::ready;
use futures::{Stream, StreamExt};

/// Extension trait providing the `on_error` operator for streams.
///
/// This trait allows any stream of `StreamItem<T>` to handle errors
/// with a custom handler function.
pub trait OnErrorExt<T>: Stream<Item = StreamItem<T>> + Sized {
    /// Handle errors in the stream with a handler function.
    ///
    /// The handler receives a reference to each error and returns:
    /// - `true` to consume the error (remove from stream)
    /// - `false` to propagate the error downstream
    ///
    /// Multiple `on_error` operators can be chained to implement the
    /// Chain of Responsibility pattern for error handling.
    ///
    /// # Examples
    ///
    /// ## Basic Error Consumption
    ///
    /// ```rust
    /// use fluxion_stream::{OnErrorExt, IntoFluxionStream};
    /// use fluxion_core::{FluxionError, StreamItem};
    /// use fluxion_test_utils::Sequenced;
    /// use futures::StreamExt;
    ///
    /// # async fn example() {
    /// let (tx, rx) = futures::channel::mpsc::unbounded();
    /// let stream = rx.into_fluxion_stream();
    ///
    /// let mut stream = stream
    ///     .on_error(|err| {
    ///         eprintln!("Error: {}", err);
    ///         true // Consume all errors
    ///     });
    ///
    /// tx.unbounded_send(Sequenced::new(1)).unwrap();
    /// assert_eq!(stream.next().await.unwrap().unwrap().into_inner(), 1);
    /// # }
    /// ```
    ///
    /// ## Chain of Responsibility
    ///
    /// ```rust
    /// use fluxion_stream::{OnErrorExt, IntoFluxionStream};
    /// use fluxion_test_utils::Sequenced;
    /// use futures::StreamExt;
    ///
    /// # async fn example() {
    /// let (tx, rx) = futures::channel::mpsc::unbounded();
    /// let stream = rx.into_fluxion_stream();
    ///
    /// let mut stream = stream
    ///     .on_error(|err| err.to_string().contains("validation"))
    ///     .on_error(|err| err.to_string().contains("network"))
    ///     .on_error(|_| true); // Catch-all
    ///
    /// tx.unbounded_send(Sequenced::new(1)).unwrap();
    /// assert_eq!(stream.next().await.unwrap().unwrap().into_inner(), 1);
    /// # }
    /// ```
    ///
    /// # See Also
    ///
    /// - [Error Handling Guide](../docs/ERROR-HANDLING.md) - Comprehensive error patterns
    fn on_error<F>(self, handler: F) -> impl Stream<Item = StreamItem<T>>
    where
        F: FnMut(&FluxionError) -> bool;
}

impl<S, T> OnErrorExt<T> for S
where
    S: Stream<Item = StreamItem<T>>,
{
    fn on_error<F>(self, mut handler: F) -> impl Stream<Item = StreamItem<T>>
    where
        F: FnMut(&FluxionError) -> bool,
    {
        self.filter_map(move |item| {
            ready(match item {
                StreamItem::Error(err) => {
                    if handler(&err) {
                        // Error handled, skip it
                        None
                    } else {
                        // Error not handled, propagate
                        Some(StreamItem::Error(err))
                    }
                }
                other => Some(other),
            })
        })
    }
}
