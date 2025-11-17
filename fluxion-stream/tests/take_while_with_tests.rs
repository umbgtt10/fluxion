// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_stream::take_while_with::TakeWhileExt;
use fluxion_test_utils::helpers::assert_no_element_emitted;
use fluxion_test_utils::sequenced::Sequenced;
use fluxion_test_utils::test_data::{
    animal_cat, animal_dog, person_alice, person_bob, person_charlie,
};
use futures::StreamExt;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

#[tokio::test]
async fn test_take_while_basic() {
    // Arrange
    let (source_tx, source_rx) = mpsc::unbounded_channel();
    let (filter_tx, filter_rx) = mpsc::unbounded_channel::<Sequenced<bool>>();

    let source_stream = UnboundedReceiverStream::new(source_rx);
    let filter_stream = UnboundedReceiverStream::new(filter_rx);

    let result_stream = source_stream.take_while_with(filter_stream, |filter_val| *filter_val);
    let mut result_stream = Box::pin(result_stream);

    // Act & Assert
    filter_tx.send(Sequenced::new(true)).unwrap();
    source_tx.send(Sequenced::new(animal_cat())).unwrap();
    let item = result_stream
        .next()
        .await
        .expect("expected first item")
        .unwrap();
    assert_eq!(item, animal_cat());

    source_tx.send(Sequenced::new(animal_dog())).unwrap();
    let item = result_stream
        .next()
        .await
        .expect("expected second item")
        .unwrap();
    assert_eq!(item, animal_dog());

    source_tx.send(Sequenced::new(person_alice())).unwrap();
    let item = result_stream
        .next()
        .await
        .expect("expected third item")
        .unwrap();
    assert_eq!(item, person_alice());

    filter_tx.send(Sequenced::new(false)).unwrap();
    source_tx.send(Sequenced::new(person_bob())).unwrap();
    source_tx.send(Sequenced::new(person_charlie())).unwrap();
    assert_no_element_emitted(&mut result_stream, 100).await;
}

#[tokio::test]
async fn test_take_while_filter_false_immediately() {
    // Arrange
    let (source_tx, source_rx) = mpsc::unbounded_channel();
    let (filter_tx, filter_rx) = mpsc::unbounded_channel::<Sequenced<bool>>();

    let source_stream = UnboundedReceiverStream::new(source_rx);
    let filter_stream = UnboundedReceiverStream::new(filter_rx);

    let result_stream = source_stream.take_while_with(filter_stream, |filter_val| *filter_val);
    let mut result_stream = Box::pin(result_stream);

    // Act & Assert
    filter_tx.send(Sequenced::new(false)).unwrap();
    source_tx.send(Sequenced::new(animal_cat())).unwrap();
    source_tx.send(Sequenced::new(animal_dog())).unwrap();
    assert_no_element_emitted(&mut result_stream, 100).await;
}

#[tokio::test]
async fn test_take_while_always_true() {
    // Arrange
    let (source_tx, source_rx) = mpsc::unbounded_channel();
    let (filter_tx, filter_rx) = mpsc::unbounded_channel::<Sequenced<bool>>();

    let source_stream = UnboundedReceiverStream::new(source_rx);
    let filter_stream = UnboundedReceiverStream::new(filter_rx);

    let result_stream = source_stream.take_while_with(filter_stream, |filter_val| *filter_val);
    let mut result_stream = Box::pin(result_stream);

    // Act & Assert
    filter_tx.send(Sequenced::new(true)).unwrap();
    source_tx.send(Sequenced::new(animal_cat())).unwrap();
    let item = result_stream
        .next()
        .await
        .expect("expected first item")
        .unwrap();
    assert_eq!(item, animal_cat());

    source_tx.send(Sequenced::new(animal_dog())).unwrap();
    let item = result_stream
        .next()
        .await
        .expect("expected second item")
        .unwrap();
    assert_eq!(item, animal_dog());

    source_tx.send(Sequenced::new(person_alice())).unwrap();
    let item = result_stream
        .next()
        .await
        .expect("expected third item")
        .unwrap();
    assert_eq!(item, person_alice());
}

