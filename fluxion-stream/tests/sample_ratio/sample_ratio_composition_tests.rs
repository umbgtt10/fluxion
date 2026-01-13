// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Composition tests for `sample_ratio` operator.

use fluxion_stream::{FilterOrderedExt, MapOrderedExt, SampleRatioExt};
use fluxion_test_utils::person::Person;
use fluxion_test_utils::test_data::{
    animal_bird, animal_cat, animal_dog, person_alice, person_bob, person_charlie, person_dave,
    person_diane, plant_rose, TestData,
};
use fluxion_test_utils::{assert_no_element_emitted, Sequenced};
use fluxion_test_utils::{helpers::unwrap_stream, unwrap_value};
use fluxion_test_utils::{test_channel, unwrap_all};

#[tokio::test]
async fn test_sample_ratio_after_filter_ordered() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel();
    let mut result = stream
        .filter_ordered(|item| matches!(item, TestData::Person(_)))
        .sample_ratio(1.0, 42);

    // Act & Assert
    tx.try_send(Sequenced::new(person_alice()))?;
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        &person_alice()
    );

    tx.try_send(Sequenced::new(animal_dog()))?; // Filtered out
    tx.try_send(Sequenced::new(person_bob()))?;
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        &person_bob()
    );

    Ok(())
}

#[tokio::test]
async fn test_sample_ratio_before_filter_ordered() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel();
    let mut result = stream
        .sample_ratio(1.0, 42)
        .filter_ordered(|item| matches!(item, TestData::Person(_)));

    // Act & Assert
    tx.try_send(Sequenced::new(person_alice()))?;
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        &person_alice()
    );

    tx.try_send(Sequenced::new(animal_dog()))?; // Sampled but then filtered
    tx.try_send(Sequenced::new(person_bob()))?;
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        &person_bob()
    );

    Ok(())
}

#[tokio::test]
async fn test_sample_ratio_after_map_ordered() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel();
    let mut result = stream
        .map_ordered(|item: Sequenced<TestData>| {
            Sequenced::new(match item.value {
                TestData::Person(p) => TestData::Person(Person {
                    name: format!("Dr. {}", p.name),
                    ..p
                }),
                other => other,
            })
        })
        .sample_ratio(1.0, 42);

    // Act
    tx.try_send(Sequenced::new(person_alice()))?;

    // Assert
    assert!(matches!(
        &unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        TestData::Person(p) if p.name.starts_with("Dr. ")
    ));

    Ok(())
}

#[tokio::test]
async fn test_sample_ratio_before_map_ordered() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel();
    let mut result = stream
        .sample_ratio(1.0, 42)
        .map_ordered(|item: Sequenced<TestData>| {
            Sequenced::new(match item.value {
                TestData::Person(p) => TestData::Person(Person {
                    name: format!("Sampled {}", p.name),
                    ..p
                }),
                other => other,
            })
        });

    // Act
    tx.try_send(Sequenced::new(person_alice()))?;

    // Assert
    assert!(matches!(
        &unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        TestData::Person(p) if p.name.starts_with("Sampled ")
    ));

    Ok(())
}

#[tokio::test]
async fn test_chained_sample_ratios() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel();
    let mut result = stream.sample_ratio(1.0, 42).sample_ratio(1.0, 99);

    // Act & Assert
    tx.try_send(Sequenced::new(person_alice()))?;
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        &person_alice()
    );

    tx.try_send(Sequenced::new(person_bob()))?;
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        &person_bob()
    );

    Ok(())
}

#[tokio::test]
async fn test_chained_sample_ratios_zero_in_chain() -> anyhow::Result<()> {
    // Arrange - second sample_ratio has ratio 0, so nothing passes
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut result = stream.sample_ratio(1.0, 42).sample_ratio(0.0, 99);

    // Act
    tx.try_send(Sequenced::new(person_alice()))?;
    tx.try_send(Sequenced::new(person_bob()))?;

    // Assert
    assert_no_element_emitted(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_sample_ratio_complex_pipeline() -> anyhow::Result<()> {
    // Arrange - filter -> sample -> map
    let (tx, stream) = test_channel();
    let mut result = stream
        .filter_ordered(|item| matches!(item, TestData::Person(_)))
        .sample_ratio(1.0, 42)
        .map_ordered(|item: Sequenced<TestData>| {
            Sequenced::new(match item.value {
                TestData::Person(p) => TestData::Person(Person {
                    name: format!("Pipeline: {}", p.name),
                    ..p
                }),
                other => other,
            })
        });

    // Act
    tx.try_send(Sequenced::new(animal_dog()))?; // Filtered
    tx.try_send(Sequenced::new(person_alice()))?; // Passes filter, sampled, mapped

    // Assert
    assert!(matches!(
        &unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        TestData::Person(p) if p.name.starts_with("Pipeline: ")
    ));

    Ok(())
}

#[tokio::test]
async fn test_sample_ratio_maintains_determinism_in_composition() -> anyhow::Result<()> {
    // Arrange - verify that composed pipeline produces deterministic results
    let items = vec![
        person_alice(),
        person_bob(),
        person_charlie(),
        person_dave(),
        person_diane(),
        animal_dog(),
        animal_cat(),
        animal_bird(),
        plant_rose(),
    ];

    // Run 1
    let (tx1, stream1) = test_channel();
    let (tx2, stream2) = test_channel();
    let mut result1 = stream1
        .filter_ordered(|item| matches!(item, TestData::Person(_)))
        .sample_ratio(0.5, 12345);
    let mut result2 = stream2
        .filter_ordered(|item| matches!(item, TestData::Person(_)))
        .sample_ratio(0.5, 12345);

    for item in &items {
        tx1.try_send(Sequenced::new(item.clone()))?;
        tx2.try_send(Sequenced::new(item.clone()))?;
    }

    // Assert
    assert_eq!(
        unwrap_all(&mut result1, 100)
            .await
            .into_iter()
            .map(|s| s.value)
            .collect::<Vec<_>>(),
        unwrap_all(&mut result2, 100)
            .await
            .into_iter()
            .map(|s| s.value)
            .collect::<Vec<_>>(),
    );

    Ok(())
}
