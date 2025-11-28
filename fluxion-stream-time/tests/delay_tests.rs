use chrono::Duration;
use fluxion_stream::FluxionStream;
use fluxion_stream_time::{ChronoStreamOps, ChronoTimestamped};
use fluxion_test_utils::{
    helpers::{assert_no_element_emitted, unwrap_stream},
    test_channel,
    test_data::{person_alice, person_bob},
    TestData,
};

#[tokio::test]
async fn test_delay_with_chrono_timestamped() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<ChronoTimestamped<TestData>>();
    let delay_duration = Duration::seconds(1);
    let mut delayed = FluxionStream::new(stream).delay(delay_duration);

    // Act - Send first value
    tx.send(ChronoTimestamped::now(person_alice()))?;

    // Assert - Should NOT arrive immediately
    assert_no_element_emitted(&mut delayed, 100).await;

    // Assert - Should arrive after delay (wait up to 1 second)
    let result = unwrap_stream(&mut delayed, 1000).await.unwrap();
    assert_eq!(result.value, person_alice());

    // Act - Send second value
    tx.send(ChronoTimestamped::now(person_bob()))?;

    // Assert - Should NOT arrive immediately
    assert_no_element_emitted(&mut delayed, 100).await;

    // Assert - Should arrive after delay (wait up to 1 second)
    let result = unwrap_stream(&mut delayed, 1000).await.unwrap();
    assert_eq!(result.value, person_bob());

    Ok(())
}
