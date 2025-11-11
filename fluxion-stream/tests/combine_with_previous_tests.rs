use fluxion_stream::FluxionChannel;
use fluxion_stream::combine_with_previous::CombineWithPreviousExt;
use fluxion_test_utils::push;
use fluxion_test_utils::test_data::{person, person_alice, person_bob, person_charlie};
use futures::StreamExt;

#[tokio::test]
async fn test_combine_with_previous_no_previous_value_emits() {
    // Arrange
    let channel = FluxionChannel::new();
    let mut stream = channel.stream.combine_with_previous();

    // Act
    push(person_alice(), &channel.sender);

    // Assert
    let result = stream.next().await.unwrap();
    assert_eq!(
        (result.0.map(|s| s.value), result.1.value),
        (None, person_alice())
    );
}

#[tokio::test]
async fn test_combine_with_previous_single_previous_value() {
    // Arrange
    let channel = FluxionChannel::new();
    let mut stream = channel.stream.combine_with_previous();

    // Act
    push(person_alice(), &channel.sender);

    // Assert
    let first_result = stream.next().await.unwrap();
    assert_eq!(
        (first_result.0.map(|s| s.value), first_result.1.value),
        (None, person_alice())
    );

    // Act
    push(person_bob(), &channel.sender);

    // Assert
    let second_result = stream.next().await.unwrap();
    assert_eq!(
        (second_result.0.map(|s| s.value), second_result.1.value),
        (Some(person_alice()), person_bob())
    );
}

#[tokio::test]
async fn test_combine_with_previous_multiple_values() {
    // Arrange
    let channel = FluxionChannel::new();
    let mut stream = channel.stream.combine_with_previous();

    // Act
    push(person_alice(), &channel.sender);

    // Assert
    let first_result = stream.next().await.unwrap();
    assert_eq!(
        (first_result.0.map(|s| s.value), first_result.1.value),
        (None, person_alice())
    );

    // Act
    push(person_bob(), &channel.sender);

    // Assert
    let second_result = stream.next().await.unwrap();
    assert_eq!(
        (second_result.0.map(|s| s.value), second_result.1.value),
        (Some(person_alice()), person_bob())
    );

    // Act
    push(person_charlie(), &channel.sender);

    // Assert
    let third_result = stream.next().await.unwrap();
    assert_eq!(
        (third_result.0.map(|s| s.value), third_result.1.value),
        (Some(person_bob()), person_charlie())
    );
}

#[tokio::test]
async fn test_combine_with_previous_stream_ends() {
    // Arrange
    let channel = FluxionChannel::new();
    let mut stream = channel.stream.combine_with_previous();

    // Act
    push(person_alice(), &channel.sender);

    // Assert
    let first_result = stream.next().await.unwrap();
    assert_eq!(
        (first_result.0.map(|s| s.value), first_result.1.value),
        (None, person_alice())
    );

    // Act
    push(person_bob(), &channel.sender);

    // Assert
    let second_result = stream.next().await.unwrap();
    assert_eq!(
        (second_result.0.map(|s| s.value), second_result.1.value),
        (Some(person_alice()), person_bob())
    );

    // Act
    drop(channel.sender);

    // Assert
    let third_result = stream.next().await;
    assert!(third_result.is_none());
}

#[tokio::test]
async fn test_combine_with_previous_for_types() {
    // Arrange
    let channel = FluxionChannel::new();
    let mut stream = channel.stream.combine_with_previous();

    // Act
    push(person_alice(), &channel.sender);

    // Assert
    let first_result = stream.next().await.unwrap();
    assert_eq!(
        (first_result.0.map(|s| s.value), first_result.1.value),
        (None, person_alice())
    );

    // Act
    push(person_bob(), &channel.sender);

    // Assert
    let second_result = stream.next().await.unwrap();
    assert_eq!(
        (second_result.0.map(|s| s.value), second_result.1.value),
        (Some(person_alice()), person_bob())
    );
}

