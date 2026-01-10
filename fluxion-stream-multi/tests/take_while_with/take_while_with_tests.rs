// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_stream_multi::take_while_with::TakeWhileExt;
use fluxion_test_utils::helpers::{assert_no_element_emitted, assert_stream_ended};
use fluxion_test_utils::test_data::{
    animal_cat, animal_dog, person_alice, person_bob, person_charlie, TestData,
};
use fluxion_test_utils::{test_channel, unwrap_stream, unwrap_value, Sequenced};

#[tokio::test]
async fn test_take_while_basic() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();

    let mut result =
        source_stream.take_while_with(filter_stream, |f| matches!(f, TestData::Person(_)));

    // Act & Assert
    filter_tx.unbounded_send(Sequenced::new(person_alice()))?;
    assert_no_element_emitted(&mut result, 100).await;

    source_tx.unbounded_send(Sequenced::new(animal_cat()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        animal_cat()
    );

    source_tx.unbounded_send(Sequenced::new(animal_dog()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        animal_dog()
    );

    source_tx.unbounded_send(Sequenced::new(person_alice()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        person_alice()
    );

    filter_tx.unbounded_send(Sequenced::new(animal_dog()))?;
    assert_no_element_emitted(&mut result, 100).await;

    source_tx.unbounded_send(Sequenced::new(person_bob()))?;
    source_tx.unbounded_send(Sequenced::new(person_charlie()))?;
    assert_no_element_emitted(&mut result, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_take_while_always_true() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();

    let mut result =
        source_stream.take_while_with(filter_stream, |f| matches!(f, TestData::Person(_)));

    // Act & Assert
    filter_tx.unbounded_send(Sequenced::new(person_alice()))?;
    assert_no_element_emitted(&mut result, 100).await;

    source_tx.unbounded_send(Sequenced::new(animal_cat()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        animal_cat()
    );

    source_tx.unbounded_send(Sequenced::new(animal_dog()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        animal_dog()
    );
    source_tx.unbounded_send(Sequenced::new(person_alice()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        person_alice()
    );

    Ok(())
}

#[tokio::test]
async fn test_take_while_complex_predicate() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();

    // Predicate: Allow only if Person name is "Alice"
    let mut result = source_stream.take_while_with(filter_stream, |f| match f {
        TestData::Person(p) => p.name == "Alice",
        _ => false,
    });

    // Act & Assert
    // 1. Filter = Alice (True)
    filter_tx.unbounded_send(Sequenced::new(person_alice()))?;
    assert_no_element_emitted(&mut result, 100).await;

    source_tx.unbounded_send(Sequenced::new(animal_cat()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        animal_cat()
    );

    source_tx.unbounded_send(Sequenced::new(animal_dog()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        animal_dog()
    );

    // 2. Filter = Bob (False)
    filter_tx.unbounded_send(Sequenced::new(person_bob()))?;
    source_tx.unbounded_send(Sequenced::new(person_alice()))?;
    source_tx.unbounded_send(Sequenced::new(person_bob()))?;
    assert_no_element_emitted(&mut result, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_take_while_interleaved_updates() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();

    let mut result =
        source_stream.take_while_with(filter_stream, |f| matches!(f, TestData::Person(_)));

    // Act & Assert
    // 1. Filter True
    filter_tx.unbounded_send(Sequenced::new(person_alice()))?;
    assert_no_element_emitted(&mut result, 100).await;

    source_tx.unbounded_send(Sequenced::new(animal_cat()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        animal_cat()
    );

    // 2. Filter True (update)
    filter_tx.unbounded_send(Sequenced::new(person_bob()))?;
    source_tx.unbounded_send(Sequenced::new(animal_dog()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        animal_dog()
    );

    // 3. Filter True (update)
    filter_tx.unbounded_send(Sequenced::new(person_charlie()))?;
    source_tx.unbounded_send(Sequenced::new(person_alice()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        person_alice()
    );

    // 4. Filter False
    filter_tx.unbounded_send(Sequenced::new(animal_dog()))?;
    source_tx.unbounded_send(Sequenced::new(person_bob()))?;
    source_tx.unbounded_send(Sequenced::new(person_charlie()))?;
    assert_no_element_emitted(&mut result, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_take_while_empty_source() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();

    let mut result =
        source_stream.take_while_with(filter_stream, |f| matches!(f, TestData::Person(_)));

    // Act
    filter_tx.unbounded_send(Sequenced::new(person_alice()))?;
    drop(source_tx);
    drop(filter_tx);

    // Assert
    assert_stream_ended(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_take_while_filer_unset() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (_, filter_stream) = test_channel::<Sequenced<TestData>>();

    let mut result =
        source_stream.take_while_with(filter_stream, |f| matches!(f, TestData::Person(_)));

    // Act
    source_tx.unbounded_send(Sequenced::new(animal_cat()))?;
    source_tx.unbounded_send(Sequenced::new(animal_dog()))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_take_while_filter_changes_back_to_true() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();

    let mut result =
        source_stream.take_while_with(filter_stream, |f| matches!(f, TestData::Person(_)));

    // Act & Assert
    filter_tx.unbounded_send(Sequenced::new(person_alice()))?;
    assert_no_element_emitted(&mut result, 100).await;

    source_tx.unbounded_send(Sequenced::new(animal_cat()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        animal_cat()
    );

    filter_tx.unbounded_send(Sequenced::new(animal_dog()))?;
    source_tx.unbounded_send(Sequenced::new(animal_dog()))?;
    assert_no_element_emitted(&mut result, 500).await;

    filter_tx.unbounded_send(Sequenced::new(person_alice()))?;
    source_tx.unbounded_send(Sequenced::new(person_alice()))?;
    assert_no_element_emitted(&mut result, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_take_while_multiple_source_items_same_filter() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();

    let mut result =
        source_stream.take_while_with(filter_stream, |f| matches!(f, TestData::Person(_)));

    // Act & Assert
    filter_tx.unbounded_send(Sequenced::new(person_alice()))?;
    source_tx.unbounded_send(Sequenced::new(animal_cat()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        animal_cat()
    );

    source_tx.unbounded_send(Sequenced::new(animal_dog()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        animal_dog()
    );

    source_tx.unbounded_send(Sequenced::new(person_alice()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        person_alice()
    );

    source_tx.unbounded_send(Sequenced::new(person_bob()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        person_bob()
    );

    filter_tx.unbounded_send(Sequenced::new(animal_dog()))?;
    source_tx.unbounded_send(Sequenced::new(person_charlie()))?;
    assert_no_element_emitted(&mut result, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_take_while_filter_updates_without_source() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();

    let mut result =
        source_stream.take_while_with(filter_stream, |f| matches!(f, TestData::Person(_)));

    // Act & Assert
    filter_tx.unbounded_send(Sequenced::new(animal_dog()))?; // False
    filter_tx.unbounded_send(Sequenced::new(person_alice()))?; // True
    filter_tx.unbounded_send(Sequenced::new(person_bob()))?; // True
    assert_no_element_emitted(&mut result, 100).await;

    source_tx.unbounded_send(Sequenced::new(animal_cat()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        animal_cat()
    );

    source_tx.unbounded_send(Sequenced::new(animal_dog()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        animal_dog()
    );
    Ok(())
}

#[tokio::test]
async fn test_take_while_with_source_before_filter() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();

    let mut result =
        source_stream.take_while_with(filter_stream, |f| matches!(f, TestData::Person(_)));

    // Act & Assert
    source_tx.unbounded_send(Sequenced::new(person_alice()))?;
    assert_no_element_emitted(&mut result, 100).await;

    filter_tx.unbounded_send(Sequenced::new(person_alice()))?;
    assert_no_element_emitted(&mut result, 100).await;

    source_tx.unbounded_send(Sequenced::new(person_alice()))?;
    assert!(matches!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        TestData::Person(_)
    ));

    Ok(())
}

#[tokio::test]
async fn test_take_while_with_filter_updates_then_source() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();

    let mut result =
        source_stream.take_while_with(filter_stream, |f| matches!(f, TestData::Person(_)));

    // Act & Assert
    filter_tx.unbounded_send(Sequenced::new(person_alice()))?;
    filter_tx.unbounded_send(Sequenced::new(animal_dog()))?;
    filter_tx.unbounded_send(Sequenced::new(person_alice()))?;
    assert_no_element_emitted(&mut result, 100).await;

    source_tx.unbounded_send(Sequenced::new(person_alice()))?;
    assert!(matches!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        TestData::Person(_)
    ));

    Ok(())
}

#[tokio::test]
async fn test_take_while_with_already_terminated() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();

    let mut result =
        source_stream.take_while_with(filter_stream, |f| matches!(f, TestData::Person(_)));

    // Act & Assert
    filter_tx.unbounded_send(Sequenced::new(person_alice()))?;
    source_tx.unbounded_send(Sequenced::new(person_alice()))?;
    assert!(matches!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        TestData::Person(_)
    ));

    filter_tx.unbounded_send(Sequenced::new(animal_dog()))?;
    source_tx.unbounded_send(Sequenced::new(person_alice()))?;
    assert_no_element_emitted(&mut result, 500).await;

    source_tx.unbounded_send(Sequenced::new(person_alice()))?;
    filter_tx.unbounded_send(Sequenced::new(person_alice()))?;
    assert_no_element_emitted(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_take_while_with_filter_false_on_first_source() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();

    let mut result =
        source_stream.take_while_with(filter_stream, |f| matches!(f, TestData::Person(_)));

    // Act & Assert
    filter_tx.unbounded_send(Sequenced::new(animal_dog()))?;
    source_tx.unbounded_send(Sequenced::new(person_alice()))?;
    assert_no_element_emitted(&mut result, 100).await;

    drop(source_tx);
    drop(filter_tx);
    assert_stream_ended(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_take_while_with_alternating_filter_values() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();

    let mut result =
        source_stream.take_while_with(filter_stream, |f| matches!(f, TestData::Person(_)));

    // Act & Assert
    filter_tx.unbounded_send(Sequenced::with_timestamp(person_alice(), 1))?;
    source_tx.unbounded_send(Sequenced::with_timestamp(person_alice(), 2))?;
    assert!(matches!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        TestData::Person(_)
    ));

    filter_tx.unbounded_send(Sequenced::with_timestamp(person_alice(), 3))?;
    source_tx.unbounded_send(Sequenced::with_timestamp(person_alice(), 4))?;
    assert!(matches!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        TestData::Person(_)
    ));

    filter_tx.unbounded_send(Sequenced::with_timestamp(person_alice(), 5))?;
    source_tx.unbounded_send(Sequenced::with_timestamp(person_alice(), 6))?;
    assert!(matches!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        TestData::Person(_)
    ));

    Ok(())
}
