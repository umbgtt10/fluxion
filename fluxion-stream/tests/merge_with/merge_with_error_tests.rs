// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Error propagation tests for `merge_with` operator.
//!
//! The `merge_with` operator now handles `StreamItem<T>` directly and passes errors
//! through unchanged, allowing error handling at the stream consumer level.

use fluxion_core::{into_stream::IntoStream, FluxionError, StreamItem};
use fluxion_stream::MergedStream;
use fluxion_test_utils::{
    assert_no_element_emitted, assert_stream_ended,
    person::Person,
    test_channel_with_errors,
    test_data::{person, person_alice, person_bob, person_diane, TestData},
    unwrap_stream, unwrap_value, Sequenced,
};
use futures::StreamExt;

#[tokio::test]
async fn test_merge_with_propagates_errors_from_first_stream() -> anyhow::Result<()> {
    // Arrange
    let (tx1, stream1) = test_channel_with_errors::<Sequenced<TestData>>();
    let (tx2, stream2) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut merged = MergedStream::seed::<Sequenced<Person>>(Person::new("Sum".to_string(), 0))
        .merge_with(stream1, |item: TestData, state| {
            if let TestData::Person(p) = item {
                state.age += p.age;
            }
            state.clone()
        })
        .merge_with(stream2, |item: TestData, state| {
            if let TestData::Person(p) = item {
                state.age += p.age - 5;
            }
            state.clone()
        });

    // Act & Assert
    tx1.send(StreamItem::Value(Sequenced::new(person_alice())))?;
    assert_eq!(unwrap_value(merged.next().await).into_inner().age, 25);

    tx1.send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    assert!(
        matches!(merged.next().await.unwrap(), StreamItem::Error(ref e) if e.to_string() == "Stream processing error: Error")
    );

    tx2.send(StreamItem::Value(Sequenced::new(person_bob())))?;
    assert_eq!(unwrap_value(merged.next().await).into_inner().age, 50);

    Ok(())
}

#[tokio::test]
async fn test_merge_with_error_at_start_filtered() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut merged = MergedStream::seed::<Sequenced<Person>>(Person::new("Sum".to_string(), 0))
        .merge_with(stream, |item: TestData, state| {
            if let TestData::Person(p) = item {
                state.age += p.age;
            }
            state.clone()
        });

    // Act & Assert
    tx.send(StreamItem::Error(FluxionError::stream_error("Early error")))?;
    assert!(
        matches!(merged.next().await.unwrap(), StreamItem::Error(ref e) if e.to_string() == "Stream processing error: Early error")
    );

    tx.send(StreamItem::Value(Sequenced::new(person_alice())))?;
    assert_eq!(unwrap_value(merged.next().await).into_inner().age, 25);
    drop(tx);

    Ok(())
}

#[tokio::test]
async fn test_merge_with_multiple_streams_error_filtering() -> anyhow::Result<()> {
    // Arrange
    let (tx1, stream1) = test_channel_with_errors::<Sequenced<TestData>>();
    let (tx2, stream2) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut merged = MergedStream::seed::<Sequenced<Person>>(Person::new("Sum".to_string(), 0))
        .merge_with(stream1, |item: TestData, state| {
            if let TestData::Person(p) = item {
                state.age += p.age;
            }
            state.clone()
        })
        .merge_with(stream2, |item: TestData, state| {
            if let TestData::Person(p) = item {
                state.age += p.age;
            }
            state.clone()
        });

    // Act & Assert
    tx1.send(StreamItem::Error(FluxionError::stream_error("Error 1")))?;
    tx2.send(StreamItem::Error(FluxionError::stream_error("Error 2")))?;

    assert!(
        matches!(merged.next().await.unwrap(), StreamItem::Error(ref e) if e.to_string().contains("Error"))
    );
    assert!(
        matches!(merged.next().await.unwrap(), StreamItem::Error(ref e) if e.to_string().contains("Error"))
    );

    tx1.send(StreamItem::Value(Sequenced::new(person_alice())))?;
    assert_eq!(unwrap_value(merged.next().await).into_inner().age, 25);

    Ok(())
}

#[tokio::test]
async fn test_merge_with_errors_interleaved_with_values() -> anyhow::Result<()> {
    // Arrange
    let (tx1, stream1) = test_channel_with_errors::<Sequenced<TestData>>();
    let (tx2, stream2) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut merged = MergedStream::seed::<Sequenced<Person>>(Person::new("Sum".to_string(), 0))
        .merge_with(stream1, |item: TestData, state| {
            if let TestData::Person(p) = item {
                state.age += p.age;
            }
            state.clone()
        })
        .merge_with(stream2, |item: TestData, state| {
            if let TestData::Person(p) = item {
                state.age += p.age * 2;
            }
            state.clone()
        });

    // Act & Assert
    tx1.send(StreamItem::Value(Sequenced::new(person_alice())))?;
    assert_eq!(unwrap_value(merged.next().await).into_inner().age, 25);

    tx2.send(StreamItem::Error(FluxionError::stream_error("Error 1")))?;
    assert!(matches!(merged.next().await.unwrap(), StreamItem::Error(_)));

    tx1.send(StreamItem::Value(Sequenced::new(person_bob())))?;
    assert_eq!(unwrap_value(merged.next().await).into_inner().age, 55);

    tx1.send(StreamItem::Error(FluxionError::stream_error("Error 2")))?;
    assert!(matches!(merged.next().await.unwrap(), StreamItem::Error(_)));

    tx2.send(StreamItem::Value(Sequenced::new(person_diane())))?;
    assert_eq!(unwrap_value(merged.next().await).into_inner().age, 135);
    drop(tx1);
    drop(tx2);

    Ok(())
}