#[tokio::test]
async fn test_combine_with_previous_high_volume_sequential() {
    // Arrange
    let channel = FluxionChannel::new();
    let mut stream = channel.stream.combine_with_previous();

    // Act: send a sequence of 200 items (mix of known fixtures cycling Alice,Bob,Charlie)
    for i in 0..200 {
        match i % 3 {
            0 => push(person_alice(), &channel.sender),
            1 => push(person_bob(), &channel.sender),
            _ => push(person_charlie(), &channel.sender),
        }
    }
    // Close input so the stream can finish
    drop(channel.sender);

    // Assert: first has no previous
    let first = stream.next().await.unwrap();
    assert_eq!(
        (first.0.map(|s| s.value), first.1.value),
        (None, person_alice())
    );

    // Then verify a few samples and the last pair to ensure previous tracking holds
    let second = stream.next().await.unwrap();
    assert_eq!(
        (second.0.map(|s| s.value), second.1.value),
        (Some(person_alice()), person_bob())
    );

    // Drain remaining 198 pairs (we already consumed 2 above)
    let mut last_prev = None;
    let mut last_curr = None;
    for _ in 0..198 {
        let (prev, curr) = stream.next().await.unwrap();
        last_prev = prev.map(|s| s.value);
        last_curr = Some(curr.value);
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
    let channel = FluxionChannel::new();
    let mut stream = channel.stream.combine_with_previous();

    // Act: Send empty string with zero value
    push(person("".to_string(), 0), &channel.sender);

    // Assert: First emission has no previous
    let result = stream.next().await.unwrap();
    assert_eq!(
        (result.0.map(|s| s.value), result.1.value),
        (None, person("".to_string(), 0))
    );

    // Act: Send another boundary value
    push(person("".to_string(), 0), &channel.sender);

    // Assert: Second emission has previous with boundary value
    let result2 = stream.next().await.unwrap();
    assert_eq!(
        (result2.0.map(|s| s.value), result2.1.value),
        (Some(person("".to_string(), 0)), person("".to_string(), 0))
    );

    // Act: Transition to normal value
    push(person_alice(), &channel.sender);

    // Assert: Should track boundary as previous
    let result3 = stream.next().await.unwrap();
    assert_eq!(
        (result3.0.map(|s| s.value), result3.1.value),
        (Some(person("".to_string(), 0)), person_alice())
    );
}

#[tokio::test]
async fn test_combine_with_previous_boundary_maximum_concurrent_streams() {
    // Arrange: Test concurrent handling with many parallel streams
    let num_concurrent = 50;
    let mut handles = Vec::new();

    for i in 0..num_concurrent {
        let handle = tokio::spawn(async move {
            let channel = FluxionChannel::new();
            let mut stream = channel.stream.combine_with_previous();

            // Act: Send first value
            push(person(format!("Person{}", i), i), &channel.sender);

            // Assert: No previous
            let result = stream.next().await.unwrap();
            assert_eq!(
                (result.0.map(|s| s.value), result.1.value),
                (None, person(format!("Person{}", i), i))
            );

            // Act: Send second value
            push(person_alice(), &channel.sender);

            // Assert: Has previous
            let result2 = stream.next().await.unwrap();
            assert_eq!(
                (result2.0.map(|s| s.value), result2.1.value),
                (Some(person(format!("Person{}", i), i)), person_alice())
            );

            // Act: Send third value
            push(person_bob(), &channel.sender);

            // Assert: Previous is Alice
            let result3 = stream.next().await.unwrap();
            assert_eq!(
                (result3.0.map(|s| s.value), result3.1.value),
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
    let channel = FluxionChannel::new();
    let mut stream = channel.stream.combine_with_previous();

    // Act: Send single value and close
    push(person_alice(), &channel.sender);
    drop(channel.sender);

    // Assert: Should emit with no previous
    let result = stream.next().await.unwrap();
    assert_eq!(
        (result.0.map(|s| s.value), result.1.value),
        (None, person_alice())
    );

    // Assert: Stream ends
    assert!(stream.next().await.is_none());
}
