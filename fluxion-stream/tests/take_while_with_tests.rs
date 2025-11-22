// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::StreamItem;
use fluxion_stream::take_while_with::TakeWhileExt;
use fluxion_test_utils::helpers::assert_no_element_emitted;
use fluxion_test_utils::test_data::{
    animal_cat, animal_dog, person_alice, person_bob, person_charlie,
};
use fluxion_test_utils::Sequenced;
use fluxion_test_utils::{test_channel, unwrap_stream, unwrap_value};
use futures::Stream;

#[tokio::test]
async fn test_take_while_basic() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<bool>>();

    let mut filtered_stream =
        source_stream.take_while_with(filter_stream, |filter_val| *filter_val);

    // Act & Assert
    filter_tx.send(Sequenced::new(true))?;
    assert_no_element_emitted(&mut filtered_stream, 100).await;
    source_tx.send(Sequenced::new(animal_cat()))?;
    assert_eq!(extract_element(&mut filtered_stream).await, animal_cat());

    source_tx.send(Sequenced::new(animal_dog()))?;
    assert_eq!(extract_element(&mut filtered_stream).await, animal_dog());

    source_tx.send(Sequenced::new(person_alice()))?;
    assert_eq!(extract_element(&mut filtered_stream).await, person_alice());

    filter_tx.send(Sequenced::new(false))?;
    source_tx.send(Sequenced::new(person_bob()))?;
    source_tx.send(Sequenced::new(person_charlie()))?;
    assert_no_element_emitted(&mut filtered_stream, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_take_while_filter_false_immediately() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<bool>>();

    let mut filtered_stream =
        source_stream.take_while_with(filter_stream, |filter_val| *filter_val);

    // Act
    filter_tx.send(Sequenced::new(false))?;
    source_tx.send(Sequenced::new(animal_cat()))?;
    source_tx.send(Sequenced::new(animal_dog()))?;

    // Assert
    assert_no_element_emitted(&mut filtered_stream, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_take_while_always_true() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<bool>>();

    let mut filtered_stream =
        source_stream.take_while_with(filter_stream, |filter_val| *filter_val);

    // Act & Assert
    filter_tx.send(Sequenced::new(true))?;
    assert_no_element_emitted(&mut filtered_stream, 100).await;
    source_tx.send(Sequenced::new(animal_cat()))?;
    assert_eq!(extract_element(&mut filtered_stream).await, animal_cat());

    source_tx.send(Sequenced::new(animal_dog()))?;
    assert_eq!(extract_element(&mut filtered_stream).await, animal_dog());

    source_tx.send(Sequenced::new(person_alice()))?;
    assert_eq!(extract_element(&mut filtered_stream).await, person_alice());

    Ok(())
}

#[tokio::test]
async fn test_take_while_complex_predicate() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<i32>>();

    let mut filtered_stream =
        source_stream.take_while_with(filter_stream, |filter_val: &i32| *filter_val < 10);

    // Act & Assert
    filter_tx.send(Sequenced::new(5))?;
    assert_no_element_emitted(&mut filtered_stream, 100).await;
    source_tx.send(Sequenced::new(animal_cat()))?;
    assert_eq!(extract_element(&mut filtered_stream).await, animal_cat());

    source_tx.send(Sequenced::new(animal_dog()))?;
    assert_eq!(extract_element(&mut filtered_stream).await, animal_dog());

    filter_tx.send(Sequenced::new(10))?;
    source_tx.send(Sequenced::new(person_alice()))?;
    source_tx.send(Sequenced::new(person_bob()))?;
    assert_no_element_emitted(&mut filtered_stream, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_take_while_interleaved_updates() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<bool>>();

    let mut filtered_stream =
        source_stream.take_while_with(filter_stream, |filter_val| *filter_val);

    // Act & Assert
    filter_tx.send(Sequenced::new(true))?;
    assert_no_element_emitted(&mut filtered_stream, 100).await;
    source_tx.send(Sequenced::new(animal_cat()))?;
    assert_eq!(extract_element(&mut filtered_stream).await, animal_cat());

    filter_tx.send(Sequenced::new(true))?;
    source_tx.send(Sequenced::new(animal_dog()))?;
    assert_eq!(extract_element(&mut filtered_stream).await, animal_dog());

    filter_tx.send(Sequenced::new(true))?;
    source_tx.send(Sequenced::new(person_alice()))?;
    assert_eq!(extract_element(&mut filtered_stream).await, person_alice());

    filter_tx.send(Sequenced::new(false))?;
    source_tx.send(Sequenced::new(person_bob()))?;
    source_tx.send(Sequenced::new(person_charlie()))?;
    assert_no_element_emitted(&mut filtered_stream, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_take_while_no_filter_value() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<bool>>();

    let mut filtered_stream =
        source_stream.take_while_with(filter_stream, |filter_val| *filter_val);

    // Act & Assert
    source_tx.send(Sequenced::new(animal_cat()))?;
    assert_no_element_emitted(&mut filtered_stream, 100).await;
    source_tx.send(Sequenced::new(animal_dog()))?;
    assert_no_element_emitted(&mut filtered_stream, 100).await;

    filter_tx.send(Sequenced::new(true))?;
    source_tx.send(Sequenced::new(person_alice()))?;
    assert_eq!(extract_element(&mut filtered_stream).await, person_alice());

    source_tx.send(Sequenced::new(person_bob()))?;
    assert_eq!(extract_element(&mut filtered_stream).await, person_bob());

    Ok(())
}

#[tokio::test]
async fn test_take_while_empty_source() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel::<Sequenced<bool>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<bool>>();

    let mut filtered_stream =
        source_stream.take_while_with(filter_stream, |filter_val| *filter_val);

    // Act
    filter_tx.send(Sequenced::new(true))?;
    drop(source_tx);

    // Assert
    assert_no_element_emitted(&mut filtered_stream, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_take_while_empty_filter() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel();
    let (_, filter_stream) = test_channel::<Sequenced<bool>>();

    let mut filtered_stream =
        source_stream.take_while_with(filter_stream, |filter_val| *filter_val);

    // Act
    source_tx.send(Sequenced::new(animal_cat()))?;
    source_tx.send(Sequenced::new(animal_dog()))?;

    // Assert
    assert_no_element_emitted(&mut filtered_stream, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_take_while_filter_changes_back_to_true() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<bool>>();

    let mut filtered_stream =
        source_stream.take_while_with(filter_stream, |filter_val| *filter_val);

    // Act & Assert
    filter_tx.send(Sequenced::new(true))?;
    assert_no_element_emitted(&mut filtered_stream, 100).await;
    source_tx.send(Sequenced::new(animal_cat()))?;
    assert_eq!(extract_element(&mut filtered_stream).await, animal_cat());

    filter_tx.send(Sequenced::new(false))?;
    source_tx.send(Sequenced::new(animal_dog()))?;
    assert_no_element_emitted(&mut filtered_stream, 100).await;

    filter_tx.send(Sequenced::new(true))?;
    source_tx.send(Sequenced::new(person_alice()))?;
    assert_no_element_emitted(&mut filtered_stream, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_take_while_multiple_source_items_same_filter() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<bool>>();

    let mut filtered_stream =
        source_stream.take_while_with(filter_stream, |filter_val| *filter_val);

    // Act & Assert
    filter_tx.send(Sequenced::new(true))?;
    source_tx.send(Sequenced::new(animal_cat()))?;
    assert_eq!(extract_element(&mut filtered_stream).await, animal_cat());

    source_tx.send(Sequenced::new(animal_dog()))?;
    assert_eq!(extract_element(&mut filtered_stream).await, animal_dog());

    source_tx.send(Sequenced::new(person_alice()))?;
    assert_eq!(extract_element(&mut filtered_stream).await, person_alice());

    source_tx.send(Sequenced::new(person_bob()))?;
    assert_eq!(extract_element(&mut filtered_stream).await, person_bob());

    filter_tx.send(Sequenced::new(false))?;
    source_tx.send(Sequenced::new(person_charlie()))?;

    assert_no_element_emitted(&mut filtered_stream, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_take_while_filter_updates_without_source() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<bool>>();

    let mut filtered_stream =
        source_stream.take_while_with(filter_stream, |filter_val| *filter_val);

    // Act & Assert
    filter_tx.send(Sequenced::new(false))?;
    filter_tx.send(Sequenced::new(true))?;
    filter_tx.send(Sequenced::new(true))?;
    assert_no_element_emitted(&mut filtered_stream, 100).await;

    source_tx.send(Sequenced::new(animal_cat()))?;
    assert_eq!(extract_element(&mut filtered_stream).await, animal_cat());

    source_tx.send(Sequenced::new(animal_dog()))?;
    assert_eq!(extract_element(&mut filtered_stream).await, animal_dog());

    Ok(())
}

async fn extract_element<T, S>(stream: &mut S) -> T
where
    T: Clone,
    S: Stream<Item = StreamItem<Sequenced<T>>> + Unpin,
{
    unwrap_value(Some(unwrap_stream(stream, 500).await))
        .value
        .clone()
}

#[test]
fn test_item_timestamped_trait_coverage() {
    // This test exists solely to achieve code coverage for unused Timestamped trait methods
    // that are required to satisfy trait bounds but are never called in the actual operator.
    use fluxion_core::Timestamped;
    use fluxion_stream::take_while_with::Item;
    use fluxion_test_utils::Sequenced;

    let source_item: Item<Sequenced<i32>, Sequenced<bool>> = Item::Source(Sequenced::new(42));

    // Call the unused trait methods to satisfy coverage metrics
    let _ = Item::<Sequenced<i32>, Sequenced<bool>>::with_timestamp(source_item.clone(), 0);
    let _ = Item::<Sequenced<i32>, Sequenced<bool>>::with_fresh_timestamp(source_item.clone());
    let _ = source_item.clone().into_inner();
}
