use chrono::Duration;
use fluxion_stream::FluxionStream;
use fluxion_stream_time::{ChronoStreamOps, ChronoTimestamped};
use fluxion_test_utils::{
    helpers::{assert_no_element_emitted, unwrap_stream},
    test_channel,
    test_data::{person_alice, person_bob, person_charlie},
    TestData,
};
use tokio::{
    task::yield_now,
    time::{advance, pause},
};

#[tokio::test]
async fn test_delay_chaining_with_map_ordered() -> anyhow::Result<()> {
    pause(); // Mock time for instant test execution
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

    // Assert - Should NOT arrive immediately (advance 100ms)
    advance(std::time::Duration::from_millis(100)).await;
    yield_now().await; // Allow tasks to process
    assert_no_element_emitted(&mut processed, 100).await;

    // Assert - Advance remaining time, should arrive transformed
    advance(std::time::Duration::from_millis(900)).await;
    yield_now().await; // Allow tasks to process
    let result = unwrap_stream(&mut processed, 100).await.unwrap();
    assert_eq!(result.value, person_bob()); // Transformed

    // Act - Send Charlie
    tx.send(ChronoTimestamped::now(person_charlie()))?;

    // Assert - Should NOT arrive immediately (advance 100ms)
    advance(std::time::Duration::from_millis(100)).await;
    yield_now().await; // Allow tasks to process
    assert_no_element_emitted(&mut processed, 100).await;

    // Assert - Advance remaining time, should arrive unchanged
    advance(std::time::Duration::from_millis(900)).await;
    yield_now().await; // Allow tasks to process
    let result = unwrap_stream(&mut processed, 100).await.unwrap();
    assert_eq!(result.value, person_charlie()); // Unchanged

    Ok(())
}

#[tokio::test]
async fn test_delay_chaining_with_filter_ordered() -> anyhow::Result<()> {
    pause(); // Mock time for instant test execution

    // Arrange
    let (tx, stream) = test_channel::<ChronoTimestamped<TestData>>();
    let delay_duration = Duration::seconds(1);

    // Chain delay with filter_ordered - keep only Alice and Charlie
    let mut processed = FluxionStream::new(stream)
        .delay(delay_duration)
        .filter_ordered(|data: &_| *data == person_alice() || *data == person_charlie());

    // Act - Send Alice - should pass filter
    tx.send(ChronoTimestamped::now(person_alice()))?;

    // Assert - Should NOT arrive immediately (advance 100ms)
    advance(std::time::Duration::from_millis(100)).await;
    yield_now().await; // Allow tasks to process
    assert_no_element_emitted(&mut processed, 100).await;

    // Assert - Advance remaining time, should arrive
    advance(std::time::Duration::from_millis(900)).await;
    yield_now().await; // Allow tasks to process
    let result = unwrap_stream(&mut processed, 100).await.unwrap();
    assert_eq!(result.value, person_alice());

    // Act - Send Bob - should be filtered out (but still delayed)
    tx.send(ChronoTimestamped::now(person_bob()))?;

    // Act - Send Charlie - should pass filter
    tx.send(ChronoTimestamped::now(person_charlie()))?;

    // Assert - Should NOT arrive immediately (Bob is still delaying)
    advance(std::time::Duration::from_millis(100)).await;
    yield_now().await; // Allow tasks to process
    assert_no_element_emitted(&mut processed, 100).await;

    // Assert - Advance time for Bob's delay (900ms) + part of Charlie's (100ms)
    advance(std::time::Duration::from_millis(1000)).await;
    yield_now().await; // Allow tasks to process
    assert_no_element_emitted(&mut processed, 100).await;

    // Assert - Advance remaining time for Charlie, should arrive
    advance(std::time::Duration::from_millis(800)).await;
    yield_now().await; // Allow tasks to process
    let result = unwrap_stream(&mut processed, 100).await.unwrap();
    assert_eq!(result.value, person_charlie());

    Ok(())
}
