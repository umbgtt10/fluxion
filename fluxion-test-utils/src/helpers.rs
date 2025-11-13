use crate::sequenced::Sequenced;
use crate::test_data::TestData;
use futures::Stream;
use futures::stream::StreamExt;
use std::time::Duration;
use tokio::time::sleep;

pub async fn assert_no_element_emitted<S, T>(stream: &mut S, timeout_ms: u64)
where
    S: Stream<Item = T> + Unpin,
{
    tokio::select! {
        _state = stream.next() => {
            panic!(
                "Unexpected combination emitted, expected no output."
            );
        }
        _ = sleep(Duration::from_millis(timeout_ms)) => {
        }
    }
}

pub async fn expect_next_value<S>(stream: &mut S, expected: TestData)
where
    S: Stream<Item = TestData> + Unpin,
{
    let item = stream.next().await.expect("expected next item");
    assert_eq!(item, expected);
}

pub async fn expect_next_timestamped<S>(stream: &mut S, expected: TestData)
where
    S: Stream<Item = Sequenced<TestData>> + Unpin,
{
    let item = stream.next().await.expect("expected next item");
    assert_eq!(item.value, expected);
}

/// Expect the next pair from a with_latest_from stream matches expected left/right by value.
pub async fn expect_next_pair<S>(stream: &mut S, expected_left: TestData, expected_right: TestData)
where
    S: Stream<Item = (Sequenced<TestData>, Sequenced<TestData>)> + Unpin,
{
    let (left, right) = stream.next().await.expect("expected next pair");
    assert_eq!((left.value, right.value), (expected_left, expected_right));
}
