use fluxion_core::StreamItem;
use futures::stream::FuturesUnordered;
use futures::{Stream, StreamExt};
use pin_project::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::time::{sleep, Sleep};

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
    T: Send + Sync + 'static,
{
    DelayStream {
        stream,
        duration,
        in_flight: FuturesUnordered::new(),
        upstream_done: false,
    }
}

#[pin_project]
struct DelayFuture<T> {
    #[pin]
    delay: Sleep,
    value: Option<T>,
}

impl<T> Future for DelayFuture<T> {
    type Output = StreamItem<T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.delay.poll(cx) {
            Poll::Ready(()) => {
                let value = this
                    .value
                    .take()
                    .expect("DelayFuture polled after completion");
                Poll::Ready(StreamItem::Value(value))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

#[pin_project]
struct DelayStream<S, T> {
    #[pin]
    stream: S,
    duration: chrono::Duration,
    in_flight: FuturesUnordered<DelayFuture<T>>,
    upstream_done: bool,
}

impl<S, T> Stream for DelayStream<S, T>
where
    S: Stream<Item = StreamItem<T>>,
    T: Send + Sync + 'static,
{
    type Item = StreamItem<T>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        // 1. Poll upstream for new items if not done
        if !*this.upstream_done {
            loop {
                match this.stream.as_mut().poll_next(cx) {
                    Poll::Ready(Some(StreamItem::Value(value))) => {
                        let std_duration = duration_to_std(this.duration);
                        let future = DelayFuture {
                            delay: sleep(std_duration),
                            value: Some(value),
                        };
                        this.in_flight.push(future);
                    }
                    Poll::Ready(Some(StreamItem::Error(err))) => {
                        // Errors pass through immediately without delay
                        return Poll::Ready(Some(StreamItem::Error(err)));
                    }
                    Poll::Ready(None) => {
                        *this.upstream_done = true;
                        break;
                    }
                    Poll::Pending => {
                        break;
                    }
                }
            }
        }

        // 2. Poll in_flight for completed delays
        match this.in_flight.poll_next_unpin(cx) {
            Poll::Ready(Some(item)) => Poll::Ready(Some(item)),
            Poll::Ready(None) => {
                if *this.upstream_done {
                    Poll::Ready(None)
                } else {
                    Poll::Pending
                }
            }
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
