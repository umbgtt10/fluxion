use chrono::Duration;
use fluxion_stream::FluxionStream;
use fluxion_stream_time::{ChronoStreamOps, ChronoTimestamped};
use fluxion_test_utils::{
    helpers::{assert_no_element_emitted, unwrap_stream},
    test_channel,
    test_data::{person_alice, person_bob, person_charlie},
    TestData,
};

#[tokio::test]
async fn test_delay_chaining_with_map_ordered() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<ChronoTimestamped<TestData>>();
    let delay_duration = Duration::seconds(1);

    // Chain delay with map_ordered - transform the data
    let mut processed = FluxionStream::new(stream)
        .delay(delay_duration)
        .map_ordered(|item: ChronoTimestamped<_>| {
            // Transform Alice to Bob
            let transformed = if item.value == person_alice() {
                person_bob()
            } else {
                item.value.clone()
            };
            ChronoTimestamped::new(transformed, item.timestamp)
        });

    // Act - Send Alice
    tx.send(ChronoTimestamped::now(person_alice()))?;

    // Assert - Should NOT arrive immediately
    assert_no_element_emitted(&mut processed, 100).await;

    // Assert - Should arrive after delay, transformed
    let result = unwrap_stream(&mut processed, 1000).await.unwrap();
    assert_eq!(result.value, person_bob()); // Transformed

    // Act - Send Charlie
    tx.send(ChronoTimestamped::now(person_charlie()))?;

    // Assert - Should NOT arrive immediately
    assert_no_element_emitted(&mut processed, 100).await;

    // Assert - Should arrive after delay, unchanged
    let result = unwrap_stream(&mut processed, 1000).await.unwrap();
    assert_eq!(result.value, person_charlie()); // Unchanged

    Ok(())
}

#[tokio::test]
async fn test_delay_chaining_with_filter_ordered() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<ChronoTimestamped<TestData>>();
    let delay_duration = Duration::seconds(1);

    // Chain delay with filter_ordered - keep only Alice and Charlie
    let mut processed = FluxionStream::new(stream)
        .delay(delay_duration)
        .filter_ordered(|data: &_| *data == person_alice() || *data == person_charlie());

    // Act - Send Alice - should pass filter
    tx.send(ChronoTimestamped::now(person_alice()))?;

    // Assert - Should NOT arrive immediately
    assert_no_element_emitted(&mut processed, 100).await;

    // Assert - Should arrive after delay
    let result = unwrap_stream(&mut processed, 1000).await.unwrap();
    assert_eq!(result.value, person_alice());

    // Act - Send Bob - should be filtered out (but still delayed)
    tx.send(ChronoTimestamped::now(person_bob()))?;

    // Act - Send Charlie - should pass filter
    tx.send(ChronoTimestamped::now(person_charlie()))?;

    // Assert - Should NOT arrive immediately (Bob is still delaying)
    assert_no_element_emitted(&mut processed, 100).await;

    // Assert - Charlie should arrive after Bob's delay + Charlie's delay
    let result = unwrap_stream(&mut processed, 2000).await.unwrap();
    assert_eq!(result.value, person_charlie());

    Ok(())
}
