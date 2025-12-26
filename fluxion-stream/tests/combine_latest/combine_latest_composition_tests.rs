// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::Timestamped;
use fluxion_stream::prelude::*;
use fluxion_stream::CombinedState;
use fluxion_test_utils::{
    test_channel,
    test_data::{animal_dog, person_alice, plant_rose},
    unwrap_stream, unwrap_value, Sequenced, TestData,
};

static COMBINE_FILTER: fn(&CombinedState<TestData, u64>) -> bool = |_| true;

#[tokio::test]
async fn test_fluxion_stream_combine_latest_composition() -> anyhow::Result<()> {
    // Arrange
    let (person_tx, person_stream) = test_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_stream) = test_channel::<Sequenced<TestData>>();
    let (plant_tx, plant_stream) = test_channel::<Sequenced<TestData>>();

    let mut combined =
        person_stream.combine_latest(vec![animal_stream, plant_stream], COMBINE_FILTER);

    // Act
    person_tx.unbounded_send(Sequenced::new(person_alice()))?;
    animal_tx.unbounded_send(Sequenced::new(animal_dog()))?;
    plant_tx.unbounded_send(Sequenced::new(plant_rose()))?;

    // Assert
    let element = unwrap_value(Some(unwrap_stream(&mut combined, 500).await));
    let inner = element.clone().into_inner();
    let state = inner.values();
    assert_eq!(state.len(), 3);
    assert_eq!(state[0], person_alice());
    assert_eq!(state[1], animal_dog());
    assert_eq!(state[2], plant_rose());

    Ok(())
}
