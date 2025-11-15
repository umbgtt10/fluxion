// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_stream::combine_with_previous::CombineWithPreviousExt;
use fluxion_test_utils::sequenced::Sequenced;
use fluxion_test_utils::test_data::{person, person_alice, person_bob, person_charlie};
use futures::StreamExt;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

#[tokio::test]
async fn test_combine_with_previous_no_previous_value_emits() {
    // Arrange
    let (tx, rx) = mpsc::unbounded_channel();
    let mut stream = UnboundedReceiverStream::new(rx);
    let mut stream = stream.combine_with_previous();

    // Act
    tx.send(Sequenced::new(person_alice())).unwrap();

    // Assert
    let result = stream.next().await.unwrap();
    assert_eq!(
        (result.previous.map(|s| s.value), result.current.value),
        (None, person_alice())
    );
}

#[tokio::test]
async fn test_combine_with_previous_single_previous_value() {
    // Arrange
    let (tx, rx) = mpsc::unbounded_channel();
    let mut stream = UnboundedReceiverStream::new(rx);
    let mut stream = stream.combine_with_previous();

    // Act
    tx.send(Sequenced::new(person_alice())).unwrap();

    // Assert
    let first_result = stream.next().await.unwrap();
    assert_eq!(
        (
            first_result.previous.map(|s| s.value),
            first_result.current.value
        ),
        (None, person_alice())
    );

    // Act
    tx.send(Sequenced::new(person_bob())).unwrap();

    // Assert
    let second_result = stream.next().await.unwrap();
    assert_eq!(
        (
            second_result.previous.map(|s| s.value),
            second_result.current.value
        ),
        (Some(person_alice()), person_bob())
    );
}

#[tokio::test]
async fn test_combine_with_previous_multiple_values() {
    // Arrange
    let (tx, rx) = mpsc::unbounded_channel();
    let mut stream = UnboundedReceiverStream::new(rx);
    let mut stream = stream.combine_with_previous();

    // Act
    tx.send(Sequenced::new(person_alice())).unwrap();

    // Assert
    let first_result = stream.next().await.unwrap();
    assert_eq!(
        (
            first_result.previous.map(|s| s.value),
            first_result.current.value
        ),
        (None, person_alice())
    );

    // Act
    tx.send(Sequenced::new(person_bob())).unwrap();

    // Assert
    let second_result = stream.next().await.unwrap();
    assert_eq!(
        (
            second_result.previous.map(|s| s.value),
            second_result.current.value
        ),
        (Some(person_alice()), person_bob())
    );

    // Act
    tx.send(Sequenced::new(person_charlie())).unwrap();

    // Assert
    let third_result = stream.next().await.unwrap();
    assert_eq!(
        (
            third_result.previous.map(|s| s.value),
            third_result.current.value
        ),
        (Some(person_bob()), person_charlie())
    );
}

#[tokio::test]
async fn test_combine_with_previous_stream_ends() {
    // Arrange
    let (tx, rx) = mpsc::unbounded_channel();
    let mut stream = UnboundedReceiverStream::new(rx);
    let mut stream = stream.combine_with_previous();

    // Act
    tx.send(Sequenced::new(person_alice())).unwrap();

    // Assert
    let first_result = stream.next().await.unwrap();
    assert_eq!(
        (
            first_result.previous.map(|s| s.value),
            first_result.current.value
        ),
        (None, person_alice())
    );

    // Act
    tx.send(Sequenced::new(person_bob())).unwrap();

    // Assert
    let second_result = stream.next().await.unwrap();
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
}

#[tokio::test]
async fn test_combine_with_previous_for_types() {
    // Arrange
    let (tx, rx) = mpsc::unbounded_channel();
    let mut stream = UnboundedReceiverStream::new(rx);
    let mut stream = stream.combine_with_previous();

    // Act
    tx.send(Sequenced::new(person_alice())).unwrap();

    // Assert
    let first_result = stream.next().await.unwrap();
    assert_eq!(
        (
            first_result.previous.map(|s| s.value),
            first_result.current.value
        ),
        (None, person_alice())
    );

    // Act
    tx.send(Sequenced::new(person_bob())).unwrap();

    // Assert
    let second_result = stream.next().await.unwrap();
    assert_eq!(
        (
            second_result.previous.map(|s| s.value),
            second_result.current.value
        ),
        (Some(person_alice()), person_bob())
    );
}

