// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{into_stream::IntoStream, FluxionError, HasTimestamp, StreamItem};
use fluxion_stream::{FilterOrderedExt, MapOrderedExt, OrderedStreamExt};
use fluxion_test_utils::test_data::{animal, person};
use fluxion_test_utils::{
    helpers::{test_channel_with_errors, unwrap_stream},
    sequenced::Sequenced,
    test_data::TestData,
};

#[tokio::test]
async fn test_ordered_merge_then_filter_with_errors() -> anyhow::Result<()> {
    // Arrange
    let (tx1, s1) = test_channel_with_errors::<Sequenced<TestData>>();
    let (tx2, s2) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut result = s1
        .ordered_merge(vec![s2])
        .into_stream()
        .filter_ordered(|x| match x {
            TestData::Person(p) => p.age % 2 == 0,
            _ => false,
        });

    // Act
    tx1.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person("P1".to_string(), 25),
        1,
    )))?;
    tx2.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error 1")))?;
    tx1.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person("P2".to_string(), 30),
        2,
    )))?;
    tx2.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person("P3".to_string(), 27),
        3,
    )))?;
    tx1.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person("P4".to_string(), 40),
        4,
    )))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));
    assert!(
        matches!(&unwrap_stream(&mut result, 100).await, StreamItem::Value(v) if matches!(v.value, TestData::Person(ref p) if p.age == 30))
    );
    assert!(
        matches!(&unwrap_stream(&mut result, 100).await, StreamItem::Value(v) if matches!(v.value, TestData::Person(ref p) if p.age == 40))
    );

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_then_map_with_errors() -> anyhow::Result<()> {
    // Arrange
    let (tx1, s1) = test_channel_with_errors::<Sequenced<TestData>>();
    let (tx2, s2) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut result = s1.ordered_merge(vec![s2]).into_stream().map_ordered(|x| {
        let ts = x.timestamp();
        let doubled = match x.into_inner() {
            TestData::Person(p) => person(p.name, p.age * 2),
            other => other,
        };
        Sequenced::with_timestamp(doubled, ts)
    });

    // Act
    tx1.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person("P1".to_string(), 10),
        1,
    )))?;
    tx2.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    tx1.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person("P2".to_string(), 15),
        2,
    )))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));
    assert!(
        matches!(&unwrap_stream(&mut result, 100).await, StreamItem::Value(v) if matches!(v.value, TestData::Person(ref p) if p.age == 20))
    );
    assert!(
        matches!(&unwrap_stream(&mut result, 100).await, StreamItem::Value(v) if matches!(v.value, TestData::Person(ref p) if p.age == 30))
    );

    Ok(())
}