#[tokio::test]
async fn test_take_while_complex_predicate() {
    // Arrange
    let (source_tx, source_rx) = mpsc::unbounded_channel();
    let (filter_tx, filter_rx) = mpsc::unbounded_channel::<Sequenced<i32>>();

    let source_stream = UnboundedReceiverStream::new(source_rx);
    let filter_stream = UnboundedReceiverStream::new(filter_rx);

    let result_stream =
        source_stream.take_while_with(filter_stream, |filter_val: &i32| *filter_val < 10);
    let mut result_stream = Box::pin(result_stream);

    // Act & Assert
    filter_tx.send(Sequenced::new(5)).unwrap();
    source_tx.send(Sequenced::new(animal_cat())).unwrap();
    let item = result_stream
        .next()
        .await
        .expect("expected first item")
        .unwrap();
    assert_eq!(item, animal_cat());

    source_tx.send(Sequenced::new(animal_dog())).unwrap();
    let item = result_stream
        .next()
        .await
        .expect("expected second item")
        .unwrap();
    assert_eq!(item, animal_dog());

    filter_tx.send(Sequenced::new(10)).unwrap();
    source_tx.send(Sequenced::new(person_alice())).unwrap();
    source_tx.send(Sequenced::new(person_bob())).unwrap();
    assert_no_element_emitted(&mut result_stream, 100).await;
}

#[tokio::test]
async fn test_take_while_interleaved_updates() {
    // Arrange
    let (source_tx, source_rx) = mpsc::unbounded_channel();
    let (filter_tx, filter_rx) = mpsc::unbounded_channel::<Sequenced<bool>>();

    let source_stream = UnboundedReceiverStream::new(source_rx);
    let filter_stream = UnboundedReceiverStream::new(filter_rx);

    let result_stream = source_stream.take_while_with(filter_stream, |filter_val| *filter_val);
    let mut result_stream = Box::pin(result_stream);

    // Act & Assert
    filter_tx.send(Sequenced::new(true)).unwrap();
    source_tx.send(Sequenced::new(animal_cat())).unwrap();
    let item = result_stream
        .next()
        .await
        .expect("expected first item")
        .unwrap();
    assert_eq!(item, animal_cat());

    filter_tx.send(Sequenced::new(true)).unwrap();
    source_tx.send(Sequenced::new(animal_dog())).unwrap();
    let item = result_stream
        .next()
        .await
        .expect("expected second item")
        .unwrap();
    assert_eq!(item, animal_dog());

    filter_tx.send(Sequenced::new(true)).unwrap();
    source_tx.send(Sequenced::new(person_alice())).unwrap();
    let item = result_stream
        .next()
        .await
        .expect("expected third item")
        .unwrap();
    assert_eq!(item, person_alice());

    filter_tx.send(Sequenced::new(false)).unwrap();
    source_tx.send(Sequenced::new(person_bob())).unwrap();
    source_tx.send(Sequenced::new(person_charlie())).unwrap();
    assert_no_element_emitted(&mut result_stream, 100).await;
}

#[tokio::test]
async fn test_take_while_no_filter_value() {
    // Arrange
    let (source_tx, source_rx) = mpsc::unbounded_channel();
    let (filter_tx, filter_rx) = mpsc::unbounded_channel::<Sequenced<bool>>();

    let source_stream = UnboundedReceiverStream::new(source_rx);
    let filter_stream = UnboundedReceiverStream::new(filter_rx);

    let result_stream = source_stream.take_while_with(filter_stream, |filter_val| *filter_val);
    let mut result_stream = Box::pin(result_stream);

    // Act & Assert
    source_tx.send(Sequenced::new(animal_cat())).unwrap();
    source_tx.send(Sequenced::new(animal_dog())).unwrap();
    assert_no_element_emitted(&mut result_stream, 100).await;

    filter_tx.send(Sequenced::new(true)).unwrap();
    source_tx.send(Sequenced::new(person_alice())).unwrap();
    let item = result_stream
        .next()
        .await
        .expect("expected first item")
        .unwrap();
    assert_eq!(item, person_alice());

    source_tx.send(Sequenced::new(person_bob())).unwrap();
    let item = result_stream
        .next()
        .await
        .expect("expected second item")
        .unwrap();
    assert_eq!(item, person_bob());
}

#[tokio::test]
async fn test_take_while_empty_source() {
    // Arrange
    let (source_tx, source_rx) = mpsc::unbounded_channel::<Sequenced<bool>>();
    let (filter_tx, filter_rx) = mpsc::unbounded_channel::<Sequenced<bool>>();

    let source_stream = UnboundedReceiverStream::new(source_rx);
    let filter_stream = UnboundedReceiverStream::new(filter_rx);

    let result_stream = source_stream.take_while_with(filter_stream, |filter_val| *filter_val);
    let mut result_stream = Box::pin(result_stream);

    // Act
    filter_tx.send(Sequenced::new(true)).unwrap();
    drop(source_tx);

    // Assert
    assert_no_element_emitted(&mut result_stream, 100).await;
}

