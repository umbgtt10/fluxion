// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::StreamItem;
use futures::Stream;
use pin_project::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::time::{sleep, Instant, Sleep};

/// Debounces values from the source stream by the specified duration.
///
/// The debounce operator waits for a pause in the input stream of at least
/// the given duration before emitting the latest value. If a new value
/// arrives before the duration elapses, the timer is reset and only the
/// newest value is eventually emitted.
///
/// This implements **trailing debounce** semantics (Rx standard):
/// - When a value arrives, start/restart the timer
/// - If no new value arrives before the timer expires, emit the latest value
/// - If a new value arrives, discard the pending value and restart the timer
/// - When the stream ends, emit any pending value immediately
///
/// Errors pass through immediately without debounce, to ensure timely
/// error propagation.
///
/// # Example
///
/// ```rust
/// use fluxion_stream_time::debounce;
/// use fluxion_core::StreamItem;
/// use fluxion_test_utils::test_data::{person_alice, person_bob};
/// use futures::stream::StreamExt;
/// use std::time::Duration;
/// use tokio::sync::mpsc;
/// use tokio_stream::wrappers::UnboundedReceiverStream;
///
/// # #[tokio::main]
/// # async fn main() {
/// let (tx, rx) = mpsc::unbounded_channel();
/// let source = UnboundedReceiverStream::new(rx).map(StreamItem::Value);
///
/// let mut debounced = debounce(source, Duration::from_millis(100));
///
/// // Alice and Bob emitted immediately. Alice should be debounced (dropped).
/// tx.send(person_alice()).unwrap();
/// tx.send(person_bob()).unwrap();
///
/// // Only Bob should remain (trailing debounce)
/// let item = debounced.next().await.unwrap().unwrap();
/// assert_eq!(item, person_bob());
/// # }
/// ```
pub fn debounce<S, T>(stream: S, duration: std::time::Duration) -> impl Stream<Item = StreamItem<T>>
where
    S: Stream<Item = StreamItem<T>>,
    T: Send,
{
    DebounceStream {
        stream,
        duration,
        sleep: Box::pin(sleep(Duration::from_millis(0))), // Initial dummy sleep
        pending_value: None,
        stream_ended: false,
    }
}

#[pin_project]
struct DebounceStream<S: Stream> {
    #[pin]
    stream: S,
    duration: std::time::Duration,
    sleep: Pin<Box<Sleep>>,
    pending_value: Option<S::Item>,
    stream_ended: bool,
}

impl<S, T> Stream for DebounceStream<S>
where
    S: Stream<Item = StreamItem<T>>,
    T: Send,
{
    type Item = StreamItem<T>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        loop {
            // If stream ended and we have a pending value, emit it immediately
            if *this.stream_ended {
                if let Some(item) = this.pending_value.take() {
                    return Poll::Ready(Some(item));
                }
                return Poll::Ready(None);
            }

            // Check if we have a pending debounced value and its timer
            if this.pending_value.is_some() {
                match this.sleep.as_mut().poll(cx) {
                    Poll::Ready(_) => {
                        // Timer expired, emit the pending value
                        let item = this.pending_value.take();
                        return Poll::Ready(item);
                    }
                    Poll::Pending => {
                        // Timer still running, check for new values
                    }
                }
            }

            // Poll the source stream for the next item
            match this.stream.as_mut().poll_next(cx) {
                Poll::Ready(Some(StreamItem::Value(value))) => {
                    // New value arrived - reset the debounce timer
                    let deadline = Instant::now() + *this.duration;
                    this.sleep.as_mut().reset(deadline);

                    // Replace any pending value with this new one
                    *this.pending_value = Some(StreamItem::Value(value));

                    // Continue polling to check the timer (it might be 0 duration)
                    continue;
                }
                Poll::Ready(Some(StreamItem::Error(err))) => {
                    // Errors pass through immediately, discarding any pending value
                    *this.pending_value = None;
                    return Poll::Ready(Some(StreamItem::Error(err)));
                }
                Poll::Ready(None) => {
                    // Stream ended - mark it and loop to emit pending value if any
                    *this.stream_ended = true;
                    continue;
                }
                Poll::Pending => {
                    // No new values from source
                    // If we have a pending value, we're waiting for its timer
                    // Otherwise, we're waiting for the next source value
                    return Poll::Pending;
                }
            }
        }
    }
}
