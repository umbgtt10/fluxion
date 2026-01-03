// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

use fluxion_core::{FluxionError, StreamItem};

use fluxion_stream::{FilterOrderedExt, MapOrderedExt, ScanOrderedExt};
use fluxion_test_utils::{
    assert_no_element_emitted, person::Person, test_channel_with_errors, unwrap_stream, Sequenced,
};

#[tokio::test]
async fn test_map_ordered_error_propagation_into_scan_ordered() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<Person>>();

    let mut result = stream
        .map_ordered(|val: Sequenced<Person>| {
            let mut p = val.into_inner();
            p.age *= 2;
            Sequenced::new(p)
        })
        .scan_ordered(0, |sum: &mut u32, val: &Person| {
            *sum += val.age;
            *sum
        });

    // Act & Assert
    tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        Person::new("Alice".to_string(), 10),
        1,
    )))?;
    assert!(matches!(
        unwrap_stream::<Sequenced<u32>, _>(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == 20
    ));

    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        Person::new("Bob".to_string(), 5),
        3,
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == 30
    ));

    drop(tx);

    Ok(())
}

#[tokio::test]
async fn test_filter_ordered_error_propagation_into_scan_ordered() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<Person>>();

    let mut result =
        stream
            .filter_ordered(|val| val.age > 10)
            .scan_ordered(0, |count: &mut i32, _: &Person| {
                *count += 1;
                *count
            });

    // Act & Assert
    tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        Person::new("Alice".to_string(), 5),
        1,
    )))?;
    assert_no_element_emitted(&mut result, 100).await;

    tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        Person::new("Bob".to_string(), 15),
        2,
    )))?;
    assert!(matches!(
        unwrap_stream::<Sequenced<i32>, _>(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == 1
    ));

    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        Person::new("Charlie".to_string(), 20),
        4,
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == 2
    ));

    drop(tx);

    Ok(())
}

#[tokio::test]
async fn test_chained_scan_ordered_error_propagation() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<Person>>();

    let mut result = stream
        .scan_ordered::<Sequenced<u32>, _, _>(0, |sum: &mut u32, value: &Person| {
            *sum += value.age;
            *sum
        })
        .scan_ordered(0, |count: &mut i32, _sum: &u32| {
            *count += 1;
            *count
        });

    // Act & Assert
    tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        Person::new("Alice".to_string(), 10),
        1,
    )))?;
    assert!(matches!(
        unwrap_stream::<Sequenced<i32>, _>(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == 1
    ));

    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        Person::new("Bob".to_string(), 20),
        3,
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == 2
    ));

    tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        Person::new("Charlie".to_string(), 30),
        4,
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == 3
    ));

    drop(tx);

    Ok(())
}
