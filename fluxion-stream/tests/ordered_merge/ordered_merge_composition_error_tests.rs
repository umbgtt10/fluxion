// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Composition + error propagation tests for `ordered_merge` operator.
//!
//! Tests combining ordered_merge with other operators while propagating errors.

use fluxion_core::{into_stream::IntoStream, FluxionError, HasTimestamp, StreamItem};
use fluxion_stream::{FilterOrderedExt, MapOrderedExt, OrderedStreamExt};
use fluxion_test_utils::test_data::{animal, person};
use fluxion_test_utils::{
    assert_stream_ended, test_channel_with_errors, unwrap_stream, Sequenced, TestData,
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
    tx1.send(StreamItem::Value(Sequenced::with_timestamp(
        person("P1".to_string(), 25),
        1,
    )))?; // age 25 odd, filtered
    tx2.send(StreamItem::Error(FluxionError::stream_error("Error 1")))?;
    tx1.send(StreamItem::Value(Sequenced::with_timestamp(
        person("P2".to_string(), 30),
        2,
    )))?; // age 30 even, passes
    tx2.send(StreamItem::Value(Sequenced::with_timestamp(
        person("P3".to_string(), 27),
        3,
    )))?; // age 27 odd, filtered
    tx1.send(StreamItem::Value(Sequenced::with_timestamp(
        person("P4".to_string(), 40),
        4,
    )))?; // age 40 even, passes

    // Assert: Error should propagate, odd numbers filtered
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    )); // 2
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    )); // 4

    drop(tx1);
    drop(tx2);
    assert_stream_ended(&mut result, 500).await;

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
    tx1.send(StreamItem::Value(Sequenced::with_timestamp(
        person("P1".to_string(), 10),
        1,
    )))?;
    tx2.send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    tx1.send(StreamItem::Value(Sequenced::with_timestamp(
        person("P2".to_string(), 15),
        2,
    )))?;

    // Assert: Error propagated immediately, then ages doubled
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    let item1 = unwrap_stream(&mut result, 100).await;
    assert!(matches!(item1, StreamItem::Value(_)));
    if let StreamItem::Value(v) = item1 {
        if let TestData::Person(p) = v.into_inner() {
            assert_eq!(p.age, 20); // 10 * 2
        }
    }

    let item2 = unwrap_stream(&mut result, 100).await;
    assert!(matches!(item2, StreamItem::Value(_)));
    if let StreamItem::Value(v) = item2 {
        if let TestData::Person(p) = v.into_inner() {
            assert_eq!(p.age, 30); // 15 * 2
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_filter_then_ordered_merge_with_errors() -> anyhow::Result<()> {
    // Arrange: Filter before merging, with errors
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
    tx1.send(StreamItem::Value(Sequenced::with_timestamp(
        person("P1".to_string(), 3),
        1,
    )))?; // age 3, filtered
    tx2.send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    tx1.send(StreamItem::Value(Sequenced::with_timestamp(
        person("P2".to_string(), 7),
        2,
    )))?; // age 7, passes
    tx2.send(StreamItem::Value(Sequenced::with_timestamp(
        person("P3".to_string(), 2),
        3,
    )))?; // age 2, filtered
    tx1.send(StreamItem::Value(Sequenced::with_timestamp(
        animal("A1".to_string(), 8),
        4,
    )))?; // legs 8, passes

    // Assert: Error propagates, only values > 5 emitted
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    let item2 = unwrap_stream(&mut result, 100).await;
    assert!(matches!(item2, StreamItem::Value(_)));
    if let StreamItem::Value(v) = item2 {
        if let TestData::Person(p) = v.into_inner() {
            assert_eq!(p.age, 7);
        }
    }

    let item3 = unwrap_stream(&mut result, 100).await;
    assert!(matches!(item3, StreamItem::Value(_)));
    if let StreamItem::Value(v) = item3 {
        if let TestData::Animal(a) = v.into_inner() {
            assert_eq!(a.legs, 8);
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_map_filter_chain_with_errors() -> anyhow::Result<()> {
    // Arrange: ordered_merge -> map -> filter with errors
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
    tx1.send(StreamItem::Value(Sequenced::with_timestamp(
        person("P1".to_string(), 1),
        1,
    )))?; // 1*2=2, filtered
    tx2.send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    tx1.send(StreamItem::Value(Sequenced::with_timestamp(
        person("P2".to_string(), 3),
        2,
    )))?; // 3*2=6, passes
    tx2.send(StreamItem::Value(Sequenced::with_timestamp(
        person("P3".to_string(), 2),
        3,
    )))?; // 2*2=4, filtered

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    let item = unwrap_stream(&mut result, 100).await;
    assert!(matches!(item, StreamItem::Value(_)));
    if let StreamItem::Value(v) = item {
        if let TestData::Person(p) = v.into_inner() {
            assert_eq!(p.age, 6);
        }
    }

    drop(tx1);
    drop(tx2);
    assert_stream_ended(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_chained_with_errors() -> anyhow::Result<()> {
    // Arrange: Chain two ordered_merge operations with errors
    let (tx1, s1) = test_channel_with_errors::<Sequenced<TestData>>();
    let (tx2, s2) = test_channel_with_errors::<Sequenced<TestData>>();
    let (tx3, s3) = test_channel_with_errors::<Sequenced<TestData>>();

    let merged_12 = s1.ordered_merge(vec![s2]);
    let mut result = merged_12.ordered_merge(vec![s3]);

    // Act: Send values and errors from all three streams
    tx1.send(StreamItem::Value(Sequenced::with_timestamp(
        person("P1".to_string(), 10),
        1,
    )))?;
    tx2.send(StreamItem::Error(FluxionError::stream_error("Error 1")))?;
    tx3.send(StreamItem::Value(Sequenced::with_timestamp(
        person("P2".to_string(), 20),
        2,
    )))?;
    tx1.send(StreamItem::Error(FluxionError::stream_error("Error 2")))?;
    tx2.send(StreamItem::Value(Sequenced::with_timestamp(
        person("P3".to_string(), 30),
        3,
    )))?;

    // Assert: Errors emitted immediately, then values in timestamp order
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    )); // Error 1
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    )); // ts=1
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    )); // Error 2
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    )); // ts=2
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    )); // ts=3

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_error_after_filter() -> anyhow::Result<()> {
    // Arrange: Test error behavior when filter removes some items
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

    // Act: Mix values (some filtered) and errors
    tx1.send(StreamItem::Value(Sequenced::with_timestamp(
        person("P1".to_string(), 1),
        1,
    )))?; // odd, filtered
    tx1.send(StreamItem::Value(Sequenced::with_timestamp(
        person("P2".to_string(), 2),
        2,
    )))?; // even, passes
    tx2.send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    tx2.send(StreamItem::Value(Sequenced::with_timestamp(
        person("P3".to_string(), 3),
        3,
    )))?; // odd, filtered
    tx1.send(StreamItem::Value(Sequenced::with_timestamp(
        person("P4".to_string(), 4),
        4,
    )))?; // even, passes

    // Assert: Error emitted immediately, then even values
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    let item1 = unwrap_stream(&mut result, 100).await;
    assert!(matches!(item1, StreamItem::Value(_)));
    if let StreamItem::Value(v) = item1 {
        if let TestData::Person(p) = v.into_inner() {
            assert_eq!(p.age, 2);
        }
    }

    let item2 = unwrap_stream(&mut result, 100).await;
    assert!(matches!(item2, StreamItem::Value(_)));
    if let StreamItem::Value(v) = item2 {
        if let TestData::Person(p) = v.into_inner() {
            assert_eq!(p.age, 4);
        }
    }

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

    // Act: Multiple errors from both streams
    tx1.send(StreamItem::Error(FluxionError::stream_error("Error 1")))?;
    tx2.send(StreamItem::Error(FluxionError::stream_error("Error 2")))?;
    tx1.send(StreamItem::Value(Sequenced::with_timestamp(
        person("P1".to_string(), 10),
        1,
    )))?;
    tx2.send(StreamItem::Value(Sequenced::with_timestamp(
        person("P2".to_string(), 20),
        2,
    )))?; // Send value with ts > 1 so ordered_merge can emit value at ts=1
    tx2.send(StreamItem::Error(FluxionError::stream_error("Error 3")))?;

    // Assert: Errors emitted immediately, values in timestamp order
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    )); // Error 1
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    )); // Error 2

    let item1 = unwrap_stream(&mut result, 100).await;
    assert!(matches!(item1, StreamItem::Value(_)));
    if let StreamItem::Value(v) = item1 {
        assert_eq!(v.into_inner(), "P1".to_string());
    }

    let item2 = unwrap_stream(&mut result, 100).await;
    assert!(matches!(item2, StreamItem::Value(_)));
    if let StreamItem::Value(v) = item2 {
        assert_eq!(v.into_inner(), "P2".to_string());
    }

    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    )); // Error 3

    drop(tx1);
    drop(tx2);

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_errors_only_through_filter() -> anyhow::Result<()> {
    // Arrange: Stream with only errors going through filter
    let (tx1, s1) = test_channel_with_errors::<Sequenced<TestData>>();
    let (tx2, s2) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut result = s1.ordered_merge(vec![s2]).filter_ordered(|x| match x {
        TestData::Person(p) => p.age > 100,
        TestData::Animal(a) => a.legs > 100,
        _ => false,
    });

    // Act: Send only errors (no values pass filter)
    tx1.send(StreamItem::Error(FluxionError::stream_error("Error 1")))?;
    tx2.send(StreamItem::Error(FluxionError::stream_error("Error 2")))?;

    // Assert: Errors should still propagate even though no values pass
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    drop(tx1);
    drop(tx2);
    assert_stream_ended(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_three_streams_filter_map_errors() -> anyhow::Result<()> {
    // Arrange: Complex pipeline with three streams
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
    tx1.send(StreamItem::Value(Sequenced::with_timestamp(
        person("P1".to_string(), 2),
        1,
    )))?; // age 2 -> 1
    tx2.send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    tx3.send(StreamItem::Value(Sequenced::with_timestamp(
        person("P2".to_string(), 3),
        2,
    )))?; // age 3 odd, filtered
    tx1.send(StreamItem::Value(Sequenced::with_timestamp(
        animal("A1".to_string(), 4),
        3,
    )))?; // legs 4 -> 2

    // Assert: Error emitted immediately, then filtered/mapped values
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    let item1 = unwrap_stream(&mut result, 100).await;
    assert!(matches!(item1, StreamItem::Value(_)));
    if let StreamItem::Value(v) = item1 {
        if let TestData::Person(p) = v.into_inner() {
            assert_eq!(p.age, 1);
        }
    }

    let item2 = unwrap_stream(&mut result, 100).await;
    assert!(matches!(item2, StreamItem::Value(_)));
    if let StreamItem::Value(v) = item2 {
        if let TestData::Animal(a) = v.into_inner() {
            assert_eq!(a.legs, 2);
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_error_preserves_timestamp_through_pipeline() -> anyhow::Result<()> {
    // Arrange: Verify that value timestamps are preserved through pipeline even with errors
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
    tx1.send(StreamItem::Value(Sequenced::with_timestamp(
        person("P1".to_string(), 10),
        5,
    )))?;
    tx2.send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    tx1.send(StreamItem::Value(Sequenced::with_timestamp(
        person("P2".to_string(), 20),
        10,
    )))?;

    // Assert: Error emitted immediately, then timestamps preserved through map
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
