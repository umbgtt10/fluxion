// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_stream::combine_with_previous::CombineWithPreviousExt;
use fluxion_test_utils::helpers::{assert_stream_ended, test_channel, unwrap_stream};
use fluxion_test_utils::sequenced::Sequenced;
use fluxion_test_utils::test_data::{person, person_alice, person_bob, person_charlie};

#[tokio::test]
async fn test_combine_with_previous_no_previous_value_emits() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel();
    let mut result = stream.combine_with_previous();

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;

    // Assert
    let result = unwrap_stream(&mut result, 500).await.unwrap();
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
    let mut result = stream.combine_with_previous();

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;

    // Assert
    let first_result = unwrap_stream(&mut result, 500).await.unwrap();
    assert_eq!(
        (
            first_result.previous.map(|s| s.value),
            first_result.current.value
        ),
        (None, person_alice())
    );

    // Act
    tx.unbounded_send(Sequenced::new(person_bob()))?;

    // Assert
    let second_result = unwrap_stream(&mut result, 500).await.unwrap();
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
    let mut result = stream.combine_with_previous();

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;

    // Assert
    let first_result = unwrap_stream(&mut result, 500).await.unwrap();
    assert_eq!(
        (
            first_result.previous.map(|s| s.value),
            first_result.current.value
        ),
        (None, person_alice())
    );

    // Act
    tx.unbounded_send(Sequenced::new(person_bob()))?;

    // Assert
    let second_result = unwrap_stream(&mut result, 500).await.unwrap();
    assert_eq!(
        (
            second_result.previous.map(|s| s.value),
            second_result.current.value
        ),
        (Some(person_alice()), person_bob())
    );

    // Act
    tx.unbounded_send(Sequenced::new(person_charlie()))?;

    // Assert
    let third_result = unwrap_stream(&mut result, 500).await.unwrap();
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
    let mut result = stream.combine_with_previous();

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;

    // Assert
    let first_result = unwrap_stream(&mut result, 500).await.unwrap();
    assert_eq!(
        (
            first_result.previous.map(|s| s.value),
            first_result.current.value
        ),
        (None, person_alice())
    );

    // Act
    tx.unbounded_send(Sequenced::new(person_bob()))?;

    // Assert
    let second_result = unwrap_stream(&mut result, 500).await.unwrap();
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
    assert_stream_ended(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_combine_with_previous_for_types() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel();
    let mut result = stream.combine_with_previous();

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;

    // Assert
    let first_result = unwrap_stream(&mut result, 500).await.unwrap();
    assert_eq!(
        (
            first_result.previous.map(|s| s.value),
            first_result.current.value
        ),
        (None, person_alice())
    );

    // Act
    tx.unbounded_send(Sequenced::new(person_bob()))?;

    // Assert
    let second_result = unwrap_stream(&mut result, 500).await.unwrap();
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
    let mut result = stream.combine_with_previous();

    // Act
    for i in 0..200 {
        match i % 3 {
            0 => tx.unbounded_send(Sequenced::new(person_alice()))?,
            1 => tx.unbounded_send(Sequenced::new(person_bob()))?,
            _ => tx.unbounded_send(Sequenced::new(person_charlie()))?,
        }
    }
    drop(tx);

    // Assert
    let first = unwrap_stream(&mut result, 500).await.unwrap();
    assert_eq!(
        (first.previous.map(|s| s.value), first.current.value),
        (None, person_alice())
    );
    let second = unwrap_stream(&mut result, 500).await.unwrap();
    assert_eq!(
        (second.previous.map(|s| s.value), second.current.value),
        (Some(person_alice()), person_bob())
    );

    // Act
    let mut last_prev = None;
    let mut last_curr = None;
    for _ in 0..198 {
        let item = unwrap_stream(&mut result, 500).await.unwrap();
        last_prev = item.previous.map(|s| s.value);
        last_curr = Some(item.current.value);
    }

    // Assert
    assert_stream_ended(&mut result, 500).await;
    assert!(last_prev.is_some());
    assert!(last_curr.is_some());

    Ok(())
}

#[tokio::test]
async fn test_combine_with_previous_boundary_empty_string_zero_values() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel();
    let mut result = stream.combine_with_previous();

    // Act
    tx.unbounded_send(Sequenced::new(person(String::new(), 0)))?;

    // Assert
    let item = unwrap_stream(&mut result, 500).await.unwrap();
    assert_eq!(
        (item.previous.map(|s| s.value), item.current.value),
        (None, person(String::new(), 0))
    );

    // Act
    tx.unbounded_send(Sequenced::new(person(String::new(), 0)))?;

    // Assert
    let result2 = unwrap_stream(&mut result, 500).await.unwrap();
    assert_eq!(
        (result2.previous.map(|s| s.value), result2.current.value),
        (Some(person(String::new(), 0)), person(String::new(), 0))
    );

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;

    // Assert
    let result3 = unwrap_stream(&mut result, 500).await.unwrap();
    assert_eq!(
        (result3.previous.map(|s| s.value), result3.current.value),
        (Some(person(String::new(), 0)), person_alice())
    );

    Ok(())
}

#[tokio::test]
async fn test_combine_with_previous_boundary_maximum_concurrent_streams() -> anyhow::Result<()> {
    // Arrange
    let num_concurrent = 50;
    let mut handles = Vec::new();

    for i in 0..num_concurrent {
        let handle = tokio::spawn(async move {
            let (tx, stream) = test_channel();
            let mut result = stream.combine_with_previous();

            // Act
            tx.unbounded_send(Sequenced::new(person(format!("Person{i}"), i)))
                .unwrap();

            // Assert
            let item = unwrap_stream(&mut result, 500).await.unwrap();
            assert_eq!(
                (item.previous.map(|s| s.value), item.current.value),
                (None, person(format!("Person{i}"), i))
            );

            // Act
            tx.unbounded_send(Sequenced::new(person_alice())).unwrap();

            // Assert
            let result2 = unwrap_stream(&mut result, 500).await.unwrap();
            assert_eq!(
                (result2.previous.map(|s| s.value), result2.current.value),
                (Some(person(format!("Person{i}"), i)), person_alice())
            );

            // Act
            tx.unbounded_send(Sequenced::new(person_bob())).unwrap();

            // Assert
            let result3 = unwrap_stream(&mut result, 500).await.unwrap();
            assert_eq!(
                (result3.previous.map(|s| s.value), result3.current.value),
                (Some(person_alice()), person_bob())
            );
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.await?;
    }

    Ok(())
}

#[tokio::test]
async fn test_combine_with_previous_single_value_stream() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel();
    let mut result = stream.combine_with_previous();

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    drop(tx);

    // Assert
    let item = unwrap_stream(&mut result, 500).await.unwrap();
    assert_eq!(
        (item.previous.map(|s| s.value), item.current.value),
        (None, person_alice())
    );

    // Assert
    assert_stream_ended(&mut result, 500).await;

    Ok(())
}
