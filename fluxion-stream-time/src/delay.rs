use fluxion_core::StreamItem;
use futures::Stream;
use pin_project::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Delays each emission from the source stream by the specified duration.
///
/// Each item is delayed independently - the delay is applied to each item
/// as it arrives. Errors are passed through without delay to ensure timely
/// error propagation.
///
/// # Example
///
/// ```rust
/// use fluxion_stream_time::delay;
/// use fluxion_core::StreamItem;
/// use futures::stream;
/// use chrono::Duration;
///
/// # async fn example() {
/// let source = stream::iter(vec![StreamItem::Value(42)]);
/// let delayed = delay(source, Duration::milliseconds(100));
/// # }
/// ```
pub fn delay<S, T>(stream: S, duration: chrono::Duration) -> impl Stream<Item = StreamItem<T>>
where
    S: Stream<Item = StreamItem<T>>,
    T: Send,
{
    DelayStream {
        stream,
        duration,
        pending: None,
    }
}

#[pin_project]
struct DelayStream<S: Stream> {
    #[pin]
    stream: S,
    duration: chrono::Duration,
    pending: Option<(S::Item, Pin<Box<tokio::time::Sleep>>)>,
}

impl<S, T> Stream for DelayStream<S>
where
    S: Stream<Item = StreamItem<T>>,
    T: Send,
{
    type Item = StreamItem<T>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();

        // Check if we have a pending delayed item
        if let Some((_item, sleep)) = this.pending {
            match sleep.as_mut().poll(cx) {
                Poll::Ready(_) => {
                    // Delay completed, emit the item
                    let result = this.pending.take().map(|(item, _)| item);
                    return Poll::Ready(result);
                }
                Poll::Pending => {
                    // Still waiting
                    return Poll::Pending;
                }
            }
        }

        // Poll the source stream for the next item
        match this.stream.poll_next(cx) {
            Poll::Ready(Some(StreamItem::Value(value))) => {
                // Convert chrono::Duration to std::time::Duration
                let std_duration = duration_to_std(this.duration);
                let sleep = Box::pin(tokio::time::sleep(std_duration));

                // Store the item and sleep future
                *this.pending = Some((StreamItem::Value(value), sleep));

                // Wake the task to poll the sleep future
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            Poll::Ready(Some(StreamItem::Error(err))) => {
                // Errors pass through immediately without delay
                Poll::Ready(Some(StreamItem::Error(err)))
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
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