#[tokio::test]
async fn test_take_while_empty_filter() {
    // Arrange
    let (source_tx, source_rx) = mpsc::unbounded_channel();
    let (_filter_tx, filter_rx) = mpsc::unbounded_channel::<Sequenced<bool>>();

    let source_stream = UnboundedReceiverStream::new(source_rx);
    let filter_stream = UnboundedReceiverStream::new(filter_rx);

    let result_stream = source_stream.take_while_with(filter_stream, |filter_val| *filter_val);
    let mut result_stream = Box::pin(result_stream);

    // Act
    source_tx.send(Sequenced::new(animal_cat())).unwrap();
    source_tx.send(Sequenced::new(animal_dog())).unwrap();

    // Assert
    assert_no_element_emitted(&mut result_stream, 100).await;
}

#[tokio::test]
async fn test_take_while_filter_changes_back_to_true() {
    // Arrange
    let (source_tx, source_rx) = mpsc::unbounded_channel();
    let (filter_tx, filter_rx) = mpsc::unbounded_channel::<Sequenced<bool>>();

    let source_stream = UnboundedReceiverStream::new(source_rx);
    let filter_stream = UnboundedReceiverStream::new(filter_rx);

    let result_stream = source_stream.take_while_with(filter_stream, |filter_val| *filter_val);
    let mut result_stream = Box::pin(result_stream);

    // Act & Assert
    filter_tx.send(Sequenced::new(true)).unwrap();
    source_tx.send(Sequenced::new(animal_cat())).unwrap();
    let item = result_stream
        .next()
        .await
        .expect("expected first item")
        .unwrap();
    assert_eq!(item, animal_cat());

    filter_tx.send(Sequenced::new(false)).unwrap();
    source_tx.send(Sequenced::new(animal_dog())).unwrap();
    assert_no_element_emitted(&mut result_stream, 100).await;

    filter_tx.send(Sequenced::new(true)).unwrap();
    source_tx.send(Sequenced::new(person_alice())).unwrap();
    assert_no_element_emitted(&mut result_stream, 100).await;
}

#[tokio::test]
async fn test_take_while_multiple_source_items_same_filter() {
    // Arrange
    let (source_tx, source_rx) = mpsc::unbounded_channel();
    let (filter_tx, filter_rx) = mpsc::unbounded_channel::<Sequenced<bool>>();

    let source_stream = UnboundedReceiverStream::new(source_rx);
    let filter_stream = UnboundedReceiverStream::new(filter_rx);

    let result_stream = source_stream.take_while_with(filter_stream, |filter_val| *filter_val);
    let mut result_stream = Box::pin(result_stream);

    // Act & Assert
    filter_tx.send(Sequenced::new(true)).unwrap();
    source_tx.send(Sequenced::new(animal_cat())).unwrap();
    let item = result_stream
        .next()
        .await
        .expect("expected first item")
        .unwrap();
    assert_eq!(item, animal_cat());

    source_tx.send(Sequenced::new(animal_dog())).unwrap();
    let item = result_stream
        .next()
        .await
        .expect("expected second item")
        .unwrap();
    assert_eq!(item, animal_dog());

    source_tx.send(Sequenced::new(person_alice())).unwrap();
    let item = result_stream
        .next()
        .await
        .expect("expected third item")
        .unwrap();
    assert_eq!(item, person_alice());

    source_tx.send(Sequenced::new(person_bob())).unwrap();
    let item = result_stream
        .next()
        .await
        .expect("expected fourth item")
        .unwrap();
    assert_eq!(item, person_bob());

    filter_tx.send(Sequenced::new(false)).unwrap();
    source_tx.send(Sequenced::new(person_charlie())).unwrap();

    assert_no_element_emitted(&mut result_stream, 100).await;
}

#[tokio::test]
async fn test_take_while_filter_updates_without_source() {
    // Arrange
    let (source_tx, source_rx) = mpsc::unbounded_channel();
    let (filter_tx, filter_rx) = mpsc::unbounded_channel::<Sequenced<bool>>();

    let source_stream = UnboundedReceiverStream::new(source_rx);
    let filter_stream = UnboundedReceiverStream::new(filter_rx);

    let result_stream = source_stream.take_while_with(filter_stream, |filter_val| *filter_val);
    let mut result_stream = Box::pin(result_stream);

    // Act & Assert
    filter_tx.send(Sequenced::new(false)).unwrap();
    filter_tx.send(Sequenced::new(true)).unwrap();
    filter_tx.send(Sequenced::new(true)).unwrap();

    assert_no_element_emitted(&mut result_stream, 100).await;

    source_tx.send(Sequenced::new(animal_cat())).unwrap();
    let item = result_stream
        .next()
        .await
        .expect("expected first item")
        .unwrap();
    assert_eq!(item, animal_cat());

    source_tx.send(Sequenced::new(animal_dog())).unwrap();
    let item = result_stream
        .next()
        .await
        .expect("expected second item")
        .unwrap();
    assert_eq!(item, animal_dog());
}
