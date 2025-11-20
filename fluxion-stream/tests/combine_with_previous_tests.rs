// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_stream::combine_with_previous::CombineWithPreviousExt;
use fluxion_test_utils::helpers::unwrap_stream;
use fluxion_test_utils::test_data::{person, person_alice, person_bob, person_charlie};
use fluxion_test_utils::ChronoTimestamped;
use fluxion_test_utils::{assert_stream_ended, test_channel};
use futures::StreamExt;

#[tokio::test]
async fn test_combine_with_previous_no_previous_value_emits() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel();
    let mut stream = stream.combine_with_previous();

    // Act
    tx.send(ChronoTimestamped::new(person_alice()))?;

    // Assert
    let result = unwrap_stream(&mut stream, 500).await.unwrap();
    assert_eq!(
        (result.previous.map(|s| s.value), result.current.value),
        (None, person_alice())
    );

    Ok(())
}

#[tokio::test]
async fn test_combine_with_previous_single_previous_value() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel();
    let mut stream = stream.combine_with_previous();

    // Act
    tx.send(ChronoTimestamped::new(person_alice()))?;

    // Assert
    let first_result = unwrap_stream(&mut stream, 500).await.unwrap();
    assert_eq!(
        (
            first_result.previous.map(|s| s.value),
            first_result.current.value
        ),
        (None, person_alice())
    );

    // Act
    tx.send(ChronoTimestamped::new(person_bob()))?;

    // Assert
    let second_result = unwrap_stream(&mut stream, 500).await.unwrap();
    assert_eq!(
        (
            second_result.previous.map(|s| s.value),
            second_result.current.value
        ),
        (Some(person_alice()), person_bob())
    );

    Ok(())
}

#[tokio::test]
async fn test_combine_with_previous_multiple_values() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel();
    let mut stream = stream.combine_with_previous();

    // Act
    tx.send(ChronoTimestamped::new(person_alice()))?;

    // Assert
    let first_result = unwrap_stream(&mut stream, 500).await.unwrap();
    assert_eq!(
        (
            first_result.previous.map(|s| s.value),
            first_result.current.value
        ),
        (None, person_alice())
    );

    // Act
    tx.send(ChronoTimestamped::new(person_bob()))?;

    // Assert
    let second_result = unwrap_stream(&mut stream, 500).await.unwrap();
    assert_eq!(
        (
            second_result.previous.map(|s| s.value),
            second_result.current.value
        ),
        (Some(person_alice()), person_bob())
    );

    // Act
    tx.send(ChronoTimestamped::new(person_charlie()))?;

    // Assert
    let third_result = unwrap_stream(&mut stream, 500).await.unwrap();
    assert_eq!(
        (
            third_result.previous.map(|s| s.value),
            third_result.current.value
        ),
        (Some(person_bob()), person_charlie())
    );

    Ok(())
}

#[tokio::test]
async fn test_combine_with_previous_stream_ends() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel();
    let mut stream = stream.combine_with_previous();

    // Act
    tx.send(ChronoTimestamped::new(person_alice()))?;

    // Assert
    let first_result = unwrap_stream(&mut stream, 500).await.unwrap();
    assert_eq!(
        (
            first_result.previous.map(|s| s.value),
            first_result.current.value
        ),
        (None, person_alice())
    );

    // Act
    tx.send(ChronoTimestamped::new(person_bob()))?;

    // Assert
    let second_result = unwrap_stream(&mut stream, 500).await.unwrap();
    assert_eq!(
        (
            second_result.previous.map(|s| s.value),
            second_result.current.value
        ),
        (Some(person_alice()), person_bob())
    );

    // Act
    drop(tx);

    // Assert
    let third_result = stream.next().await;
    assert!(third_result.is_none());

    Ok(())
}

#[tokio::test]
async fn test_combine_with_previous_for_types() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel();
    let mut stream = stream.combine_with_previous();

    // Act
    tx.send(ChronoTimestamped::new(person_alice()))?;

    // Assert
    let first_result = unwrap_stream(&mut stream, 500).await.unwrap();
    assert_eq!(
        (
            first_result.previous.map(|s| s.value),
            first_result.current.value
        ),
        (None, person_alice())
    );

    // Act
    tx.send(ChronoTimestamped::new(person_bob()))?;

    // Assert
    let second_result = unwrap_stream(&mut stream, 500).await.unwrap();
    assert_eq!(
        (
            second_result.previous.map(|s| s.value),
            second_result.current.value
        ),
        (Some(person_alice()), person_bob())
    );

    Ok(())
}

