// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::FluxionStream;
use fluxion_test_utils::{
    person::Person, test_channel_with_errors, test_data::TestData, unwrap_stream, Sequenced,
};

#[tokio::test]
async fn test_ordered_merge_error_propagation_with_map_ordered() -> anyhow::Result<()> {
    // Arrange
    let (tx1, stream1) = test_channel_with_errors::<Sequenced<TestData>>();
    let (tx2, stream2) = test_channel_with_errors::<Sequenced<TestData>>();

    let s1 = FluxionStream::new(stream1).map_ordered(|seq| {
        let inner = seq.into_inner();
        let new_inner = match inner {
            TestData::Person(mut p) => {
                p.age += 1;
                TestData::Person(p)
            }
            other => other,
        };
        Sequenced::new(new_inner)
    });

    let s2 = FluxionStream::new(stream2).map_ordered(|seq| {
        let inner = seq.into_inner();
        let new_inner = match inner {
            TestData::Person(mut p) => {
                p.age += 2;
                TestData::Person(p)
            }
            other => other,
        };
        Sequenced::new(new_inner)
    });

    let mut result = s1.ordered_merge(vec![s2]);

    // Act & Assert
    tx1.send(StreamItem::Value(Sequenced::with_timestamp(
        TestData::Person(Person::new("Alice".to_string(), 10)),
        1,
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(v) if matches!(v.value, TestData::Person(ref p) if p.age == 11)
    ));

    tx2.send(StreamItem::Error(FluxionError::stream_error("Error2")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    tx1.send(StreamItem::Error(FluxionError::stream_error("Error1")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    tx2.send(StreamItem::Value(Sequenced::with_timestamp(
        TestData::Person(Person::new("Bob".to_string(), 20)),
        4,
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(v) if matches!(v.value, TestData::Person(ref p) if p.age == 22)
    ));

    drop(tx1);
    drop(tx2);

    Ok(())
}