#[tokio::test]
async fn test_combine_with_previous_high_volume_sequential() {
    // Arrange
    let (tx, rx) = mpsc::unbounded_channel();
    let mut stream = UnboundedReceiverStream::new(rx);
    let mut stream = stream.combine_with_previous();

    // Act: send a sequence of 200 items (mix of known fixtures cycling Alice,Bob,Charlie)
    for i in 0..200 {
        match i % 3 {
            0 => tx.send(Sequenced::new(person_alice())).unwrap(),
            1 => tx.send(Sequenced::new(person_bob())).unwrap(),
            _ => tx.send(Sequenced::new(person_charlie())).unwrap(),
        }
    }
    // Close input so the stream can finish
    drop(tx);

    // Assert: first has no previous
    let first = stream.next().await.unwrap();
    assert_eq!(
        (first.previous.map(|s| s.value), first.current.value),
        (None, person_alice())
    );

    // Then verify a few samples and the last pair to ensure previous tracking holds
    let second = stream.next().await.unwrap();
    assert_eq!(
        (second.previous.map(|s| s.value), second.current.value),
        (Some(person_alice()), person_bob())
    );

    // Drain remaining 198 pairs (we already consumed 2 above)
    let mut last_prev = None;
    let mut last_curr = None;
    for _ in 0..198 {
        let item = stream.next().await.unwrap();
        last_prev = item.previous.map(|s| s.value);
        last_curr = Some(item.current.value);
    }
    // Stream should now be ended
    assert!(stream.next().await.is_none());
    // Last pair should have a previous
    assert!(last_prev.is_some());
    assert!(last_curr.is_some());
}

#[tokio::test]
async fn test_combine_with_previous_boundary_empty_string_zero_values() {
    // Arrange: Test with boundary values (empty strings, zero numeric values)
    let (tx, rx) = mpsc::unbounded_channel();
    let mut stream = UnboundedReceiverStream::new(rx);
    let mut stream = stream.combine_with_previous();

    // Act: Send empty string with zero value
    tx.send(Sequenced::new(person(String::new(), 0))).unwrap();

    // Assert: First emission has no previous
    let result = stream.next().await.unwrap();
    assert_eq!(
        (result.previous.map(|s| s.value), result.current.value),
        (None, person(String::new(), 0))
    );

    // Act: Send another boundary value
    tx.send(Sequenced::new(person(String::new(), 0))).unwrap();

    // Assert: Second emission has previous with boundary value
    let result2 = stream.next().await.unwrap();
    assert_eq!(
        (result2.previous.map(|s| s.value), result2.current.value),
        (Some(person(String::new(), 0)), person(String::new(), 0))
    );

    // Act: Transition to normal value
    tx.send(Sequenced::new(person_alice())).unwrap();

    // Assert: Should track boundary as previous
    let result3 = stream.next().await.unwrap();
    assert_eq!(
        (result3.previous.map(|s| s.value), result3.current.value),
        (Some(person(String::new(), 0)), person_alice())
    );
}

#[tokio::test]
async fn test_combine_with_previous_boundary_maximum_concurrent_streams() {
    // Arrange: Test concurrent handling with many parallel streams
    let num_concurrent = 50;
    let mut handles = Vec::new();

    for i in 0..num_concurrent {
        let handle = tokio::spawn(async move {
            let (tx, rx) = mpsc::unbounded_channel();
            let mut stream = UnboundedReceiverStream::new(rx);
            let mut stream = stream.combine_with_previous();

            // Act: Send first value
            tx.send(Sequenced::new(person(format!("Person{i}"), i)))
                .unwrap();

            // Assert: No previous
            let result = stream.next().await.unwrap();
            assert_eq!(
                (result.previous.map(|s| s.value), result.current.value),
                (None, person(format!("Person{i}"), i))
            );

            // Act: Send second value
            tx.send(Sequenced::new(person_alice())).unwrap();

            // Assert: Has previous
            let result2 = stream.next().await.unwrap();
            assert_eq!(
                (result2.previous.map(|s| s.value), result2.current.value),
                (Some(person(format!("Person{i}"), i)), person_alice())
            );

            // Act: Send third value
            tx.send(Sequenced::new(person_bob())).unwrap();

            // Assert: Previous is Alice
            let result3 = stream.next().await.unwrap();
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
}

#[tokio::test]
async fn test_combine_with_previous_single_value_stream() {
    // Arrange: Stream that emits only one value
    let (tx, rx) = mpsc::unbounded_channel();
    let mut stream = UnboundedReceiverStream::new(rx);
    let mut stream = stream.combine_with_previous();

    // Act: Send single value and close
    tx.send(Sequenced::new(person_alice())).unwrap();
    drop(tx);

    // Assert: Should emit with no previous
    let result = stream.next().await.unwrap();
    assert_eq!(
        (result.previous.map(|s| s.value), result.current.value),
        (None, person_alice())
    );

    // Assert: Stream ends
    assert!(stream.next().await.is_none());
}