#[tokio::test]
async fn test_filter_then_ordered_merge_with_errors() -> anyhow::Result<()> {
    // Arrange
    let (tx1, s1) = test_channel_with_errors::<Sequenced<TestData>>();
    let (tx2, s2) = test_channel_with_errors::<Sequenced<TestData>>();

    let filtered1 = s1.into_stream().filter_ordered(|x| match x {
        TestData::Person(p) => p.age > 5,
        TestData::Animal(a) => a.legs > 5,
        _ => false,
    });
    let filtered2 = s2.into_stream().filter_ordered(|x| match x {
        TestData::Person(p) => p.age > 5,
        TestData::Animal(a) => a.legs > 5,
        _ => false,
    });

    let mut result = filtered1.ordered_merge(vec![filtered2]);

    // Act
    tx1.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person("P1".to_string(), 3),
        1,
    )))?;
    tx2.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    tx1.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person("P2".to_string(), 7),
        2,
    )))?;
    tx2.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person("P3".to_string(), 2),
        3,
    )))?;
    tx1.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        animal("A1".to_string(), 8),
        4,
    )))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));
    assert!(
        matches!(&unwrap_stream(&mut result, 100).await, StreamItem::Value(v) if matches!(v.value, TestData::Person(ref p) if p.age == 7))
    );
    assert!(
        matches!(&unwrap_stream(&mut result, 100).await, StreamItem::Value(v) if matches!(v.value, TestData::Animal(ref a) if a.legs == 8))
    );

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_map_filter_chain_with_errors() -> anyhow::Result<()> {
    // Arrange
    let (tx1, s1) = test_channel_with_errors::<Sequenced<TestData>>();
    let (tx2, s2) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut result = s1
        .ordered_merge(vec![s2])
        .map_ordered(|x| {
            let ts = x.timestamp();
            let doubled = match x.into_inner() {
                TestData::Person(p) => person(p.name, p.age * 2),
                TestData::Animal(a) => animal(a.species, a.legs * 2),
                other => other,
            };
            Sequenced::with_timestamp(doubled, ts)
        })
        .filter_ordered(|x| match x {
            TestData::Person(p) => p.age > 4,
            TestData::Animal(a) => a.legs > 4,
            _ => false,
        });

    // Act
    tx1.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person("P1".to_string(), 1),
        1,
    )))?;
    tx2.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    tx1.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person("P2".to_string(), 3),
        2,
    )))?;
    tx2.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person("P3".to_string(), 2),
        3,
    )))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));
    assert!(
        matches!(&unwrap_stream(&mut result, 100).await, StreamItem::Value(v) if matches!(v.value, TestData::Person(ref p) if p.age == 6))
    );

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_chained_with_errors() -> anyhow::Result<()> {
    // Arrange
    let (tx1, s1) = test_channel_with_errors::<Sequenced<TestData>>();
    let (tx2, s2) = test_channel_with_errors::<Sequenced<TestData>>();
    let (tx3, s3) = test_channel_with_errors::<Sequenced<TestData>>();

    let merged_12 = s1.ordered_merge(vec![s2]);
    let mut result = merged_12.ordered_merge(vec![s3]);

    // Act
    tx1.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person("P1".to_string(), 10),
        1,
    )))?;
    tx2.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error 1")))?;
    tx3.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person("P2".to_string(), 20),
        2,
    )))?;
    tx1.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error 2")))?;
    tx2.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person("P3".to_string(), 30),
        3,
    )))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));
    assert!(
        matches!(&unwrap_stream(&mut result, 100).await, StreamItem::Value(v) if matches!(v.value, TestData::Person(ref p) if p.age == 10))
    );
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));
    assert!(
        matches!(&unwrap_stream(&mut result, 100).await, StreamItem::Value(v) if matches!(v.value, TestData::Person(ref p) if p.age == 20))
    );
    assert!(
        matches!(&unwrap_stream(&mut result, 100).await, StreamItem::Value(v) if matches!(v.value, TestData::Person(ref p) if p.age == 30))
    );

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_error_after_filter() -> anyhow::Result<()> {
    // Arrange
    let (tx1, s1) = test_channel_with_errors::<Sequenced<TestData>>();
    let (tx2, s2) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut result = s1
        .ordered_merge(vec![s2])
        .into_stream()
        .filter_ordered(|x| match x {
            TestData::Person(p) => p.age % 2 == 0,
            TestData::Animal(a) => a.legs % 2 == 0,
            _ => false,
        });

    // Act
    tx1.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person("P1".to_string(), 1),
        1,
    )))?;
    tx1.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person("P2".to_string(), 2),
        2,
    )))?;
    tx2.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    tx2.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person("P3".to_string(), 3),
        3,
    )))?;
    tx1.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person("P4".to_string(), 4),
        4,
    )))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));
    assert!(
        matches!(&unwrap_stream(&mut result, 100).await, StreamItem::Value(v) if matches!(v.value, TestData::Person(ref p) if p.age == 2))
    );
    assert!(
        matches!(&unwrap_stream(&mut result, 100).await, StreamItem::Value(v) if matches!(v.value, TestData::Person(ref p) if p.age == 4))
    );

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_multiple_errors_through_map() -> anyhow::Result<()> {
    // Arrange
    let (tx1, s1) = test_channel_with_errors::<Sequenced<TestData>>();
    let (tx2, s2) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut result = s1.ordered_merge(vec![s2]).into_stream().map_ordered(|x| {
        let ts = x.timestamp();
        let name = match x.into_inner() {
            TestData::Person(p) => p.name,
            TestData::Animal(a) => a.species,
            TestData::Plant(pl) => pl.species,
        };
        Sequenced::with_timestamp(name, ts)
    });

    // Act
    tx1.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error 1")))?;
    tx2.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error 2")))?;
    tx1.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person("P1".to_string(), 10),
        1,
    )))?;
    tx2.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person("P2".to_string(), 20),
        2,
    )))?;
    tx2.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error 3")))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));
    assert!(
        matches!(&unwrap_stream(&mut result, 100).await, StreamItem::Value(v) if v.value == "P1")
    );
    assert!(
        matches!(&unwrap_stream(&mut result, 100).await, StreamItem::Value(v) if v.value == "P2")
    );
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_errors_only_through_filter() -> anyhow::Result<()> {
    // Arrange
    let (tx1, s1) = test_channel_with_errors::<Sequenced<TestData>>();
    let (tx2, s2) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut result = s1.ordered_merge(vec![s2]).filter_ordered(|x| match x {
        TestData::Person(p) => p.age > 100,
        TestData::Animal(a) => a.legs > 100,
        _ => false,
    });

    // Act
    tx1.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error 1")))?;
    tx2.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error 2")))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_three_streams_filter_map_errors() -> anyhow::Result<()> {
    // Arrange
    let (tx1, s1) = test_channel_with_errors::<Sequenced<TestData>>();
    let (tx2, s2) = test_channel_with_errors::<Sequenced<TestData>>();
    let (tx3, s3) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut result = s1
        .ordered_merge(vec![s2, s3])
        .into_stream()
        .filter_ordered(|x| match x {
            TestData::Person(p) => p.age % 2 == 0,
            TestData::Animal(a) => a.legs % 2 == 0,
            _ => false,
        })
        .map_ordered(|x| {
            let ts = x.timestamp();
            let halved = match x.into_inner() {
                TestData::Person(p) => person(p.name, p.age / 2),
                TestData::Animal(a) => animal(a.species, a.legs / 2),
                other => other,
            };
            Sequenced::with_timestamp(halved, ts)
        });

    // Act
    tx1.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person("P1".to_string(), 2),
        1,
    )))?;
    tx2.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    tx3.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person("P2".to_string(), 3),
        2,
    )))?;
    tx1.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        animal("A1".to_string(), 4),
        3,
    )))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));
    assert!(
        matches!(&unwrap_stream(&mut result, 100).await, StreamItem::Value(v) if matches!(v.value, TestData::Person(ref p) if p.age == 1))
    );
    assert!(
        matches!(&unwrap_stream(&mut result, 100).await, StreamItem::Value(v) if matches!(v.value, TestData::Animal(ref a) if a.legs == 2))
    );

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_error_preserves_timestamp_through_pipeline() -> anyhow::Result<()> {
    // Arrange
    let (tx1, s1) = test_channel_with_errors::<Sequenced<TestData>>();
    let (tx2, s2) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut result = s1.ordered_merge(vec![s2]).into_stream().map_ordered(|x| {
        let ts = x.timestamp();
        let doubled = match x.into_inner() {
            TestData::Person(p) => person(p.name, p.age * 2),
            TestData::Animal(a) => animal(a.species, a.legs * 2),
            other => other,
        };
        Sequenced::with_timestamp(doubled, ts)
    });

    // Act
    tx1.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person("P1".to_string(), 10),
        5,
    )))?;
    tx2.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    tx1.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person("P2".to_string(), 20),
        10,
    )))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    let item1 = unwrap_stream(&mut result, 100).await;
    if let StreamItem::Value(v) = item1 {
        assert_eq!(v.timestamp(), 5);
        if let TestData::Person(p) = v.into_inner() {
            assert_eq!(p.age, 20);
        }
    } else {
        panic!("Expected Value");
    }

    let item2 = unwrap_stream(&mut result, 100).await;
    if let StreamItem::Value(v) = item2 {
        assert_eq!(v.timestamp(), 10);
        if let TestData::Person(p) = v.into_inner() {
            assert_eq!(p.age, 40);
        }
    } else {
        panic!("Expected Value");
    }

    Ok(())
}