#[tokio::test]
async fn test_combine_with_previous_high_volume_sequential() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel();
    let mut stream = stream.combine_with_previous();

    // Act: send a sequence of 200 items (mix of known fixtures cycling Alice,Bob,Charlie)
    for i in 0..200 {
        match i % 3 {
            0 => tx.send(ChronoTimestamped::new(person_alice()))?,
            1 => tx.send(ChronoTimestamped::new(person_bob()))?,
            _ => tx.send(ChronoTimestamped::new(person_charlie()))?,
        }
    }
    // Close input so the stream can finish
    drop(tx);

    // Assert: first has no previous
    let first = unwrap_stream(&mut stream, 500).await.unwrap();
    assert_eq!(
        (first.previous.map(|s| s.value), first.current.value),
        (None, person_alice())
    );

    // Then verify a few samples and the last pair to ensure previous tracking holds
    let second = unwrap_stream(&mut stream, 500).await.unwrap();
    assert_eq!(
        (second.previous.map(|s| s.value), second.current.value),
        (Some(person_alice()), person_bob())
    );

    // Drain remaining 198 pairs (we already consumed 2 above)
    let mut last_prev = None;
    let mut last_curr = None;
    for _ in 0..198 {
        let item = unwrap_stream(&mut stream, 500).await.unwrap();
        last_prev = item.previous.map(|s| s.value);
        last_curr = Some(item.current.value);
    }
    // Stream should now be ended
    assert_stream_ended(&mut stream, 500).await;
    // Last pair should have a previous
    assert!(last_prev.is_some());
    assert!(last_curr.is_some());

    Ok(())
}

#[tokio::test]
async fn test_combine_with_previous_boundary_empty_string_zero_values() -> anyhow::Result<()> {
    // Arrange: Test with boundary values (empty strings, zero numeric values)
    let (tx, stream) = test_channel();
    let mut stream = stream.combine_with_previous();

    // Act: Send empty string with zero value
    tx.send(ChronoTimestamped::new(person(String::new(), 0)))?;

    // Assert: First emission has no previous
    let result = unwrap_stream(&mut stream, 500).await.unwrap();
    assert_eq!(
        (result.previous.map(|s| s.value), result.current.value),
        (None, person(String::new(), 0))
    );

    // Act: Send another boundary value
    tx.send(ChronoTimestamped::new(person(String::new(), 0)))?;

    // Assert: Second emission has previous with boundary value
    let result2 = unwrap_stream(&mut stream, 500).await.unwrap();
    assert_eq!(
        (result2.previous.map(|s| s.value), result2.current.value),
        (Some(person(String::new(), 0)), person(String::new(), 0))
    );

    // Act: Transition to normal value
    tx.send(ChronoTimestamped::new(person_alice()))?;

    // Assert: Should track boundary as previous
    let result3 = unwrap_stream(&mut stream, 500).await.unwrap();
    assert_eq!(
        (result3.previous.map(|s| s.value), result3.current.value),
        (Some(person(String::new(), 0)), person_alice())
    );

    Ok(())
}

#[tokio::test]
async fn test_combine_with_previous_boundary_maximum_concurrent_streams() -> anyhow::Result<()> {
    // Arrange: Test concurrent handling with many parallel streams
    let num_concurrent = 50;
    let mut handles = Vec::new();

    for i in 0..num_concurrent {
        let handle = tokio::spawn(async move {
            let (tx, stream) = test_channel();
            let mut stream = stream.combine_with_previous();

            // Act: Send first value
            tx.send(ChronoTimestamped::new(person(format!("Person{i}"), i)))
                .unwrap();

            // Assert: No previous
            let result = unwrap_stream(&mut stream, 500).await.unwrap();
            assert_eq!(
                (result.previous.map(|s| s.value), result.current.value),
                (None, person(format!("Person{i}"), i))
            );

            // Act: Send second value
            tx.send(ChronoTimestamped::new(person_alice())).unwrap();

            // Assert: Has previous
            let result2 = unwrap_stream(&mut stream, 500).await.unwrap();
            assert_eq!(
                (result2.previous.map(|s| s.value), result2.current.value),
                (Some(person(format!("Person{i}"), i)), person_alice())
            );

            // Act: Send third value
            tx.send(ChronoTimestamped::new(person_bob())).unwrap();

            // Assert: Previous is Alice
            let result3 = unwrap_stream(&mut stream, 500).await.unwrap();
            assert_eq!(
                (result3.previous.map(|s| s.value), result3.current.value),
                (Some(person_alice()), person_bob())
            );
        });

        handles.push(handle);
    }

    // Wait for all concurrent streams to complete
    for handle in handles {
        handle
            .await
            .expect("Concurrent stream task should complete successfully");
    }

    Ok(())
}

#[tokio::test]
async fn test_combine_with_previous_single_value_stream() -> anyhow::Result<()> {
    // Arrange: Stream that emits only one value
    let (tx, stream) = test_channel();
    let mut stream = stream.combine_with_previous();

    // Act: Send single value and close
    tx.send(ChronoTimestamped::new(person_alice()))?;
    drop(tx);

    // Assert: Should emit with no previous
    let result = unwrap_stream(&mut stream, 500).await.unwrap();
    assert_eq!(
        (result.previous.map(|s| s.value), result.current.value),
        (None, person_alice())
    );

    // Assert: Stream ends
    assert_stream_ended(&mut stream, 500).await;

    Ok(())
}
