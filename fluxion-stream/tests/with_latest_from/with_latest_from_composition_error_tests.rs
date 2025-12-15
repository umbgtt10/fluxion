// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, HasTimestamp, StreamItem};

use fluxion_stream::{MapOrderedExt, WithLatestFromExt};
use fluxion_test_utils::{person::Person, test_channel_with_errors, unwrap_stream, Sequenced};

#[tokio::test]
async fn test_with_latest_from_error_propagation_at_end_of_chain() -> anyhow::Result<()> {
    // Arrange
    let (tx1, stream1) = test_channel_with_errors::<Sequenced<Person>>();
    let (tx2, stream2) = test_channel_with_errors::<Sequenced<Person>>();

    let mut result = stream1
        .map_ordered(|x| {
            let ts = HasTimestamp::timestamp(&x);
            let mut p = x.into_inner();
            p.age *= 2;
            Sequenced::with_timestamp(p, ts)
        })
        .with_latest_from(stream2, |state| {
            let values = state.values();
            let age_sum = values[0].age + values[1].age;
            Sequenced::new(Person::new("Combined".to_string(), age_sum))
        });

    // Act
    tx2.send(StreamItem::Value(Sequenced::with_timestamp(
        Person::new("Secondary".to_string(), 10),
        0,
    )))?;
    tx2.send(StreamItem::Value(Sequenced::with_timestamp(
        Person::new("Secondary".to_string(), 999),
        1000,
    )))?;

    tx1.send(StreamItem::Value(Sequenced::with_timestamp(
        Person::new("Main".to_string(), 5),
        1,
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value.age == 20
    ));

    tx1.send(StreamItem::Error(FluxionError::stream_error("Error1")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    tx1.send(StreamItem::Value(Sequenced::with_timestamp(
        Person::new("Main".to_string(), 6),
        2,
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value.age == 22
    ));

    drop(tx1);
    drop(tx2);

    Ok(())
}
