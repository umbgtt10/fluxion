use fluxion_core::StreamItem;
use futures::Stream;
use pin_project::pin_project;
use tokio::time::{Sleep, sleep};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

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
/// use futures::stream;
/// use chrono::Duration;
///
/// # async fn example() {
/// let source = stream::iter(vec![StreamItem::Value(42)]);
/// let debounced = debounce(source, Duration::milliseconds(100));
/// # }
/// ```
pub fn debounce<S, T>(stream: S, duration: chrono::Duration) -> impl Stream<Item = StreamItem<T>>
where
    S: Stream<Item = StreamItem<T>>,
    T: Send,
{
    DebounceStream {
        stream,
        duration,
        pending: None,
        stream_ended: false,
    }
}

#[pin_project]
struct DebounceStream<S: Stream> {
    #[pin]
    stream: S,
    duration: chrono::Duration,
    pending: Option<(S::Item, Pin<Box<Sleep>>)>,
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
                if let Some((item, _)) = this.pending.take() {
                    return Poll::Ready(Some(item));
                }
                return Poll::Ready(None);
            }

            // Check if we have a pending debounced value and its timer
            if let Some((_, sleep)) = this.pending.as_mut() {
                match sleep.as_mut().poll(cx) {
                    Poll::Ready(_) => {
                        // Timer expired, emit the pending value
                        let result = this.pending.take().map(|(item, _)| item);
                        return Poll::Ready(result);
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
                    let std_duration = duration_to_std(this.duration);
                    let sleep = Box::pin(sleep(std_duration));

                    // Replace any pending value with this new one
                    *this.pending = Some((StreamItem::Value(value), sleep));

                    // Continue polling to check the timer
                    continue;
                }
                Poll::Ready(Some(StreamItem::Error(err))) => {
                    // Errors pass through immediately, discarding any pending value
                    *this.pending = None;
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

/// Convert chrono::Duration to std::time::Duration
fn duration_to_std(duration: &chrono::Duration) -> std::time::Duration {
    let millis = duration.num_milliseconds();
    if millis < 0 {
        std::time::Duration::from_millis(0)
    } else {
        std::time::Duration::from_millis(millis as u64)
    }
}
