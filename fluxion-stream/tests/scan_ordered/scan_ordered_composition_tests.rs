// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_stream::FluxionStream;
use fluxion_test_utils::{
    helpers::unwrap_stream,
    test_channel,
    test_data::{person_alice, person_bob, person_charlie, TestData},
    unwrap_value, Sequenced,
};

#[tokio::test]
async fn test_scan_ordered_chained() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();

    // First scan: running sum of ages
    // Second scan: count of emissions
    let mut result = FluxionStream::new(stream)
        .scan_ordered::<Sequenced<i32>, _, _>(0, |sum: &mut i32, value: &TestData| {
            if let TestData::Person(p) = value {
                *sum += p.age as i32;
            }
            *sum
        })
        .scan_ordered(0, |count: &mut i32, _sum: &i32| {
            *count += 1;
            *count
        });

    // Act & Assert
    tx.send(Sequenced::new(person_alice()))?; // age=25, sum=25, count=1
    assert_eq!(
        unwrap_value(Some(
            unwrap_stream::<Sequenced<i32>, _>(&mut result, 500).await,
        ))
        .value,
        1
    );

    tx.send(Sequenced::new(person_bob()))?; // age=28, sum=53, count=2
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        2
    );

    tx.send(Sequenced::new(person_charlie()))?; // age=35, sum=88, count=3
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        3
    );

    drop(tx);

    Ok(())
}
