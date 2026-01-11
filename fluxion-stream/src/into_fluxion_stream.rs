// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::boxed::Box;
#[cfg(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std",
    feature = "runtime-wasm",
    feature = "runtime-embassy",
    target_arch = "wasm32",
    feature = "alloc"
))]
use async_channel::Receiver;
use core::fmt::Debug;
use core::pin::Pin;
use fluxion_core::{StreamItem, Timestamped};
use futures::Stream;
#[cfg(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std",
    feature = "runtime-wasm",
    feature = "runtime-embassy",
    target_arch = "wasm32",
    feature = "alloc"
))]
use futures::StreamExt;

/// Extension trait to convert futures channels into fluxion streams.
///
/// This trait provides a simple way to wrap a futures `UnboundedReceiver` into
/// a stream that emits `StreamItem::Value` for each received item.
#[cfg(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std"
))]
pub trait IntoFluxionStream<T> {
    /// Converts this receiver into a fluxion stream.
    ///
    /// Each item received from the channel is wrapped in `StreamItem::Value`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fluxion_stream::IntoFluxionStream;
    /// use async_channel::unbounded;
    ///
    /// let (tx, rx) = unbounded::<i32>();
    /// let stream = rx.into_fluxion_stream();
    /// ```
    fn into_fluxion_stream(self) -> impl Stream<Item = StreamItem<T>> + Send + Sync;

    fn into_fluxion_stream_map<U, F>(
        self,
        mapper: F,
    ) -> Pin<Box<dyn Stream<Item = StreamItem<U>> + Send + Sync>>
    where
        F: FnMut(T) -> U + Send + Sync + 'static,
        U: Timestamped<Inner = U> + Clone + Debug + Ord + Send + Sync + Unpin + 'static;
}

// Single-threaded version (WASM, Embassy)
#[cfg(not(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std"
)))]
pub trait IntoFluxionStream<T> {
    fn into_fluxion_stream(self) -> impl Stream<Item = StreamItem<T>>;

    fn into_fluxion_stream_map<U, F>(self, mapper: F) -> Pin<Box<dyn Stream<Item = StreamItem<U>>>>
    where
        F: FnMut(T) -> U + 'static,
        U: Timestamped<Inner = U> + Clone + Debug + Ord + Unpin + 'static;
}

#[cfg(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std"
))]
impl<T: Send + 'static> IntoFluxionStream<T> for Receiver<T> {
    fn into_fluxion_stream(self) -> impl Stream<Item = StreamItem<T>> + Send + Sync {
        Box::pin(self.map(StreamItem::Value))
    }

    fn into_fluxion_stream_map<U, F>(
        self,
        mut mapper: F,
    ) -> Pin<Box<dyn Stream<Item = StreamItem<U>> + Send + Sync>>
    where
        F: FnMut(T) -> U + Send + Sync + 'static,
        U: Timestamped<Inner = U> + Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    {
        Box::pin(self.map(move |value| StreamItem::Value(mapper(value))))
    }
}

// Single-threaded impl (WASM, Embassy, or no runtime)
#[cfg(not(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std"
)))]
impl<T: 'static> IntoFluxionStream<T> for Receiver<T> {
    fn into_fluxion_stream(self) -> impl Stream<Item = StreamItem<T>> {
        Box::pin(self.map(StreamItem::Value))
    }

    fn into_fluxion_stream_map<U, F>(
        self,
        mut mapper: F,
    ) -> Pin<Box<dyn Stream<Item = StreamItem<U>>>>
    where
        F: FnMut(T) -> U + 'static,
        U: Timestamped<Inner = U> + Clone + Debug + Ord + Unpin + 'static,
    {
        Box::pin(self.map(move |value| StreamItem::Value(mapper(value))))
    }
}
