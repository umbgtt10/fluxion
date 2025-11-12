use crate::test_data::TestData;
use fluxion_stream::combine_latest::CombinedState;
use fluxion_stream::timestamped::Timestamped;
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

pub async fn expect_next_value<S>(stream: &mut S, expected: TestData)
where
    S: Stream<Item = TestData> + Unpin,
{
    let item = stream.next().await.expect("expected next item");
    assert_eq!(item, expected);
}

pub async fn expect_next_timestamped<S>(stream: &mut S, expected: TestData)
where
    S: Stream<Item = Timestamped<TestData>> + Unpin,
{
    let item = stream.next().await.expect("expected next item");
    assert_eq!(item.value, expected);
}

/// Expect the next pair from a with_latest_from stream matches expected left/right by value.
pub async fn expect_next_pair<S>(stream: &mut S, expected_left: TestData, expected_right: TestData)
where
    S: Stream<Item = (Timestamped<TestData>, Timestamped<TestData>)> + Unpin,
{
    let (left, right) = stream.next().await.expect("expected next pair");
    assert_eq!((left.value, right.value), (expected_left, expected_right));
}

pub async fn expect_next_combined_equals<S>(stream: &mut S, expected: &[TestData])
where
    S: Stream<Item = CombinedState<Timestamped<TestData>>> + Unpin,
{
    let state = stream.next().await.expect("expected next combined state");
    let actual: Vec<TestData> = state.get_state().iter().map(|s| s.value.clone()).collect();
    assert_eq!(actual, expected);
}
