// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_stream::take_while_with::TakeWhileExt;
use fluxion_test_utils::helpers::assert_no_element_emitted;
use fluxion_test_utils::sequenced::Sequenced;
use fluxion_test_utils::test_channel;
use fluxion_test_utils::test_data::{
    animal_cat, animal_dog, person_alice, person_bob, person_charlie,
};
use futures::StreamExt;

#[tokio::test]
async fn test_take_while_basic() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<bool>>();

    let result_stream = source_stream.take_while_with(filter_stream, |filter_val| *filter_val);
    let mut result_stream = Box::pin(result_stream);

    // Act & Assert
    filter_tx.send(Sequenced::new(true))?;
    source_tx.send(Sequenced::new(animal_cat()))?;
    let item = result_stream
        .next()
        .await
        .expect("expected first item")
        .unwrap()
        .get()
        .clone();
    assert_eq!(item, animal_cat());

    source_tx.send(Sequenced::new(animal_dog()))?;
    let item = result_stream
        .next()
        .await
        .expect("expected second item")
        .unwrap()
        .get()
        .clone();
    assert_eq!(item, animal_dog());

    source_tx.send(Sequenced::new(person_alice()))?;
    let item = result_stream
        .next()
        .await
        .expect("expected third item")
        .unwrap()
        .get()
        .clone();
    assert_eq!(item, person_alice());

    filter_tx.send(Sequenced::new(false))?;
    source_tx.send(Sequenced::new(person_bob()))?;
    source_tx.send(Sequenced::new(person_charlie()))?;
    assert_no_element_emitted(&mut result_stream, 100).await;
    Ok(())
}

#[tokio::test]
async fn test_take_while_filter_false_immediately() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<bool>>();

    let result_stream = source_stream.take_while_with(filter_stream, |filter_val| *filter_val);
    let mut result_stream = Box::pin(result_stream);

    // Act & Assert
    filter_tx.send(Sequenced::new(false))?;
    source_tx.send(Sequenced::new(animal_cat()))?;
    source_tx.send(Sequenced::new(animal_dog()))?;
    assert_no_element_emitted(&mut result_stream, 100).await;
    Ok(())
}

#[tokio::test]
async fn test_take_while_always_true() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<bool>>();

    let result_stream = source_stream.take_while_with(filter_stream, |filter_val| *filter_val);
    let mut result_stream = Box::pin(result_stream);

    // Act & Assert
    filter_tx.send(Sequenced::new(true))?;
    source_tx.send(Sequenced::new(animal_cat()))?;
    let item = result_stream
        .next()
        .await
        .expect("expected first item")
        .unwrap()
        .get()
        .clone();
    assert_eq!(item, animal_cat());

    source_tx.send(Sequenced::new(animal_dog()))?;
    let item = result_stream
        .next()
        .await
        .expect("expected second item")
        .unwrap()
        .get()
        .clone();
    assert_eq!(item, animal_dog());

    source_tx.send(Sequenced::new(person_alice()))?;
    let item = result_stream
        .next()
        .await
        .expect("expected third item")
        .unwrap()
        .get()
        .clone();
    assert_eq!(item, person_alice());
    Ok(())
}

#[tokio::test]
async fn test_take_while_complex_predicate() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<i32>>();

    let result_stream =
        source_stream.take_while_with(filter_stream, |filter_val: &i32| *filter_val < 10);
    let mut result_stream = Box::pin(result_stream);

    // Act & Assert
    filter_tx.send(Sequenced::new(5))?;
    source_tx.send(Sequenced::new(animal_cat()))?;
    let item = result_stream
        .next()
        .await
        .expect("expected first item")
        .unwrap()
        .get()
        .clone();
    assert_eq!(item, animal_cat());

    source_tx.send(Sequenced::new(animal_dog()))?;
    let item = result_stream
        .next()
        .await
        .expect("expected second item")
        .unwrap()
        .get()
        .clone();
    assert_eq!(item, animal_dog());

    filter_tx.send(Sequenced::new(10))?;
    source_tx.send(Sequenced::new(person_alice()))?;
    source_tx.send(Sequenced::new(person_bob()))?;
    assert_no_element_emitted(&mut result_stream, 100).await;
    Ok(())
}

#[tokio::test]
async fn test_take_while_interleaved_updates() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<bool>>();

    let result_stream = source_stream.take_while_with(filter_stream, |filter_val| *filter_val);
    let mut result_stream = Box::pin(result_stream);

    // Act & Assert
    filter_tx.send(Sequenced::new(true))?;
    source_tx.send(Sequenced::new(animal_cat()))?;
    let item = result_stream
        .next()
        .await
        .expect("expected first item")
        .unwrap()
        .get()
        .clone();
    assert_eq!(item, animal_cat());

    filter_tx.send(Sequenced::new(true))?;
    source_tx.send(Sequenced::new(animal_dog()))?;
    let item = result_stream
        .next()
        .await
        .expect("expected second item")
        .unwrap()
        .get()
        .clone();
    assert_eq!(item, animal_dog());

    filter_tx.send(Sequenced::new(true))?;
    source_tx.send(Sequenced::new(person_alice()))?;
    let item = result_stream
        .next()
        .await
        .expect("expected third item")
        .unwrap()
        .get()
        .clone();
    assert_eq!(item, person_alice());

    filter_tx.send(Sequenced::new(false))?;
    source_tx.send(Sequenced::new(person_bob()))?;
    source_tx.send(Sequenced::new(person_charlie()))?;
    assert_no_element_emitted(&mut result_stream, 100).await;
    Ok(())
}