#[tokio::test]
async fn test_merge_with_state_preserved_despite_filtered_errors() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut merged = MergedStream::seed::<Sequenced<Person>>(Person::new("Sum".to_string(), 0))
        .merge_with(stream, |item: TestData, state| {
            if let TestData::Person(p) = item {
                state.age += p.age;
            }
            state.clone()
        });

    // Act & Assert
    tx.send(StreamItem::Value(Sequenced::new(person_alice())))?;
    assert_eq!(unwrap_value(merged.next().await).into_inner().age, 25);

    tx.send(StreamItem::Value(Sequenced::new(person_bob())))?;
    assert_eq!(unwrap_value(merged.next().await).into_inner().age, 55);

    tx.send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    assert!(matches!(merged.next().await.unwrap(), StreamItem::Error(_)));

    tx.send(StreamItem::Value(Sequenced::new(person_diane())))?;
    assert_eq!(unwrap_value(merged.next().await).into_inner().age, 95);

    Ok(())
}

#[tokio::test]
async fn test_merge_with_error_before_stream_ends() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut merged = MergedStream::seed::<Sequenced<Person>>(Person::new("Sum".to_string(), 0))
        .merge_with(stream, |item: TestData, state| {
            if let TestData::Person(p) = item {
                state.age += p.age;
            }
            state.clone()
        });

    // Act & Assert
    tx.send(StreamItem::Value(Sequenced::new(person_alice())))?;
    assert_eq!(unwrap_value(merged.next().await).into_inner().age, 25);

    tx.send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    assert!(matches!(merged.next().await.unwrap(), StreamItem::Error(_)));

    assert_no_element_emitted(&mut merged, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_merge_with_empty_stream_with_only_errors() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut merged = MergedStream::seed::<Sequenced<Person>>(Person::new("Sum".to_string(), 0))
        .merge_with(stream, |item: TestData, state| {
            if let TestData::Person(p) = item {
                state.age += p.age;
            }
            state.clone()
        });

    // Act & Assert
    tx.send(StreamItem::Error(FluxionError::stream_error("Error 1")))?;
    tx.send(StreamItem::Error(FluxionError::stream_error("Error 2")))?;

    assert!(matches!(merged.next().await.unwrap(), StreamItem::Error(_)));
    assert!(matches!(merged.next().await.unwrap(), StreamItem::Error(_)));

    drop(tx);
    assert_stream_ended(&mut merged, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_merge_with_three_streams_with_filtered_errors() -> anyhow::Result<()> {
    // Arrange
    let (tx1, stream1) = test_channel_with_errors::<Sequenced<TestData>>();
    let (tx2, stream2) = test_channel_with_errors::<Sequenced<TestData>>();
    let (tx3, stream3) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut merged = MergedStream::seed::<Sequenced<Person>>(Person::new("Sum".to_string(), 0))
        .merge_with(stream1, |item: TestData, state| {
            if let TestData::Person(p) = item {
                state.age += p.age;
            }
            state.clone()
        })
        .merge_with(stream2, |item: TestData, state| {
            if let TestData::Person(p) = item {
                state.age += p.age * 2;
            }
            state.clone()
        })
        .merge_with(stream3, |item: TestData, state| {
            if let TestData::Person(p) = item {
                state.age += p.age * 3;
            }
            state.clone()
        });

    // Act & Assert
    tx1.send(StreamItem::Value(Sequenced::new(person_alice())))?;
    assert_eq!(unwrap_value(merged.next().await).into_inner().age, 25);

    tx2.send(StreamItem::Value(Sequenced::new(person_bob())))?;
    assert_eq!(unwrap_value(merged.next().await).into_inner().age, 85);

    tx2.send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    assert!(matches!(merged.next().await.unwrap(), StreamItem::Error(_)));

    tx3.send(StreamItem::Value(Sequenced::new(person_diane())))?;
    assert_eq!(unwrap_value(merged.next().await).into_inner().age, 205);

    Ok(())
}

#[tokio::test]
async fn test_merge_with_into_fluxion_stream_error_handling() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut merged = MergedStream::seed::<Sequenced<Person>>(Person::new("Sum".to_string(), 0))
        .merge_with(stream, |item: TestData, state| {
            if let TestData::Person(p) = item {
                state.age += p.age;
            }
            state.clone()
        })
        .into_stream();

    // Act & Assert
    tx.send(StreamItem::Value(Sequenced::new(person_alice())))?;
    assert_eq!(unwrap_value(merged.next().await).into_inner().age, 25);

    tx.send(StreamItem::Error(FluxionError::stream_error("Filtered")))?;
    assert!(matches!(merged.next().await.unwrap(), StreamItem::Error(_)));

    tx.send(StreamItem::Value(Sequenced::new(person_bob())))?;
    assert_eq!(unwrap_value(merged.next().await).into_inner().age, 55);

    drop(tx);

    Ok(())
}

#[tokio::test]
async fn test_merge_with_poll_pending_simulation() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut merged = MergedStream::seed::<Sequenced<Person>>(Person::new("Sum".to_string(), 0))
        .merge_with(stream, |item: TestData, state| {
            if let TestData::Person(p) = item {
                state.age += p.age;
            }
            state.clone()
        });

    // Act & Assert
    tokio::select! {
        _ = tokio::time::sleep(tokio::time::Duration::from_millis(10)) => {
            tx.send(StreamItem::Value(Sequenced::new(person("A".to_string(), 10))))?;
        }
        result = merged.next() => {
            if let Some(StreamItem::Value(v)) = result {
                assert_eq!(v.into_inner().age, 10);
            }
        }
    }

    assert_eq!(
        unwrap_stream(&mut merged, 100)
            .await
            .unwrap()
            .into_inner()
            .age,
        10
    );

    drop(tx);

    Ok(())
}
