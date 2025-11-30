// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_stream::FluxionStream;
use fluxion_test_utils::{
    helpers::unwrap_stream,
    test_channel,
    test_data::{animal_dog, person_alice, person_bob, plant_rose, TestData},
    unwrap_value, Sequenced,
};

#[tokio::test]
async fn test_ordered_merge_filter_ordered() -> anyhow::Result<()> {
    // Arrange - merge two streams, then filter for specific types
    let (s1_tx, s1_rx) = test_channel::<Sequenced<TestData>>();
    let (s2_tx, s2_rx) = test_channel::<Sequenced<TestData>>();

    let mut stream = FluxionStream::new(s1_rx)
        .ordered_merge(vec![FluxionStream::new(s2_rx)])
        .filter_ordered(|test_data| !matches!(test_data, TestData::Animal(_))); // Filter out animals

    // Act & Assert
    s1_tx.send(Sequenced::new(person_alice()))?;
    {
        let val = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
        assert_eq!(&val.value, &person_alice());
    }

    s2_tx.send(Sequenced::new(animal_dog()))?; // Filtered out
    s1_tx.send(Sequenced::new(plant_rose()))?;
    {
        let val = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
        assert_eq!(&val.value, &plant_rose());
    }

    s2_tx.send(Sequenced::new(person_bob()))?;
    {
        let val = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
        assert_eq!(&val.value, &person_bob());
    }

    Ok(())
}