#[tokio::test]
async fn test_take_while_no_filter_value() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<bool>>();

    let result_stream = source_stream.take_while_with(filter_stream, |filter_val| *filter_val);
    let mut result_stream = Box::pin(result_stream);

    // Act & Assert
    source_tx.send(Sequenced::new(animal_cat()))?;
    source_tx.send(Sequenced::new(animal_dog()))?;
    assert_no_element_emitted(&mut result_stream, 100).await;

    filter_tx.send(Sequenced::new(true))?;
    source_tx.send(Sequenced::new(person_alice()))?;
    let item = result_stream
        .next()
        .await
        .expect("expected first item")
        .unwrap()
        .get()
        .clone();
    assert_eq!(item, person_alice());

    source_tx.send(Sequenced::new(person_bob()))?;
    let item = result_stream
        .next()
        .await
        .expect("expected second item")
        .unwrap()
        .get()
        .clone();
    assert_eq!(item, person_bob());
    Ok(())
}

#[tokio::test]
async fn test_take_while_empty_source() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel::<Sequenced<bool>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<bool>>();

    let result_stream = source_stream.take_while_with(filter_stream, |filter_val| *filter_val);
    let mut result_stream = Box::pin(result_stream);

    // Act
    filter_tx.send(Sequenced::new(true))?;
    drop(source_tx);

    // Assert
    assert_no_element_emitted(&mut result_stream, 100).await;
    Ok(())
}

#[tokio::test]
async fn test_take_while_empty_filter() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel();
    let (_filter_tx, filter_stream) = test_channel::<Sequenced<bool>>();

    let result_stream = source_stream.take_while_with(filter_stream, |filter_val| *filter_val);
    let mut result_stream = Box::pin(result_stream);

    // Act
    source_tx.send(Sequenced::new(animal_cat()))?;
    source_tx.send(Sequenced::new(animal_dog()))?;

    // Assert
    assert_no_element_emitted(&mut result_stream, 100).await;
    Ok(())
}

#[tokio::test]
async fn test_take_while_filter_changes_back_to_true() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<bool>>();

    let result_stream = source_stream.take_while_with(filter_stream, |filter_val| *filter_val);
    let mut result_stream = Box::pin(result_stream);

    // Act & Assert
    filter_tx.send(Sequenced::new(true))?;
    source_tx.send(Sequenced::new(animal_cat()))?;
    let item = result_stream
        .next()
        .await
        .expect("expected first item")
        .unwrap()
        .get()
        .clone();
    assert_eq!(item, animal_cat());

    filter_tx.send(Sequenced::new(false))?;
    source_tx.send(Sequenced::new(animal_dog()))?;
    assert_no_element_emitted(&mut result_stream, 100).await;

    filter_tx.send(Sequenced::new(true))?;
    source_tx.send(Sequenced::new(person_alice()))?;
    assert_no_element_emitted(&mut result_stream, 100).await;
    Ok(())
}

#[tokio::test]
async fn test_take_while_multiple_source_items_same_filter() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<bool>>();

    let result_stream = source_stream.take_while_with(filter_stream, |filter_val| *filter_val);
    let mut result_stream = Box::pin(result_stream);

    // Act & Assert
    filter_tx.send(Sequenced::new(true))?;
    source_tx.send(Sequenced::new(animal_cat()))?;
    let item = result_stream
        .next()
        .await
        .expect("expected first item")
        .unwrap()
        .get()
        .clone();
    assert_eq!(item, animal_cat());

    source_tx.send(Sequenced::new(animal_dog()))?;
    let item = result_stream
        .next()
        .await
        .expect("expected second item")
        .unwrap()
        .get()
        .clone();
    assert_eq!(item, animal_dog());

    source_tx.send(Sequenced::new(person_alice()))?;
    let item = result_stream
        .next()
        .await
        .expect("expected third item")
        .unwrap()
        .get()
        .clone();
    assert_eq!(item, person_alice());

    source_tx.send(Sequenced::new(person_bob()))?;
    let item = result_stream
        .next()
        .await
        .expect("expected fourth item")
        .unwrap()
        .get()
        .clone();
    assert_eq!(item, person_bob());

    filter_tx.send(Sequenced::new(false))?;
    source_tx.send(Sequenced::new(person_charlie()))?;

    assert_no_element_emitted(&mut result_stream, 100).await;
    Ok(())
}

#[tokio::test]
async fn test_take_while_filter_updates_without_source() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<bool>>();

    let result_stream = source_stream.take_while_with(filter_stream, |filter_val| *filter_val);
    let mut result_stream = Box::pin(result_stream);

    // Act & Assert
    filter_tx.send(Sequenced::new(false))?;
    filter_tx.send(Sequenced::new(true))?;
    filter_tx.send(Sequenced::new(true))?;

    assert_no_element_emitted(&mut result_stream, 100).await;

    source_tx.send(Sequenced::new(animal_cat()))?;
    let item = result_stream
        .next()
        .await
        .expect("expected first item")
        .unwrap()
        .get()
        .clone();
    assert_eq!(item, animal_cat());

    source_tx.send(Sequenced::new(animal_dog()))?;
    let item = result_stream
        .next()
        .await
        .expect("expected second item")
        .unwrap()
        .get()
        .clone();
    assert_eq!(item, animal_dog());
    Ok(())
}
