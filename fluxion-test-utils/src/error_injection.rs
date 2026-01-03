// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Test utilities for error injection in streams.
//!
//! This module provides stream wrappers that can inject `StreamItem::Error` values
//! into streams for testing error propagation behavior in stream operators.

use core::pin::Pin;
use core::task::{Context, Poll};
use fluxion_core::{FluxionError, StreamItem, Timestamped};
use futures::Stream;

/// A stream wrapper that injects errors at specified positions.
///
/// This wrapper takes a stream that produces ordered values and wraps them in
/// `StreamItem::Value`, optionally injecting `StreamItem::Error` at a specified position.
///
/// # Examples
///
/// ```rust
/// use fluxion_test_utils::Sequenced;
/// use fluxion_test_utils::ErrorInjectingStream;
/// use fluxion_core::{StreamItem, Timestamped };
/// use futures::{stream, StreamExt};
///
/// # async fn example() {
/// let items = vec![
///     <Sequenced<i32> >::with_timestamp(1, 1),
///     <Sequenced<i32> >::with_timestamp(2, 2),
///     <Sequenced<i32> >::with_timestamp(3, 3),
/// ];
///
/// let base_stream = stream::iter(items);
/// let mut error_stream = ErrorInjectingStream::new(base_stream, 1);
///
/// // First item is a value
/// let first = error_stream.next().await.unwrap();
/// assert!(matches!(first, StreamItem::Value(_)));
///
/// // Second item is the injected error
/// let second = error_stream.next().await.unwrap();
/// assert!(matches!(second, StreamItem::Error(_)));
///
/// // Third item is a value again
/// let third = error_stream.next().await.unwrap();
/// assert!(matches!(third, StreamItem::Value(_)));
/// # }
/// ```
pub struct ErrorInjectingStream<S> {
    inner: S,
    inject_error_at: Option<usize>,
    count: usize,
}

impl<S> ErrorInjectingStream<S> {
    /// Creates a new error-injecting stream wrapper.
    ///
    /// # Arguments
    ///
    /// * `inner` - The base stream to wrap
    /// * `inject_error_at` - The position (0-indexed) at which to inject an error
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fluxion_test_utils::{Sequenced, ErrorInjectingStream};
    /// use futures::stream;
    ///
    /// let items = vec![Sequenced::new(1), Sequenced::new(2)];
    /// let base = stream::iter(items);
    /// let error_stream = ErrorInjectingStream::new(base, 1);
    /// // Will inject error at position 1 (after first value)
    /// ```
    pub fn new(inner: S, inject_error_at: usize) -> Self {
        Self {
            inner,
            inject_error_at: Some(inject_error_at),
            count: 0,
        }
    }
}

impl<S> Stream for ErrorInjectingStream<S>
where
    S: Stream + Unpin,
    S::Item: Timestamped,
{
    type Item = StreamItem<S::Item>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Check if we should inject an error at this position
        if let Some(error_pos) = self.inject_error_at {
            if self.count == error_pos {
                self.inject_error_at = None; // Only inject once
                self.count += 1;
                return Poll::Ready(Some(StreamItem::Error(FluxionError::stream_error(
                    "Injected test error",
                ))));
            }
        }

        // Otherwise, poll the inner stream and wrap in StreamItem::Value
        match Pin::new(&mut self.inner).poll_next(cx) {
            Poll::Ready(Some(item)) => {
                self.count += 1;
                Poll::Ready(Some(StreamItem::Value(item)))
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}
