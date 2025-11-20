// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Test utilities for error injection in streams.
//!
//! This module provides stream wrappers that can inject `StreamItem::Error` values
//! into streams for testing error propagation behavior in stream operators.

use fluxion_core::{FluxionError, StreamItem, Timestamped};
use futures::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};

/// A stream wrapper that injects errors at specified positions.
///
/// This wrapper takes a stream that produces ordered values and wraps them in
/// `StreamItem::Value`, optionally injecting `StreamItem::Error` at a specified position.
///
/// # Examples
///
/// ```rust
/// use fluxion_test_utils::Timestamped;
/// use fluxion_test_utils::ErrorInjectingStream;
/// use fluxion_core::{StreamItem, Timestamped };
/// use futures::{stream, StreamExt};
///
/// # async fn example() {
/// let items = vec![
///     <Timestamped<i32> >::with_timestamp(1, 1),
///     <Timestamped<i32> >::with_timestamp(2, 2),
///     <Timestamped<i32> >::with_timestamp(3, 3),
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
    /// use fluxion_test_utils::{Timestamped, ErrorInjectingStream};
    /// use futures::stream;
    ///
    /// let items = vec![Timestamped::new(1), Timestamped::new(2)];
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
                return Poll::Ready(Some(StreamItem::Error(FluxionError::lock_error(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ChronoTimestamped;
    use fluxion_core::Timestamped;
    use futures::{stream, StreamExt};

    #[tokio::test]
    async fn test_error_injection_at_position() {
        let items = vec![
            <ChronoTimestamped<_>>::with_timestamp(1, 1_000_000_000),
            <ChronoTimestamped<_>>::with_timestamp(2, 2_000_000_000),
            <ChronoTimestamped<_>>::with_timestamp(3, 3_000_000_000),
        ];

        let base_stream = stream::iter(items);
        let mut error_stream = ErrorInjectingStream::new(base_stream, 1);

        // Position 0: value
        let first = error_stream.next().await.unwrap();
        assert!(matches!(first, StreamItem::Value(_)));

        // Position 1: injected error
        let second = error_stream.next().await.unwrap();
        assert!(matches!(second, StreamItem::Error(_)));

        // Position 2: value
        let third = error_stream.next().await.unwrap();
        assert!(matches!(third, StreamItem::Value(_)));
    }

    #[tokio::test]
    async fn test_error_injection_at_start() {
        let items = vec![<ChronoTimestamped<_>>::with_timestamp(1, 1_000_000_000)];
        let base_stream = stream::iter(items);
        let mut error_stream = ErrorInjectingStream::new(base_stream, 0);

        // First emission is the error
        let first = error_stream.next().await.unwrap();
        match first {
            StreamItem::Error(e) => {
                assert!(matches!(e, FluxionError::LockError { .. }));
            }
            StreamItem::Value(_) => panic!("Expected error at position 0"),
        }

        // Second emission is the value
        let second = error_stream.next().await.unwrap();
        assert!(matches!(second, StreamItem::Value(_)));
    }
}
