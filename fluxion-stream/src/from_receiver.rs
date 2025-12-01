// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Convenience constructors for creating `FluxionStream` from tokio channels.

use crate::FluxionStream;
use fluxion_core::StreamItem;
use futures::{Stream, StreamExt};
use std::pin::Pin;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio_stream::wrappers::UnboundedReceiverStream;

/// Extension trait to convert tokio channels into `FluxionStream`.
///
/// This trait provides a simple way to wrap a tokio `UnboundedReceiver` into
/// a `FluxionStream` that emits `StreamItem::Value` for each received item.
pub trait ReceiverStreamExt<T> {
    /// Converts this receiver into a `FluxionStream`.
    ///
    /// Each item received from the channel is wrapped in `StreamItem::Value`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fluxion_stream::ReceiverStreamExt;
    /// use tokio::sync::mpsc;
    ///
    /// let (tx, rx) = mpsc::unbounded_channel::<i32>();
    /// let stream = rx.to_fluxion_stream();
    /// ```
    fn to_fluxion_stream(
        self,
    ) -> FluxionStream<Pin<Box<dyn Stream<Item = StreamItem<T>> + Send + Sync>>>;
}

impl<T: Send + 'static> ReceiverStreamExt<T> for UnboundedReceiver<T> {
    fn to_fluxion_stream(
        self,
    ) -> FluxionStream<Pin<Box<dyn Stream<Item = StreamItem<T>> + Send + Sync>>> {
        FluxionStream::new(Box::pin(
            UnboundedReceiverStream::new(self).map(StreamItem::Value),
        ))
    }
}
