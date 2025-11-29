use fluxion_core::{FluxionError, StreamItem};
use futures::Stream;
use pin_project::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::time::{sleep_until, Instant, Sleep};

/// Errors if the source stream does not emit any value within the specified duration.
///
/// The timeout operator monitors the time interval between emissions from the source stream.
/// If the source stream does not emit any value within the specified duration, the operator
/// emits a `FluxionError::TimeoutError` with "Timeout" context and terminates the stream.
///
/// - If the source emits a value, the timer is reset.
/// - If the source completes, the timeout operator completes.
/// - If the source errors, the error is passed through and the timer is reset (or stream ends depending on error handling).
///
/// # Example
///
/// ```rust
/// use fluxion_stream_time::timeout;
/// use fluxion_core::StreamItem;
/// use fluxion_test_utils::test_data::person_alice;
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
/// let mut timed_out = timeout(source, Duration::from_millis(100));
///
/// tx.send(person_alice()).unwrap();
///
/// let item = timed_out.next().await.unwrap().unwrap();
/// assert_eq!(item, person_alice());
/// # }
/// ```
pub fn timeout<S, T>(stream: S, duration: Duration) -> impl Stream<Item = StreamItem<T>>
where
    S: Stream<Item = StreamItem<T>>,
{
    TimeoutStream {
        stream,
        duration,
        sleep: Box::pin(sleep_until(Instant::now() + duration)),
        is_done: false,
    }
}

#[pin_project]
struct TimeoutStream<S> {
    #[pin]
    stream: S,
    duration: Duration,
    sleep: Pin<Box<Sleep>>,
    is_done: bool,
}

impl<S, T> Stream for TimeoutStream<S>
where
    S: Stream<Item = StreamItem<T>>,
{
    type Item = StreamItem<T>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();

        if *this.is_done {
            return Poll::Ready(None);
        }

        // 1. Poll the stream
        match this.stream.poll_next(cx) {
            Poll::Ready(Some(item)) => {
                // Reset the timer
                let next_deadline = Instant::now() + *this.duration;
                this.sleep.as_mut().reset(next_deadline);
                // We need to poll the sleep to register the new waker, but we can't do it easily here without potentially blocking?
                // Actually, `reset` just updates the deadline. We need to ensure the waker is registered.
                // However, since we are returning Ready, the consumer will likely poll us again soon?
                // But if the consumer processes the item and takes a long time, the timer should be running.
                // `sleep` is a future. If we don't poll it, it might not wake us up?
                // Tokio's `Sleep` needs to be polled to register the waker.
                // But we just got an item, so we are returning.
                // The next time `poll_next` is called, we will poll the sleep again.
                // This is fine because the timeout is "between emissions".
                // If we return an item, the "clock" for the next timeout starts now.
                // If `poll_next` isn't called for a while, that's backpressure, and usually timeout applies to "time waiting for upstream".
                // If downstream is slow, that's not an upstream timeout.
                return Poll::Ready(Some(item));
            }
            Poll::Ready(None) => {
                *this.is_done = true;
                return Poll::Ready(None);
            }
            Poll::Pending => {
                // Stream is pending, check the timer
            }
        }

        // 2. Poll the timer
        match this.sleep.as_mut().poll(cx) {
            Poll::Ready(_) => {
                // Timer fired, emit error and terminate
                *this.is_done = true;
                Poll::Ready(Some(StreamItem::Error(FluxionError::timeout_error(
                    "Timeout",
                ))))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
