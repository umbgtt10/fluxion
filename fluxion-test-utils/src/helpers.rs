use futures::Stream;
use futures::stream::StreamExt;
use std::fmt::Debug;
use std::time::Duration;
use tokio::time::sleep;

pub async fn assert_no_element_emitted<S, T>(stream: &mut S, timeout_ms: u64)
where
    S: Stream<Item = T> + Unpin,
    T: Debug,
{
    tokio::select! {
        state = stream.next() => {
            panic!(
                "Unexpected combination emitted: {:?}, expected no output.",
                state
            );
        }
        _ = sleep(Duration::from_millis(timeout_ms)) => {
        }
    }
}
