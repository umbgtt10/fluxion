use fluxion_core::StreamItem;
use futures::Stream;
use pin_project::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::time::{sleep, Instant, Sleep};

/// Throttles values from the source stream by the specified duration.
///
/// The throttle operator emits the first value, then ignores subsequent values
/// for the specified duration. After the duration expires, it accepts the next
/// value and repeats the process.
///
/// This implements **leading throttle** semantics:
/// - When a value arrives and we are not throttling:
///   - Emit the value immediately
///   - Start the throttle timer
///   - Ignore subsequent values until the timer expires
/// - When the timer expires:
///   - We become ready to accept a new value
///
/// Errors pass through immediately without throttling, to ensure timely
/// error propagation.
///
/// # Example
///
/// ```rust
/// use fluxion_stream_time::throttle;
/// use fluxion_core::StreamItem;
/// use futures::stream;
/// use std::time::Duration;
///
/// # async fn example() {
/// let source = stream::iter(vec![StreamItem::Value(42)]);
/// let throttled = throttle(source, Duration::from_millis(100));
/// # }
/// ```
pub fn throttle<S, T>(stream: S, duration: std::time::Duration) -> impl Stream<Item = StreamItem<T>>
where
    S: Stream<Item = StreamItem<T>>,
    T: Send,
{
    ThrottleStream {
        stream,
        duration,
        sleep: Box::pin(sleep(std::time::Duration::from_millis(0))), // Initial dummy sleep
        throttling: false,
    }
}

#[pin_project]
struct ThrottleStream<S: Stream> {
    #[pin]
    stream: S,
    duration: std::time::Duration,
    sleep: Pin<Box<Sleep>>,
    throttling: bool,
}

impl<S, T> Stream for ThrottleStream<S>
where
    S: Stream<Item = StreamItem<T>>,
    T: Send,
{
    type Item = StreamItem<T>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        loop {
            // 1. Check timer if throttling
            if *this.throttling {
                // Check if deadline has passed explicitly to handle cases where Sleep poll lags
                if Instant::now() >= this.sleep.deadline() {
                    *this.throttling = false;
                } else {
                    match this.sleep.as_mut().poll(cx) {
                        Poll::Ready(_) => {
                            *this.throttling = false;
                            // Timer expired, we are open to new values.
                        }
                        Poll::Pending => {
                            // Timer running.
                        }
                    }
                }
            }

            // 2. Poll stream
            match this.stream.as_mut().poll_next(cx) {
                Poll::Ready(Some(StreamItem::Value(value))) => {
                    if !*this.throttling {
                        // Not throttling: Emit value, start timer
                        let deadline = Instant::now() + *this.duration;
                        this.sleep.as_mut().reset(deadline);
                        *this.throttling = true;
                        return Poll::Ready(Some(StreamItem::Value(value)));
                    } else {
                        // Throttling: Drop value, continue polling stream
                        continue;
                    }
                }
                Poll::Ready(Some(StreamItem::Error(err))) => {
                    // Errors pass through immediately
                    return Poll::Ready(Some(StreamItem::Error(err)));
                }
                Poll::Ready(None) => {
                    return Poll::Ready(None);
                }
                Poll::Pending => {
                    // If we are throttling, we need to ensure we are woken up by the timer.
                    // this.sleep.poll(cx) above already registered the waker if it was Pending.
                    // this.stream.poll_next(cx) registered the waker if it was Pending.
                    return Poll::Pending;
                }
            }
        }
    }
}
