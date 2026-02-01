// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_stream::take_while_with::TakeWhileExt;
use fluxion_test_utils::helpers::{
    assert_no_element_emitted, assert_stream_ended, test_channel, unwrap_stream, unwrap_value,
};
use fluxion_test_utils::sequenced::Sequenced;
use fluxion_test_utils::test_data::{
    animal_cat, animal_dog, person_alice, person_bob, person_charlie, TestData,
};

#[tokio::test]
async fn test_take_while_basic() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();

    let mut result =
        source_stream.take_while_with(filter_stream, |f| matches!(f, TestData::Person(_)));

    // Act
    filter_tx.unbounded_send(Sequenced::new(person_alice()))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    source_tx.unbounded_send(Sequenced::new(animal_cat()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        animal_cat()
    );

    // Act
    source_tx.unbounded_send(Sequenced::new(animal_dog()))?;
    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        animal_dog()
    );

    // Act
    source_tx.unbounded_send(Sequenced::new(person_alice()))?;
    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        person_alice()
    );

    // Act
    filter_tx.unbounded_send(Sequenced::new(animal_dog()))?;
    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    source_tx.unbounded_send(Sequenced::new(person_bob()))?;
    source_tx.unbounded_send(Sequenced::new(person_charlie()))?;
    // Assert
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

    // Act
    filter_tx.unbounded_send(Sequenced::new(person_alice()))?;
    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    source_tx.unbounded_send(Sequenced::new(animal_cat()))?;
    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        animal_cat()
    );

    // Act
    source_tx.unbounded_send(Sequenced::new(animal_dog()))?;
    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        animal_dog()
    );

    // Act
    source_tx.unbounded_send(Sequenced::new(person_alice()))?;
    // Assert
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

    // Act
    filter_tx.unbounded_send(Sequenced::new(person_alice()))?;
    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    source_tx.unbounded_send(Sequenced::new(animal_cat()))?;
    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        animal_cat()
    );

    // Act
    source_tx.unbounded_send(Sequenced::new(animal_dog()))?;
    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        animal_dog()
    );

    // Act
    filter_tx.unbounded_send(Sequenced::new(person_bob()))?;
    source_tx.unbounded_send(Sequenced::new(person_alice()))?;
    source_tx.unbounded_send(Sequenced::new(person_bob()))?;
    // Assert
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

    // Act
    filter_tx.unbounded_send(Sequenced::new(person_alice()))?;
    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    source_tx.unbounded_send(Sequenced::new(animal_cat()))?;
    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        animal_cat()
    );

    // Act
    filter_tx.unbounded_send(Sequenced::new(person_bob()))?;
    source_tx.unbounded_send(Sequenced::new(animal_dog()))?;
    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        animal_dog()
    );

    // Act
    filter_tx.unbounded_send(Sequenced::new(person_charlie()))?;
    source_tx.unbounded_send(Sequenced::new(person_alice()))?;
    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        person_alice()
    );

    // Act
    filter_tx.unbounded_send(Sequenced::new(animal_dog()))?;
    source_tx.unbounded_send(Sequenced::new(person_bob()))?;
    source_tx.unbounded_send(Sequenced::new(person_charlie()))?;
    // Assert
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

    // Act
    filter_tx.unbounded_send(Sequenced::new(person_alice()))?;
    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    source_tx.unbounded_send(Sequenced::new(animal_cat()))?;
    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        animal_cat()
    );

    // Act
    filter_tx.unbounded_send(Sequenced::new(animal_dog()))?;
    source_tx.unbounded_send(Sequenced::new(animal_dog()))?;
    // Assert
    assert_no_element_emitted(&mut result, 500).await;

    // Act
    filter_tx.unbounded_send(Sequenced::new(person_alice()))?;
    source_tx.unbounded_send(Sequenced::new(person_alice()))?;
    // Assert
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

    // Act
    filter_tx.unbounded_send(Sequenced::new(person_alice()))?;
    source_tx.unbounded_send(Sequenced::new(animal_cat()))?;
    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        animal_cat()
    );

    // Act
    source_tx.unbounded_send(Sequenced::new(animal_dog()))?;
    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        animal_dog()
    );

    // Act
    source_tx.unbounded_send(Sequenced::new(person_alice()))?;
    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        person_alice()
    );

    // Act
    source_tx.unbounded_send(Sequenced::new(person_bob()))?;
    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        person_bob()
    );

    // Act
    filter_tx.unbounded_send(Sequenced::new(animal_dog()))?;
    source_tx.unbounded_send(Sequenced::new(person_charlie()))?;
    // Assert
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

    // Act
    filter_tx.unbounded_send(Sequenced::new(animal_dog()))?; // False
    filter_tx.unbounded_send(Sequenced::new(person_alice()))?; // True
    filter_tx.unbounded_send(Sequenced::new(person_bob()))?; // True
                                                             // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    source_tx.unbounded_send(Sequenced::new(animal_cat()))?;
    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        animal_cat()
    );

    // Act
    source_tx.unbounded_send(Sequenced::new(animal_dog()))?;
    // Assert
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

    // Act
    source_tx.unbounded_send(Sequenced::new(person_alice()))?;
    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    filter_tx.unbounded_send(Sequenced::new(person_alice()))?;
    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    source_tx.unbounded_send(Sequenced::new(person_alice()))?;
    // Assert
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

    // Act
    filter_tx.unbounded_send(Sequenced::new(person_alice()))?;
    filter_tx.unbounded_send(Sequenced::new(animal_dog()))?;
    filter_tx.unbounded_send(Sequenced::new(person_alice()))?;
    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    source_tx.unbounded_send(Sequenced::new(person_alice()))?;
    // Assert
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

    // Act
    filter_tx.unbounded_send(Sequenced::new(person_alice()))?;
    source_tx.unbounded_send(Sequenced::new(person_alice()))?;
    // Assert
    assert!(matches!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        TestData::Person(_)
    ));

    // Act
    filter_tx.unbounded_send(Sequenced::new(animal_dog()))?;
    source_tx.unbounded_send(Sequenced::new(person_alice()))?;
    // Assert
    assert_no_element_emitted(&mut result, 500).await;

    // Act
    source_tx.unbounded_send(Sequenced::new(person_alice()))?;
    filter_tx.unbounded_send(Sequenced::new(person_alice()))?;
    // Assert
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

    // Act
    filter_tx.unbounded_send(Sequenced::new(animal_dog()))?;
    source_tx.unbounded_send(Sequenced::new(person_alice()))?;
    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    drop(source_tx);
    drop(filter_tx);
    // Assert
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

    // Act
    filter_tx.unbounded_send(Sequenced::with_timestamp(person_alice(), 1))?;
    source_tx.unbounded_send(Sequenced::with_timestamp(person_alice(), 2))?;
    // Assert
    assert!(matches!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        TestData::Person(_)
    ));

    // Act
    filter_tx.unbounded_send(Sequenced::with_timestamp(person_alice(), 3))?;
    source_tx.unbounded_send(Sequenced::with_timestamp(person_alice(), 4))?;
    // Assert
    assert!(matches!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        TestData::Person(_)
    ));

    // Act
    filter_tx.unbounded_send(Sequenced::with_timestamp(person_alice(), 5))?;
    source_tx.unbounded_send(Sequenced::with_timestamp(person_alice(), 6))?;
    // Assert
    assert!(matches!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        TestData::Person(_)
    ));

    Ok(())
}
